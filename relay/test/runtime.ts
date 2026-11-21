import assert from "node:assert/strict";
import { build } from "esbuild";
import { fileURLToPath } from "node:url";
import { Miniflare } from "miniflare";

const bundle = build({
  entryPoints: [fileURLToPath(new URL("./harness.ts", import.meta.url))],
  bundle: true, write: false, format: "esm", platform: "browser",
  external: ["cloudflare:workers"],
}).then(result => result.outputFiles![0]!.text);

export async function createRuntime(inspect = false, dialChecks = true, dialReplies = true, relayDelayMs = 0, targetWatches = true): Promise<Miniflare> {
  assert.ok(Number.isFinite(relayDelayMs) && relayDelayMs >= 0);
  assert.ok(inspect || relayDelayMs === 0);
  const contents = await bundle;
  return new Miniflare({
    workers: [{
      config: {
        name: "relay", compatibilityDate: "2026-11-01",
        manifest: {
          mainModule: "index.js",
          modules: { "index.js": { type: "esm", contents } },
        },
        exports: { RelayCoordinator: { type: "durable-object", storage: "sqlite" } },
        env: {
          TEST_OPTIONS: { type: "json", value: { inspect, dialChecks, dialReplies, relayDelayMs, targetWatches } },
          HTTP_LIMIT: { type: "rate-limit", namespace: "7349101", simple: { limit: 1200, period: 60 } },
          CONNECT_LIMIT: { type: "rate-limit", namespace: "7349102", simple: { limit: 120, period: 60 } },
          COORDINATOR: { type: "durable-object", worker: "relay", exportName: "RelayCoordinator" },
        },
      },
    }],
  });
}
