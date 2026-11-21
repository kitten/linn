import assert from "node:assert/strict";
import { test } from "node:test";
import { authenticate, challengeMessage, clientMessage, decode, forward, frame, hex, unhex } from "../src/protocol.ts";

test("challenge proves key possession and cannot be replayed for a new challenge", async () => {
  const pair = await crypto.subtle.generateKey("Ed25519", true, ["sign", "verify"]);
  const publicKey = new Uint8Array(await crypto.subtle.exportKey("raw", pair.publicKey));
  const challenge = crypto.getRandomValues(new Uint8Array(16));
  const signature = new Uint8Array(await crypto.subtle.sign("Ed25519", pair.privateKey, challengeMessage(challenge)));
  const payload = new Uint8Array(97);
  payload.set(publicKey);
  payload[32] = 64;
  payload.set(signature, 33);
  const auth = frame(1, payload);
  assert.equal(await authenticate(auth.buffer, challenge), hex(publicKey));
  await assert.rejects(authenticate(auth.buffer, crypto.getRandomValues(new Uint8Array(16))));
  await assert.rejects(authenticate(frame(1, payload.subarray(0, 96)).buffer, challenge));
});

test("forwarding preserves datagram batches while binding the real source", () => {
  const source = "11".repeat(32);
  const payload = new Uint8Array(39);
  payload.fill(0x22, 0, 32);
  payload[32] = 2;
  payload[33] = 0;
  payload[34] = 2;
  payload.set([1, 2, 3, 4], 35);
  const message = clientMessage(frame(5, payload).buffer);
  assert.equal(message.type, "datagram");
  if (message.type !== "datagram") throw new Error("expected datagram");
  assert.equal(message.destination, "22".repeat(32));
  const output = decode(forward(source, message).buffer);
  assert.equal(output.kind, 7);
  assert.equal(hex(output.payload.subarray(0, 32)), source);
  assert.deepEqual(output.payload.subarray(32), payload.subarray(32));
});

test("invalid control messages and oversized/truncated datagrams are rejected", () => {
  for (const bytes of [new Uint8Array(), new Uint8Array([0x40]), frame(2), frame(4, new Uint8Array(32)),
    frame(9, new Uint8Array(7)), frame(4, new Uint8Array(65537)), frame(5, new Uint8Array(35))]) {
    assert.throws(() => clientMessage(bytes.buffer));
  }
  assert.deepEqual(clientMessage(frame(9, new Uint8Array(8)).buffer).type, "ping");
});

test("dial check has an exact target/token payload and cannot be confused with a disconnect", () => {
  const payload = new Uint8Array(48);
  payload.fill(0x11, 0, 32); payload.fill(0x22, 32);
  const request = clientMessage(frame(14, payload).buffer);
  assert.equal(request.type, "dial-check");
  if (request.type !== "dial-check") throw new Error("expected dial-check");
  assert.equal(request.destination, "11".repeat(32));
  assert.equal(hex(request.token), "22".repeat(16));
  for (const length of [0, 32, 47, 49]) assert.throws(() => clientMessage(frame(14, new Uint8Array(length)).buffer));
  assert.throws(() => clientMessage(frame(8, new Uint8Array(32)).buffer));
});

test("registration subscription frames require exact target/token lengths", () => {
  const payload = new Uint8Array(48);
  payload.fill(7);
  assert.deepEqual(unhex(hex(payload.subarray(32)), 16), payload.subarray(32));
  assert.throws(() => unhex(hex(payload.subarray(32))));
  for (const [tag, type] of [[16, "target-watch"], [17, "target-unwatch"]] as const) {
    const decoded = clientMessage(frame(tag, payload).buffer);
    assert.equal(decoded.type, type);
    if (decoded.type !== type) throw new Error("wrong subscription type");
    assert.deepEqual(decoded.token, payload.subarray(32));
    for (const length of [0, 32, 47, 49]) assert.throws(() => clientMessage(frame(tag, new Uint8Array(length)).buffer));
  }
  assert.throws(() => clientMessage(frame(18, payload).buffer));
});
