import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { Miniflare } from "miniflare";
import { createRuntime } from "./runtime.ts";
import { canonical, EVENT_DOMAIN, hash, sign, type Action, type Event, type Acceptance } from "../src/membership.ts";
import { authenticate, challengeMessage, decode, frame, hex } from "../src/protocol.ts";

let runtime: Miniflare;
const bootstrap = "private-local-test-bootstrap";
before(async () => {
  runtime = await createRuntime(bootstrap);
});
after(async () => { await runtime?.dispose(); });

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
  const keys = await Promise.all([0, 1, 2].map(async () => {
    const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
    const publicKey = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
    return { pair, publicKey, id: hex(publicKey) };
  }));
  await enroll(keys.slice(0, 2).map((key) => key.pair));
  const connect = async (key: typeof keys[number]) => {
    const response = await runtime.dispatchFetch("https://local/relay", {
      headers: { upgrade: "websocket", "sec-websocket-protocol": "iroh-relay-v2" },
    });
    assert.equal(response.status, 101);
    assert.equal(response.headers.get("sec-websocket-protocol"), "iroh-relay-v2");
    const socket = response.webSocket!;
    socket.accept();
    const challengeFrame = await next(socket);
    const challenge = decode(challengeFrame).payload;
    const signature = new Uint8Array(await crypto.subtle.sign("Ed25519", key.pair.privateKey, challengeMessage(challenge)));
    const payload = new Uint8Array(97);
    payload.set(key.publicKey);
    payload[32] = 64;
    payload.set(signature, 33);
    const auth = frame(1, payload);
    assert.equal(await authenticate(auth.buffer, challenge), key.id);
    const result = next(socket);
    socket.send(auth);
    return { socket, result };
  };
  const a = await connect(keys[0]!);
  assert.equal(decode(await a.result).kind, 2);
  const b = await connect(keys[1]!);
  assert.equal(decode(await b.result).kind, 2);
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
  // A control-message flood is disconnected, while the other member stays usable.
  const flooded = closed(a.socket);
  for (let i = 0; i < 256; i++) a.socket.send(frame(9, new Uint8Array(8)));
  assert.equal(await flooded, 1008);
  const pong = next(b.socket, 10);
  b.socket.send(frame(9, new Uint8Array(8)));
  assert.equal(decode(await pong).kind, 10);
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
      method: "POST", headers: { authorization: `Bearer ${bootstrap}` }, body: canonical(proposal),
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
  const isolated = await createRuntime(bootstrap);
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
