import { randomBytes } from "node:crypto";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

const bootstrap = randomBytes(32).toString("hex");
const runtime = await createRuntime(bootstrap);
const directory = await mkdtemp(join(tmpdir(), "linn-control-interop-"));
try {
  const secretFile = join(directory, "bootstrap");
  await writeFile(secretFile, bootstrap, { mode: 0o600 });
  const address = await runtime.ready;
  const result = await new Promise<number | null>((resolve, reject) => {
    const command = spawn("cargo", ["test", "--test", "mesh_control", "--test", "mesh_pairing", "--", "--ignored", "--skip", "offline_daemons_rediscover_changed_loopback_ports", "--nocapture"], {
      cwd: new URL("../../", import.meta.url),
      env: { ...process.env, LINN_RELAY_TEST_URL: address.href, LINN_RELAY_BOOTSTRAP_FILE: secretFile },
      stdio: "inherit", timeout: 120_000,
    });
    command.once("error", reject); command.once("exit", resolve);
  });
  if (result !== 0) process.exitCode = 1;
} finally {
  await runtime.dispose(); await rm(directory, { recursive: true, force: true });
}
