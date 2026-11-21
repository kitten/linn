// This entry point is bundled only by the native test runner, never deployed.
import worker, { RelayCoordinator as Coordinator, type Env } from "../src/index.ts";
interface TestEnv extends Env {
  TEST_OPTIONS: { inspect: boolean; dialChecks: boolean; dialReplies: boolean; relayDelayMs: number; targetWatches: boolean };
}
const alarmOperations = { sets: 0, deletes: 0 };
const heartbeatTraffic = { applicationPings: 0, applicationPongs: 0 };
const probeTraffic = { https: 0, captivePortal: 0 };
let relayTraffic = traffic();
function traffic() { return { inboundFrames: {} as Record<number, number>, outboundFrames: {} as Record<number, number>, inboundBytes: 0, outboundBytes: 0 }; }
export class RelayCoordinator extends Coordinator {
  private readonly options: TestEnv["TEST_OPTIONS"];
  private delayGone = false;
  private readonly delayedGone: Array<[WebSocket, Uint8Array]> = [];
  constructor(ctx: DurableObjectState, env: TestEnv) {
    super(ctx, env);
    this.options = env.TEST_OPTIONS;
  }
  protected override alarmOperation(operation: "sets" | "deletes"): void {
    alarmOperations[operation]++;
  }
  override async fetch(request: Request): Promise<Response> {
    if (this.options.inspect && new URL(request.url).pathname === "/_inspect") {
      const query = new URL(request.url).searchParams;
      if (query.has("resetTraffic")) relayTraffic = traffic();
      if (query.has("delayGone")) this.delayGone = true;
      if (query.has("releaseGone")) {
        this.delayGone = false;
        for (const [socket, data] of this.delayedGone.splice(0)) super.send(socket, data);
      }
      if (query.has("disconnect")) for (const socket of this.ctx.getWebSockets()) socket.close(1012, "test reconnect");
      if (query.has("fire")) { await this.ctx.storage.deleteAlarm(); await this.alarm(); }
      return Response.json({ alarmOperations, heartbeatTraffic, probeTraffic, relayTraffic,
        watchCount: this.ctx.storage.sql.exec<{n: number}>("SELECT COUNT(*) AS n FROM target_watches").toArray()[0]!.n,
        delayedGone: this.delayedGone.length, alarm: await this.ctx.storage.getAlarm(),
        sockets: this.ctx.getWebSockets().map(socket => socket.deserializeAttachment()) });
    }
    const headers = new Headers(request.headers);
    if (!this.options.dialChecks) headers.delete("x-linn-dial-check");
    if (!this.options.targetWatches) headers.delete("x-linn-target-watch");
    return super.fetch(new Request(request, { headers }));
  }
  protected override send(socket: WebSocket, data: Uint8Array): void {
    if (this.options.inspect) {
      relayTraffic.outboundBytes += data.byteLength;
      relayTraffic.outboundFrames[data[0]!] = (relayTraffic.outboundFrames[data[0]!] ?? 0) + 1;
    }
    if (data[0] === 15 && !this.options.dialReplies) return;
    if (this.delayGone && data[0] === 8) { this.delayedGone.push([socket, data]); return; }
    if (this.options.relayDelayMs && (data[0] === 6 || data[0] === 7)) {
      const copy = data.slice();
      this.ctx.waitUntil(new Promise<void>(resolve => setTimeout(() => {
        super.send(socket, copy); resolve();
      }, this.options.relayDelayMs)));
      return;
    }
    super.send(socket, data);
  }
  override async webSocketMessage(socket: WebSocket, input: string | ArrayBuffer): Promise<void> {
    const tag = typeof input === "string" ? -1 : new Uint8Array(input)[0]!;
    if (this.options.inspect) {
      relayTraffic.inboundBytes += typeof input === "string" ? input.length : input.byteLength;
      relayTraffic.inboundFrames[tag] = (relayTraffic.inboundFrames[tag] ?? 0) + 1;
      if (tag === 9) heartbeatTraffic.applicationPings++;
      if (tag === 10) heartbeatTraffic.applicationPongs++;
    }
    return super.webSocketMessage(socket, input);
  }
}
export default {
  fetch(request: Request, env: TestEnv) {
    const path = new URL(request.url).pathname;
    if (env.TEST_OPTIONS.inspect) {
      if (path === "/ping") probeTraffic.https++;
      if (path === "/generate_204") probeTraffic.captivePortal++;
      if (path === "/_inspect") return env.COORDINATOR.get(env.COORDINATOR.idFromName("private-mesh-v1")).fetch(request);
    }
    return worker.fetch(request, env);
  },
} satisfies ExportedHandler<TestEnv>;
