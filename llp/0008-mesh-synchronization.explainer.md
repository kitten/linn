# LLP 0008: Mesh synchronization

**Type:** Explainer\
**Status:** Active\
**Systems:** Sync, Runtime, Store, CLI, Relay\
**Related:** LLP 0000, LLP 0001, LLP 0003, LLP 0004, LLP 0007

## Intent and delivery boundary

Pair with any current member using `linn sync:pair`, then run `linn sync` to
exchange original records and permanent deletions through available members.
The [background runtime](0007-background-runtime.guide.md) owns each account's
canonical store, serves authenticated requests and performs serialized store
steps outside network waits. Writes, startup, timers and reconnects do not
initiate record synchronization.

The native product is one Rust binary. One separately deployable Cloudflare
Worker with a SQLite-backed Durable Object provides ordered membership control
and the private Iroh WebSocket relay. Original records remain in local libSQL
stores and are encrypted between native peers. Membership metadata is visible
to the service; possession of its receipt key cannot invent device approval.

This LLP owns mesh identity, pairing, replication, publication and file-bundle
contracts. SSH synchronization and remote-management commands are absent.
Migration 7 removes remote configuration; migration 8 compacts derived
membership caches and indexes root history. Migration 9 removes the unused hint
table. Migration 10 adds durable incoming-item staging for atomic publication.
Original records, history and tombstones survive these migrations. Local
RPC compatibility is 6, mesh job/ALPN profile is 1, and signed membership
event/receipt profile is 1.

Address discovery consists of IPv4 LAN/loopback mDNS and Iroh's connection
mechanisms through the configured relay. Linn has no cloud address-hint API,
peer-label facility, cross-service federation, automatic record scheduler or
cross-job connection coalescing. Direct remote paths are opportunistic; pairing
and relay-only connectivity depend on the configured service. Already-paired
same-host endpoints can rediscover each other with the service unavailable.

## Accepted product decisions

- Record synchronization is manual: `linn sync` initiates a finite mesh
  job. Local writes, reconnects and timers do not initiate record exchange.
- All members have equal authority to invite, merge meshes and revoke members.
- A mesh replicates the whole selected store: immutable history, close envelopes
  and all permanent tombstones, not merely currently visible search results.
- Creating an invitation delegates joining/merging authority. Redemption alone
  authorizes the merge; no second approval on the issuing machine is required.
- Membership changes require device signatures. Cloudflare distributes and
  persists them but cannot authorize an invented member.
- Membership changes form one ordered sequence per connected mesh. Devices sign
  changes against its current head; the Worker accepts them using atomic
  compare-and-swap. Pairing, merging and revocation require the service.
- Known peers continue syncing with last verified membership during service
  outages. Revocation is eventual, not instantaneous across partitions.
- Already-paired same-host accounts must rediscover and sync without Internet
  access. First-time pairing and mesh merging still require the service.
- Pairing existing meshes joins their transitive membership and replication
  domains, including offline members and future admissions. Distinct trust
  groups require separate stores. There are no per-repository mesh filters.

## Peer identity and local state

A peer is an account-owned store instance, not a machine, hostname, user email,
IP address or SSH key. Operator and agent accounts on the same host are
distinct peers, with independently generated keys and explicit pairing.
Different stores under one account are also distinct peers. Existing
canonical-path routing continues to select the local daemon; it is not the
network identity.

Create a persistent Ed25519 endpoint identity when mesh enrollment begins, not
on an ordinary read. Use it with Iroh and domain-separated signatures for mesh
control messages. Keep the secret and mesh control state in the account-private
local libSQL file, with the existing permissions and schema fencing. Rebuild
never changes identity or membership. Original-data export excludes keys,
invitations and local network configuration.

A full database backup contains the identity and must be protected accordingly.
Restoring in place preserves it. A copy intended to run alongside the original
must run `sync:reset-identity` before network activity and enroll as a new
peer. Reset removes the copy's local enrollment and secret, checkpointing WAL
before success under LLP 0001, without revoking the original peer or altering records and tombstones. Duplicate live endpoint
identities produce a visible error rather than endlessly replacing each other's
connections. Key loss is recovered by enrollment with a fresh identity and
revocation of the old one; there is no cloud recovery secret. The reset
operation must suppress automatic network startup before touching the copied
identity; starting peer transport and then resetting is unsafe. Retain verified
conflicting-history evidence through identity reset: a fresh key must not
silently authorize a component already known to have forked.

Persist signed membership events, verified heads, revocations and the service
URL in the selected database. Discovery hints are ephemeral in-memory state;
restarting an owner rebuilds them from discovery. Status identifies peers by
endpoint key. Network state never enters original envelopes or another client
database.

## XDG locations and account context

Mesh support preserves LLP 0004's Unix path precedence: explicit `--store`,
then `LINN_STORE`, then `store` in `$XDG_CONFIG_HOME/linn/config.toml`, then
`$XDG_DATA_HOME/linn/store.db`. Unset or empty configuration/data variables use
`$HOME/.config` and `$HOME/.local/share`. Honor absolute overrides
independently for every invoking account; never substitute a hardcoded home,
shared machine directory or cwd. Resolve the selection in the CLI and pass the
canonical store identity to its owner. A resident daemon must not resolve a
later client's request using its own startup HOME or XDG environment.

Keys, signed membership, revocations and enrollment are durable store data and
remain in the selected libSQL file. They must survive runtime-directory cleanup
and must not live in `$XDG_RUNTIME_DIR`, `$XDG_CACHE_HOME` or an ad hoc
`~/.linn` directory. There is no second network-state file under
`$XDG_STATE_HOME`. Address hints remain in memory. Account configuration stays
in the XDG configuration location; mesh transport creates no persistent log
file or cache directory.

Local IPC sockets use the account's valid `$XDG_RUNTIME_DIR/linn/` as in LLP
0007, including explicit overrides on macOS. The directory must be absolute,
private, account-owned and local. Retain the documented private Linux/macOS
fallbacks when unset/empty; these are replacement runtime directories, not
locations for durable peer keys. Do not manufacture a login session or assume
`/run/user/<uid>` exists for a headless account. Preserve the daemon's
store-adjacent ownership coordination sidecars under LLP 0007; do not relocate
its owner locks on an XDG change and accidentally create two owners. They
contain no peer secret or record copy. An existing owner's authenticated
endpoint remains authoritative even if a later client supplies a different
valid runtime directory.

Validate the XDG paths Linn consumes under its established rules. Linn rejects
relative explicit XDG values instead of silently ignoring them; this existing
strictness is deliberate, not a new mesh policy. Do not chmod or chown existing
XDG base directories. Create missing application-owned subdirectories
privately, and enforce secret-store privacy on the selected file and its own
containing directory. A default store's parent is `$XDG_DATA_HOME/linn`, not
the entire `$XDG_DATA_HOME` tree. Unsafe explicit store locations fail
enrollment with a clear error rather than silently moving the store or changing
another directory's permissions. These path rules describe the supported Unix runtime; they do not claim
Windows mesh support.

## Multiple accounts on one machine

Operator and agent accounts are independent installations, not two clients of a
machine-wide service. Each account resolves its own store/configuration and
starts its own per-store owner under LLP 0007. That owner alone opens its
store, loads its endpoint secret and serves explicitly requested network work.
More stores in either account create more independent peers. Do not derive
network keys from a UID, hostname, shared installation path or machine
identifier; the same username on another host must also get an independent key.

Keep local IPC account-restricted. The current Unix runtime derives its routing
key from effective UID plus canonical store path, places ownership locks under
an account-specific private directory, and checks the connecting effective UID.
Network identity remains the persistent endpoint key, not that routing hash. Do
not relax these checks to sync two local accounts: they connect through the
same authenticated peer protocol as accounts on different hosts. No shared SQL
file, cross-account socket access, privileged broker, SSH login or shared
secret file is required. They may use the same installed binary and service
URL.

