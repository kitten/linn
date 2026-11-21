import { DurableObject } from "cloudflare:workers";
import { authenticate, clientMessage, forward, frame, hex, unhex } from "./protocol.ts";
import { ingress, type IngressEnv } from "./ingress.ts";
import { admitMessage, heartbeatDeadline, type Heartbeat, type MessageBudget } from "./limits.ts";
import { ControlPlane } from "./control.ts";

export interface Env extends IngressEnv {
  COORDINATOR: DurableObjectNamespace<RelayCoordinator>;
}
interface Attachment extends MessageBudget, Heartbeat {
  version: 1;
  challenge: number[];
  deadline: number;
  endpoint?: string;
  authenticating?: boolean;
  dialCheck?: boolean;
  targetWatch?: boolean;
  session?: string;
  group?: string;
  budget: number;
  budgetAt: number;
  pongToken?: string;
  queuedBytes?: number;
  checkpointBytes?: number;
}

const MAX_WATCHES = 1024;
const MAX_SOCKETS = 128;
const MAX_PENDING = 16;
const MAX_BUFFER = 256 * 1024;
const CHECKPOINT_BYTES = 64 * 1024;
const BYTES_PER_SECOND = 4 * 1024 * 1024;
const HANDSHAKE_MS = 10_000;

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const response = await ingress(request, env);
    if (response) return response;
    return env.COORDINATOR.get(env.COORDINATOR.idFromName("private-mesh-v1")).fetch(request);
  },
} satisfies ExportedHandler<Env>;

