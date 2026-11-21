import assert from "node:assert/strict";
import { test } from "node:test";
import { createRuntime } from "./runtime.ts";

for (const enabled of [false, true]) {
  test(`test bindings negotiate extensions ${enabled ? "on" : "off"} without editing the bundle`, async () => {
    const runtime = await createRuntime(true, enabled, true, 0, enabled);
    try {
      const response = await runtime.dispatchFetch("https://local/relay", { headers: {
        upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v2",
        "x-linn-dial-check": "1", "x-linn-target-watch": "1",
      }});
      assert.equal(response.headers.get("x-linn-dial-check"), enabled ? "1" : null);
      assert.equal(response.headers.get("x-linn-target-watch"), enabled ? "1" : null);
      response.webSocket!.accept();
      response.webSocket!.close();
    } finally { await runtime.dispose(); }
  });
}