Before generating, loading or persisting a mesh secret, validate the store file
and its containing directory as private and owned by the effective account. On
Unix require an account-owned mode-0600 regular store and an account-owned
mode-0700 parent, with the existing canonical-path/hard-link constraints. Cover
database journals/WAL and other secret-bearing
sidecars too. Creation modes alone do not validate pre-existing files: current
Store/Replica opens create private files but do not establish this stronger
enrollment check for every existing store. Fail enrollment/network startup with
an actionable permissions error; do not chmod or chown another account's data.
Revalidate on owner restart and around identity changes. Reject attempts to use
one physical database as two accounts' independent replicas. Machine
administrators who can bypass account permissions are outside this isolation
guarantee. Pairing grants the intended replicated data, not filesystem access.

Running a CLI under another account must use that account's HOME,
configuration, store and runtime directory. An inherited XDG_RUNTIME_DIR owned
by the invoking account fails the existing privacy check; do not weaken it or
reuse that socket directory. Headless accounts may leave it unset and use the
native private fallback. Enrollment needs no desktop login, shared keychain or
administrator access or deployment credentials. Any user may create a new mesh
or join an existing one using an invitation.

Every endpoint binds independent UDP sockets using OS-assigned ports (port 0),
and maintains its own outbound relay connection when the service is reachable.
No fixed host-wide peer-transfer port or shared peer socket is required. Local
discovery's multicast socket reuse is separate from peer-transfer sockets. IPv6
unavailability or inability to form a direct path must not prevent fully
relayed synchronization. Same-host peers are not self-connections: skip only
the local endpoint ID, never matching IP, hostname or account label. Use normal
endpoint authentication for local/LAN paths, with relay fallback. Offline
same-host sync uses explicit loopback discovery below, not NAT hairpinning or a
cross-account IPC shortcut.

The Durable Object indexes sockets, presence, membership, reconnect state and
peer quotas by authenticated endpoint ID. Two accounts behind the same public
IP coexist as distinct endpoint identities. A new connection for endpoint B
must not replace endpoint A's socket because their IP or hostname matches.
Relay admission and datagram budgets use endpoint identity, not a shared public
IP or a one-connection-per-IP limit. Duplicate-identity handling applies only
to the same endpoint key.

Stopping, upgrading, leaving or revoking the agent peer affects that identity,
not the operator peer or every peer on the host. Independent daemons may use
compatible different builds; network version checks happen per connection,
separately from each account's local IPC/schema compatibility. One account's
dispatcher and ownership locks are never shared with another account. A slow
peer can still backpressure an outgoing original-item write and its delivery
lease on the other account, up to the idle-write deadline below; receiving a
slow acknowledgement must not hold that lane. They also share host CPU, memory
and network capacity. Measure concurrent behavior rather than promising
resource isolation or zero cross-peer delay.

## Pairing and mesh membership

An unpaired store initializes a mesh with a signed genesis event and a random
stable root ID. An event contains `version`, `service`, random `id`, sorted
expected `heads` and a typed `action`. A proposal adds a map of endpoint public
keys to domain-separated signatures. Roots are derived from predecessor
history; only genesis declares a new root. The proposal hash is its
canonical-JSON hash. Verify complete proof chains back to locally trusted
genesis/admission anchors before accepting members, using bounded pages rather
than a bound on total retained history.

Each connected mesh has one current head. An ordinary change names that head; a
merge names the current heads of both disjoint components. The Worker checks
the signed changes' authority and expected heads, then atomically appends the
event and replaces the affected heads. There is no deployment-wide sequence
that unrelated meshes must download. After a merge, both histories feed one
ordered successor sequence. All affected heads and invitation consumption are
updated in the same Durable Object transaction. An ordinary stale-head conflict
returns a retryable conflict without applying anything: fetch and verify the
latest history, recheck authority and intent, and sign a fresh proposal. Bound
retries and report persistent contention. Never silently reuse a signature on
different heads or convert a stale merge into a different operation.

Create an invitation with a fresh ephemeral signing key. The issuing member
signs a delegation binding its public key to the service, mesh, issuing
identity, invitation ID and expiry. The shareable token contains the ephemeral
secret and the trust anchors needed to verify the invitation. Never send that
secret to the Worker, log it, put it in an HTTP query, or persist it after
completion. The service receives only the public delegation. A leaked
invitation grants access: the invitation command must say that it can merge
entire stores and their meshes. Display known membership and the offline-member
implication.

Invitations expire after ten minutes and have one atomic redemption slot. The
redeemer signs a transcript binding the delegation to its own endpoint, its
current mesh roots and heads, using both its endpoint key and the delegated
ephemeral key. Presenting the token through the explicit join command is
consent to merging; there is no subsequent confirmation prompt. The issuer may
be offline after creating the invitation. Expired, cancelled, replayed or
malformed invitations fail before original-data exchange. Single-use
enforcement relies on the coordinator's atomic durable slot: signatures cannot
prevent a malicious coordinator from accepting several otherwise valid
redemptions from a token holder. It still cannot generate those signatures or
redeem a token it never received. A lost success reply is retried with the same
signed redemption ID, returning the same result, not admitting a second
endpoint. Invitation-consumption state is durable. Restoring an older service
snapshot is unsupported: it can reopen consumed invitations and conflict with
heads already accepted by native peers.

Admission into one mesh adds the endpoint. Redemption by an existing member of
another mesh produces a merge authorized by the issuing delegation and
redeeming member. Preserve both sets of roots and signed history; do not
rewrite original data or require every offline member to sign. Accepted merge
history is retained and repeated redemption is idempotent. At acceptance the
issuing member must still be authorized at the issuer component's current head;
a delegation does not survive its issuer's revocation. The issuer need not be
online: the redeemer fetches both current histories and signs the head-bound
proposal with its own and the delegated ephemeral key. A stale-head retry does
not consume an unused invitation. After success, retries of that exact signed
redemption return its original accepted result even if later membership changes
have advanced the head; this does not restore subsequently revoked authority.
The Worker uses the lowest root ID as its component routing/group identifier,
not as a replacement trust anchor. Native status uses endpoint keys. New
members learn the entire verified connected component. Joining through another
member does not require contact with every existing peer.

One currently authorized member's signed invitation is the mesh's approval.
Once the service accepts redemption and the joining store durably verifies and
records that acceptance, pairing is successful and the endpoint is admitted. No
quorum, acknowledgement, membership fan-out or record synchronization is
required. Every existing member, including the inviter, may be offline during
redemption. Existing members learn the accepted admission when they next
refresh membership or verify a peer's proof; this propagation does not hold
admission open. Local transport startup runs asynchronously after admission. A
bind, discovery or relay-connectivity failure is reported as transport status
and must not fail or roll back pairing. Record exchange remains a separate
explicit `linn sync` job; neither successful synchronization nor reaching the
other mesh members is a pairing prerequisite. The service is still required for
ordered membership acceptance, with the existing expiry and revocation checks.
The Worker likewise returns the durable acceptance without waiting for relay
socket maintenance. Refreshing socket attachments and closing revoked sockets
runs in the background; relay authentication and packet routing still check
current durable membership, so this does not extend a revoked peer's authority.
After a reset and successful re-enrollment, an immediately requested sync must
consume the pending transport-resume intent even if its request reaches the
driver before the background wake-up. Reset/leave/shutdown discard older resume
intent. This ordering must not reintroduce a transport wait into pairing.

Each store uses one service origin. Meshes on the same deployment may merge;
cross-deployment federation is rejected explicitly. New meshes default to
`https://linn.kitten.sh`; `--service <url>` selects a custom deployment during
initialization. An enrolled store retains its service, and invitations carry
the service origin for joining devices. The Worker cannot replace the
invitation's anchors or the endpoint authenticated by Iroh.

## Revocation and partition semantics

Any current member may publish a signed revocation. Keep revocation events
permanently and union them across merged meshes. On receipt, cancel affected
jobs and deny revoked endpoints before each replica request and original-byte
chunk. The Worker closes relay sockets for revoked endpoints. A revoked
endpoint ID cannot be re-added by an old admission or invitation. Rejoining
requires a fresh key and a fresh invitation. Removing a peer does not delete
its copies.