// @ref LLP 0008#worker-and-durable-object-architecture
export class RelayCoordinator extends DurableObject<Env> {
  private readonly control = new ControlPlane(this.ctx);

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    // Socket routing metadata survives hibernation; it contains no sync results.
    this.ctx.storage.sql.exec(`CREATE TABLE IF NOT EXISTS target_watches (
      session TEXT NOT NULL, token TEXT NOT NULL, target TEXT NOT NULL,
      active INTEGER NOT NULL DEFAULT 0, PRIMARY KEY(session, token))`);
    this.ctx.storage.sql.exec("DROP INDEX IF EXISTS target_watch_target");
    this.ctx.storage.sql.exec("CREATE INDEX IF NOT EXISTS target_watch_observer ON target_watches(session, target, active)");
    const sessions = new Set(this.ctx.getWebSockets().map(socket =>
      (socket.deserializeAttachment() as Attachment | null)?.session).filter(Boolean));
    for (const row of this.ctx.storage.sql.exec<{session: string}>("SELECT DISTINCT session FROM target_watches").toArray()) {
      if (!sessions.has(row.session)) this.ctx.storage.sql.exec("DELETE FROM target_watches WHERE session = ?", row.session);
    }
  }

  async fetch(request: Request): Promise<Response> {
    const path = new URL(request.url).pathname;
    if (["/v1/service", "/v1/events", "/v1/history"].includes(path)) {
      const result = await this.control.fetch(request);
      if (path === "/v1/events" && result.ok) this.ctx.waitUntil(this.refreshEnrollment());
      return result;
    }
    // Only the validated Worker entry point forwards requests to this object.
    if (path !== "/relay") return new Response("not found", { status: 404 });
    for (const socket of this.ctx.getWebSockets()) {
      const state = socket.deserializeAttachment() as Attachment | null;
      if (state && !state.endpoint && Date.now() >= state.deadline) this.close(socket, 1008, "authentication timed out");
    }
    // Closing sockets still consume capacity until the runtime removes them.
    const sockets = this.ctx.getWebSockets();
    const pending = sockets.filter((socket) => !(socket.deserializeAttachment() as Attachment | null)?.endpoint).length;
    if (sockets.length >= MAX_SOCKETS || pending >= MAX_PENDING) return new Response("busy", { status: 503 });
    const pair = new WebSocketPair();
    const challenge = crypto.getRandomValues(new Uint8Array(16));
    const attachment: Attachment = {
      version: 1, challenge: [...challenge], deadline: Date.now() + HANDSHAKE_MS,
      budget: BYTES_PER_SECOND, budgetAt: Date.now(),
      dialCheck: request.headers.get("x-linn-dial-check") === "1",
      targetWatch: request.headers.get("x-linn-target-watch") === "1",
      session: hex(challenge),
    };
    pair[1].serializeAttachment(attachment);
    this.ctx.acceptWebSocket(pair[1]);
    pair[1].send(frame(0, challenge));
    await this.armAlarm();
    return new Response(null, {
      status: 101, webSocket: pair[0], headers: { "sec-websocket-protocol": "iroh-relay-v2",
        ...(attachment.dialCheck ? { "x-linn-dial-check": "1" } : {}),
        ...(attachment.targetWatch ? { "x-linn-target-watch": "1" } : {}) },
    });
  }

  async webSocketMessage(socket: WebSocket, input: string | ArrayBuffer): Promise<void> {
    try {
      if (typeof input === "string") throw new Error("binary frames required");
      const state = socket.deserializeAttachment() as Attachment;
      if (!state || state.version !== 1) throw new Error("invalid session");
      if (!state.endpoint) {
        await this.authenticateSocket(socket, state, input);
        return;
      }
      const now = Date.now();
      const message = clientMessage(input);
      const acknowledged = message.type === "pong" && hex(message.payload) === state.pongToken;
      // Replies to our own checkpoints are paced by us, not unsolicited traffic.
      if (!admitMessage(state, now, message.type === "ping" || (message.type === "pong" && !acknowledged))) throw new Error("message rate exceeded");
      state.budget = Math.min(BYTES_PER_SECOND, state.budget + Math.max(0, now - state.budgetAt) * BYTES_PER_SECOND / 1000);
      state.budgetAt = Math.max(now, state.budgetAt);
      if (message.type === "datagram") {
        if (input.byteLength > state.budget) {
          socket.serializeAttachment(state);
          return; // QUIC loss recovery backs off instead of losing its session.
        }
        state.budget -= input.byteLength;
      }
      if (acknowledged) {
        state.awaitingPong = undefined;
        state.pongToken = undefined;
        state.queuedBytes = Math.max(0, (state.queuedBytes ?? 0) - (state.checkpointBytes ?? 0));
        state.checkpointBytes = 0;
      }
      socket.serializeAttachment(state);
      if (message.type === "ping") return this.send(socket, frame(10, message.payload));
      if (message.type === "pong") {
        if (acknowledged) {
          if ((state.queuedBytes ?? 0) >= CHECKPOINT_BYTES) this.checkpoint(socket);
        }
        return;
      }
      if (message.type === "target-watch" || message.type === "target-unwatch") {
        if (!state.targetWatch || !state.session) throw new Error("target watch not negotiated");
        const token = hex(message.token);
        if (message.type === "target-unwatch") {
          this.ctx.storage.sql.exec("DELETE FROM target_watches WHERE session = ? AND token = ? AND target = ?", state.session, token, message.destination);
          return;
        }
        const existing = this.ctx.storage.sql.exec<{target: string}>("SELECT target FROM target_watches WHERE session = ? AND token = ?", state.session, token).toArray()[0];
        if (existing && existing.target !== message.destination) throw new Error("conflicting watch token");
        const count = this.ctx.storage.sql.exec<{n: number}>("SELECT COUNT(*) AS n FROM target_watches WHERE session = ?", state.session).toArray()[0]!.n;
        if (!existing && count >= MAX_WATCHES) throw new Error("target watch capacity");
        this.ctx.storage.sql.exec("INSERT OR IGNORE INTO target_watches(session, token, target) VALUES (?, ?, ?)", state.session, token, message.destination);
        if (!await this.control.sameGroup(state.endpoint, message.destination)) throw new Error("mesh authorization denied");
        if ((socket.deserializeAttachment() as Attachment | null)?.session !== state.session) return;
        // Unsubscribe/close may interleave with the membership authorization await.
        const activated = this.ctx.storage.sql.exec("UPDATE target_watches SET active = 1 WHERE session = ? AND token = ? AND target = ? RETURNING target", state.session, token, message.destination).toArray();
        if (activated.length && this.find(message.destination)) this.notifyWatch(socket, state.session, message.destination, token);
        return;
      }
      if (!await this.control.sameGroup(state.endpoint, message.destination)) throw new Error("mesh authorization denied");
      if ((socket.deserializeAttachment() as Attachment | null)?.endpoint !== state.endpoint) return;
      const destination = this.find(message.destination);
      if (message.type === "dial-check") {
        if (!state.dialCheck) throw new Error("dial check not negotiated");
        const payload = new Uint8Array(49);
        payload.set(unhex(message.destination));
        payload.set(message.token, 32);
        payload[48] = destination ? 1 : 0;
        return this.send(socket, frame(15, payload));
      }
      if (!destination) return this.send(socket, frame(8, unhex(message.destination)));
      this.send(destination, forward(state.endpoint, message));
    } catch {
      // Do not log packet, key, challenge, token or authentication bytes.
      this.close(socket, 1008, "invalid or unauthorized relay message");
    }
  }

  private async authenticateSocket(socket: WebSocket, state: Attachment, input: ArrayBuffer): Promise<void> {
    if (Date.now() >= state.deadline) throw new Error("authentication timed out");
    if (state.authenticating) throw new Error("authentication already in progress");
    state.authenticating = true;
    socket.serializeAttachment(state);
    const endpoint = await authenticate(input, new Uint8Array(state.challenge));
    const group = await this.control.group(endpoint);
    if (!group) throw new Error("endpoint not enrolled");
    if (!(socket.deserializeAttachment() as Attachment | null)?.authenticating) return;
    if (Date.now() >= state.deadline) throw new Error("authentication timed out");
    if (this.find(endpoint)) {
      socket.send(frame(13, new Uint8Array([1])));
      return this.close(socket, 1008, "duplicate endpoint identity");
    }
    state.endpoint = endpoint;
    state.authenticating = undefined;
    state.group = group;
    state.challenge = [];
    socket.serializeAttachment(state);
    socket.send(frame(2));
    await this.notifyRegistration(endpoint);
  }

  webSocketClose(socket: WebSocket): void { this.close(socket, 1000, "closed"); }
  webSocketError(socket: WebSocket): void { this.close(socket, 1011, "transport failure"); }

  private async refreshEnrollment(): Promise<void> {
    for (const socket of this.ctx.getWebSockets()) {
      const state = socket.deserializeAttachment() as Attachment | null;
      if (!state?.endpoint) continue;
      const group = await this.control.group(state.endpoint);
      const current = socket.deserializeAttachment() as Attachment | null;
      if (current?.endpoint !== state.endpoint) continue;
      if (!group) this.close(socket, 1008, "membership revoked");
      else { current.group = group; socket.serializeAttachment(current); }
    }
  }

  async alarm(): Promise<void> {
    const now = Date.now();
    for (const socket of this.ctx.getWebSockets()) {
      const state = socket.deserializeAttachment() as Attachment | null;
      if (!state) continue;
      if (!state.endpoint) {
        if (now >= state.deadline) this.close(socket, 1008, "authentication timed out");
        continue;
      }
      if (now < heartbeatDeadline(state)) continue;
      if (state.awaitingPong !== undefined) this.close(socket, 1008, "heartbeat timed out");
    }
    await this.armAlarm();
  }

  private notifyWatch(socket: WebSocket, session: string, target: string, token: string): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    if (state?.session !== session) return;
    if ((state.queuedBytes ?? 0) + 49 > MAX_BUFFER) {
      this.close(socket, 1013, "registration notification backpressure");
      return;
    }
    const payload = new Uint8Array(48);
    payload.set(unhex(target));
    payload.set(unhex(token, 16), 32);
    this.send(socket, frame(18, payload));
  }
  private watchTokens(session: string, target: string): Set<string> {
    return new Set(this.ctx.storage.sql.exec<{token: string}>(
      "SELECT token FROM target_watches WHERE session = ? AND target = ? AND active = 1", session, target,
    ).toArray().map(row => row.token));
  }
  private async notifyRegistration(target: string): Promise<void> {
    for (const socket of this.ctx.getWebSockets()) {
      const observer = socket.deserializeAttachment() as Attachment | null;
      if (!observer?.endpoint || !observer.session) continue;
      const tokens = this.watchTokens(observer.session, target);
      if (!tokens.size || !await this.control.sameGroup(observer.endpoint, target)) continue;
      if (!this.find(target)) continue;
      // Authorization may yield to unsubscribe/close. Recheck the captured
      // tokens without including subscriptions that activated during the await.
      const current = this.watchTokens(observer.session, target);
      for (const token of tokens) {
        if (current.has(token)) this.notifyWatch(socket, observer.session, target, token);
      }
    }
  }

  private find(endpoint: string): WebSocket | undefined {
    return this.ctx.getWebSockets().find((socket) => {
      const state = socket.deserializeAttachment() as Attachment;
      return state?.endpoint === endpoint;
    });
  }
  protected send(socket: WebSocket, data: Uint8Array): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    if (!state?.endpoint) return;
    // Workers exposes no awaitable send backpressure. A matching protocol Pong
    // confirms all earlier frames crossed the ordered WebSocket. Bound the
    // unconfirmed window; dropping relay datagrams lets QUIC retransmit them.
    if ((state.queuedBytes ?? 0) + data.length > MAX_BUFFER) {
      this.checkpoint(socket);
      return;
    }
    state.queuedBytes = (state.queuedBytes ?? 0) + data.length;
    socket.serializeAttachment(state);
    try { socket.send(data); } catch { this.close(socket, 1011, "write failed"); }
    if ((state.queuedBytes ?? 0) >= CHECKPOINT_BYTES && state.awaitingPong === undefined) this.checkpoint(socket);
  }
  private checkpoint(socket: WebSocket): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    if (!state?.endpoint || state.awaitingPong !== undefined) return;
    const token = crypto.getRandomValues(new Uint8Array(8));
    state.awaitingPong = Date.now();
    state.pongToken = hex(token);
    state.checkpointBytes = state.queuedBytes ?? 0;
    socket.serializeAttachment(state);
    this.ctx.waitUntil(this.armAlarm());
    try { socket.send(frame(9, token)); } catch { this.close(socket, 1011, "write failed"); }
  }
  private close(socket: WebSocket, code: number, reason: string): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    // Clear before notifications so a closing socket cannot be selected again.
    socket.serializeAttachment(null);
    try {
      if (state?.session) this.ctx.storage.sql.exec("DELETE FROM target_watches WHERE session = ?", state.session);
    } catch { /* Closing still invalidates observers; orphan rows are pruned on rehydration. */ }
    try { socket.close(code, reason); } catch { /* Already closed. */ }
    this.ctx.waitUntil(this.armAlarm(true));
    if (state?.endpoint) {
      for (const other of this.ctx.getWebSockets()) {
        if (other === socket) continue;
        const target = other.deserializeAttachment() as Attachment | null;
        if (target?.endpoint && target.group === state.group) {
          this.send(other, frame(8, unhex(state.endpoint)));
        }
      }
    }
  }
  protected alarmOperation(_operation: "sets" | "deletes"): void {}

  private async armAlarm(clearIdle = false): Promise<void> {
    await this.ctx.storage.transaction(async (storage) => {
      const current = await storage.getAlarm();
      let next = Infinity;
      for (const socket of this.ctx.getWebSockets()) {
        const state = socket.deserializeAttachment() as Attachment | null;
        if (!state) continue;
        next = Math.min(next, state.endpoint ? heartbeatDeadline(state) : state.deadline);
      }
      if (Number.isFinite(next)) {
        if (current === null || current > next) {
          this.alarmOperation("sets");
          await storage.setAlarm(next);
        }
      } else if (clearIdle && current !== null) {
        this.alarmOperation("deletes");
        await storage.deleteAlarm();
      }
    });
  }
}
