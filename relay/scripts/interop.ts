import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

const runtime = await createRuntime(true);
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
  else {
    const state = await (await runtime.dispatchFetch("https://local/_inspect")).json() as {
      alarmOperations: { sets: number; deletes: number };
      heartbeatTraffic: { applicationPings: number; applicationPongs: number };
      alarm: number | null;
    };
    assert.equal(state.heartbeatTraffic.applicationPings, 0, "native health pings must not reach Worker JavaScript");
    assert.ok(state.heartbeatTraffic.applicationPongs > 0, "transfers must acknowledge application checkpoints");
    console.log("relay alarm operations:", JSON.stringify(state.alarmOperations));
    assert.ok(state.alarmOperations.sets < 100, "alarm persistence must be coalesced across transfer checkpoints");
    assert.equal(state.alarm, null, "closed native endpoints must leave no alarm");
  }
} finally {
  await runtime.dispose();
}