Validate authority by replaying the verified signed sequence, not wall-clock
timestamps, delivery order or an unsigned revision number supplied by
Cloudflare. A revocation accepted first prevents later changes by that
identity, including redemptions of its outstanding invitations. Historical
admissions and merges accepted before their sponsor's revocation remain valid:
revoking a sponsor does not implicitly revoke everyone it previously
introduced. Revoke unwanted members explicitly. When two members attempt to
revoke each other, the first accepted change wins; the other proposal becomes
stale and its actor is no longer authorized to retry. There is no
concurrent-event quarantine or endorsement operation in this model.

Revoked endpoint IDs remain permanently denied across merges. Union permanent
revocations before deriving merged membership; an old admission, imported
history or outstanding invitation cannot re-add a revoked key. A redeemer or
issuer denied by either input component cannot authorize that merge. Ordering
cannot retract bytes exchanged before a revocation became known. Retain
original records under the ordinary immutable-store rules. A newly delivered
revocation never undoes a permanent data tombstone.

Peers exchange control events before original data. Remember verified history
and reject rollback below local knowledge; an untrusted service may suppress
updates or deny connectivity but cannot forge authorization. A disconnected
peer cannot know a newly issued revocation: existing verified peers may keep
syncing until they receive it. No claim of immediate revocation, global
freshness or fork-free delivery from a malicious service is made.
Issuing/cancelling invitations, redeeming, merging and revoking require the
service; ordinary known-peer exchange does not. An offline attempt to revoke
reports unavailability and does not claim a globally accepted revocation. Local
leave can disable this store's networking immediately, but cannot revoke it for
other peers offline.

An incompatible signed successor history is a security conflict, distinct from
an ordinary stale proposal. Persist the conflicting evidence and suspend that
component's network authorization and record exchange; never choose a branch by
timestamp, length or Worker assertion. Status identifies the conflict without
exposing secrets. There is no in-place fork repair command: recovery requires
explicitly establishing a fresh mesh with fresh endpoint identities and new
invitations through a trusted deployment. Keep original records/tombstones and
the old evidence for deliberate recovery; do not automatically reset identity
or reinterpret old invitations. Cached membership permits outage operation only
while no incompatible history is known.

## Worker and Durable Object architecture

One Worker entry point and one SQLite-backed Durable Object class in `relay/`
serve an operator-controlled domain. Each deployment uses one named coordinator
object, `private-mesh-v1`. All endpoint sockets and membership components
therefore share one routing authority; merging does not need to move sockets
between objects or build cross-object packet forwarding. This is one Worker
deployment, not one Worker for each peer or mesh.

The control API is `GET /v1/service`, `POST /v1/events` and `POST /v1/history`.
The relay upgrade is `/relay`; `/ping` is a latency probe and `/health` is a
minimal health response. Health probes require no version header. Control API
compatibility is carried by the `/v1/` paths and versioned payloads; no separate
HTTP version header is required. Relay upgrades must offer
`Sec-WebSocket-Protocol: iroh-relay-v2`; reject missing or unsupported offers
before opening a socket and echo the selected subprotocol on success.
Genesis creation is open to users without deployment authorization. The
creating device must sign its genesis proposal and bind the service receipt
key. Deployment credentials and bootstrap files are not part of mesh creation.
Existing members and invitation delegations authorize subsequent changes.
Relay registration requires current mesh membership. Malformed
UTF-8/JSON, key encodings and origins are bad requests. Unexpected storage or
signing failures return HTTP 503 without internal details; callers retain
uncertain proposals and may use verified cached membership for service
unavailability. A service failure is never reported as definite rejection.

Authenticate relay endpoints with Iroh's challenge/signature protocol. Use the
challenge path rather than requiring TLS exporter access behind Cloudflare.
Endpoint authentication proves key ownership; separately check verified mesh
membership. Read both endpoints' current membership in one storage snapshot for
forwarding, before looking up destination presence, and do not forward through
sockets closed during that read. Cross-mesh packets receive the same denial
whether the destination is connected or offline. Recheck
the handshake deadline after asynchronous authentication work, and recheck signed
history-read expiry when its storage transaction starts. Membership refresh
must preserve the latest attachment's counters and closure state across awaits.
Forward packets only within the same authorized component and bind
the source field to the authenticated socket, never a caller-supplied identity.
Unregistered WebSockets receive a challenge but cannot forward datagrams before
endpoint authentication and membership authorization succeed.

The relay implements the pinned wire protocol: WebSocket subprotocol
negotiation, binary datagrams and batches, ping/pong, endpoint-gone/status
messages, frame validation and duplicate connection behavior. The TypeScript
adapter interoperates with the pinned stock Rust Iroh client; Iroh's transport
and cryptographic implementations remain upstream dependencies.

Use accepted hibernatable sockets and per-socket attachments for authenticated
identity and control state. Reconstruct routing after wake; in-memory maps are
never the only membership authority. Protocol-level keepalives may wake the
object even when WebSocket-level automatic responses are possible. Measure idle
cost with the real client rather than assuming hibernation makes it free.

Signed membership and public invitation-consumption state live in Durable
Object storage. Live authenticated WebSockets supply ephemeral presence. Relay
queues are bounded and transient; do not store original records or packet
payloads. End-to-end encryption terminates at Linn peers. The service can
observe identities, membership, IPs, timing and traffic volume. Logs exclude
invitations, credentials, record bytes and ciphertext payloads.

## Connectivity and propagation

The native endpoint uses the enrolled deployment as its custom relay, disables
public relay/address-lookup defaults and installs a private in-memory lookup
populated by mDNS. One endpoint belongs to each enrolled store. Relay
connection failure does not prevent loopback discovery or direct transfers.

Iroh uses known direct candidates and relayed connectivity and may establish
direct UDP paths when available. The Worker provides no UDP address-discovery
service or signed address-hint publication. Remote NAT traversal is
opportunistic; there is no promise of direct connectivity across every NAT. The
service remains an availability dependency for pairing and relay-only peers.
Cached membership alone does not locate a remote endpoint after every possible
address change.

Use a versioned Linn mesh ALPN and bounded authenticated control stream,
separate from local IPC. Membership exchange precedes data authorization. Adapt
the original-item protocol and merge functions in [sync](../src/sync/mod.rs),
[framing](../src/sync/protocol.rs) and [replica
storage](../src/storage/sync.rs) to asynchronous streams; preserve original
bytes, hashes, tombstone precedence, conflicts and publication recovery
specified in LLP 0001 and the retained contracts below. Network protocol
compatibility is independent of the local application/schema compatibility
number. Use the typed [mesh store queue](../src/mesh/store.rs) and serialized
application steps for inventory/get/needs/apply/finish. Network workers never
hold the store lane while waiting for peer acknowledgements. Original-byte
chunks retain only their bounded delivery lease.

Connections belong to individual job branches and close when those branches are
dropped. At most 32 inbound and outbound connections combined are admitted per
store; excess dials fail with capacity errors. Inbound connections without active
application streams close after 30 seconds. Eight handshake slots and two slots
per transfer direction have 30-second acquisition deadlines. Membership handshakes
have a five-minute total deadline, and an accepted connection must begin its job
within 30 seconds of authorization. Independent jobs can contact the same peer; there is no elected dial
owner or cross-job connection cache. The frozen cohort determines traversal
order. Establishing a connection does not itself exchange records.

### Manual mesh jobs

`linn sync` captures the verified membership head and starts a finite job with
an unpredictable run ID. Only an explicit local command initiates a root job. A
peer may perform delegated steps of that job, but receiving records, local
writes, startup, timers and reconnects never start another root job.

Collect reachable peers' originals/tombstones toward the initiator, then
redistribute the merged state through the reached peers. An intermediate peer
may contact members the initiator cannot reach, using authenticated bounded job
requests, not recursive CLI execution. Deduplicate by run ID and endpoint ID in
each phase; use a first-accepted parent to form the job's traversal tree.
Return cached/in-progress status on repeat visits instead of waiting on a
descendant that is waiting on this peer. Collection completes or reports a
failed branch before distribution begins. Returning data over the tree
preserves the existing idempotent original-item merge and publication rules. A
single pre-sync pass at each remote is insufficient because it would not
distribute the initiator's new records to descendants.

