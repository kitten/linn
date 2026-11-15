import assert from "node:assert/strict";
import { test } from "node:test";
import { ingress, type IngressEnv } from "../src/ingress.ts";
import { admitMessage, heartbeatDeadline, type MessageBudget } from "../src/limits.ts";

function limits(success = true) {
  const keys: string[] = [];
  const binding = { async limit({ key }: { key: string }) { keys.push(key); return { success }; } };
  return { keys, env: { HTTP_LIMIT: binding, CONNECT_LIMIT: binding } as IngressEnv };
}

test("edge rejects invalid requests without rate-limit or coordinator work", async () => {
  const { env, keys } = limits();
  for (const [path, init, status] of [
    ["/unknown", {}, 404],
    ["/v1/events", {}, 405],
    ["/health", { method: "POST" }, 405],
    ["/v1/history", { method: "POST", headers: { "content-length": "16385" } }, 413],
    ["/v1/history", { method: "POST", headers: { "content-length": "bad" } }, 400],
    ["/relay", {}, 426],
    ["/relay", { headers: { upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v1" } }, 400],
  ] as const) {
    assert.equal((await ingress(new Request(`https://local${path}`, init), env))?.status, status);
  }
  assert.deepEqual(keys, []);
});

test("edge limits known HTTP routes and independently limits valid upgrades", async () => {
  const { env, keys } = limits();
  const headers = { "cf-connecting-ip": "192.0.2.1", "x-forwarded-for": "untrusted" };
  assert.equal((await ingress(new Request("https://local/health", { headers }), env))?.status, 200);
  assert.equal(await ingress(new Request("https://local/v1/service", { headers }), env), undefined);
  const upgrade = new Request("https://local/relay", { headers: { ...headers, upgrade: "websocket", "sec-websocket-protocol": "other, iroh-relay-v2" } });
  assert.equal(await ingress(upgrade, env), undefined);
  assert.deepEqual(keys, Array(4).fill("192.0.2.1"));
  for (const blocked of [limits(false).env, { ...env, CONNECT_LIMIT: limits(false).env.CONNECT_LIMIT }]) {
    const response = await ingress(upgrade, blocked);
    assert.equal(response?.status, 429);
    assert.equal(response?.headers.get("retry-after"), "60");
  }
  const unavailable = { ...env, HTTP_LIMIT: { async limit() { throw new Error("offline"); } } } as IngressEnv;
  assert.equal((await ingress(upgrade, unavailable))?.status, 503);
});

test("message budgets permit bursts, bound tiny-message floods and survive wake", () => {
  const state: MessageBudget = {};
  for (let i = 0; i < 64; i++) assert.equal(admitMessage(state, 1000, true), true);
  assert.equal(admitMessage(state, 1000, true), false);
  const restored = JSON.parse(JSON.stringify(state)) as MessageBudget;
  assert.equal(admitMessage(restored, 1000, true), false);
  assert.equal(admitMessage(restored, 2000, true), true);
  // Ordinary data still has its independent, much larger allowance.
  const data: MessageBudget = {};
  for (let i = 0; i < 16384; i++) assert.equal(admitMessage(data, 1000, false), true);
  assert.equal(admitMessage(data, 1000, false), false);
  assert.equal(admitMessage(data, 2000, false), true);
  assert.equal(admitMessage(state, 0, true), false); // Clock reversal grants no extra refill.
  assert.equal(admitMessage(state, 999, true), false);
  // Old hibernatable attachments without these fields initialize safely.
  assert.equal(admitMessage({}, 2000, false), true);
});

test("handshake alarms do not advance heartbeats and Pong deadlines are exact", () => {
  assert.equal(heartbeatDeadline({ pingAt: 15000 }, 10000), 15000);
  assert.equal(heartbeatDeadline({ awaitingPong: 2500, pingAt: 17500 }, 15000), 32500);
  assert.equal(heartbeatDeadline({ awaitingPong: 0 }, 10000), 30000);
  assert.equal(heartbeatDeadline({ pingAt: 17500 }, 16000), 17500);
  assert.equal(heartbeatDeadline({}, 10000), 10000); // Initialize an older attachment once.
});
