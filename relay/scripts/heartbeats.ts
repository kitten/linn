import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer, connect, type Socket } from "node:net";
import { once } from "node:events";
import { performance } from "node:perf_hooks";
import { createRuntime } from "../test/runtime.ts";

// Forward real workerd Pongs: delay the second, discard the third. Never invent
// replies or inspect/log authentication payloads.
const runtime = await createRuntime(true);
const sockets = new Set<Socket>();
const timers = new Set<ReturnType<typeof setTimeout>>();
const pongTimes: number[] = [];
const server = createServer();
let command: ReturnType<typeof spawn> | undefined;
try {
  const address = await runtime.ready;
  server.on("connection", client => {
    const upstream = connect({ host: address.hostname, port: Number(address.port) });
    for (const socket of [client, upstream]) {
      sockets.add(socket);
      socket.on("close", () => sockets.delete(socket));
      socket.on("error", () => { client.destroy(); upstream.destroy(); });
    }
    client.on("close", () => upstream.destroy());
    upstream.on("close", () => client.destroy());
    client.pipe(upstream);
    let pending = Buffer.alloc(0);
    let mode: "headers" | "http" | "websocket" = "headers";
    upstream.on("data", chunk => {
      if (mode === "http") { client.write(chunk); return; }
      pending = Buffer.concat([pending, chunk]);
      if (mode === "headers") {
        const end = pending.indexOf("\r\n\r\n");
        if (end < 0) return;
        const header = pending.subarray(0, end + 4);
        mode = /^HTTP\/1\.[01] 101 /.test(header.toString("ascii")) ? "websocket" : "http";
        client.write(header);
        pending = pending.subarray(end + 4);
        if (mode === "http") { client.write(pending); pending = Buffer.alloc(0); return; }
      }
      while (pending.length >= 2) {
        const opcode = pending[0]! & 15;
        let length = pending[1]! & 127;
        let headerLength = 2;
        if (length === 126) {
          if (pending.length < 4) return;
          length = pending.readUInt16BE(2); headerLength = 4;
        } else if (length === 127) {
          if (pending.length < 10) return;
          const wide = pending.readBigUInt64BE(2);
          assert.ok(wide <= 1_048_576n, "test frames must remain bounded");
          length = Number(wide); headerLength = 10;
        }
        assert.equal(pending[1]! & 128, 0, "server frames must be unmasked");
        if (pending.length < headerLength + length) return;
        const frame = Buffer.from(pending.subarray(0, headerLength + length));
        pending = pending.subarray(headerLength + length);
        if (opcode === 10) {
          pongTimes.push(performance.now());
          if (pongTimes.length === 3) continue;
          if (pongTimes.length === 2) {
            const timer = setTimeout(() => {
              timers.delete(timer);
              if (!client.destroyed) client.write(frame);
            }, 2_000);
            timers.add(timer);
            continue;
          }
        }
        client.write(frame);
      }
    });
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const proxy = server.address();
  assert.ok(proxy && typeof proxy !== "string");
  command = spawn("cargo", ["test", "--test", "relay_interop", "native_iroh_relay_only_roundtrip", "--", "--ignored", "--nocapture"], {
    cwd: new URL("../../", import.meta.url),
    env: { ...process.env, LINN_RELAY_TEST_URL: `http://127.0.0.1:${proxy.port}/`, LINN_RELAY_TEST_HEARTBEATS: "1" },
    stdio: "inherit", timeout: 150_000,
  });
  const result = await new Promise<number | null>((resolve, reject) => {
    command!.once("error", reject);
    command!.once("exit", resolve);
  });
  assert.equal(result, 0, "native client must tolerate delay and recover from a missing Pong");
  assert.ok(pongTimes.length >= 4, "reconnected client must receive a real Pong");
  assert.ok(pongTimes[1]! - pongTimes[0]! >= 29_000, "idle checks must use the longer interval");
  assert.ok(pongTimes[2]! - pongTimes[1]! >= 31_000, "received Pong must postpone the next check");
  assert.ok(pongTimes[3]! - pongTimes[2]! >= 14_000, "missing Pong must receive the periodic grace period");
  const state = await (await runtime.dispatchFetch("https://local/_inspect")).json() as { probeTraffic: { https: number; captivePortal: number } };
  assert.deepEqual(state.probeTraffic, { https: 0, captivePortal: 0 });
  console.log("periodic control heartbeat delay, timeout and reconnect checks passed");
} finally {
  if (command && command.exitCode === null) command.kill("SIGTERM");
  for (const timer of timers) clearTimeout(timer);
  for (const socket of sockets) socket.destroy();
  if (server.listening) await new Promise<void>(resolve => server.close(() => resolve()));
  await runtime.dispose();
}
