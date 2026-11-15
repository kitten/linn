import assert from "node:assert/strict";
import { test } from "node:test";
import { readState, writeState, deleteState } from "../src/state.ts";
import { migrateDerivedIndexes } from "../src/control.ts";
import { MAX_EVENT_BYTES, type Membership } from "../src/membership.ts";

// Enforce the real per-value ceiling while exercising transactional migration.
class Storage {
  values = new Map<string, unknown>();
  transactions = 0;
  failAt = Infinity;
  async get<T>(key: string | string[]): Promise<T | Map<string, T> | undefined> {
    if (Array.isArray(key)) return new Map(key.filter((k) => this.values.has(k)).map((k) => [k, structuredClone(this.values.get(k)) as T]));
    return structuredClone(this.values.get(key)) as T | undefined;
  }
  async put(key: string, value: unknown) {
    assert.ok(Buffer.byteLength(JSON.stringify(value)) + key.length < 2 * 1024 * 1024);
    this.values.set(key, structuredClone(value));
  }
  async delete(key: string | string[]) { for (const k of typeof key === "string" ? [key] : key) this.values.delete(k); }
  async list<T>(options: { prefix: string; startAfter?: string; limit: number }) {
    return new Map([...this.values].filter(([k]) => k.startsWith(options.prefix) && (!options.startAfter || k > options.startAfter))
      .sort(([a], [b]) => a.localeCompare(b)).slice(0, options.limit).map(([k, v]) => [k, structuredClone(v) as T]));
  }
  async transaction<T>(fn: (tx: DurableObjectTransaction) => Promise<T>): Promise<T> {
    if (++this.transactions === this.failAt) throw new Error("interrupted upgrade");
    const before = structuredClone(this.values);
    try { return await fn(this.tx()); } catch (error) { this.values = before; throw error; }
  }
  tx() { return this as unknown as DurableObjectTransaction; }
  storage() { return this as unknown as DurableObjectStorage; }
}
function state(head = "head"): Membership {
  return { head, service: "https://mesh.example", receipt_key: "receipt", roots: ["root"], members: ["owner"], revoked: [], invitations: {} };
}

test("current cache exceeds one Worker value without losing revocations or retaining ancestor caches", async () => {
  const store = new Storage();
  const first = state();
  first.revoked = Array.from({ length: 40_000 }, (_, i) => i.toString(16).padStart(64, "0"));
  assert.ok(Buffer.byteLength(JSON.stringify(first)) > 2 * 1024 * 1024);
  await writeState(store.tx(), first);
  assert.deepEqual(await readState(store.tx(), first.head), first);
  await writeState(store.tx(), { ...first, head: "next" });
  await deleteState(store.tx(), first.head);
  assert.equal(await readState(store.tx(), first.head), undefined);
  assert.ok([...store.values.keys()].every((k) => k.startsWith("state:next")));
});

test("legacy cache and root-history upgrade resumes after a committed batch and preserves signed events", async () => {
  const store = new Storage();
  for (let i = 1; i <= 130; i++) {
    const cursor = String(i).padStart(16, "0");
    store.values.set(`order:${cursor}`, { hash: `event${i}`, root: "root" });
    store.values.set(`state:event${i}`, state(`event${i}`));
    store.values.set(`event:event${i}`, { original: `signed bytes ${i}` });
  }
  store.values.set("root:root", "event130");
  store.failAt = 2;
  await assert.rejects(migrateDerivedIndexes(store.storage()), /interrupted upgrade/);
  assert.equal(store.values.get("derived-index-cursor"), `order:${String(1).padStart(16, "0")}`);
  store.failAt = Infinity;
  await migrateDerivedIndexes(store.storage());
  await migrateDerivedIndexes(store.storage());
  assert.deepEqual(await readState(store.tx(), "event130"), state("event130"));
  assert.equal([...store.values.keys()].filter((k) => k.startsWith("state:")).length, 2);
  assert.equal([...store.values.keys()].filter((k) => k.startsWith("component-order:")).length, 130);
  for (let i = 1; i <= 130; i++) assert.deepEqual(store.values.get(`event:event${i}`), { original: `signed bytes ${i}` });
  assert.equal(store.values.has("derived-index-cursor"), false);
});

test("a newer derived format is rejected without rewriting service data", async () => {
  const store = new Storage();
  store.values.set("derived-index-version", 3);
  store.values.set("future-only", "preserve");
  const before = structuredClone(store.values);
  await assert.rejects(migrateDerivedIndexes(store.storage()), /unsupported membership cache version/);
  assert.deepEqual(store.values, before);
});

