import assert from "node:assert/strict";
import { test } from "node:test";
import { instrumentAlarmOperations } from "./runtime.ts";

const bundle = "await storage.setAlarm(next);\nawait storage.deleteAlarm();";
for (const operation of ["setAlarm(next)", "deleteAlarm()"] as const) {
  const target = `await storage.${operation}`;
  for (const scenario of ["missing", "duplicated"] as const) {
    test(`inspection rejects ${scenario} ${operation} instrumentation`, () => {
      const changed = scenario === "missing" ? bundle.replace(target, "") : `${bundle}\n${target};`;
      assert.throws(() => instrumentAlarmOperations(changed), error =>
        error instanceof assert.AssertionError &&
        error.message.startsWith(`expected exactly one alarm instrumentation target: ${operation}`));
    });
  }
}
