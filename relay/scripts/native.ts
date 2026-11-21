import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createRuntime } from "../test/runtime.ts";

type Scenario = {
  group: string;
  runtime: Parameters<typeof createRuntime>;
  tests: string[];
  env?: Record<string, string>;
};
const scenarios: Scenario[] = [
  ...[[true, true], [false, true], [true, false]].map(([checks, replies]): Scenario => ({
    group: "dial-check",
    runtime: [true, checks, replies],
    tests: ["--test", "relay_interop", "native_correlated_"],
    env: {
      ...(checks && replies ? { LINN_RELAY_TEST_DELAYED_GONE: "1" } : { LINN_RELAY_TEST_OLD_RELAY: "1" }),
      ...(!replies ? { LINN_RELAY_TEST_NO_REPLY: "1" } : {}),
    },
  })),
  {
    group: "adoption", runtime: [true],
    tests: ["--lib", "native_speculative_"],
  },
  {
    group: "registration", runtime: [true],
    tests: ["--lib", "native_registration_wakes_"],
  },
  {
    group: "registration", runtime: [true, true, true, 0, false],
    tests: ["--test", "relay_interop", "native_registration_notification_old_relay"],
  },
  {
    group: "pipeline", runtime: [true, true, true, 20],
    tests: ["--lib", "native_relay_transfer_latency_matched_coverage"],
  },
];
const group = process.argv[2];
assert.ok(!group || scenarios.some(scenario => scenario.group === group), "unknown native test group");
for (const scenario of scenarios.filter(scenario => !group || scenario.group === group)) {
  const runtime = await createRuntime(...scenario.runtime);
  try {
    const address = await runtime.ready;
    const code = await new Promise<number | null>((resolve, reject) => {
      const child = spawn("cargo", ["test", ...scenario.tests, "--", "--ignored", "--nocapture", "--test-threads=1"], {
        cwd: new URL("../../", import.meta.url),
        env: { ...process.env, LINN_RELAY_TEST_URL: address.href, ...scenario.env },
        stdio: "inherit", timeout: 300_000,
      });
      child.once("error", reject);
      child.once("exit", resolve);
    });
    assert.equal(code, 0, `${scenario.group} acceptance must pass`);
  } finally {
    await runtime.dispose();
  }
}
if (!group) {
  await import("./interop.ts");
  await import("./control.ts");
  await import("./offline.ts");
  await import("./backoff.ts");
  await import("./heartbeats.ts");
}
