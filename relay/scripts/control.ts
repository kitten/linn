import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

const runtime = await createRuntime();
try {
  const address = await runtime.ready;
  const result = await new Promise<number | null>((resolve, reject) => {
    const command = spawn("cargo", ["test", "--test", "mesh_control", "--test", "mesh_pairing", "--", "--ignored", "--skip", "offline_daemons_rediscover_changed_loopback_ports", "--skip", "sync_latency_matched_coverage", "--nocapture"], {
      cwd: new URL("../../", import.meta.url),
      env: { ...process.env, LINN_RELAY_TEST_URL: address.href },
      stdio: "inherit", timeout: 120_000,
    });
    command.once("error", reject); command.once("exit", resolve);
  });
  if (result !== 0) process.exitCode = 1;
} finally {
  await runtime.dispose();
}
