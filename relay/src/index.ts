import { DurableObject } from "cloudflare:workers";
import { authenticate, clientMessage, forward, frame, hex, unhex } from "./protocol.ts";
import { ControlPlane } from "./control.ts";

interface Env {
  COORDINATOR: DurableObjectNamespace<RelayCoordinator>;
  BOOTSTRAP_TOKEN?: string;
}
interface Attachment {
  version: 1;
  challenge: number[];
  deadline: number;
  endpoint?: string;
  authenticating?: boolean;
  group?: string;
  budget: number;
  budgetAt: number;
  awaitingPong?: number;
  pongToken?: string;
  queuedBytes?: number;
  checkpointBytes?: number;
}

const MAX_SOCKETS = 128;
const MAX_BUFFER = 256 * 1024;
const BYTES_PER_SECOND = 4 * 1024 * 1024;
const HANDSHAKE_MS = 30_000;
const PING_MS = 15_000;

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return env.COORDINATOR.get(env.COORDINATOR.idFromName("private-mesh-v1")).fetch(request);
  },
} satisfies ExportedHandler<Env>;

// @ref LLP 0008#worker-and-durable-object-architecture
export class RelayCoordinator extends DurableObject<Env> {
  private readonly control = new ControlPlane(this.ctx, this.env);

  async fetch(request: Request): Promise<Response> {
    const path = new URL(request.url).pathname;
    if (["/v1/service", "/v1/events", "/v1/history"].includes(path)) {
      const result = await this.control.fetch(request);
      if (path === "/v1/events" && result.ok) this.ctx.waitUntil(this.refreshEnrollment());
      return result;
    }
    if (request.method === "GET" && (path === "/health" || path === "/ping")) {
      return new Response(path === "/health" ? "linn-relay: v1\n" : "", {
        headers: { "cache-control": "no-store" },
      });
    }
    if (path !== "/relay" || request.method !== "GET") return new Response("not found", { status: 404 });
    if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
      return new Response("WebSocket required", { status: 426 });
    }
    const versions = (request.headers.get("sec-websocket-protocol") ?? "").split(",").map((x) => x.trim());
    if (!versions.includes("iroh-relay-v2")) return new Response("iroh-relay-v2 required", { status: 400 });
    if (this.ctx.getWebSockets().length >= MAX_SOCKETS) return new Response("busy", { status: 503 });
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
        if (Date.now() > state.deadline) throw new Error("authentication timed out");
        if (state.authenticating) throw new Error("authentication already in progress");
        state.authenticating = true;
        socket.serializeAttachment(state);
        const endpoint = await authenticate(input, new Uint8Array(state.challenge));
        const group = await this.control.group(endpoint);
        if (!group) throw new Error("endpoint not enrolled");
        if (!(socket.deserializeAttachment() as Attachment | null)?.authenticating) return;
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
      state.budget = Math.min(BYTES_PER_SECOND, state.budget + (now - state.budgetAt) * BYTES_PER_SECOND / 1000);
      state.budgetAt = now;
      if (message.type === "datagram") {
        if (input.byteLength > state.budget) {
          socket.serializeAttachment(state);
          return; // QUIC loss recovery backs off instead of losing its session.
        }
        state.budget -= input.byteLength;
      }
      if (message.type === "pong" && hex(message.payload) === state.pongToken) {
        state.awaitingPong = undefined;
        state.pongToken = undefined;
        state.queuedBytes = Math.max(0, (state.queuedBytes ?? 0) - (state.checkpointBytes ?? 0));
        state.checkpointBytes = 0;
      }
      socket.serializeAttachment(state);
      if (message.type === "ping") return this.send(socket, frame(10, message.payload));
      if (message.type === "pong") return;
      const destination = this.find(message.destination);
      if (!destination) return this.send(socket, frame(8, unhex(message.destination)));
      const target = destination.deserializeAttachment() as Attachment;
      const sourceGroup = await this.control.group(state.endpoint);
      const targetGroup = await this.control.group(target.endpoint!);
      if (!sourceGroup || targetGroup !== sourceGroup) throw new Error("mesh authorization denied");
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
      if (!group) this.close(socket, 1008, "membership revoked");
      else { state.group = group; socket.serializeAttachment(state); }
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
      if (state.awaitingPong && now - state.awaitingPong >= 2 * PING_MS) {
        this.close(socket, 1008, "heartbeat timed out");
        continue;
      }
      if (!state.awaitingPong) this.checkpoint(socket);
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
    if ((state.queuedBytes ?? 0) + data.length > MAX_BUFFER) return;
    state.queuedBytes = (state.queuedBytes ?? 0) + data.length;
    socket.serializeAttachment(state);
    try { socket.send(data); } catch { this.close(socket, 1011, "write failed"); }
    if ((state.queuedBytes ?? 0) >= 16 * 1024 && !state.awaitingPong) this.checkpoint(socket);
  }
  private checkpoint(socket: WebSocket): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    if (!state?.endpoint || state.awaitingPong) return;
    const token = crypto.getRandomValues(new Uint8Array(8));
    state.awaitingPong = Date.now();
    state.pongToken = hex(token);
    state.checkpointBytes = state.queuedBytes ?? 0;
    socket.serializeAttachment(state);
    try { socket.send(frame(9, token)); } catch { this.close(socket, 1011, "write failed"); }
  }
  private close(socket: WebSocket, code: number, reason: string): void {
    const state = socket.deserializeAttachment() as Attachment | null;
    // Clear before notifications so a closing socket cannot be selected again.
    socket.serializeAttachment(null);
    try { socket.close(code, reason); } catch { /* Already closed. */ }
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
  private async armAlarm(): Promise<void> {
    if (this.ctx.getWebSockets().length && await this.ctx.storage.getAlarm() === null) {
      await this.ctx.storage.setAlarm(Date.now() + PING_MS);
    }
  }
}
