import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { createRuntime } from "../test/runtime.ts";

const measurements: unknown[] = [];
const cases = process.env.LINN_SYNC_BENCH_CASE ? [process.env.LINN_SYNC_BENCH_CASE] : ["unchanged", "small-change", "initial", "sleeping"];
const paths = process.env.LINN_SYNC_BENCH_PATHS ? [process.env.LINN_SYNC_BENCH_PATHS] : ["cold", "warm"];
const repeats = Number(process.env.LINN_SYNC_BENCH_REPEATS ?? "1");
assert.ok(Number.isInteger(repeats) && repeats > 0);
for (const workload of cases) for (const pathState of paths) for (let sample = 0; sample < repeats; sample++) {
  const runtime = await createRuntime(true);
  try {
    const address = await runtime.ready;
    let output = "";
    const code = await new Promise<number | null>((resolve, reject) => {
      const command = spawn("cargo", ["test", "--test", "mesh_pairing", "sync_latency_matched_coverage", "--", "--ignored", "--nocapture"], {
        cwd: new URL("../../", import.meta.url),
        env: { ...process.env, LINN_RELAY_TEST_URL: address.href, LINN_SYNC_BENCH_CASE: workload, LINN_SYNC_BENCH_PATHS: pathState },
        stdio: ["ignore", "pipe", "inherit"], timeout: 120_000,
      });
      command.stdout.on("data", chunk => { output += chunk; process.stdout.write(chunk); });
      command.once("error", reject);
      command.once("exit", resolve);
    });
    assert.equal(code, 0, `${workload}, ${pathState}, sample ${sample}`);
    const line = output.split("\n").find(line => line.startsWith("MEASURE "));
    assert.ok(line, "coverage assertions must finish before a measurement is accepted");
    measurements.push({ sample, ...JSON.parse(line.slice(8)) });
  } finally {
    await runtime.dispose();
  }
}
if (process.env.LINN_SYNC_BENCH_OUTPUT) {
  await writeFile(process.env.LINN_SYNC_BENCH_OUTPUT, JSON.stringify(measurements, null, 2) + "\n");
}
