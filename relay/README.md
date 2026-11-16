# Linn relay

This package implements device-signed mesh membership and the Iroh 1.4.1
WebSocket relay in [LLP 0008](../llp/0008-mesh-synchronization.explainer.md).
The native daemon uses it for pairing, membership updates and private relayed
peer connections. Explicit `linn sync` jobs exchange originals between peers;
the Worker stores membership metadata and never holds a store replica.
The native protocol and account-isolation contracts are specified in the LLP.

Only current members of the same mesh may exchange datagrams. Relay tests use
ordinary device-signed enrollment too; there is no test-only authorization mode.

## Development

Run from the repository root through its flake (Node 24, Rust and build assets):

```sh
nix develop -c npm --prefix relay ci
nix develop -c npm --prefix relay run typecheck
nix develop -c npm --prefix relay run build
nix develop -c npm --prefix relay test
nix develop -c npm --prefix relay run test:interop
nix develop -c npm --prefix relay run test:control
nix develop -c npm --prefix relay run test:offline
```

The Worker tests execute in workerd, including challenge verification, rejection,
source-bound datagram routing, duplicate identity isolation and a hibernation/
wake cycle. The interoperability runner starts an isolated local Worker and uses
the pinned native Iroh client with the control-heartbeat patch, with all IP
transports and public discovery removed. It exercises 2 and 10 relay connections, idle heartbeats, and
an 8 MiB bidirectional transfer, then closes endpoints and the runtime. It is a
development test; the packaged Linn CLI never requires Node or Wrangler.

The control runner checks native/Worker membership interoperability and detached
CLI pairing, explicit record exchange and three-store merging. Each daemon has
its own private store and HOME/XDG roots. The offline runner pairs two daemons,
stops its local Worker, holds both former IPv4 ports and restarts the daemons.
It checks fresh loopback discovery, explicit replication, immutable correction
history, permanent deletion and refusal of first-time offline pairing. These
are independent-process tests under one UID; actual Linux/macOS cross-account
tests are deferred and remain distinct security checks.

`npm test` requires the dry-run bundle from `npm run build`. Never commit `dist`,
`.wrangler`, `.dev.vars`, credentials or node_modules. Dependency versions
and integrity hashes are committed in package-lock.json; the flake pins Node.

## Deployment and real-domain gate

The Worker uses one SQLite-backed `RelayCoordinator` Durable Object binding and
the `v1` migration tag. It routes all groups through one named object. Choose a
domain under the operator's Cloudflare account; it is a deployment argument,
selectable with `linn sync:pair --service <url>`. The native default is
`https://linn.kitten.sh`. Authenticate Wrangler headlessly with an API token
supplied privately, or an existing operator-managed OAuth login. Do not commit
an account ID, secret or domain-specific configuration.

Deploy using:

```sh
nix develop -c npm --prefix relay run deploy -- --domain "$relay_domain"
```

Mesh creation requires no deployment secret. Users create independent meshes
with signed genesis proposals; existing members authorize subsequent membership
changes, and invitation redeemers need only their invitation.

For the relay staging gate:

```sh
LINN_RELAY_TEST_URL="https://$relay_domain" \
  nix develop -c cargo test --test relay_interop -- --ignored --nocapture
```

The test creates fresh signed meshes using the normal genesis, invitation and
redemption APIs; it does not replace any existing enrollment. Endpoint private
keys remain in the test process. Normal test/CI runs ignore this network gate.
A local pass does not establish public TLS/Cloudflare interoperability.

Native membership and CLI tests use `mesh_control` and `mesh_pairing` instead
of `relay_interop`; skip the offline test against a real domain, since its runner
must own and stop the service.

`GET /health` returns the relay protocol marker; `/ping` is the Iroh latency
probe; both are answered by the Worker without waking the coordinator.
The Worker rejects invalid routes, methods, relay protocols and declared oversized
bodies before forwarding. Per-IP rate-limit bindings allow 1,200 valid HTTP
requests and 120 relay connection attempts per minute, independently. Limits
are per Cloudflare location; HTTP 429 includes a 60-second Retry-After.
Reserve binding namespaces 7349101/7349102 for this deployment's HTTP/connection
limits. The local runtime uses the same bindings. Do not trust client forwarding
headers as the IP identity.

At most 16 of the 128 relay sockets can await authentication, for ten seconds.
Authenticated sockets allow 8,192 messages/s (16,384 burst), with a separate
32/s client-ping/unsolicited-pong budget (64 burst), in addition to the byte/queue
limits in the LLP. Matching replies to server checkpoints use only the general
message budget.
Exceeding a message budget disconnects that socket. These guards bound transient
work; they do not cap durable membership history or guarantee a billing ceiling.

Logs/observability are disabled and packet/authentication bodies must not
be logged. Relay payloads are end-to-end encrypted between native endpoints.

## Limits and lifecycle

Control requests admit four concurrent operations, bound bodies at 16 KiB and
expire incomplete reads after ten seconds. Cache upgrades checkpoint each entry
and yield between bounded scheduling slices; retry a 503 upgrade response.
Durable signed history is intentionally uncapped. Current authorization caches
and historical replay can still outgrow available resources; chunked storage
bounds rows, not the aggregate working set.

The relay permits 128 sockets, a 256 KiB unconfirmed send window per
receiver and a 4 MiB/s per-sender datagram budget. A matching random protocol Pong
confirms delivery of all earlier frames on the ordered WebSocket, allowing its
window to advance. Excess UDP datagrams are dropped for QUIC loss recovery;
relay queues do not grow without bound and rate limiting does not reset a valid
QUIC session. Protocol heartbeat timeout disconnects unavailable endpoints.
These transport budgets change no retrieval arithmetic or stored record bytes.

Socket attachments retain authenticated identity, challenge phase, window and
rate state across hibernation. Native packet acknowledgements are unrelated to
the daemon's original-byte delivery leases. Membership enforcement is always active.

Rollback code must support the stored cache/index version. Preserve the receipt
key, accepted membership and invitation consumption; an older code version that
expects aggregate snapshots cannot read the chunked cache format. Reverting
service data behind accepted native heads or consumed invitations is not a
supported rollback. Recovery from incomplete accepted history requires a fresh
trusted deployment and fresh mesh enrollment, retaining native records and
conflict evidence. The service has no backup of a Linn store.
