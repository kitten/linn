import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { Miniflare } from "miniflare";
import { createRuntime } from "./runtime.ts";
import { canonical, EVENT_DOMAIN, hash, sign, type Action, type Event, type Acceptance } from "../src/membership.ts";
import { authenticate, challengeMessage, decode, frame, hex } from "../src/protocol.ts";

let runtime: Miniflare;
before(async () => {
  runtime = await createRuntime(true);
});
after(async () => { await runtime?.dispose(); });

async function device() {
  const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
  const publicKey = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
  return { pair, publicKey, id: hex(publicKey) };
}
async function connect(key: Awaited<ReturnType<typeof device>>, extensions: Record<string, string> = {}) {
  const response = await runtime.dispatchFetch("https://local/relay", {
    headers: { upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v2", ...extensions },
  });
  assert.equal(response.status, 101);
  assert.equal(response.headers.get("sec-websocket-protocol"), "iroh-relay-v2");
  for (const header of ["x-linn-dial-check", "x-linn-target-watch"]) {
    assert.equal(response.headers.get(header), extensions[header] ?? null);
  }
  const socket = response.webSocket!;
  socket.accept();
  const challenge = decode(await next(socket)).payload;
  const payload = new Uint8Array(97);
  payload.set(key.publicKey); payload[32] = 64;
  payload.set(new Uint8Array(await crypto.subtle.sign("Ed25519", key.pair.privateKey, challengeMessage(challenge))), 33);
  const auth = frame(1, payload);
  assert.equal(await authenticate(auth.buffer, challenge), key.id);
  const result = next(socket);
  socket.send(auth);
  return { socket, result };
}
async function connected(key: Awaited<ReturnType<typeof device>>, extensions: Record<string, string> = {}) {
  const { socket, result } = await connect(key, extensions);
  assert.equal(decode(await result).kind, 2);
  return socket;
}
function targetToken(target: Uint8Array, token: number): Uint8Array {
  const payload = new Uint8Array(48);
  payload.set(target);
  new DataView(payload.buffer).setUint32(44, token);
  return payload;
}

test("relay rejects setup without credentials and does not expose an unversioned socket", async () => {
  assert.equal((await runtime.dispatchFetch("https://local/v1/interop", { method: "PUT", body: "[]" })).status, 404);
  assert.equal((await runtime.dispatchFetch("https://local/relay")).status, 426);
  const health = await runtime.dispatchFetch("https://local/health");
  assert.equal(health.status, 200);
  assert.equal(await health.text(), "linn-relay: v1\n");
  assert.equal(health.headers.get("cache-control"), "no-store");
  assert.equal((await runtime.dispatchFetch("https://local/healthz")).status, 404);
  for (const protocol of [undefined, "iroh-relay-v1", "iroh-relay-v20"]) {
    const headers: Record<string, string> = { upgrade: "websocket" };
    if (protocol) headers["sec-websocket-protocol"] = protocol;
    assert.equal((await runtime.dispatchFetch("https://local/relay", { headers })).status, 400);
  }
});

test("real Worker authentication, routing, rejection and duplicate-key isolation", async () => {
  const keys = await Promise.all(Array.from({ length: 4 }, device));
  await enroll(keys.slice(0, 2).map((key) => key.pair));
  await enroll([keys[3]!.pair]);
  const inspect = async () => (await runtime.dispatchFetch("https://local/_inspect")).json() as Promise<{
    alarmOperations: { sets: number; deletes: number };
    alarm: number | null; sockets: Array<{ endpoint?: string; awaitingPong?: number }>;
  }>;
  const a = await connect(keys[0]!);
  assert.equal(decode(await a.result).kind, 2);
  assert.notEqual((await inspect()).alarm, null);
  const b = await connect(keys[1]!);
  assert.equal(decode(await b.result).kind, 2);
  assert.notEqual((await inspect()).alarm, null);
  await runtime.unsafeEvictDurableObject("relay", "RelayCoordinator", {
    name: "private-mesh-v1", webSockets: "hibernate",
  });
  const duplicate = await connect(keys[0]!);
  assert.equal(decode(await duplicate.result).kind, 13);
  const unauthorized = await connect(keys[2]!);
  await assert.rejects(unauthorized.result);

  // Pending unauthenticated connections cannot consume all established capacity.
  const pending: WebSocket[] = [];
  for (let i = 0; i < 16; i++) {
    const response = await runtime.dispatchFetch("https://local/relay", {
      headers: { upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v2" },
    });
    assert.equal(response.status, 101);
    const socket = response.webSocket!;
    socket.accept();
    await next(socket);
    pending.push(socket);
  }
  assert.equal((await runtime.dispatchFetch("https://local/relay", {
    headers: { upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v2" },
  })).status, 503);

  const packet = new Uint8Array(37);
  packet.set(keys[1]!.publicKey);
  packet.set([0, 1, 2, 3, 4], 32);
  const received = next(b.socket);
  a.socket.send(frame(4, packet));
  const forwarded = decode(await received);
  assert.equal(forwarded.kind, 6);
  assert.equal(hex(forwarded.payload.subarray(0, 32)), keys[0]!.id);
  assert.deepEqual(forwarded.payload.subarray(32), packet.subarray(32));
  for (const socket of pending) socket.close();

  // Delay the first checkpoint acknowledgement while later data fills the
  // window. Its Pong must checkpoint the residual data, including after wake.
  const large = new Uint8Array(64 * 1024);
  large.set(keys[1]!.publicKey);
  const data = next(b.socket, 6);
  const checkpoint = next(b.socket, 9);
  a.socket.send(frame(4, large));
  await data;
  const token = decode(await checkpoint).payload;
  const queued = [60, 60, 60].map(kib => {
    const packet = new Uint8Array(kib * 1024);
    packet.set(keys[1]!.publicKey);
    return packet;
  });
  for (const packet of queued) {
    const delivered = next(b.socket, 6);
    a.socket.send(frame(4, packet));
    await delivered;
  }
  // An earlier retained alarm must only reconcile deadlines, never expire a
  // still-live checkpoint or emit another Ping.
  const outstanding = await (await runtime.dispatchFetch("https://local/_inspect?fire")).json() as Awaited<ReturnType<typeof inspect>>;
  const receiver = outstanding.sockets.find(s => s.endpoint === keys[1]!.id)!;
  assert.ok(outstanding.alarm! <= receiver.awaitingPong! + 30_000);
  await runtime.unsafeEvictDurableObject("relay", "RelayCoordinator", {
    name: "private-mesh-v1", webSockets: "hibernate",
  });
  const residualCheckpoint = next(b.socket, 9);
  b.socket.send(frame(10, token));
  const residualToken = decode(await residualCheckpoint).payload;
  const residual = await inspect();
  assert.deepEqual(residual.alarmOperations, outstanding.alarmOperations);
  const pendingReceiver = residual.sockets.find(s => s.endpoint === keys[1]!.id)!;
  assert.ok(residual.alarm! <= pendingReceiver.awaitingPong! + 30_000);
  b.socket.send(frame(10, residualToken));
  // A subsequent ping/pong orders the inspection after the acknowledgement.
  const barrier = next(b.socket, 10);
  b.socket.send(frame(9, new Uint8Array(8)));
  await barrier;
  assert.notEqual((await inspect()).alarm, null);
  const reconciled = await (await runtime.dispatchFetch("https://local/_inspect?fire")).json() as { alarm: number | null };
  assert.equal(reconciled.alarm, null);
  // A full-sized datagram must fit again once the residual checkpoint clears.
  const retried = next(b.socket, 6);
  const retryCheckpoint = next(b.socket, 9);
  a.socket.send(frame(4, large));
  assert.equal(decode(await retried).payload.length, large.length);
  b.socket.send(frame(10, decode(await retryCheckpoint).payload));
  const retryBarrier = next(b.socket, 10);
  b.socket.send(frame(9, new Uint8Array(8)));
  await retryBarrier;
  assert.notEqual((await inspect()).alarm, null);
  // More than the unsolicited-control burst must still allow requested Pongs.
  let checkpoints = 0;
  const acknowledge = (event: { data: ArrayBuffer }) => {
    const message = decode(event.data);
    if (message.kind === 9) { checkpoints++; b.socket.send(frame(10, message.payload)); }
  };
  b.socket.addEventListener("message", acknowledge);
  const bulk = new Uint8Array(64 * 1024);
  bulk.set(keys[1]!.publicKey);
  for (let i = 0; i < 96; i++) {
    const delivered = next(b.socket, 6);
    a.socket.send(frame(4, bulk));
    await delivered;
    if (i % 16 === 15) await new Promise(resolve => setTimeout(resolve, 125));
  }
  b.socket.removeEventListener("message", acknowledge);
  assert.ok(checkpoints > 64);
  // A control-message flood is disconnected, while the other member stays usable.
  const flooded = closed(a.socket);
  for (let i = 0; i < 256; i++) a.socket.send(frame(9, new Uint8Array(8)));
  assert.equal(await flooded, 1008);
  const pong = next(b.socket, 10);
  b.socket.send(frame(9, new Uint8Array(8)));
  assert.equal(decode(await pong).kind, 10);
  // Another mesh gets the same denial whether the target is offline or online.
  for (const target of [keys[0]!, keys[1]!]) {
    for (const tag of [4, 14, 16]) {
      const extensions: Record<string, string> = tag === 14 ? { "x-linn-dial-check": "1" }
        : tag === 16 ? { "x-linn-target-watch": "1" } : {};
      const outsider = await connect(keys[3]!, extensions);
      assert.equal(decode(await outsider.result).kind, 2);
      const replies: number[] = [];
      outsider.socket.addEventListener("message", event => { replies.push(decode(event.data as ArrayBuffer).kind); });
      const denied = closed(outsider.socket);
      const packet = tag === 4 ? new Uint8Array(33) : targetToken(target.publicKey, 1);
      packet.set(target.publicKey);
      outsider.socket.send(frame(tag, packet));
      assert.equal(await denied, 1008);
      assert.deepEqual(replies, [], "foreign-mesh checks/watches must not reveal registration");
    }
  }
  b.socket.close();
});

function next(socket: { addEventListener: Function; removeEventListener: Function }, kind?: number): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timer);
      socket.removeEventListener("message", message);
      socket.removeEventListener("close", closed);
    };
    const message = (event: { data: ArrayBuffer }) => {
      if (kind !== undefined && decode(event.data).kind !== kind) return;
      cleanup(); resolve(event.data);
    };
    const closed = () => { cleanup(); reject(new Error("socket closed")); };
    const timer = setTimeout(() => { cleanup(); reject(new Error("message timeout")); }, 3000);
    socket.addEventListener("message", message);
    socket.addEventListener("close", closed);
  });
}