Freeze the job's membership cohort, visit each reached peer at most once per
phase and report unreachable/failed branches. Membership changes and concurrent
writes can require another explicit run; a sync must not chase a moving mesh or
wait forever for quiescence. Keep at most four active jobs per store. Each job
collects children sequentially in sorted cohort order and retains successful
child controls for distribution. The daemon admits at most four CLI mesh
workers, plus the reserved local-control slot described below, and rejects excess
commands; ordinary local commands use the separate
serialized store lane. Failures in one branch do not cancel durable progress
elsewhere. The completion result distinguishes collection failures,
distribution failures and peers not reached; it is not a claim that every mesh
member has converged.

Retry transient connections only within the active job, with bounded attempts
and backoff. No periodic repair round or post-command reconnect-to-sync loop is
part of this delivery. Cancellation/initiator loss stops scheduling delegated
work, completes or recovers accepted publication, releases transport leases and
retires the run state. Bound active run bookkeeping and its retention; run IDs
are not permanent store records or authoritative acknowledgement cursors.

Keep transport readiness separate from record exchange. An enrolled resident
owner may advertise bounded presence, maintain its relay connection, discover
local peers and accept an authenticated explicit sync job while its CLI is
idle. No record bytes move solely because another endpoint appeared. Transport
retries must not hold the dispatcher, and remote jobs must remain subject to
local admission, cancellation and membership checks. This passive serving is
required for a single user's command to reach other accounts without a second
CLI there. It is not automatic synchronization triggered by those accounts'
writes.

The run's received count tracks durably accepted items immediately. Phase
snapshots preserve that counter even if a later item or branch fails; a fresh
retry counts only newly applied items and cannot double-count existing ones.

Replica streams are read-only sources bound to the authenticated peer and the
active run phase. During collection the parent pulls from its collected child;
during distribution the child pulls from its distributing parent. Check current
membership on every source request and every original-byte chunk, and close
source streams when their job becomes terminal. Use two incoming and two
outgoing transfer slots per store. Publish after 256 changed items or 4 MiB of
changed envelopes, and at successful or failed phase completion. Control-stream
reading and heartbeat writing continue independently of original-item transfers
and delegated work. A control wait expires after five minutes without application
progress; repeated control messages do not count. Heartbeats carry monotonic progress
revisions from actual replica bytes and cohort traversal. Child progress can
advance its parent's revision; parent progress only refreshes a child's wait,
preventing heartbeat feedback cycles. A progressing job has no fixed total-duration
deadline. Each replica source captures the initial record and tombstone counts
and limits its emitted inventories to those per-class budgets, so concurrent
writes cannot extend that stream forever. This is a finite scan, not a point-in-time
snapshot; a later explicit sync reconciles concurrent writes.

Reserve one additional resident command slot for local status, leave or reset
when the ordinary four mesh command workers are occupied. This preserves a way
to inspect or shut down transport while record jobs are busy.

Persistent membership and a running daemon enable passive serving only. There
are no dirty-generation, periodic-repair or automatic root-job timers.

## Offline same-host discovery

Already-paired accounts on the same host must rediscover and synchronize while
the Internet, DNS and Worker are unavailable, including after both daemons
restart and their OS-assigned ports change. Loopback networking must remain
available and permitted by host policy; network namespaces that cannot reach
each other's loopback are not the same local connectivity domain. First-time
pairing, invitation redemption and mesh merging continue to require the
service. There is no offline auto-enrollment based on sharing a machine.

Use `mdns-sd` 0.21.7 with default features disabled and only its async event
feature enabled. [The adapter](../src/mesh/discovery.rs) publishes and browses
`_linn._udp.local.` on standard mDNS port 5353, without an external daemon. The
instance label is the endpoint's 52-character z-base-32 public key, the
hostname is that key followed by `.local.`, and TXT `v=1` identifies this
profile. Advertise only the actual UDP transfer port and IPv4 addresses. mDNS
discovery supports IPv4 LAN and loopback addresses. It does not advertise IPv6
candidates; Iroh's IPv6 and relay transports remain available independently.

For a wildcard-bound Iroh IPv4 socket, publish automatically updated addresses
on IPv4 interfaces, including loopback. For a socket bound to `127.0.0.1`,
restrict the service and daemon to IPv4 loopback. Let the library scope address
records to their interfaces; additionally reject a loopback hint unless its
receive-interface metadata identifies a local loopback interface. Never use a
remote host's `127.0.0.1` as our own. Interface changes and DNS record expiry/
goodbyes update hints; removed and locally revoked endpoints lose their hints.

Only approved, non-self endpoint IDs with profile version 1 and nonzero ports
enter Linn's address lookup, with at most 32 IPv4 addresses per approved
endpoint. Prioritize loopback within that cap. Re-browse cached records when
membership changes so a newly approved peer need not reannounce. The library's
internal DNS cache also contains untrusted records: sample its record counters
once per second and stop discovery with a visible error when their sum exceeds
4096. This is a sampled shutdown budget, not a strict allocator/RSS cap; do not
claim that unknown senders allocate no library state. Event channels are
bounded by the pinned library. Discovery failure must leave relay transport
usable.

Monitor asynchronous daemon errors and identity-name conflicts. Report
interface errors in transport status, and stop discovery on a local name
conflict rather than silently treating a renamed identity as a new peer.
Pairing never waits for socket setup, discovery results or record exchange.
Stop the discovery task, request daemon shutdown and clear hints before closing
the Iroh endpoint. The discovery wire format is DNS-SD/mDNS.

Other local users and LAN participants may observe the public endpoint/address
records. Never publish mesh membership, labels, account names, store paths,
credentials or originals. Keep loopback hints local: never publish them to
Cloudflare or forward them as usable addresses on another host. No shared
filesystem discovery directory, XDG permission relaxation, privileged helper or
desktop mDNS service is required. Process-isolation tests do not establish
multi-UID socket coexistence or filesystem/IPC isolation.

Treat discovery results as untrusted bounded address hints, not authorization.
Only dial already-approved endpoint IDs and always verify Iroh's authenticated
identity before exchanging the mesh control/data protocol. Spoofed, stale or
reused ports cannot admit another key. Expire stale hints, process
announcements after restart, and keep known membership in the account's durable
store. Exchange signed membership/revocations before records as with any other
path. Reject known revoked identities even while Cloudflare is down;
revocations not yet received remain subject to the accepted eventual-revocation
rule.

Start the endpoint and local lookup without awaiting service login, address
publication, DNS or `endpoint.online()` relay readiness. Relay failures/backoff
must not consume the local discovery/transfer budget or occupy the dispatcher.
Local lookup failure is visible in status, while other transports continue.
Loopback rediscovery is exercised by the independent-daemon offline tests. Host
firewalls and network namespaces can still prevent local connectivity; there is
no cross-account IPC or shared-file fallback.

## Tailscale and other routed overlays

An operator-provided OS-routed overlay is an ordinary UDP connectivity path
when Iroh has a usable endpoint/address candidate. An address prefix or tailnet
membership grants no Linn authority. Endpoints still authenticate with their
keys and verify signed mesh membership on every path.

Linn does not invoke the Tailscale CLI, call its local/control API, link an
SDK, read private overlay state, enumerate a tailnet or modify
routes/firewalls. It does not scan overlay subnets, assume an interface name or
use MagicDNS as an account/store identity. Separate stores on one overlay IP
keep independent keys and transfer ports. Proxy-only userspace networking is
not an OS UDP path.

The Worker does not distribute routed-address hints, and peers do not forward
such hints. Direct overlay connectivity therefore depends on candidates Iroh
already knows or addresses learned by local discovery. The configured Iroh
relay remains the fallback. A path over Tailscale does not prove a physically
direct link because the overlay may itself relay traffic.

## Runtime, failure and command contracts

Keep sockets, timers and Iroh work outside the store dispatcher. Submit each
bounded replica operation through the daemon's internal queue. Never hold a
database transaction or dispatcher slot while waiting for a peer. Preserve the
runtime's delivery barrier for original bytes: a queued network write must not
escape after a forget operation has been acknowledged. Cancel transport work
and complete or retain pending publication using the existing durable rules.
Retain a mesh delivery lease through each authorized chunk write, dropping it
on progress or transport cancellation before obtaining another chunk or waiting
for a peer acknowledgement. The chunk revalidation rules are specified below.
Never put original bytes into a later application send queue after releasing
that lease. Bytes already handed to the transport may remain in flight, as with
existing QUIC transfers; forgetting cannot retract them or erase remote copies.