test("storage failures are unavailable while malformed requests remain bad input", async () => {
  const { ControlPlane } = await import("../src/control.ts");
  const { canonical, READ_DOMAIN, EVENT_DOMAIN, sign } = await import("../src/membership.ts");
  const { hex } = await import("../src/protocol.ts");
  const store = new Storage();
  const context = { storage: store.storage(), blockConcurrencyWhile: <T>(fn: () => Promise<T>) => fn() } as unknown as DurableObjectState;
  const plane = new ControlPlane(context, { BOOTSTRAP_TOKEN: "bootstrap" });
  const service = "https://mesh.example";
  const response = await plane.fetch(new Request(`${service}/v1/service`));
  assert.equal(response.status, 200);
  const receipt = await response.json() as { receipt_key: string };
  const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
  const actor = hex(new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey)));
  const head = "4".repeat(64);
  const root = "2".repeat(64);
  await writeState(store.tx(), { ...state(head), service, receipt_key: receipt.receipt_key, roots: [root], members: [actor] });
  store.values.set(`root:${root}`, head);
  const delegated = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
  const key = hex(new Uint8Array(await crypto.subtle.exportKey("raw", delegated.publicKey)));
  const now = Math.floor(Date.now() / 1000);
  const event = { version: 1, service, id: "1".repeat(64), heads: [head],
    action: { kind: "invite", actor, key, receipt_key: receipt.receipt_key, not_before: now, expires: now + 600 } };
  const proposal = { event, signatures: { [actor]: await sign(pair.privateKey, EVENT_DOMAIN, event) } };
  store.failAt = store.transactions + 1;
  const unavailable = await plane.fetch(new Request(`${service}/v1/events`, { method: "POST", headers: { authorization: "Bearer bootstrap" }, body: canonical(proposal) }));
  assert.equal(unavailable.status, 503);
  assert.deepEqual(await unavailable.json(), { error: "control service unavailable" });
  store.failAt = Infinity;
  // The exact proposal remains retryable, not a rejected or partially consumed event.
  assert.equal((await plane.fetch(new Request(`${service}/v1/events`, { method: "POST", headers: { authorization: "Bearer bootstrap" }, body: canonical(proposal) }))).status, 200);
  const read = { version: 1, service, key: actor, expires: Math.floor(Date.now() / 1000) + 60, nonce: "3".repeat(64), after: null, invitation: null };
  store.failAt = store.transactions + 1;
  assert.equal((await plane.fetch(new Request(`${service}/v1/history`, { method: "POST", body: canonical({ request: read, signature: await sign(pair.privateKey, READ_DOMAIN, read) }) }))).status, 503);
  for (const input of ["{", new Uint8Array([0xff]), canonical({ ...proposal, event: { ...event, service: "bad url" } }), canonical({ ...proposal, event: { ...event, id: "bad key" } })]) {
    assert.equal((await plane.fetch(new Request(`${service}/v1/events`, { method: "POST", body: input }))).status, 400);
  }
});


test("body bounds include stalled readers and cancellation cannot stall cleanup", async () => {
  const { body } = await import("../src/control.ts");
  const request = (stream: ReadableStream<Uint8Array>) => new Request("https://mesh.example/v1/events", {
    method: "POST", body: stream, duplex: "half",
  } as RequestInit);
  let cancelled = false;
  const stalled = request(new ReadableStream({
    cancel() { cancelled = true; return new Promise<void>(() => {}); },
  }));
  await assert.rejects(body(stalled, 20), /timed out/);
  assert.equal(cancelled, true);
  const oversized = request(new ReadableStream({ start(c) { c.enqueue(new Uint8Array(MAX_EVENT_BYTES + 1)); } }));
  await assert.rejects(body(oversized, 100), /too large/);
  assert.deepEqual(await body(new Request("https://mesh.example", { method: "POST", body: "{}" }), 100), {});
});

test("busy control requests reject promptly and release capacity after invalid input", async () => {
  const { ControlPlane } = await import("../src/control.ts");
  const store = new Storage();
  const context = { storage: store.storage(), blockConcurrencyWhile: <T>(fn: () => Promise<T>) => fn() } as unknown as DurableObjectState;
  const plane = new ControlPlane(context, {});
  await plane.fetch(new Request("https://mesh.example/v1/service"));
  const controllers: ReadableStreamDefaultController<Uint8Array>[] = [];
  const pending = Array.from({ length: 4 }, () => plane.fetch(new Request("https://mesh.example/v1/events", {
    method: "POST", body: new ReadableStream({ start(c) { controllers.push(c); } }), duplex: "half",
  } as RequestInit)));
  assert.equal((await plane.fetch(new Request("https://mesh.example/v1/service"))).status, 503);
  for (const controller of controllers) controller.close();
  assert.ok((await Promise.all(pending)).every((response) => response.status === 400));
  assert.equal((await plane.fetch(new Request("https://mesh.example/v1/service"))).status, 200);
});


test("corrupt chunk headers cannot turn cache deletion into an unbounded loop", async () => {
  for (const chunks of [Infinity, -1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    const store = new Storage();
    store.values.set("state:bad", { chunks });
    await assert.rejects(readState(store.tx(), "bad"), /invalid state cache/);
    await assert.rejects(deleteState(store.tx(), "bad"), /invalid state cache/);
    assert.ok(store.values.has("state:bad"));
  }
});


test("a bounded upgrade step checkpoints and yields instead of blocking startup", async () => {
  const store = new Storage();
  for (let i = 1; i <= 3; i++) store.values.set(`order:${String(i).padStart(16, "0")}`, { hash: `event${i}`, root: "root" });
  assert.equal(await migrateDerivedIndexes(store.storage(), 0), false);
  assert.equal(store.values.get("derived-index-cursor"), `order:${String(1).padStart(16, "0")}`);
  assert.equal(await migrateDerivedIndexes(store.storage()), true);
});


test("cache deletion enumerates stored chunks instead of trusting a huge count", async () => {
  const store = new Storage();
  store.values.set("state:bad", { chunks: Number.MAX_SAFE_INTEGER });
  store.values.set("state:bad:0", "orphan");
  await assert.rejects(readState(store.tx(), "bad"), /incomplete state cache/);
  await deleteState(store.tx(), "bad");
  assert.equal(store.values.size, 0);
});