async function enroll(keys: CryptoKeyPair[]): Promise<void> {
  const service = "https://local";
  const receipt = await (await runtime.dispatchFetch(`${service}/v1/service`)).json() as { receipt_key: string };
  const id = () => hex(crypto.getRandomValues(new Uint8Array(32)));
  const publicKey = async (key: CryptoKeyPair) => hex(new Uint8Array(await crypto.subtle.exportKey("raw", key.publicKey)));
  let head: string | undefined;
  const commit = async (action: Action, signers: CryptoKeyPair[]) => {
    const event: Event = { version: 1, service, id: id(), heads: head ? [head] : [], action };
    const proposal = { event, signatures: Object.fromEntries(await Promise.all(signers.map(async (key) => [await publicKey(key), await sign(key.privateKey, EVENT_DOMAIN, event)]))) };
    const response = await runtime.dispatchFetch(`${service}/v1/events`, {
      method: "POST", body: canonical(proposal),
    });
    assert.equal(response.status, 200, await response.clone().text());
    head = hash((await response.json() as Acceptance).proposal);
    return head;
  };
  const owner = keys[0]!;
  await commit({ kind: "genesis", actor: await publicKey(owner), root: id(), receipt_key: receipt.receipt_key }, [owner]);
  for (const joining of keys.slice(1)) {
    const delegated = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
    const now = Math.floor(Date.now() / 1000);
    const invitation = await commit({ kind: "invite", actor: await publicKey(owner), key: await publicKey(delegated), receipt_key: receipt.receipt_key, not_before: now, expires: now + 600 }, [owner]);
    await commit({ kind: "redeem", actor: await publicKey(joining), invitation }, [joining, delegated]);
  }
}