The dispatcher uses the per-chunk delivery barrier specified below. The
no-network-wait rule above excludes this bounded original-byte delivery lease,
not arbitrary peer response waits. Use 30-second TLS-acceptance and
idle-frame/write-progress deadlines; chunk leases follow the protocol
resolutions below, and stalled transports are cancelled; an ongoing healthy
reconciliation has no whole-operation deadline. Network backpressure can delay
forget within the bounded chunk-write lease.

Reserve fair execution for local commands. A failed peer does not stop
unrelated peers or local ingest/search. A partially completed exchange remains
retry-safe; publish successfully accepted batches before reporting failure.
Classify auth, version, conflict, publication, timeout and offline failures
separately. Automatic retry is for idempotent replica reconciliation, never
arbitrary uncertain local mutations. Stop drains or cancels bounded transport
jobs and repairs publication before releasing store ownership. Verified enabled
enrollment resumes passive transport on daemon restart. Uncertain membership
proposals retry through explicit control operations, not startup. The daemon
creates no OS boot service and does not wake sleeping hardware.

CLI presentation keeps colon-separated command names. Top-level help hides
`sync:*` administration commands, except `sync:pair` when the selected store
has no registered mesh. `sync -h` documents all of them; individual command
help remains available. Determine enrollment through the daemon, never by
opening SQL in the CLI. Missing stores need no daemon lookup. Bound help's
lookup to two seconds; if enrollment cannot be determined, omit `sync:pair`
from top-level help while retaining it in `sync -h`.

Creating or redeeming an invitation requires a notice that both entire stores
and meshes, including offline members, can merge. Prompt with default no before
reading a token or issuing a membership mutation. `-y` / `--yes` bypasses the
prompt; noninteractive callers (including JSON and piped tokens) must pass it.
Cancellation of an unused invitation does not require this prompt. Existing
mesh enrollment is legitimate for invitations and merging; explain that fact.
Suggest `sync:reset-identity` specifically for a copied store intended to run
alongside its original, explaining that records and deletions survive.

Invitation presentation labels invitation ID, known peer IDs and readable UTC
expiry, explains single use and gives the join/paste next step. Label the token
on terminal stdout; redirected stdout remains the raw token. JSON remains one
typed result. Notices and instructions go to stderr. Peer IDs identify
account-owned store instances, not machines.

Sync progress is on stderr, deduplicated when unchanged, and updates a single
line on terminals. Peer collection/distribution counts include this peer and
show cohort denominators. Completion appears once on stdout; JSON retains the
structured report without a second human completion summary. Applied-change
counts describe this store's records and deletions, not mesh-wide transfers.
Zero local changes must not be described as a mesh-wide no-op. Incomplete text
reports name missing peer phases, typed failure categories and recovery actions,
and explain durable partial progress. Membership-operation results explain the
outcome, affected peer/invitation where available and whether a first sync ran.

`sync:status` remains local by default, explicitly marking remote reachability
unknown. Optional `--check` tests transport connectivity to known remote peers,
with at most eight concurrent checks and a ten-second probe deadline. Queued or
unfinished peers remain unknown; a connection failure is reported as unreachable
at check time. This is an authenticated endpoint transport snapshot, not proof
of membership authorization or successful record exchange. No records transfer,
no membership refresh is requested and no mesh job begins. Reachability results
are returned only for checked status, as an optional JSON `reachability` map;
ordinary status JSON is unchanged. The additional local check-status request is
fenced by application compatibility 6; the mesh wire profile is unchanged.

The public synchronization commands are:

- `linn sync:pair` creates an invitation, initializing a genesis mesh if
  unpaired. New meshes use `https://linn.kitten.sh` by default; optional
  `--service <url>` selects a custom deployment. No bootstrap file or deployment
  authorization is required. An enrolled store uses its existing service and
  cannot silently switch services. This grants joining/merging authority, not
  a record transfer.
- `linn sync:pair --join [invitation]` redeems an invitation. An argument value is
  accepted at the user's discretion. With no value, read piped stdin or prompt
  in a terminal. `--join -` explicitly selects stdin. Joining requires `--join`;
  positional invitation input is rejected. Token input is limited to 32 KiB.
- Admission completes before optional first sync. An interactive join offers
  the first sync (default yes); `--sync` starts it explicitly and `--no-sync`
  skips it. Noninteractive and JSON joins skip the offer unless `--sync` is
  supplied. Show progress on stderr. A failed first sync leaves membership
  admitted, returns incomplete/failure status and can be retried with
  `linn sync`; it never rolls admission back. JSON joins with `--sync` return
  one typed admission object with `initial_sync` or `initial_sync_error` attached.
  The daemon performs explicitly requested first sync after committing admission,
  releasing its membership-command lock and sending an admission event. Initial
  sync errors retain the structured application error, including category and
  uncertain-publication information. If the connection fails after an admission
  event, the client transport preserves that known admission with a typed initial
  sync error; without that event it reports ordinary uncertain completion.
- `linn sync` initiates the finite manual job above. It exchanges membership and
  records using the selected store's daemon and returns structured per-peer and
  phase outcomes. Offline members remain visible. An unreachable or failed branch
  yields the ordinary command error with durable partial progress reported;
  an empty mesh is a successful no-op that still repairs pending publication.

Keep `sync:status` for inspection, `sync:revoke <peer-id>` and
`sync:leave` for membership changes, `sync:pair --cancel <invitation-id>` for
an unused invitation, and `sync:reset-identity` for an explicitly copied store.
`sync:revoke` requires a full peer ID; use `sync:leave` for this endpoint.
There is no `sync:peers` command or `sync:revoke self` alias. Do
not add separate `sync:init`, `sync:invite` or `sync:join` public commands.
Preserve explicit store selection. Pairing/control operations may exchange
their signed control state but never make original replication an admission
prerequisite. Explicit `--sync` is daemon-owned post-admission orchestration.
The interactive offer stays in the CLI after admission; accepting it submits an
ordinary sync request. Neither path introduces automatic synchronization.
The daemon allocates run IDs and streams typed progress on the request's local
RPC connection; clients render progress without periodic status requests.

Status exposes verified heads and conflicting-history state. Control failures,
including stale proposals after bounded retries, are returned by their command.
Persist local pending-leave intent before transport suspension so
drain/publication failures cannot hide an outstanding remote revocation.
`sync:leave` disables local transport, persists the outstanding intent and
cancels active jobs before attempting service-side self-revocation. If the
service is unavailable, leave still disables local transport and explicitly
reports that remote revocation remains outstanding. Persist that local pending
intent, not a purported accepted event. A later explicit leave retries against
the latest verified head; daemon restart does not silently publish membership
mutations or reenable transport. Removing the last member requires a new mesh,
not cloud key recovery.

`sync:status` reports enrolled membership, local transport readiness, active or
last manual job outcomes, local bound addresses, relay/discovery readiness and
failures without exposing secrets. It must not contact peers or start a sync
merely to inspect state. Do not label one peer's last successful exchange as
proof that all its descendants are current. New commands remain thin adapters
over typed application operations. Resident ownership never means a scheduled
sync exists.

## Deployment and verification

The self-contained `relay/` package contains source, protocol tests, package
metadata/lockfile, `wrangler.jsonc`, third-party notices and a deployment
[README](../relay/README.md). The flake pins development tooling; the native
runtime requires no Node or Wrangler. Domains and Cloudflare credentials are
operator deployment inputs, not native defaults or committed secrets. Rust
builds do not deploy the Worker.

