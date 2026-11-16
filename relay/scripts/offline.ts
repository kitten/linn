import { mkdtemp, writeFile, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

const runtime = await createRuntime();
const directory = await mkdtemp(join(tmpdir(), "linn-offline-"));
let stopped = false;
let checking = false;
const timer = setInterval(async () => {
  if (checking || stopped) return;
  checking = true;
  try {
    if (await stat(join(directory, "ready")).then(() => true, () => false)) {
      await runtime.dispose();
      stopped = true;
      await writeFile(join(directory, "offline"), "");
    }
  } finally { checking = false; }
}, 20);
try {
  const address = await runtime.ready;
  const result = await new Promise<number | null>((resolve, reject) => {
    const command = spawn("cargo", ["test", "--test", "mesh_pairing", "offline_daemons_rediscover_changed_loopback_ports", "--", "--ignored", "--nocapture"], {
      cwd: new URL("../../", import.meta.url),
      env: { ...process.env, LINN_RELAY_TEST_URL: address.href, LINN_OFFLINE_BARRIER: directory },
      stdio: "inherit", timeout: 120_000,
    });
    command.once("error", reject); command.once("exit", resolve);
  });
  if (result !== 0) process.exitCode = 1;
} finally {
  clearInterval(timer);
  if (!stopped) await runtime.dispose();
  await rm(directory, { recursive: true, force: true });
}