function closed(socket: { addEventListener: Function; removeEventListener: Function }, timeout = 3000): Promise<number> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { socket.removeEventListener("close", listener); reject(new Error("close timeout")); }, timeout);
    const listener = (event: { code: number }) => { clearTimeout(timer); socket.removeEventListener("close", listener); resolve(event.code); };
    socket.addEventListener("close", listener);
  });
}

test("pending handshake expires without another request", async () => {
  const isolated = await createRuntime();
  try {
    const address = new URL("/relay", await isolated.ready);
    address.protocol = "ws:";
    const socket = new WebSocket(address, "iroh-relay-v2");
    socket.binaryType = "arraybuffer";
    await next(socket);
    // Workerd can keep TCP teardown pending after delivering the Close frame.
    // Node exposes CLOSING before its final close event; no further request is sent.
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => { clearInterval(poll); reject(new Error("handshake expiry timeout")); }, 15000);
      const poll = setInterval(() => {
        if (socket.readyState !== WebSocket.OPEN) { clearInterval(poll); clearTimeout(timer); resolve(); }
      }, 25);
    });
    assert.ok(socket.readyState === WebSocket.CLOSING || socket.readyState === WebSocket.CLOSED);
  } finally { await isolated.dispose(); }
});

test("negotiated dial checks correlate absence and survive target reconnect without trusting disconnect notices", async () => {
  const keys = await Promise.all([device(), device()]);
  await enroll(keys.map(key => key.pair));
  const source = await connected(keys[0]!, { "x-linn-dial-check": "1" });
  const check = async (value: number) => {
    const payload = new Uint8Array(48);
    payload.set(keys[1]!.publicKey);
    payload.fill(value, 32);
    const reply = next(source, 15);
    source.send(frame(14, payload));
    const result = decode(await reply).payload;
    assert.deepEqual(result.subarray(0, 48), payload);
    return result[48];
  };
  assert.equal(await check(1), 0);
  const target = await connected(keys[1]!);
  assert.equal(await check(2), 1);
  const notice = next(source, 8);
  const departed = closed(target);
  target.close();
  await departed;
  const oldNotice = await notice;
  assert.equal(decode(oldNotice).payload.length, 32);
  const replacement = await connected(keys[1]!);
  assert.equal(await check(3), 1);
  assert.equal(decode(oldNotice).kind, 8); // no nonce: this remains advisory
  await disconnect(replacement, source);
});

