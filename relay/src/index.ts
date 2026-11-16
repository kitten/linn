import { DurableObject } from "cloudflare:workers";
import { authenticate, clientMessage, forward, frame, hex, unhex } from "./protocol.ts";
import { ingress, type IngressEnv } from "./ingress.ts";
import { admitMessage, heartbeatDeadline, type Heartbeat, type MessageBudget } from "./limits.ts";
import { ControlPlane } from "./control.ts";

interface Env extends IngressEnv {
  COORDINATOR: DurableObjectNamespace<RelayCoordinator>;
}
interface Attachment extends MessageBudget, Heartbeat {
  version: 1;
  challenge: number[];
  deadline: number;
  endpoint?: string;
  authenticating?: boolean;
  group?: string;
  budget: number;
  budgetAt: number;
  pongToken?: string;
  queuedBytes?: number;
  checkpointBytes?: number;
}

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
    };
    pair[1].serializeAttachment(attachment);
    this.ctx.acceptWebSocket(pair[1]);
    pair[1].send(frame(0, challenge));
    await this.armAlarm();
    return new Response(null, {
      status: 101, webSocket: pair[0], headers: { "sec-websocket-protocol": "iroh-relay-v2" },
    });
  }

  async webSocketMessage(socket: WebSocket, input: string | ArrayBuffer): Promise<void> {
    try {
      if (typeof input === "string") throw new Error("binary frames required");
      const state = socket.deserializeAttachment() as Attachment;
      if (!state || state.version !== 1) throw new Error("invalid session");
      if (!state.endpoint) {
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
      if (!await this.control.sameGroup(state.endpoint, message.destination)) throw new Error("mesh authorization denied");
      if ((socket.deserializeAttachment() as Attachment | null)?.endpoint !== state.endpoint) return;
      const destination = this.find(message.destination);
      if (!destination) return this.send(socket, frame(8, unhex(message.destination)));
      this.send(destination, forward(state.endpoint, message));
    } catch {
      // Do not log packet, key, challenge, token or authentication bytes.
      this.close(socket, 1008, "invalid or unauthorized relay message");
    }
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

  private find(endpoint: string): WebSocket | undefined {
    return this.ctx.getWebSockets().find((socket) => {
      const state = socket.deserializeAttachment() as Attachment;
      return state?.endpoint === endpoint;
    });
  }
  private send(socket: WebSocket, data: Uint8Array): void {
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
        if (current === null || current > next) await storage.setAlarm(next);
      } else if (clearIdle && current !== null) {
        await storage.deleteAlarm();
      }
    });
  }
}
