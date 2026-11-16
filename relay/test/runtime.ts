import { readFile } from "node:fs/promises";
import { Miniflare } from "miniflare";

export function instrumentAlarmOperations(source: string): string {
  for (const [operation, counter] of [["setAlarm(next)", "sets"], ["deleteAlarm()", "deletes"]] as const) {
    const target = `await storage.${operation}`;
    source = source.replace(target, `(alarmOperations.${counter}++, ${target})`);
  }
  return "let alarmOperations = { sets: 0, deletes: 0 };\n" + source;
}

export async function createRuntime(inspect = false): Promise<Miniflare> {
  const bundle = await readFile(new URL("../dist/index.js", import.meta.url), "utf8");
  const contents = inspect ? instrumentAlarmOperations(bundle) : bundle;
  return new Miniflare({
    workers: [{
      config: {
        name: "relay", compatibilityDate: "2026-11-01",
        manifest: {
          mainModule: "index.js",
          modules: { "index.js": { type: "esm", contents: contents + (inspect ? `
const heartbeatTraffic = { applicationPings: 0, applicationPongs: 0 };
const originalMessage = RelayCoordinator.prototype.webSocketMessage;
RelayCoordinator.prototype.webSocketMessage = function (socket, input) {
  const tag = typeof input === "string" ? -1 : new Uint8Array(input)[0];
  if (tag === 9) heartbeatTraffic.applicationPings++;
  if (tag === 10) heartbeatTraffic.applicationPongs++;
  return originalMessage.call(this, socket, input);
};
const originalObjectFetch = RelayCoordinator.prototype.fetch;
RelayCoordinator.prototype.fetch = async function (request) {
  if (new URL(request.url).pathname === "/_inspect") {
    if (new URL(request.url).searchParams.has("fire")) {
      await this.ctx.storage.deleteAlarm();
      await this.alarm();
    }
    return Response.json({ alarmOperations, heartbeatTraffic, alarm: await this.ctx.storage.getAlarm(), sockets: this.ctx.getWebSockets().map(s => s.deserializeAttachment()) });
  }
  return originalObjectFetch.call(this, request);
};
const originalWorkerFetch = index_default.fetch;
index_default.fetch = function (request, env) {
  if (new URL(request.url).pathname === "/_inspect") {
    return env.COORDINATOR.get(env.COORDINATOR.idFromName("private-mesh-v1")).fetch(request);
  }
  return originalWorkerFetch(request, env);
};
` : "") } },
        },
        exports: { RelayCoordinator: { type: "durable-object", storage: "sqlite" } },
        env: {
          HTTP_LIMIT: { type: "rate-limit", namespace: "7349101", simple: { limit: 1200, period: 60 } },
          CONNECT_LIMIT: { type: "rate-limit", namespace: "7349102", simple: { limit: 120, period: 60 } },
          COORDINATOR: { type: "durable-object", worker: "relay", exportName: "RelayCoordinator" },
        },
      },
    }],
  });
}
