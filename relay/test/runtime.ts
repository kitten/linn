import { readFile } from "node:fs/promises";
import { Miniflare } from "miniflare";

export async function createRuntime(bootstrap: string): Promise<Miniflare> {
  return new Miniflare({
    workers: [{
      config: {
        name: "relay", compatibilityDate: "2026-11-01",
        manifest: {
          mainModule: "index.js",
          modules: { "index.js": { type: "esm", contents: await readFile(new URL("../dist/index.js", import.meta.url), "utf8") } },
        },
        exports: { RelayCoordinator: { type: "durable-object", storage: "sqlite" } },
        env: {
          COORDINATOR: { type: "durable-object", worker: "relay", exportName: "RelayCoordinator" },
          BOOTSTRAP_TOKEN: { type: "json", value: bootstrap },
        },
      },
    }],
  });
}