The service uses the `COORDINATOR` binding, the SQLite-backed
`RelayCoordinator` class and the retained `v1` storage migration tag.
Cache/index upgrades follow [derived membership
storage](#derived-membership-storage-and-history); original signed control
history and invitation consumption must survive deployment. Rolling back
accepted membership or invitation consumption is unsupported. Recovery from
incomplete accepted history requires a fresh trusted deployment and fresh mesh
enrollment, retaining native records and conflict evidence. The service has no
copy or backup of native record stores.

Control HTTP work admits at most four concurrent requests. Bodies have a
10-second total read deadline and a 16 KiB bound; cancellation cleanup does not wait for an unresponsive reader.
Capacity and deadline failures leave retries safe. Before calling the coordinator,
the Worker answers health/latency probes, rejects unknown paths, wrong methods,
invalid relay upgrades and declared bodies over 16 KiB. Streaming bodies remain
bounded by the coordinator regardless of Content-Length. Worker rate-limit
bindings allow 1,200 valid HTTP requests and, independently, 120 valid relay
connection attempts per IP per minute. Use Cloudflare's supplied connecting IP,
not caller-supplied forwarding headers; local requests without that header share
one key. Excess returns HTTP 429 with Retry-After; limiter failures return 503.
Counters are approximate and local to each Cloudflare location, not global
quotas. Binding namespaces 7349101 and 7349102 are reserved for these limits.
The application keeps no persistent IP counters and adds no lifetime history quota.

The relay accepts at most 128 WebSockets, of which at most 16 may await
authentication or complete closure. Closing sockets count toward capacity until
the runtime removes them, preventing connection churn from bypassing the bounds.
Pending sockets have a ten-second challenge deadline and only
one in-flight authentication per socket. Admission closes expired pending sockets;
alarms run by the earliest pending deadline without postponing existing alarms.
Each authenticated socket has a token budget of 8,192 incoming messages per
second with a 16,384-message burst. Client pings and unsolicited pongs additionally
share a 32-per-second budget with a 64-message burst. A matching reply to the
server's outstanding random checkpoint is charged only to the general message
budget, so server-requested acknowledgments do not throttle normal transfers. Exceeding a message budget closes
the socket; these budgets survive hibernation and initialize on older
attachments. Each sender has a 4 MiB/s datagram budget, and
each receiver has a 256 KiB unconfirmed send window. Ordered protocol Pong
checkpoints release that window; excess datagrams are dropped for QUIC
recovery. Protocol pings run at 15-second intervals, with a 30-second
outstanding-pong limit. Heartbeats use per-socket deadlines; an earlier pending
handshake alarm must not send premature heartbeats to authenticated sockets.
Schedule the next alarm no later than the earliest heartbeat or pending handshake
deadline. Hibernatable attachments preserve identity, deadlines
and budgets. These limits bound application queues, not all platform buffering
or billing.

Relay tests enroll endpoints through the same device-signed membership protocol
as production. There is no test-only relay allowlist or alternate enrollment path.

Run formatter, strict workspace Clippy/tests, native build and the
LLP-reference checker through the flake. Worker verification includes
typecheck, dry-run build and tests. Native relay interoperability is exercised
with IP transports and public discovery disabled; public-domain testing checks
actual Cloudflare TLS/relay behavior. Unit/local-runtime tests do not establish
operational cost or real-domain behavior by themselves. Do not commit generated
bundles, credentials or transfer output.

### Process isolation and local networking

[tests/mesh_pairing.rs](../tests/mesh_pairing.rs) uses independent native
owners, private stores and separate HOME/XDG roots. The control runner covers
one-member approval with offline inviters, admission without record transfer,
mesh merging, explicit synchronization, optional first-sync failure/retry and
self-revocation. The offline runner stops its service, holds the old transfer
ports and restarts two owners. It verifies fresh loopback discovery,
original/correction/tombstone transfer, no record propagation on writes alone,
and failure of first pairing offline. The runners are `npm --prefix relay run
test:control` and `test:offline` inside `nix develop`.

These are processes under one UID. They do not establish isolation between
actual OS users, Linux/macOS socket coexistence with every host mDNS service,
firewall behavior or physically disconnected interfaces. Distinct-UID tests
must separately verify private IPC/store access, independent lifecycle, XDG
roots and shared-IP relay coexistence. Use isolated test environments; testing
does not authorize creating accounts or changing permissions on an operator's
live machine. Windows is outside the supported mesh runtime.

Keep coverage for immutable-byte/hash preservation, idempotence, permanent
forget, cached-membership outage operation, signed revocation order and fork
rejection, copied-store identity reset, interrupted publication, cancellation,
cyclical/duplicate visits and bounded transfer leases. Typed failures must
survive both local IPC and remote stream/connection termination. Cache
migrations must preserve signed bytes, resume after interruption and reject
newer formats.

## Replication and publication

The pending-publication marker introduced in migration 4 is shared by local
writes, imports and replication (LLP 0001). Keep that marker when retiring
remote configuration. Every replica operation uses a schema-checked
transaction, including read snapshots, configuration writes, apply and the
pending check before finish. Apply and configuration writes acquire their
SQLite write lock before checking the schema, so a previously opened replica
cannot write after a newer migration. Local paths, keys and legacy remote
configuration never replicate as records. Signed mesh membership travels only
on the separate control protocol. Exchange original envelope bytes and their
SHA-256, body SHA-256, and tombstone IDs; never copy SQL files, indexes,
vectors or source files. Validate hashes and envelope ID before accepting a
record. Unknown JSON fields remain in the immutable envelope but are not
indexed as author terms. Rebuild preserves existing envelopes byte for byte.
Existing ID and identical envelope hash is a no-op; different hashes are an
explicit conflict, never a last-writer-wins overwrite. Tombstones always win
and are never garbage collected. Close envelopes, parent references and writer
metadata travel unchanged.

Mesh exchange has no repository-scope selector. Keep original scope unchanged
as envelope metadata, not as a network authorization boundary. Writer email is
unverified attribution, never peer authentication. Membership grants read/write
replication of the whole store; removal cannot retract existing copies.

Each received item commits durably to the private incoming table. Replica
inventory/get/needs include these accepted originals, preserving idempotence,
conflict detection and tombstone precedence before publication. Ordinary queries
continue using the preceding complete publication. Mesh transfers retain their 256-item/4 MiB publication
bound. Finish consumes durable staging in pages of at most 256 items or a 4 MiB
prefix, allowing one independently bounded item to exceed that byte target.
Validated file imports perform one bulk publication rather than repeatedly
rebuilding the whole store for each input page. Additional working memory for
staging is bounded; durable accepted originals awaiting publication remain
uncapped like other original history.

Finish moves staged originals and tombstones, builds all derived indexes and
clears staging in one checked immediate transaction. Internal joined transaction scopes cannot expose an intermediate publication. A crash or failure restores staging and the
preceding complete query publication; retry finish through sync or rebuild. The
legacy pending marker is recovered under LLP 0001. Apply acknowledges durable
acceptance; finish acknowledges complete publication. Before successful finish
with tombstones, drain old original-data responses and checkpoint/truncate WAL.
Drain replayed staging even when every item is already published or tombstoned;
discarded original bytes and legacy pending recovery require WAL erasure too.
Forget retains secure deletion, FTS `delete-all` on an empty index and segment
optimization. Sync resumes from durable originals/tombstones, not an
acknowledgement cursor.
Rebuilding may recompute the documented lifecycle projection, just as explicit
rebuild does. Concurrent writers are serialized by SQL write transactions;
publication uses the existing source-version check. A sync is not a distributed
transaction: concurrent changes can require another run to converge.

## Replica wire contract

Keep bounded, typed original-item exchanges separate from membership control,
Iroh relay datagrams and local IPC. Mesh profile 1 uses length-prefixed bounded
JSON on authenticated QUIC streams with a 16 MiB frame limit. Retain inventory,
get, needs, apply and finish operations internally; the public stdio protocol
and its handshake are removed. Negotiate the mesh ALPN before exchanging data.

Every session negotiates compatibility before original-data transfer. The Hello
includes the caller's locally verified membership head. An equal head lets the
receiver authorize the endpoint against its current local state and return
Authorized immediately; the caller still checks local authorization. Otherwise
Ack requests the full verified proof exchange. A claimed head never grants
membership. This negotiation adds no Worker request.

Inventory pages contain at most 256 IDs/hashes, ID-ordered with an exclusive keyset
cursor. Tombstones precede records; send only missing bodies. A conflicting
hash fails without transferring replacement bytes. Transfer one envelope at a
time. The mesh adapter revalidates and leases each bounded chunk under the
protocol resolutions below. Release before peer application acknowledgement
waits. Oversized items fail rather than truncate. Malformed, oversized or
truncated frames and untyped replies fail explicitly. Unexpected EOF fails the
exchange; explicit finish and failure cleanup publish accepted pending items.
Inventories are bounded database reads, never permanent in-memory corpus
copies. The mesh transport does not raise the record/frame limit or change item
hashing implicitly.

Keep corpus-free tests for raw-byte preservation, idempotence, tombstone
precedence, immutable conflicts, interrupted publication, version mismatch and
bounded framing. Existing retrieval pages on unchanged stores remain identical.
Network activity is daemon-owned. Search, ingest and other ordinary local
operations do not initiate record exchange or wait for peer availability. Only
an explicit manual sync job transfers records; there is no automatic
propagation.

## File export and import

`export` writes a versioned JSONL bundle; `import <file|->` merges it without
changing IDs, original envelope/body bytes, timestamps, scope or writer
attribution. This is distinct from `add`, which creates a new record from
original text. Imports use the same immutable conflicts, tombstone precedence
and publication rules as sync. Only import bundles from trusted writers:
deletions are permanent.

Export defaults to current word-eligible records at one captured UTC clock.
`--all-states` includes candidates, expired records, close envelopes and
version history. Filters are ANDed: `--scope` matches original scope with
SQLite NOCASE; `--kind note|doc`; `--since` is an inclusive creation timestamp,
accepting RFC3339 or a positive whole duration in m/h/d/w relative to that
clock; repeatable `--id <ref>` selects the union of resolved whole-record
references. No ranked search selects exports. A fetch reference exports its
whole record, not its span. Unknown references are errors; known records
excluded by another filter yield no item. References use the ordinary
caller-scope resolution rule.

Selection is a consistent SQL snapshot. References and lifecycle edges do not
implicitly export other bodies. A history selection containing a closed or
superseded record must also contain at least one accepted envelope or child
that keeps it hidden; otherwise fail and request a broader selection. A stale
hidden projection with no surviving witness requires `index:rebuild` first.
Unresolved outbound references stay unresolved at the recipient.

Only an unfiltered `--all-states` export includes all tombstones, making it the
complete original-data backup. A filtered export cannot recover the scope, kind
or creation date of a deleted record; it never guesses or leaks unrelated IDs.
Explicit tombstone IDs can be exported with `--all-states --id` when no scope,
kind or since filter is supplied. Other exports omit tombstones. Export is a
snapshot, not ongoing deletion delivery; use sync for that purpose.

Bundle version 1 has a header `{"format":"linn-export","version":1}`, then
tagged `item` frames containing the sync Item fields, then an `end` frame with
the item count and SHA-256 of the exact item lines including newlines.
Tombstones precede records; each group is ID-ordered. Each line is bounded to
16 MiB. The end frame is mandatory and trailing content is rejected. Original
envelopes are strings inside frames, preserving formatting and unknown fields
unchanged. No SQL indexes, vectors, remote configuration or source files
travel.

Export stages into a private temporary file before releasing bytes; an error
does not emit a partial bundle. `--output <file>` creates a new mode-0600 file,
refusing existing paths; omission or `-` writes stdout. Import stages and
checks the complete framing, count, digest and every item hash before any
merge. A conflict during merge can leave earlier items committed; publish those
before returning the error, naming recovery if publication also fails. Retry is
safe. Successful import publishes indexes once and reports changed/unchanged
item counts (JSON with `--json`). Export always emits the bundle format
regardless of `--json`; errors go to stderr, with no routine exported-item
count. Empty bundles succeed.

## Derived membership storage and history

Durable membership history has no lifetime quota. Preserve immutable signed events and receipts. Persist derived aggregate state
only for current leaf heads, deleting consumed parent caches in the same
transaction that accepts a child or merge. Facts such as permanent revocations,
used delegation keys and root ancestry remain available; do not expire them to
make storage appear bounded. Each fact appears in a current component cache,
not in a permanent full snapshot for every historical event. History-page merges
retain only the next 16 candidates while iterating component roots. This keeps
retained cache space linear in history rather than quadratic. Original store
history, tombstones and signed membership metadata remain unbounded by policy.
Current component caches and historical replay can therefore still exceed
available disk, memory or execution limits at sufficiently large scale; chunked
rows do not bound the aggregate working set. Deployment storage and resource
monitoring remain necessary; exhaustion can prevent membership changes,
including revocations. Do not silently prune authorization facts. If leave/reset
cannot persist shutdown intent, still request suspension of the current daemon
and report that a restart may restore authorization.

Native migration 8 removes non-leaf snapshots and reduces each event's derived
root index to one representative root. Replay an uncached historical state
iteratively from its signed predecessors when an old invitation or conflict
check needs it; verify signatures/receipts during reconstruction and do not
persist that historical snapshot again. Duplicate accepted event writes compare
immutable bytes; proofs acknowledge already-known proposal hashes using indexed
existence checks without rebuilding old states.
Only state consumers request historical reconstruction. Original records,
tombstones and original membership envelope bytes are unchanged.

For peer handshakes and ordinary enrolled-device HTTP refreshes, verify unfamiliar ancestry in a transient staging area with
a 4 MiB accounting budget for envelopes, derived states and node overhead. Keep
only staged leaf aggregates, releasing consumed parent states; reject forks of
consumed unfamiliar heads. Import
only the ancestors of a verified event whose roots intersect the locally trusted
mesh. Reject leftover unrelated components at proof completion; duplicate known
hashes use indexed existence checks. Invitation redemption deliberately imports
the offered mesh's ancestry using its explicit invitation authority. HTTP history
refreshes send the locally verified head in `x-linn-known-head`. The Worker checks
signed reader authorization and current membership/invitation validity first;
when the head matches, it returns an empty completed page. The native reader
returns its already-verified matching state. A differing head uses ordinary
proof replay. These conditional reads introduce no new authority or persistent
peer cursor. HTTP history refreshes capture a finite target head and have a
five-minute total deadline;
a service that never supplies that target cannot hold a command indefinitely. A merge exceeding the staging budget fails
with a capacity error rather than partially importing unrelated history. Ordinary
connected membership history is not subject to a lifetime staging quota.

The Worker stores each current cache as canonical JSON in 64 KiB string chunks
with a small count header. No single cache row grows with total membership
history. Remove all parent chunks atomically on head replacement. Read previous
aggregate caches during upgrade, then convert current heads and delete old
snapshots. A restartable upgrade checkpoints individual historical index
entries; the admission/control service waits for this derived-data conversion,
resuming from committed progress after interruption. Production upgrade slices
have a 100 ms scheduling budget, finishing the current entry before yielding;
control requests return retryable 503 while more entries remain. Startup does
not wait for the entire historical scan. Original signed events are
retained unchanged. Only Worker versions supporting the stored cache/index
format can run against it. Rollback must preserve the receipt key, accepted
heads and invitation consumption; restoring an older signed history is not a
cache downgrade.

Components only merge; they never split. Each accepted event is indexed once
under a representative root from its verified ancestry. History reads take at
most 16 entries after the cursor from each root belonging to the requesting
component, merge those bounded ranges by acceptance cursor, and return the
first 16. Preserve topological acceptance order and include both sides of old
merges. Never scan unrelated components or return empty pages merely because
another mesh has traffic. Native history uses the corresponding indexed root
ranges.

### Typed job failures

Reports retain peer, phase, bounded message and a stable typed `kind`:
`authentication`, `version`, `conflict`, `publication`, `timeout`, `offline`,
`protocol`, `cancelled`, `capacity` or `hop_limit`. Failures identify their source
category. Preserve that category through local RPC and delegated reports without parsing
error message text. Schema/ALPN mismatches are version failures,
signature/membership denials are authentication failures, immutable-content or
signed-history forks are conflicts, and store/publication failures retain their
publication category. No-progress deadlines are timeouts; unreachable peers are
offline failures.

Mesh profile 1 reserves QUIC application codes `0x4c00` through `0x4c09` for
these categories in the enum's explicit order. A failing control stream is reset
with its category before cancellation drops its owner; connection closes use the same codes. Receivers decode either path,
preserving the remote category even when the rest of a failure response cannot
arrive. Only service-unavailable failures permit cached-membership fallback;
explicit authorization, protocol or verification failures are returned. A
publication failure takes precedence over a concurrent network failure so
recovery is not hidden. Typed failure reporting does not authorize automatic
membership mutations or unbounded retries.

## Protocol resolutions

Use RFC 8785 canonical JSON for signed mesh events and hashes, with explicit
versioned schemas, sorted unique expected-head/root arrays and domain-separated
Ed25519 signatures. Reject unknown fields, duplicate IDs with different bytes,
missing parents and cycles. Persist original signed control bytes independently
of original record envelopes. Bound transport pages, not total durable history;
verify history incrementally rather than requiring a whole log in one frame.

The Worker serializes membership changes by comparing signed expected heads
inside the same durable transaction that appends the event, changes component
heads and consumes any invitation. Verify signatures before entering the
transaction, then recheck all authority, head, cancellation and expiry
conditions against transactional state. Async signature verification must not
create a check-then-write race. A rejected proposal has no partial effects. An
uncertain reply retries the identical proposal ID/bytes before creating a
replacement; matching accepted proposals return the original result, while
reused IDs with different bytes fail. A stale proposal may be replaced only
after verifying the new head and rechecking the requested operation. Staleness
does not itself mean a fork; incompatible accepted histories do.

Replay the same authorization rules in native peers and the Worker. Corpus-free
fixtures must cover admission then sponsor revocation, sponsor revocation then
admission, both orders of mutual revocation, two simultaneous merges sharing a
component, stale merge retries, replayed old admissions, a permanently revoked
key in either merge input, lost success replies and incompatible signed heads.
No stale invitation or proposal may restore a revoked identity. There is no
partial acceptance of a permission graph or endorsement protocol.

Every accepted membership event carries a separate durable receipt signed by
the deployment's receipt key, pinned in genesis and invitation delegations. A
device signature alone proves a proposal, not coordinator acceptance; peers
must reject proposals without valid receipts. The receipt binds the complete
signed event hash, service identity and acceptance time. For redemption, the
event binds the invitation hash (and thus its expiry), actor and expected
heads; its unique event ID serves as the consumption ID. Hash the signed
proposal independently of its receipt to avoid a circular hash. The coordinator
atomically persists the accepted event/receipt, head changes and consumption
and returns identical bytes on retry. Peers verify the device authorizations
and this receipt, then validate acceptance time against the delegation window;
historical admissions do not expire merely because a peer receives them later.
Service admission/timing and single-use enforcement are trusted coordinator
functions. A compromised coordinator can backdate or fork receipts but cannot
mint a device/delegated-key signature: this is an explicit trust limit, not a
cryptographic guarantee of trusted time. Reject service receipt-key changes
until explicitly reprovisioned. There is no transparent key rotation.
Expired/cancelled unused invitations are rejected at admission. Service restore
cannot roll back consumption or transparently revalidate old invitations. Test
late receipt delivery separately from late redemption.

The version-1 membership wire profile requires canonical JSON on the wire (also
rejecting duplicate properties) and uses lowercase hex keys/hashes/signatures,
integer Unix seconds within JSON's exact safe-integer range, an explicit
service origin and sorted unique expected-head arrays. Actions are genesis,
invitation, redemption, cancellation and revocation. Genesis binds a random
root ID and the receipt key; invitation binds its issuer, delegated key,
receipt key and ten-minute window. Redemption signs both component heads with
the joining member and delegated key. Cancellation/revocation require a current
member's signature. Genesis and ordinary actions have zero and one expected
heads respectively; redemption has one for admission or two for a merge. A
history proof is a topologically ordered page of accepted events; missing
predecessors require more history rather than best-effort authorization. Root
IDs and active/revoked members are derived from signed history, never accepted
from an unsigned service snapshot. Membership is bounded to 256 active
endpoints per component; this is independent of paged permanent history.
Revocation subjects must be known members, not arbitrary future keys. A service
receipt is not an independent membership grant.

A 30-second timeout means no write progress, not an entire frame's wall time.
Use bounded chunk authorization in the mesh adapter: before each write of at
most 64 KiB, the dispatcher checks record identity/hash, current
tombstone/publication state and peer authority, and lends a delivery lease.
Hold it through that one asynchronous write attempt. Release on successful
partial/full progress or on transport cancellation; reacquire and revalidate
before sending the next bytes. A blocked attempt has the idle-write deadline,
after which the transport is cancelled before release. A forget can therefore
interleave between chunks and prohibit all future writes of the old record.
Reset/discard the incomplete incoming frame on cancellation; apply only
complete validated items atomically. Already handed-off transport bytes remain
in flight under the ordinary limitation. Preserve the 16 MiB item/frame limit;
slow but progressing connections need not restart a complete record every 30
seconds. Test many partial writes, source forget between chunks, revoked peers,
stalled writers and cancellation racing lease release. Original item validation
and immutable merge semantics remain unchanged.

Manual job frames are Begin, Accepted, Collected, Distribute and Completed,
versioned separately from replica item operations. Cancellation propagates through
stream/connection teardown and owned task cancellation. A single Progress heartbeat
carries revision zero before tracking starts, then the tracked revision; there are
no separate mesh Ping/Pong frames. Iroh relay Ping/Pong remains independent.
Begin binds a random run ID, initiating endpoint, frozen membership head/cohort and the
initiator's signature and maximum hop count. Each authenticated forwarding peer
extends the bounded arrival path described below. Keep at most four active jobs
per store, at most 256 cohort endpoints per job with sequential child
collection; reject excess active work explicitly. A job moves once through
collecting, collected, distributing and terminal complete/failed/cancelled
states. Duplicate Begin requests return status immediately, never recursively
wait on the requesting branch. Collect children and merge them locally before
returning the subtree to the parent. Distribute the parent's merged state
locally before visiting children. Record per-branch outcomes; a member lacking
successful collection or distribution makes the result incomplete. A failed
path does not make the result incomplete if another branch reaches that member.
Changed membership or concurrent writes require another explicit job.
Independent transport heartbeat/cancellation runs outside the dispatcher; loss
of the parent control stream cancels the subtree. Bound terminal status
retention to ten minutes in memory and discard it at owner shutdown; rerunning
reconciliation after a lost result is safe, whereas pairing receipt retries use
their separate durable IDs. Tests must exercise two paths to one child, cycles,
overlapping roots, parent loss, cohort limits and failed collection followed by
partial distribution.

## Bounded forwarding and duplicate control

Mesh job profile 1 signs a maximum of 16 forwarding edges with its frozen
cohort, head and random run ID. Each forwarding request carries the ordered
visited endpoint path; the authenticated sender must be its last entry. The
path starts at the signed initiator, contains only cohort endpoints, is unique
and cannot exceed the signed hop budget. Each new receiver appends itself. A
full-budget node publishes its own collected state but schedules no further
descendants; unreached members produce an incomplete report rather than an
endless traversal. This is a trusted-member forwarding rule, not a
cryptographic proof of each intermediate hop; an authorized device can already
initiate its own jobs.

Skip ancestors already in the arrival path. Register a run before contacting
children. Compare duplicate IDs by immutable signed job bytes, independent of
the arrival path; reject conflicting reuse. Return cached/in-progress status
immediately for duplicate visits, without waiting on a descendant. Keep the
first accepted parent. Early/repeated Distribute requests during
collection/distribution return phase status and do not apply data or restart
work. Source streams remain phase- and peer-bound. QUIC orders frames on each
stream; reordered visits across streams are handled by this run registry. The
registry retains at most 68 entries, with at most four active jobs; terminal
entries expire after ten minutes and can be evicted sooner for space. After
owner shutdown all run bookkeeping is discarded; after eviction, replay
reconciliation remains idempotent and hop bounded. Parent loss cancels
descendant work, releases delivery leases and repairs accepted publication.
Retry starts a fresh finite run and frozen cohort.
