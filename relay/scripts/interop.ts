import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

const runtime = await createRuntime();
try {
  const address = await runtime.ready;
  const result = await new Promise<number | null>((resolve, reject) => {
    const command = spawn("cargo", ["test", "--test", "relay_interop", "--", "--ignored", "--nocapture"], {
      cwd: new URL("../../", import.meta.url),
      env: { ...process.env, LINN_RELAY_TEST_URL: address.href },
      stdio: "inherit", timeout: 180_000,
    });
    command.once("error", reject);
    command.once("exit", resolve);
  });
  if (result !== 0) process.exitCode = 1;
} finally {
  await runtime.dispose();
}
