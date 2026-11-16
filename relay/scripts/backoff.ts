import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer, connect, type Socket, type Server } from "node:net";
import { once } from "node:events";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";
import { createRuntime } from "../test/runtime.ts";

// Fail upgrades until the native client's idle backoff has grown. Then allow
// real handshakes and check that traffic/network signals wake the retry.
const runtime = await createRuntime(true);
const directory = await mkdtemp(join(tmpdir(), "linn-backoff-"));
const sockets = new Set<Socket>();
const servers: Server[] = [];
const failures = new Map<string, number[]>();
const recoveries = new Map<string, number>();
let command: ReturnType<typeof spawn> | undefined;
let markerFailure: unknown;
try {
  const backend = await runtime.ready;
  const addresses: Record<string, string> = {};
  for (const mode of ["traffic", "network", "cancel"]) {
    const attempts: number[] = [];
    failures.set(mode, attempts);
    const rejectCount = mode === "cancel" ? 2 : 5;
    const server = createServer(client => {
      sockets.add(client);
      client.on("close", () => sockets.delete(client));
      client.on("error", () => client.destroy());
      let header = Buffer.alloc(0);
      const inspect = (chunk: Buffer) => {
        header = Buffer.concat([header, chunk]);
        if (!header.includes("\r\n\r\n")) return;
        client.off("data", inspect);
        const upgrade = /^GET \/relay\s/.test(header.toString("ascii"));
        if (upgrade && (attempts.length < rejectCount || mode === "cancel")) {
          attempts.push(performance.now());
          client.end("HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
          if (attempts.length === rejectCount) {
            void writeFile(join(directory, `${mode}-ready`), "").catch(error => { markerFailure = error; });
          }
          return;
        }
        if (upgrade && !recoveries.has(mode)) recoveries.set(mode, performance.now());
        const upstream = connect({ host: backend.hostname, port: Number(backend.port) });
        sockets.add(upstream);
        upstream.on("close", () => { sockets.delete(upstream); client.destroy(); });
        upstream.on("error", () => { client.destroy(); upstream.destroy(); });
        client.on("close", () => upstream.destroy());
        upstream.write(header);
        client.pipe(upstream);
        upstream.pipe(client);
      };
      client.on("data", inspect);
    });
    servers.push(server);
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    addresses[mode] = `http://127.0.0.1:${address.port}/`;
  }
  command = spawn("cargo", ["test", "--test", "relay_interop", "native_iroh_relay_only_roundtrip", "--", "--ignored", "--nocapture"], {
    cwd: new URL("../../", import.meta.url),
    env: { ...process.env, LINN_RELAY_TEST_URL: addresses.traffic!, LINN_RELAY_TEST_BACKOFF: "1",
      LINN_RELAY_BACKOFF_BACKEND: backend.href, LINN_RELAY_BACKOFF_BARRIER: directory,
      LINN_RELAY_BACKOFF_TRAFFIC: addresses.traffic!, LINN_RELAY_BACKOFF_NETWORK: addresses.network!,
      LINN_RELAY_BACKOFF_CANCEL: addresses.cancel! },
    stdio: "inherit", timeout: 180_000,
  });
  const result = await new Promise<number | null>((resolve, reject) => {
    command!.once("error", reject); command!.once("exit", resolve);
  });
  assert.equal(markerFailure, undefined);
  assert.equal(result, 0, "native backoff recovery must succeed");
  for (const mode of ["traffic", "network"]) {
    const attempts = failures.get(mode)!;
    assert.equal(attempts.length, 5, "quiet failures must not reset the backoff");
    assert.ok(attempts[4]! - attempts[3]! >= 7_900, "idle backoff must grow before the wakeup");
    const recovered = recoveries.get(mode);
    assert.ok(recovered !== undefined);
    assert.ok(recovered - attempts[4]! >= 900, "demand must respect the minimum retry pause");
    assert.ok(recovered - attempts[4]! < 4_000, "new work must not wait for the long backoff");
  }
  assert.equal(failures.get("cancel")!.length, 2);
  console.log("idle retry growth, direct local work, outbound/network wakeups and cancellation passed");
} finally {
  if (command && command.exitCode === null) command.kill("SIGTERM");
  for (const socket of sockets) socket.destroy();
  for (const server of servers) {
    if (server.listening) await new Promise<void>(resolve => server.close(() => resolve()));
  }
  await runtime.dispose();
  await rm(directory, { recursive: true, force: true });
}