test("negotiated registration watches survive notifications, close setup races and fence unsubscribe", async () => {
  const keys = await Promise.all([device(), device()]);
  await enroll(keys.map(key => key.pair));
  const source = await connected(keys[0]!, { "x-linn-target-watch": "1" });
  const notifications: Uint8Array[] = [];
  source.addEventListener("message", event => {
    const parsed = decode(event.data as ArrayBuffer);
    if (parsed.kind === 18) notifications.push(parsed.payload);
  });
  const watch = (token: number) => {
    const payload = new Uint8Array(48);
    payload.set(keys[1]!.publicKey); payload.fill(token, 32);
    source.send(frame(16, payload));
    return payload;
  };
  const first = next(source, 18);
  const payload = watch(1);
  const target = await connected(keys[1]!);
  assert.deepEqual(decode(await first).payload, payload);
  const current = next(source, 18);
  const secondPayload = watch(2);
  assert.deepEqual(decode(await current).payload, secondPayload);
  await runtime.unsafeEvictDurableObject("relay", "RelayCoordinator", {
    name: "private-mesh-v1", webSockets: "hibernate",
  });
  const gone = next(source, 8); const ended = closed(target); target.close(); await ended; await gone;
  const retired = watch(3);
  source.send(frame(17, retired));
  const fenced = next(source, 10); source.send(frame(9, new Uint8Array(8))); await fenced;
  const replacement = await connected(keys[1]!);
  const pong = next(source, 10); source.send(frame(9, new Uint8Array(8))); await pong;
  assert.equal(notifications.filter(value => value[32] === 3).length, 0);
  assert.equal(notifications.filter(value => value[32] === 1).length, 2);
  await disconnect(replacement, source);
});

async function disconnect(...sockets: Awaited<ReturnType<typeof connected>>[]) {
  const ended = sockets.map(socket => closed(socket));
  for (const socket of sockets) socket.close();
  await Promise.all(ended);
}
async function watchCount(): Promise<number> {
  const state = await (await runtime.dispatchFetch("https://local/_inspect")).json() as { watchCount: number };
  return state.watchCount;
}

async function subscribeToCapacity(socket: Awaited<ReturnType<typeof connected>>, target: Uint8Array) {
  const barrier = next(socket, 10);
  for (let token = 0; token < 1024; token++) socket.send(frame(16, targetToken(target, token)));
  socket.send(frame(9, new Uint8Array(8)));
  await barrier;
}

test("target watch capacity and close retirement survive hibernation", async () => {
  const keys = await Promise.all([device(), device()]);
  await enroll(keys.map(key => key.pair));
  const source = await connected(keys[0]!, { "x-linn-target-watch": "1" });
  await subscribeToCapacity(source, keys[1]!.publicKey);
  assert.equal(await watchCount(), 1024);
  const ended = closed(source);
  source.send(frame(16, targetToken(keys[1]!.publicKey, 1024)));
  assert.equal(await ended, 1008);
  assert.equal(await watchCount(), 0);
  // A replacement observer has its own empty routing state, including after wake.
  await runtime.unsafeEvictDurableObject("relay", "RelayCoordinator", {
    name: "private-mesh-v1", webSockets: "hibernate",
  });
  const replacement = await connected(keys[0]!, { "x-linn-target-watch": "1" });
  await subscribeToCapacity(replacement, keys[1]!.publicKey);
  assert.equal(await watchCount(), 1024);
  await disconnect(replacement);
  assert.equal(await watchCount(), 0);
});

test("registration fan-out closes an exhausted observer rather than dropping its trigger", async () => {
  const keys = await Promise.all([device(), device()]);
  await enroll(keys.map(key => key.pair));
  const source = await connected(keys[0]!, { "x-linn-target-watch": "1" });
  await subscribeToCapacity(source, keys[1]!.publicKey);
  const ended = closed(source, 10_000);
  let observerClosed = false;
  source.addEventListener("close", () => { observerClosed = true; });
  for (let registration = 0; registration < 6 && !observerClosed; registration++) {
    const notified = next(source, 18).then(() => {}, () => {});
    const target = await connected(keys[1]!);
    await Promise.race([notified, ended]);
    const departed = closed(target);
    target.close();
    await departed;
  }
  assert.equal(await ended, 1013);
  assert.equal(await watchCount(), 0);
});
