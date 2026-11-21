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
Migration 11 (`src/storage/migrations/0011-sync-sizes.sql`) adds the current-item inventory index: hashes, original scopes,
encoded-response/original-envelope sizes and monotonic ordinals in the same
libSQL file. Original envelopes remain unchanged.
Original records, history and tombstones survive these migrations. Local
RPC compatibility is 7, mesh job/ALPN profile is 2, and signed membership
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
adapter interoperates with stock Rust Iroh clients. The pinned native client uses
`vendor/iroh-relay` to translate its application health pings into WebSocket control
pings and translate eight-byte control Pongs back into health-tracker replies.
Only native client connections enable this translation; server connections and
browser clients retain upstream behavior. Application Ping/Pong remains supported
for stock peers and ordered send-window checkpoints. Cryptography is unchanged.

Use accepted hibernatable sockets and per-socket attachments for authenticated
identity and control state. Reconstruct routing after wake; in-memory maps are
never the only membership authority. Client control pings receive runtime control Pongs without waking the object.
Control Pongs prove transport liveness, never application delivery. Measure idle
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
populated by mDNS. One endpoint belongs to each enrolled store. Exactly one
configured relay per node is a permanent design assumption, not an initial
rollout limit. Do not add multiple relay selection, failover or relay ranking.
The `(relay, target)` outcome map remains one per-node map for each active job.
Relay connection failure does not prevent loopback discovery or direct transfers.

Iroh uses known direct candidates and relayed connectivity and may establish
direct UDP paths when available. The Worker provides no UDP address-discovery
service or signed address-hint publication. Remote NAT traversal is
opportunistic; there is no promise of direct connectivity across every NAT. The
service remains an availability dependency for pairing and relay-only peers.
Cached membership alone does not locate a remote endpoint after every possible
address change.

Use a versioned Linn mesh ALPN and bounded authenticated control stream,
separate from local IPC. Membership exchange precedes data authorization. Adapt
the bounded wire types in [sync](../src/sync.rs), and the merge functions in [replica
storage](../src/storage/sync.rs) to asynchronous streams; preserve original
bytes, hashes, tombstone precedence, conflicts and publication recovery
specified in LLP 0001 and the retained contracts below. Network protocol
compatibility is independent of the local application/schema compatibility
number. Use the typed [mesh store queue](../src/mesh/store.rs) and serialized
application steps for inventory/get/needs/apply/finish. Use an explicit
authorization step instead of an empty Needs request. Source Finish rechecks
the requesting peer in the same serialized step as publication; local failure
cleanup still publishes without requiring a remote peer. Network workers never
hold the store lane while waiting for peer acknowledgements. Original-byte
chunks retain only their bounded delivery lease.

Active connections belong to individual job sessions and close when those
sessions finish or are cancelled. Reuse healthy connections and release idle
branches only under capacity pressure, as specified below. Derive the combined
inbound/outbound connection limit per store from current verified signed
membership. Let `N` include this device, with `1 <= N <= 256`, and use
`C(N) = min(1028, max(32, 4 * (N - 1) + 8))`. The floor is 32 connections; the
hard ceiling is 1028. Budget up to `N - 1` retained tree edges for each of the
four independent jobs, plus eight connections of admission/probe headroom.
This is a per-store resource allowance, not a target connection count or a
reason to preconnect. It does not guarantee room for every simultaneous losing
dial: queue excess work and retire duplicate contacts under the existing limits.
Count connecting and admitted transports in both directions against the same
budget, including status probes, duplicate contacts and connections still
waiting for application authorization; do not exempt unverified arrivals.

Compute `N` only from locally verified current signed membership, never an
unsigned coordinator response, address hint, proposed cohort or larger
historical frozen job. Before current membership is verified, use only the
32-connection floor. Recompute the target on verified membership changes;
starting/finishing a job does not multiply or reset this store-wide allowance.
On growth, make the added capacity available without reconnecting healthy
sessions. On shrink, permit no new reservation that would exceed the new
limit. Drain excess idle connections, and let already admitted authorized work
finish under its unchanged progress deadlines; a transient excess while it
drains is not permission to admit more connections. Do not cancel progressing
transfers solely to reach the lower target. Revocation, leave, reset and shutdown
retain their normal immediate authority/cancellation barriers. Never exceed the
1028-connection ceiling, including during growth or concurrent reservation.

Inbound connections without active application streams close after 30 seconds.
Keep eight handshake slots and two slots per transfer direction, with their
30-second acquisition deadlines; connection-limit growth raises neither limit.
Incoming membership handshakes
have a five-minute total defensive deadline; outgoing sync admission uses the
per-attempt admission deadlines below. An accepted connection must begin its job
within 30 seconds of authorization. Independent jobs can contact the same peer;
there is no elected dial owner or cross-job connection cache. The frozen cohort
bounds traversal; ready results are not blocked by endpoint ordering. Establishing a connection does not itself exchange records.

### Manual mesh jobs

`linn sync` starts one finite, initiator-coordinated job with an unpredictable
run ID and a frozen verified membership head/cohort. Only an explicit local
command starts a root job. Writes, startup, timers, reconnects and receipt of
records never start jobs. Keep at most four active jobs per store and 256 peers
per cohort. Independent explicit jobs retain independent authority and budgets.

Every sync starts with fresh job state. No job results, learned reachability,
confirmed-gone/offline or absence results, version results, status, job-owned
connections or connection pools carry between jobs. Create fresh job-owned
transport/client state and empty outcome/assignment/delta maps
for each explicit run, and close/drop them after delivering its terminal result.
Reuse connections, observations and cached duplicate replies only within that
same active run. Concurrent independent runs do not share job state either.

Durable original records/tombstones, identity and verified signed membership
remain authoritative storage, not reusable job state. The enrolled daemon's
passive listener, independent discovery, relay presence and Iroh's retained
per-peer endpoint path state remain passive transport infrastructure. Iroh path
state may persist between jobs and be shared across concurrent jobs; do not add
per-job path isolation inside Iroh. Never use that retained path state to skip
or shorten a contact attempt, infer or report absence, or change which peers
are tried. Transport path selection may use it within an independently started
contact attempt. Apply a shared preferred path to each connection only after
that path is established on that connection. While it is opening, retain the
connection's existing usable paths and statuses. When pruning redundant IP
paths, count live paths and decrement that count after each successful close;
never prune every live IP path because another connection already has the
preferred path. Exclude locally closed paths from selection and live counts
until their delayed close events retire the cached entries; keep this pending-
closure bookkeeping bounded by those existing entries. Keep this transport
handling shared and independent of jobs.
Passive infrastructure neither preserves a completed run nor
authorizes adopting its job connections or outcomes into another run. Use
current discovery observations for each new job and independently establish its
peer connections.
Do not create duplicate endpoint identities to obtain a cold job or disrupt
unrelated active work when dropping that job's resources.

Optimize cold-job sync latency and traffic for meshes of 5–20 devices across
a few networks, while preserving correctness and bounded resources up to the
existing 256-peer cap. Separate the initial direct data round from final
redistribution, but overlap missing-peer discovery with the initial round.
A receiver of Begin registers the job and accepts its assigned role; it does not recursively probe the cohort. Contacting a peer, learning an
address or receiving an advertisement does not authorize record exchange.

#### L1 and subsequent levels

The initiator is the scheduler and initial frontier. L1 consists of directly
admitted peers. Start bounded contact attempts and process ready admissions and
transfers immediately; endpoint order breaks scheduling ties, never blocks a
ready peer. As soon as a missing peer's first contact attempt fails, assign an
eligible reached intermediary to it without waiting for L1 collection or
publication to finish. A failed relay route does not cancel a still-useful direct
attempt; expansion can proceed while that other route remains pending.

L1 fixes its contribution set once all original direct contact/admission attempts
have settled, not merely once they have been issued. Settle an attempt by
acceptance, failure, timeout or cancellation after another route wins. Peers
already admitted through an intermediary need no further initial direct attempt.
Do not extend L1 for a later relay-refresh retry or failed-branch recovery.
Continue ready upward collection and expansion while original attempts settle;
their unresolved outcomes may delay L1's downward pass. Capture
its data-round baseline and acknowledgements independently of overlapping
expansion; do not repeatedly enlarge L1 or wait for all descendants to finish it.
Keep publication/store operations serialized as before. The scheduler routes
bounded assignments through reached parents; intermediaries never independently
probe the entire cohort or create another root job. A newly reached island may
expand toward its assigned missing peers immediately, under the same run,
ledger and deadlines. Derive contact levels from accepted parent depth, not a
global wave barrier. Retain older untried gateways as alternatives.

Collect contributions from newly reached peers toward the root as soon as their
sources are ready, including while L1 work is active. Tag contributions outside
an L1 peer's acknowledged baseline for final redistribution. This overlap hides
contact delays behind useful transfer/publication work instead of adding the
sum of L1 and expansion durations. Do not create a full data round per level.
After discovery exhausts eligible work or reaches the discovery cap, finish L1 and any admitted upward collection under their
existing progress deadlines, then redistribute the merged state once to every reached branch. At most two
logical data rounds exist: initial L1 and final redistribution. If no new
contribution or unsynchronized admitted peer remains, reuse existing completed
coverage instead of performing an empty final pass. Concurrent external writes
alone do not trigger an unbounded third round.

For A reaching B/C directly while only B can reach D, schedule D through B as
soon as A's first D attempt fails. B can discover D while A/C still exchange
records. D's originals flow through B toward A; final redistribution carries
those originals to C and the rest of the reached group. Do not delay D's
admission until A/B/C finish L1. A completed initial contribution is not final
coverage after another branch introduces records or deletions.

#### Assignment ownership and amplification bounds

Keep one authoritative assignment ledger at the initiator. Record source,
target, route class (direct or relay), assignment ID, level, state and outcome.
For a missing target, start the preferred source first and stagger a second
eligible source after a fixed internal delay of 750 milliseconds if the
target remains unadmitted. Allow at most two concurrent discovery assignments
per target while relay state is unknown, both charged to the existing
eight-contact job-wide limit. Once a current attempt supplies a validated
correlated Gone outcome, let remaining direct-only routes to that target use
all available job-wide contact slots, without a per-target limit or stagger.
Invalidating Gone restores the ordinary two-assignment/stagger rule for new
assignments; do not cancel already owned direct attempts to enforce it. Keep
route eligibility, deduplication, deadlines and the discovery cap unchanged.
Do not wait for a full failed dial before using an available second route. After an
attempt ends, continue untried eligible routes until admission, exhaustion or
the discovery cap; staggering changes order,
not route eligibility. The cap always bounds discovery.
Register first acceptance atomically, keep one parent, cancel losing attempts
and fence delayed accepts without cancelling the winning connection. An endpoint's
successful admission is job-wide knowledge: do not collect it again through
another route or let intermediaries independently schedule the cohort. Duplicate
contacts return registered status immediately without waiting on their caller.

For direct discovery, try each eligible reached-source/missing-target pair at
most once. A source with no known direct address for the target has no eligible
direct route: return a distinct NoDirectRoute response without starting a
DirectOnly dial or spending its three-second deadline. In a combined attempt,
try the relay normally. Select its initial paths after passive transport
readiness, including direct addresses learned during that wait. Report the
source's actual selected direct-route eligibility in both successful and failed
contact replies, not an earlier address snapshot or only the requested route
flags. Carry this same per-dial state through verified speculative adoption.
A concrete non-timeout direct failure remains fatal even if another route later
admits the target. If no direct route was selected, retain that outcome. Record
these source/target pairs separately in the traversal report and human output;
they are neither timed-out routes nor evidence of absence. Configuration may
also disable direct transport. Do not consult passive Iroh path state to skip or
shorten attempts. Prioritize existing address hints, then use endpoint order fairly;
hints affect order, not whether a possible source is eventually considered.
Among currently available sources retain hint priority, then prefer the shallowest
accepted tree level among equally hinted sources and rotate fairly in endpoint
order within that level. Apply depth preference only to sources reporting a free
outgoing transfer slot now. Never wait on a known busy shallower source while a
deeper eligible source reports an idle slot. Unknown availability is not a free
slot. If every eligible source is busy or unknown, preserve eligibility and use
fair hint/endpoint ordering without a depth preference under existing deadlines.

Advertise live outgoing transfer-slot availability in the accepted response and
as bounded, event-driven capacity deltas over existing authenticated job control
streams. Forward descendant deltas along the registered tree, coalesce superseded
values and bound each map/batch to the frozen cohort. Advertise only a boolean
indicating whether any outgoing transfer slot is free; retain two actual slots.
Use the newest received report at selection time; remote reports are scheduling
advice, not reservations, reachability, absence or admission. The actual source
semaphore remains authoritative when a transfer starts; concurrent acquisition
races retain the existing two-slot limit and 30-second acquisition deadline.
Send updates when acquisition/release changes availability between zero and
nonzero free slots; one-to-two or two-to-one free-slot changes do not affect
selection and must not create traffic. Coalesce intermediate transitions before
forwarding. Do not poll or send capacity heartbeats, and do not count capacity
advertisements as application progress. Discard these maps
with the job and invalidate failed/replaced source roles. Do not change attempted
route ledgers, reset direct attempts or grant another relay refresh.
Retain older sources as alternatives. Admit newly found sources into the same
work queue and continue until there are no pending assignments and no untried
eligible routes from any reached source to a missing target. Do not stop at
an empty newest frontier while an older gateway remains untried.

Do not cap a target at three sources or require a hint before trying another
source. Consider every eligible route unless user cancellation or the discovery
cap ends discovery; name remaining targets as partial/unresolved,
never as proved unreachable. At most `N * (N - 1)` distinct directed direct
source-target pairs exist in a frozen N-peer cohort; completed admissions
remove targets from that queue early. Worst-case direct exploration can be
quadratic when topology is unknown. Bound duplicate work and concurrency rather
than claiming a linear bound by omitting routes.

Sources may report passive direct-address hints in bounded pages, limited to
cohort targets. Send newly available hints once, not a repeatedly polled all-pairs
reachability matrix. Never cap hint reports at eight and treat the omitted hints
as evidence of absence. The scheduler need not collect every hint before starting
work: an assigned source consults its current local lookup. Where no direct
address can be resolved, report no known direct route, not a failed relay dial.
mDNS hints are unauthenticated location evidence; authenticate actual connections.

Keep at most eight pending contact assignments job-wide, subject also to each
store's connection, handshake and transfer limits. Reserve a slot before issuing
an assignment; release it on terminal outcome. Queue work fairly across missing
targets and sources. Forwarding a command over an existing tree edge does not
consume another target-dial slot. Cancellation drops queued/in-flight work and
its owned connections. There is no recursive CLI execution or independent
intermediary fan-out.

#### Relay-aware expansion beyond L2

Separate relay reachability from direct-path opportunities. L1 may use direct
and relay paths. At L2, intermediaries retain direct discovery and may use the relay for missing
targets whose relay result remains unknown. Reuse an already conclusive per-target
relay result at L2 as well: learning a direct path does not require repeating a
relay attempt known to be absent. Do not suppress L2's local P2P opportunity
merely because the initiator already has relay connectivity. For newly admitted
targets, prefer a confirmed useful direct route when choosing their first parent;
otherwise use the working relay route without waiting for speculative optimization.
Beyond L2, stop repeating relay sweeps at every descendant: reuse per-target
conclusive outcomes through the job's `(relay, target)` map, scoped to the
verified frozen membership cohort. Unknown map entries retain eligible sources. This rule never suppresses eligible direct attempts
to still-missing peers at any depth.

Keep one job-scoped map keyed by `(relay, target)` with reached, confirmed-gone
or unknown outcome. Every node dials through its sole configured relay. The
initiator supplies the initial contacts. For expansion, select a usable source
with the hint, capacity, depth and rotation rules above. Selecting a source
does not select another relay for that node.
No source-specific negative cache or separate source/target relay map is needed.
The relay key names the node's sole configured relay. It does not introduce
multi-relay discovery, selection, failover or ranking. Keep the single map local
to this node and active job; carry validated observations through job messages
without retaining them for another run. Retain observer connection generation
only as validation metadata, not as another map.
Success to B proves neither C's availability nor a working relay path when B
was reached directly. Scope sharing to the verified frozen membership cohort.

Negotiate the correlated dial extension on each native relay WebSocket upgrade:
the client offers `x-linn-dial-check: 1` and enables checks only when the server
accepts with the same response header. Keep the base `iroh-relay-v2` subprotocol.
Older clients receive no new frame, and an older relay or missing acceptance
header leaves ordinary connection timeouts unchanged. Browser clients without
negotiation also retain the base protocol.

A negotiated DialCheck frame (tag 14) contains a 32-byte target endpoint followed
by a fresh unpredictable 16-byte attempt token. Send it only for an active dial,
not as periodic polling or an enumeration API. Authenticate/enforce same-group
membership before inspecting registration. Reply with DialStatus (tag 15): the
same target/token followed by one byte, 0 for not registered and 1 for registered.
Process that observation against current target socket registration after the
authorization await. Registration is a point-in-time route observation, never
successful peer admission or a promise about future reconnects. Apply existing
message-rate, framing, authentication and relay queue limits to both frames.

Bound pending subscriptions and remove them on completion/cancellation. Accept
only a reply matching the current target, token, relay and observing connection
generation. Drop pending checks to unknown on source reconnect; never accept
an old reply into a new attempt, even when the target is unchanged. Registered
replies leave QUIC admission running. Missing, malformed, unnegotiated or delayed
replies do not fabricate absence; retain normal timeout/error handling. A relay
that accepts the extension but never replies still falls back to the existing
dial deadline. Unsolicited EndpointGone (tag 8) remains advisory, including after
a target reconnect; it has no attempt token and never confirms current absence.

Surface a valid correlated not-registered result to the sync scheduler. A usable
or still-possible known direct route continues independently; prefer completed
connection success over an absence observation arriving at the same time. Only
a relay-only attempt may end early on the matching negative. Do not consult
retained Iroh peer-path state to rule out a possible direct path or shorten a
mixed direct/relay attempt; only the assigned route policy and configured
transport availability can exclude direct dialing. Continue eligible direct paths and preserve reachability coverage:
absence at one relay is not absence from a LAN. Keep one active negotiated
check per dial and the single `(relay, target)` outcome map; do not add independent
source-specific negative caches or relay sweeps. Invalidate stale mapped evidence
on verified new presence/reconnect under the job's bounded refresh rule.

Negotiate unresolved-target registration notifications separately with
`x-linn-target-watch: 1`, offered and accepted on the observer WebSocket upgrade.
Keep DialCheck/DialStatus unchanged. TargetWatch (tag 16) and TargetUnwatch
(tag 17) each contain a 32-byte target followed by a fresh 16-byte job-scoped
token. TargetRegistered (tag 18) carries that same target/token. Bind the native
subscription to the relay and observing WebSocket generation; ignore wrong,
retired or earlier-generation notifications. Subscribe only for still-unresolved
targets after an attempt has settled. Retain the receiver while ordinary
discovery is quiescent and through final reconciliation. Remove the subscription
when its target is admitted, the job finalizes or
is cancelled, or the discovery cap expires. Retire an observer's subscriptions
to unknown on disconnect without automatically resubscribing or granting a retry.

The relay authenticates same-group membership before activating a subscription
or notifying an observer. Group registration fan-out by observer socket, using
its session/target routing index. Authorize each observer against the current
membership snapshot, then recheck its captured active tokens after the await;
unsubscribe/close must still fence every notification. Exclude subscriptions
activated after that observer's capture, and load at most one observer's bounded
token sets at a time. Notify on each target registration while subscribed; also
notify if it is already registered when the subscription activates, closing the race
between a failed attempt and subscription setup. A notification is only a
trigger for the single `(relay, target)` refresh allowance. It never establishes
presence, reachability, admission, absence, contribution or an acknowledgement.
Keep a still-unresolved target's subscription after the refresh allowance is
consumed. Further notifications only invalidate stale absence; they cannot
schedule a second relay retry or reset direct attempts. Coalesce notifications
per subscription in bounded memory. Keep direct-route deduplication unchanged.
Missing negotiation, subscription
capacity or observer reconnect leaves the target unresolved;
neither create offline success nor add polling, resweeps or keepalives.

Bound the native endpoint's active subscriptions plus queued retirements to
1024, covering four jobs at the 256-member cap. Bound relay subscription routing
to 1024 entries per observer socket. Store only socket-generation, opaque token,
target and activation state in the existing Durable Object SQL storage so
hibernation preserves routing; store no run result, original, inventory or
reachability cache. Remove rows on unsubscribe and socket close,
and discard orphaned socket rows on rehydration. Bound authorization-pending
rows in the same limit and let unsubscribe/close fence activation after an
asynchronous authorization check. Apply existing relay framing/rate/queue limits. If notification queue capacity
is exhausted, close the observer connection to invalidate subscriptions to
unknown rather than silently dropping a registration trigger.

The bounded refresh rule permits one event-triggered relay retry per
`(relay, target)` per job after verified new target presence or target reconnect
invalidates stale relay evidence. Permit that retry only while discovery
is open: before finalization or cancellation and
before discovery-cap expiry. Exhaustion makes ordinary discovery quiescent; it
does not close the recovery window. Do not poll or resweep the cohort while
quiescent. Resume only for a verified target-presence event under this single
refresh allowance or to recover an actually failed branch over untried routes.
It may retry a previously tried relay route; consume the single refresh
allowance when issuing it. Repeated events
neither replenish the allowance nor restart discovery or its deadlines. Do not
reset direct-route attempts, add polling, or extend the discovery window.
After discovery closes or the allowance is consumed, invalidate stale evidence
without another relay retry and report an unreached target as unresolved;
stale absence must not turn it into an offline success. A source observer's
reconnect alone is not verified target presence: pending checks become unknown
under their existing attempt deadlines, without earning a refresh retry.
Conflicting, overlapping relay snapshots have no cross-observer ordering proof.
Keep such outcomes unknown rather than treating their arrival order as a
verified new registration. Doubt alone grants no retry of an already tried
route; the refresh allowance requires fresh verified target presence.

Keep L2's P2P benefit in direct discovery of missing targets, hint-based initial
parent selection and Iroh's direct-path upgrades on existing connections. Do not
probe already reached pairs or change a healthy peer's parent to optimize traffic.
Freeze its working parent through both data rounds. Do not add optional
between-round route handoff, old/new parent negotiation or topology optimization
controls to this delivery. Address hints do not alone establish a better usable
path; choose confirmed connectivity without delaying admission or data transfer.
Recovery of an actually failed parent is separate from elective optimization and
must never silently reclassify missing distribution as success.

#### Attempt deadlines and reliable coverage

Use fixed internal per-attempt limits, never runtime tuning flags. Retain the
three-second target connection and ten-second total admission caps, with admission
covering stream opening, verified membership negotiation and Begin/Accepted.
Keep queue time separate: queued assignments do not time out before they are
issued, and a new level does not add a new blanket waiting period. Start known
direct hints and eligible relay work promptly without a per-target discovery
sleep. Source-local monotonic deadlines terminate stalled work even if root
cancellation cannot arrive. Duplicate messages do not restart an attempt.

Apply a fixed twenty-second overall discovery cap, with no runtime tuning knob.
Use negotiated correlated relay absence; relays without that signal retain the
ordinary attempt timeouts and unresolved outcomes. Start the cap's monotonic
clock only after membership refresh completes successfully or takes the existing
verified-cache fallback, never at command start. Do not reset it at L2, a new
island, replacement source or repeated frame. Speculative dials keep their own
original attempt deadlines; refresh does not restart them.

Keep the cap's measurement basis separate from admitted-transfer duration.
Select the fixed twenty seconds from completed real-store release-binary runs
at the 5–20-device design target, not equal-head in-process fixtures alone.
Fresh verified membership reconciliation is part of admission and consumes this
window. With a separately extended validation budget, completed same-host
20-device unchanged syncs took 9.982–10.130 seconds cold and 0.138–0.139 seconds
warm; initial syncs took 10.535–11.647 seconds cold and 0.713–0.727 seconds warm
(two samples per workload/path state). Each completed run verified every
reachable store's exact originals. The provisional ten-second budget cut short
all-online 20-device runs and is not an accepted coverage baseline. Retain
20 seconds as headroom over completed design-target measurements, not a WAN
latency guarantee or a shorter command-time bound. Keep cold/warm Iroh path
runs separate; prior synchronization can also warm durable membership metadata.
Exercise 256-device full-coverage data-plane stress separately with an explicitly
extended test budget: its roughly 59-second single-host four-job discovery does
not redefine the production cap or establish production capped coverage.
Verify capped partials independently and never compare their latency to completed
runs as matched coverage.

The cap bounds only finding and waiting on peers: contact, admission negotiation,
discovery and expansion attempts. When it expires,
stop scheduling new attempts and cancel incomplete admission/discovery work.
Do not cancel admitted transfers, group commits, publication, final distribution
or their acknowledgements because this cap expired. Opening an on-demand session
on an already admitted recorded edge is admitted reconciliation, not renewed
discovery; retain its existing per-attempt/progress limits after discovery expiry.
Searching an untried replacement route remains discovery. An admission completed
before expiry remains admitted work; a delayed acceptance cannot start new
exchange after its discovery assignment has expired. Finish progressing
admitted work under existing frame, idle and no-progress deadlines, then return
an explicit partial result naming unreached peers and missing final acks.
Deadline exhaustion is not evidence of relay absence. Timeout-only outcomes
remain unresolved. Discovery expiry does not force error return while admitted
work is still healthy, and it does not promise a wall-clock command deadline.

User cancellation and real transfer/no-progress failures retain durable-prefix
cleanup: commit valid pending groups, publish/recover accepted staging and
release delivery leases before reporting incomplete work. An explicit discovery
cap can leave reachable peers unadmitted; report
that latency/coverage tradeoff separately from successful admitted transfers.
Explore until eligible work is exhausted, the discovery cap expires or the job is cancelled.
Full completion still requires final acknowledged coverage and honest cohort
membership. Slow admitted transfers may outlast discovery expiry; moving
topology or unadmitted peers can require another explicit run. Never infer
timeless absence from a bounded discovery attempt.

Allow one minute without application progress on an accepted control wait;
repeated heartbeats do not refresh it. Actual replica bytes, completed store
steps and assignment outcomes advance revisions. Child progress may advance
its parent's revision; parent progress only refreshes a child's wait. Do not
create heartbeat feedback cycles. Admitted transfers making progress have no
fixed total deadline; discovery-cap expiry  does not shorten their existing frame/idle/no-progress deadlines.
Capture a finite current-item ordinal cutoff and apply it to both inventory
classes, so later insertions cannot extend a transfer forever or displace an
earlier original. Preserve an ID's ordinal through staging/publication and
in-place metadata rebuilding. Keep only current items and tombstones in this
index; it is not a durable item/history log. Do not hold a read transaction
across network waits. Retain the 30-second frame/write idle
deadlines and bounded delivery leases.

#### Round reconciliation and convergence

Assign one parent when a peer first accepts a valid assignment. Keep a healthy
parent for the run; neither a cycle, duplicate arrival nor speculative shorter
path reparents a peer. Change a parent only to recover an actual failed branch,
with no old transfer active and the initiator coordinating the new role. Collect bottom-up and distribute top-down with bounded sibling
concurrency under the two incoming/two outgoing transfer slots per store. A
parent can admit and collect a sibling without waiting for an unrelated sibling's
connection attempt. Serialize immutable merge/publication through the existing
store lane; arrival order never overrides tombstone precedence or conflicts.

Every data round has a monotonically increasing number and a root-owned
membership set drawn from admitted peers. Send each recipient its own child
roles rather than the entire membership set on every edge. Finish accepted
publications before sending its collected acknowledgement.

Reuse L1's completed upward contributions for branches with no new descendants.
In the final round, collect newly admitted subtrees and their ancestor paths;
do not request another upward inventory from an unrelated L1 leaf. The existing
tree and L1 acknowledgements determine this set; no durable per-job item log,
content surrogate or new persistent publication-generation mechanism is required. A changed ancestor uses fresh
finite inventories after merging its new children. Local writes after a branch's
completed L1 contribution may require the next explicit run, just as writes
after any finite source inventory do.

After final collection reaches the root, redistribute through every reached
branch, including earlier levels, so later contributions reach L1 peers.
Earlier downward acknowledgements cannot certify equality after a later merge. Use bounded original inventories,
not generated summaries or probabilistic equality claims. Publication rules and
original-byte hashes remain unchanged. A peer acknowledges distribution only
after its local pull/publication and its retained children's acknowledgements.

For an L1 peer with a confirmed baseline, final redistribution may use a
transient in-memory list of exact original items/tombstones received outside
that baseline instead of full inventories. Use one shared job-wide list bounded
to 4096 ID/hash entries and 1 MiB of retained metadata. Bound the mutation event
channel to 256 events of at most 4 KiB each and each delta page to 256 entries
or 256 KiB, whichever is reached first; overflow or a missed event disables the
optimization. The list contains metadata, not retained original bodies; send bounded chunks, preserving tombstone precedence,
original hashes, Needs/Apply validation and publication acknowledgements. A
contribution received during overlapping expansion but not covered by that
peer's L1 acknowledgement belongs to its delta, even if it arrived before other
L1 peers finished. Track baseline identity per peer; wall-clock “after L1” alone
cannot establish complete coverage. Invalidate delta eligibility on any local
mutation or publication transition not accounted for by the list; never omit an
untracked tombstone merely because it did not arrive over a replica stream.
Confirm a baseline only when the source remains unchanged throughout its finite
L1 downward inventory and acknowledgement. On the receiver, track unaccounted
mutations from job admission, not only from the end of L1. Receiving tracked
items in this job does not itself invalidate the baseline. Keep the delta list
at the initiator; receivers need only a bounded mutation watch and their L1
completion state. A missed mutation notification is doubt and disables the
affected optimization. Keep the source list unchanged throughout each delta
stream and recheck both source and receiver eligibility before accepting it.

The delta is an optimization, never the authoritative store or durable job log.
Retrieve originals by indexed key when sending; any retained body is subject to
secure forget. Overflow of the shared list invalidates delta eligibility for
every branch job-wide: discard the incomplete list and use full original
inventories for all final branches, including those whose deltas were already
partly sent. No branch may certify coverage from a shared list that overflowed.
Final acknowledgements obtained from a complete delta before overflow remain valid for the items they covered; later additions use full inventories.
For branch-local missing/uncertain baseline, hash/ID mismatch, missing item,
recovery handoff or doubt, use full original inventories for that branch.
Restart loses the shared list and therefore makes every branch use full
inventories. An optimization mismatch
falls back to authoritative reconciliation; a real immutable conflict still
fails normally. Never use a truncated delta as proof of equality. Release this
memory at job completion/cancellation and require the same final durable
acknowledgement as full inventory exchange.
On a delta metadata/item mismatch or incomplete delta stream, publish the valid
prefix and retry full inventories once on a fresh replica stream of the same
admitted connection, with fresh current-item ordinal cutoffs. This is reconciliation,
not another contact or a deadline reset. A lost connection instead follows the
existing bounded failed-branch recovery rule. Preserve real authentication,
version, immutable-conflict, publication and protocol-framing failures as errors.

If a parent fails during the final round, fence/cancel its active work and retain
committed contributions. Mark its affected subtree's final acknowledgement
pending. Assign an untried healthy route while discovery remains open (before
discovery-cap expiry);
acknowledge the child's fenced old role and new acyclic parent without waiting
for an offline parent's ack. Reconcile the recovered branch using full original
inventories in the same final round, not a speculative delta or a new root job.
A retained positive relay outcome keeps an untried relay route eligible for
actual failed-branch recovery; it is not negative evidence. It does not reset a
tried source-target route or grant a relay refresh beyond the single verified-
presence allowance. Do not probe a healthy admitted peer for this purpose.
Do not invalidate successful unrelated branches unless the recovery introduces
new originals/tombstones; in that case distribute those additions to all peers
whose earlier final ack did not cover them. The final round may repair its
coverage before closing; “two rounds” is not permission to omit recovered data.
If discovery expiry, unavailable routes or failed
fencing prevents safe recovery,
finish durable cleanup and report the named subtree incomplete. Never count
an old parent's partial distribution as a child's final acknowledgement.

Count each peer once in final coverage, not once per round. Final collected
coverage describes contributions incorporated into the root. Final distributed
coverage describes peers that acknowledged the final round's merged state.
When the final round starts, previous distribution acknowledgements remain
visible as L1 progress but do not satisfy final coverage. Concurrent external
writes remain outside any point-in-time convergence promise and may require
another explicit run.

Stop expansion on cohort completion or exhaustion of all eligible untried
source/target/routes, after pending assignments have returned. Then complete the final
reconciliation round if new committed contributions or an admitted peer still need distribution;
otherwise retain completed L1 coverage without a redundant rescan. Discovery
expiry  stops new routes, not admitted collection,
publication or final distribution. A failed branch does not
cancel successful publications elsewhere. If a branch disappears after an early
round, retain its durable contribution and continue other branches. Try an
untried route to its affected peers through a healthy reached source before
freezing the final tree. Recovery requires the child to stop old work, fence the
old parent role and acknowledge its new parent; the initiator checks an acyclic
rooted tree. Notify the old parent when possible, but do not require an offline
parent's acknowledgement. If safe recovery cannot be confirmed, mark final
distribution incomplete. Never claim success for a route
that was discovered but lost before final acknowledgement.
Missing peers and uncertain remote outcomes remain explicit in the result.

An unresolved transport loss during a delta is uncertainty, not absence. Use the
existing bounded full-inventory fallback and require its ordinary byte coverage
and final acknowledgements. If that fallback cannot finish, report unresolved
failure; never convert the loss into offline partial success.

#### Startup overlap and bounded transfer pipelining

Overlap independent startup work. Start passive endpoint readiness and bounded
fresh transport-only dials to the first eight non-revoked remote members in the
stored verified membership's endpoint order while signed coordinator
membership refresh runs. Reserve speculative contact permits before spawning
their tasks; verified contacts must never hold those permits while awaiting
unpolled speculative tasks queued for the same budget. Charge speculative dials
to the same connection/contact limits and deduplicate them when the verified job
adopts a connection. No mesh
Begin, source request, original bytes or accepted peer contribution precedes
refresh success or the existing service-unavailable-only verified-cache fallback.
After refresh, recheck identity, enabled state, conflict, cancellation and current
membership; freeze the verified cohort, update endpoint authorization and close
connections to peers no longer authorized. A successful speculative transport
connection is neither admission nor evidence of current membership. Cancel
speculation on refresh denial, reset, leave or identity change.

Adopt a usable successful or still-pending speculative dial as the verified
route's single contact attempt; preserve its original dial deadline. If the
speculative dial is unusable at verified adoption—failed, expired, or successful
but already closed—allow at most one fresh verified contact attempt per route.
Treat these cases as one allowance; they do not accumulate separate retries.
Do not count unusable speculation as the verified direct source/target pair's
attempt. Remove speculation from job state when adoption consumes it. A pending
dial adopted while usable gets no fresh attempt if it subsequently fails. This
allowance never resets an adopted dial's deadline or admits a peer before the
membership gate. Preserve existing transport idle deadlines.

After membership verification, adopt a still-current token/target/relay-generation-
correlated reply with its still-usable speculative dial and count that evidence
once for the assigned contact. A reply observed before verification remains
transport-local until this adoption boundary; it is not membership or admission.
Retain the dial's validated observation lease and recheck its observer generation
at adoption. Never adopt stale evidence or evidence from a failed, expired,
already-closed or otherwise discarded dial; a fresh verified attempt starts with
empty evidence. Attach the assignment's observation consumer atomically with
replaying any current cached reply, so replies racing with adoption are neither
lost nor counted twice. Forward subsequent current correlated replies while the
adopted dial remains pending, including a not-registered reply that ends that
dial early. Normal dial completion/cancellation still retires its token. A pending
dial adopted while usable remains that route's attempt if it later fails or
closes; it does not acquire the discarded-dial fresh-attempt allowance.
For an adopted speculative connection, start the ten-second total admission
clock at verified adoption, when the membership gate permits admission. Preserve
the speculative transport dial's original three-second deadline; verification
does not restart that dial deadline. Time spent waiting for verification on a
successful transport connection does not consume the admission allowance.
Keep a speculative task's local capacity queue outside that allowance: if
contact has not yet been issued at adoption, start admission when its capacity
reservation permits contact. Otherwise use verified adoption. Neither case
restarts an issued transport dial's three-second deadline.

Keep membership refresh's request-scoped HTTP client behavior; do not add a
pooled or shared HTTP client, including within a job. Preserve the configured
service, TLS/request policy and five-second per-request sync timeout. Do not
retain a warm job peer connection or job-level learned route for the next sync. The passive Iroh
path-state exception above still applies. Persistent verified membership
still permits the documented service-unavailable fallback; it is not a cached
reachability result. Do not skip freshness verification or introduce automatic
record sync for speed.

Process an inventory page in one bounded local Needs operation, not one store
queue round trip and read transaction per entry. Retain the 256-entry page limit,
ID order and exact tombstone/staging/published-hash precedence. Return ordered
per-entry decisions for the valid prefix plus the first typed error, if any.
Apply the valid prefix in its original order before reporting that error; an
error later in a page must not discard earlier durable progress. Do not fetch
entries after that error. The original item's Apply still revalidates current
state, immutable conflicts and tombstone precedence: Needs is a scheduling hint,
not a durable authorization or reservation. Reuse indexed queries/prepared work
inside the short store operation; do not build corpus-wide maps or hold a read
transaction while waiting for network bytes. Keep writer/query fairness between
pages and propagate storage failures without treating them as an empty page.
Read Needs hashes from the transactionally maintained current-item inventory
index; do not parse staged envelopes to make these scheduling decisions.

Prefetch at most one next inventory page while processing the current page's
Needs/Get/Apply work. Track expected response kinds in FIFO order so an
inventory reply cannot be mistaken for an Item; metadata prefetch has its own
bounded page slot and does not bypass body-byte reservations. The InventoryPage
frame carries sized entries and an explicit end-of-list flag;
a full last page does not require an empty follow-up request. Keep exclusive
ID cursors, finite current-item ordinal cutoffs and at most 256 entries per page. Bound page
metadata memory to current plus one prefetched page. Validate cursor ordering,
class and end flag before using the page; do not Apply the later page until the
current valid prefix retires. Cancel prefetched work on earlier failure and
never prefetch record pages before tombstone processing completes.

Encode profile-2 inventory entries as compact four-element rows
`[id, hash, encoded_frame_bytes, original_envelope_bytes]`. Keep file-bundle
inventory entry formats unchanged. Include
the original-envelope length to reserve exact publication-batch capacity;
compact rows avoid paying field-name overhead on every item. Inventory entries
include encoded Item-frame byte size with ID/hash.
Size is a conservative reservation bound including JSON escaping and frame
overhead, not an unchecked hint. Migration 11 maintains
them, the current hash/scope and stable ordinal as rebuildable derived metadata
in the same libSQL file. Preserve ordinals for existing IDs when repairing this
index. Never alter the immutable
envelope or introduce another persistent data file. Validate actual bytes and the 16 MiB frame limit
before allocation/application; an understated size cannot expand the window.
An obsolete size, mismatched original or invalid end flag falls back to validated
inventory/refetch or fails typed framing/conflict checks; never truncate bytes.
Maintain this metadata with original ingest/staging in the same transaction.
Read inventory hashes, scopes and sizes from the index; do not load or parse
original bodies to construct an inventory page.

Size the Get pipeline by a fixed internal byte budget rather than a depth of
two. Reserve enough bytes for each response before issuing its Get; include
received/uncommitted items and decoded working allocations in the bounded memory
accounting. Bound request/metadata count by a page as well, so zero-size tombstones
or tiny records cannot create an unbounded request queue. A valid item larger
than the ordinary window is transferred alone under the existing frame maximum,
not rejected or waited on forever. Consume responses and Apply decisions in
inventory order using FIFO framing; no per-item QUIC streams, GetMany operation
or parallel Apply workers are required. One-item/no-change paths do not wait to
fill a window. An independently driven reader drains all credited responses
while the store worker applies an earlier prefix. Keep source writes serialized
and chunk-leased; do not queue unleased original bodies for later transmission.

Retire requests in inventory order. Validate each ID and hash against its own
inventory entry and apply no response after the first failing entry. On mismatch or source loss, stop issuing requests and retain the ordered valid
response prefix before the first failed/incomplete response. Retire that prefix
under normal Apply validation before returning the error; an independent reader
must not let a later EOF discard an earlier complete valid response. A completed
full-inventory Get miss is a no-op within that prefix. Discard
responses after the failure. Forget, revocation and explicit cancellation retain
their existing precedence and can prohibit pending application; prefix handling
never overrides a current authorization denial or tombstone. Durably accepted
items still publish through existing failure cleanup. Recheck Apply
against concurrent local tombstones even for a fully received prefetched item.
Bytes already delivered retain the ordinary non-retractability limitation.

Do not stop issuing Get credits or drain the fetch window at a group commit
or group-flush timer. Let the next bounded group fill while the preceding group
commits, subject to the byte window and available remaining publication-batch
capacity. Keep at most one group committing plus one group filling, with ordered
commit/acceptance and no third uncommitted group.

Bound a staging group to 32 items or 512 KiB of encoded Item frames, and flush
a nonempty ready prefix after two milliseconds. Transfer a larger valid item
alone. Reserve a 128 MiB per-stream fetch working budget, charging 128 times
each encoded frame size plus 4096 bytes per credited item for raw/decoded
originals, JSON validation working allocations and metadata. Include both
committing and filling groups in this accounting; retain the existing two
transfer slots per direction. This conservative reservation does not permit
an understated inventory size to allocate beyond its credit.

Keep encoding/decoding work bounded to 64 KiB when running inline on the async
task. Use the worst-case JSON escaping bound to select inline encoding; offload
larger work while retaining registered-buffer ownership. Poll an incoming
original chunk once under its buffer ownership lock, copy and drop a ready
chunk before releasing that lock, and release the lock on Pending. Never hold
the lock across a network wait or retain an unregistered application chunk
across a forget acknowledgement.

The local native matched-coverage acceptance fixture uses 5, 10 and 20 peers,
96 root items of 512 bytes or 16 KiB each, and three repetitions with cold and
warm Iroh path state reported separately. Against an otherwise identical
one-credit/one-item-group baseline, this configuration measured 7–14% lower
median job latency with effectively unchanged application-frame traffic.
Every run verifies original-byte parity. These unoptimized loopback/mock-store
measurements select internal pipeline bounds; they establish neither WAN nor
durable/full-store latency improvements.
The five-peer relay-only acceptance fixture binds no IP transports and injects
20 milliseconds before each forwarded relay frame. With 96 root items of
512 bytes and three samples per path state, its measured median falls from
10.94 to 1.61 seconds with cold Iroh paths and from 10.95 to 1.67 seconds with
warm paths against the same one-at-a-time baseline. Original-item bytes are
identical; measured combined relay ingress/egress bytes fall by about 12%.
Treat this as controlled local-delay evidence, not a WAN forecast.

At a 256-item/4 MiB publication (Finish) boundary, stop issuing new Get credits.
Drain already credited responses into their reserved slots, commit both valid
groups in order and pause the source with no uncredited original writes
outstanding while Finish publishes; resume only after successful publication. A slow store or publication must not turn
intentional pipeline backpressure into a new idle-write failure. Between complete
requests, a replica source with no outstanding response waits under the existing
job control/progress and cancellation lifecycle; it must not independently expire
a 30-second frame-header timer while the receiver performs a legitimate Finish.
Once request/response bytes start, retain bounded frame/write idle deadlines.
Do not disable job no-progress detection or keep a source alive through repeated
non-progress messages. Preserve the same contiguous-prefix publication boundaries: receiving ahead does not count
as applying or publishing. Tombstone transfer completes before record requests
begin, and final Finish/Done occurs only after outstanding requests are retired.
Group staged Apply writes within a publication batch under fixed count, byte
and elapsed-time bounds, flushing when any bound is reached. Groups never exceed
the 256-item/4 MiB publication batch, except one independently valid oversized
item alone. A short fixed flush deadline commits a sparse group without waiting
for more network data. Use the two-millisecond deadline and 128 MiB byte-window
budget defined above; these are fixed algorithms, not user knobs.

Commit a valid group before Finish, any acceptance acknowledgement, received
count, progress claim of acceptance, successful return or error return. Commit
the valid prefix before reporting a later item/page error. A local helper may
stage ordered items privately, but no individual Apply may return an accepted
result before its group commit. Reader/source progress from bytes is distinct
from durable accepted-item counts. Do not hold an open SQLite write transaction
or monopolize the store lane while awaiting network input or the flush timer;
assemble bounded pending work, then perform the short grouped write/commit.
Include all credited, prefetched and buffered uncommitted items in the two
groups' combined count/byte accounting. Reserve capacity for the committing and
filling groups before issuing credits; do not exceed the byte window or the
remaining publication-batch budget. No third uncommitted group may be fetched.
A valid item exceeding the ordinary byte window remains a single-item transfer
under the existing frame limit. If commit itself fails, return a typed publication failure without
counting or acknowledging the group; recover durable preceding staging normally.
Keep WAL FULL durability, per-item validation, tombstone precedence, secure-forget
barriers and transfer-slot limits unchanged. Crash recovery refetches at most
two uncommitted groups per receiving stream (one committing, one filling);
committed groups are idempotent
no-ops on retry and Finish still publishes accepted staging atomically.

On local forget, synchronously invalidate and drop that record's prefetched or
buffered bodies, decoded copies, uncommitted group entries and transient-delta
body copies before acknowledging forget. Cancel/fence in-flight fetches and
outbound chunks for that record so a delayed response cannot repopulate a
forgotten buffer or resurrect it at Apply. Keep tombstone metadata as required.
For received tombstones, invalidate registered buffers for the committed prefix
before replying to Apply. A rejected suffix or failed commit does not invalidate
other transfers.
Apply authoritative tombstone checks even after complete reception. Peer-local
forget still cannot retract bytes already delivered elsewhere; preserve that
existing transport limitation and per-chunk delivery lease contract.

#### Traffic and connection working sets

Bound contact attempts and original-data reconciliation separately. Suppressed
relay repeats do not establish a linear bound on total discovery traffic: routing
control through a depth-h tree costs forwarding hops, inventories contain IDs/
hashes, and replicas must receive original bytes missing from their stores.
Keep assignments target-local and reports incremental. Preserve the signed frozen
run/cohort descriptor and send it on initial Begin; later controls reference
its run ID. Do not repeat the full cohort or tree in each assignment. Send only
the receiving peer's expected child roles with its round command. Keep the
existing verified membership negotiation; do not introduce a cohort-digest
negotiation or a new signature for each scheduling message merely to reduce
already bounded control fields.

At most two data rounds bound repeated full inventory work independently of
contact depth. Unchanged L1-only runs perform one round. A run that discovers
new peers recollects only their subtrees and ancestor paths, then redistributes
through all reached branches. That can repeat downward inventories on L1 edges,
so metadata traffic can exceed a single recursive pass. The existing Needs/hash checks suppress
already-held original bodies; do not resend them merely because a round changes.
A bounded transient final-round delta list is permitted as specified above;
do not add a durable per-job item log, probabilistic equality shortcut or new
corpus-wide cache to avoid the second scan. Preserve tombstone-first transfer,
immutable conflict handling and secure forget. Later tombstones can invalidate
earlier useful L1 work; this is a cost of early publication, not evidence of
final equality. Claim traffic improvement only from measured original bytes,
inventory bytes and control bytes at matched final coverage.

Reuse healthy job connections through collection, expansion and distribution
while capacity permits. A normal small mesh must not disconnect and renegotiate
membership between phases or rounds. Reuse its control streams for multiple
assignments; permit accounting and batching are local scheduling concerns, not
new wire handshakes.

Apply the membership-scaled combined connection limit `C(N)` defined above.
Retain healthy job connections while that budget permits; do not evict or reopen
branches merely because a store passes 32 connections. Under actual capacity
pressure, including a shrinking verified-membership target, release only an idle
branch after its assigned work has acknowledged completion.
Keep its existing run/parent/phase entry; do not introduce separate Park/Resume
messages or a second session state machine. Reopen that recorded edge with the
ordinary authenticated handshake and duplicate Begin/status exchange, then issue
the next valid phase command. A duplicate Begin does not restart traversal or
change parents. EOF with delegated work pending fails that work; an intentionally
released idle connection has no outstanding work to cancel. Identify that intentional release with the application
connection-close code `0x4d00`, after the last owned operation acknowledged
completion. An ordinary disconnect, aborted admission or partial frame is not
an intentional idle release and cannot preserve a resumable role by itself.
This uses the existing connection close, not a Park/Resume control exchange.

Prefer retaining routes that are needed for upcoming work. Queue excess work
fairly before dialing, release permits on idle close and never evict an active
transfer or publication to admit another contact. Connection pressure must not
be reported as a remote offline failure. If no idle branch can release capacity,
queue fairly without holding a network/store worker. Cancellation or inability
to make progress remains explicit; do not misclassify local pressure as absence.

Bound pressure-induced reopening to one attempt per admitted edge per data-round
direction and one additional attempt during expansion. Permit one additional
control-only finalization/cancellation reopen per admitted edge under the existing
connection/admission and no-progress deadlines. That allowance sends only the
ordinary authenticated registration/status and terminal Finish/Completed or
cancellation control; never inventories, Get/Apply, another data round or a new
parent. It remains admitted cleanup after discovery-cap expiry, not discovery.
Consume it on issue, never replenish it, and report incomplete cleanup if it
fails. Existing healthy sessions consume no such attempt and require no new
handshake. Failed reopening reports incomplete work; it does not trigger a
discovery retry or reconnect loop. Count these attempts separately from the
source-target-route discovery ledger. Intermediate
routing shares active edge connections rather than reopening a path per target.
Keep consumed reopening allowances in the bounded run metadata when an edge is
detached. On parent loss, fence all work and source roles and release transports;
retain only non-resumable role metadata and its remaining reopening accounting.
An ordinary continuation cannot resume that failed role; the bounded control-only
terminal/cancellation continuation may only reclaim it. A fenced parent-recovery
admission inherits the remaining accounting instead of granting fresh allowances.
If that accounting was pressure-evicted, return expired/unknown rather than
recreating it or resetting an edge budget.

An idle entry of an active run holds no network worker, live job connection,
transfer permit, pending group or retained original body. Keep one role entry
per run per store, with at most four entries total across executing, idle and
orphaned runs. Bound each entry's roles, paths, assignments and duplicate-control
metadata by its signed cohort and the existing finite assignment/round budgets.
Retain a cryptographic fingerprint of canonical command fields, the compact
reply and the next-hop/child cancellation identity for completed controls;
do not retain a full forwarded path for every cached reply. Keep complete
commands only while their bounded owned work is active. Conflicting ID reuse
still fails, and duplicate replies never start work again.
Idle entries consume metadata capacity, not an execution worker. Reacquire an
execution slot fairly before running another phase; keep at most four executing
jobs and retain all existing transfer and handshake limits.

Retain idle metadata by count, without elapsed expiry. Do not apply a 60-second
lease, an inactivity timer, peer absence or the discovery cap to retained idle
roles: their root may still be making progress on another branch. Reclaim a
role only on explicit terminal/cancel control, revocation/shutdown (including
leave/reset), or the entry-pressure eviction rule below. A larger connection
allowance neither raises the four-entry bound nor creates cross-job reuse.

Apply entry-pressure eviction only when registering a distinct run would exceed
the four-entry bound. First reclaim roles already explicitly ended. Otherwise,
choose the least recently used eligible idle role, with run ID as the stable
tie-breaker. Age orders candidates; it never expires one by itself or supplies
reachability evidence. Do not evict a locally executing root, an attached active
control session, an outstanding assignment/transfer, publication or a role with
owned work or delivery leases. If no role is eligible, return a typed capacity
failure for the new run without disturbing existing work. Never evict metadata
merely to free a connection; releasing an idle transport preserves its role.

Fence an evicted role atomically before admitting its replacement: end its old
authority, discard its duplicate-control state and prevent delayed work from
serving sources or acquiring another phase worker. Keep durable originals,
tombstones and accepted publication unchanged. Return expired/unknown for an
attempted continuation; the initiating job reports the named affected branch
incomplete or performs only its already permitted bounded failed-branch recovery.
Eviction is neither peer absence nor a successful acknowledgement and does not
reset discovery, direct-route attempts, reopening budgets or the relay-refresh
allowance. Dropped state returns expired/unknown rather than fabricating phase
completion. An explicitly continued Begin performs lookup only and never creates
a missing run. Mark a
terminal-only continuation in the ordinary Begin exchange and reject source
streams, assignments and reconciliation through it. Dropping idle metadata
never restarts a job or fabricates an acknowledgement. On termination,
deliver the final result to the initiating RPC and discard the run's entries,
connections and observations. Do not retain a terminal registry for later jobs
or status calls. A disconnected orphan may retain only bounded active-run
cancellation/drain state until its lifecycle finishes; it is never a reusable
completed-job cache. This is within-job pressure handling, not cross-job reuse.

#### Lifecycle, security and reporting

Preserve manual-only passive serving: an enrolled daemon advertises bounded
presence, maintains relay connectivity and accepts explicit authenticated jobs
while its CLI is idle. Local writes, receipt of records and newly discovered
addresses never initiate a root job. Optional coordinator reads retain their
five-second per-request timeout and verified-cache fallback only on service
unavailability. Membership mutations retain their own uncertain-outcome rules.

Verify frozen signed job authority and current membership on admission and each
source request. Bind replica sources to the authenticated parent/child role,
run, round and active transfer phase. Recheck current authority on every
original-byte chunk. Commit pending groups before publishing after 256 changed items or 4 MiB of
changed envelopes, and on successful or failed transfer completion. The received counter
tracks durable new applications to this store immediately, survives a later
branch failure and does not count duplicates again on retry.

Keep control reading, heartbeat writing and cancellation independent of contact
and transfer work. Root cancellation or loss of an active parent session cancels its delegated
work, releases assignment/transfer slots, closes item sources and completes
or recovers accepted publication. Cancellation must not discard accepted publication or claim final distribution
that did not finish.
Bound active-run bookkeeping and discard terminal state after result delivery as
specified below. Job scheduling
state is transient, not a second persistent store or permanent acknowledgement
cursor. After owner restart, another explicit sync reconciles original state.

Report direct-round completion, current expansion level, reached peers, pending
contacts, final distribution and local applied items separately. Include direct/
relay route outcomes and reuse from the `(relay, target)` outcome map. Separate
confirmed relay absence, no known direct address and unresolved timeouts. Full
cohort completion requires every frozen member's final acknowledgement. When
all supported reachable peers finish but some members are explicitly offline,
report partial cohort coverage and preserve JSON `complete: false`; do not claim
the offline members participated. Unknown, cancelled or failed branches remain
explicitly incomplete, never evidence that all reachable peers were synchronized.
After every eligible direct source/target route has been tried and settled
(or a source explicitly reports no known direct route),
report an unreached peer as offline when its relay absence was confirmed by a
validated correlated not-registered reply for a current attempt in this job and
that evidence remains valid. Direct timeouts alone do not prevent this offline-only
partial result from exiting successfully. Never derive it from EndpointGone,
an old-relay timeout, stale/invalidated evidence, an untried route or a contact
cancelled by the discovery cap. A losing direct contact cancelled on admission
also remains unknown if that target later needs recovery; do not count it as a
settled timeout when certifying absence. Any non-timeout direct failure, including version,
handshake or authentication failure, still fails the sync. Preserve that guard
through quiescent wakeup and final-round recovery; later timeouts cannot erase it.
Carry whether direct transport was enabled in a failed contact response so a
source with no configured direct transport does not fabricate a direct failure.
Use `offline` only for scheduler-certified absence. Map generic transport,
stream, missing-edge and local I/O losses to `unresolved`, not `offline` or an
elapsed-attempt timeout. Keep actual elapsed deadlines as `timeout`. Neither an
unresolved direct loss nor a missing parent final acknowledgement can create
partial success. Before final report normalization, downgrade every `offline`
outcome not certified by the current scheduler to `timeout`; an admitted peer
cannot be certified absent. Preserve non-timeout direct failures even if another
route subsequently admits the target. Distinguish a source-confirmed attempted
route, an explicit no-known-route result and an unknown source outcome. A lost
forwarding/control path to a source is unknown, not proof of either a target's
direct failure or no known address. It blocks absence certification; actual
acknowledged recovery of the target can retire that routing failure. A successfully recovered data-phase loss
may be retired only by real acknowledged coverage, under the existing recovery
rules.
Expected offline-only outcomes may exit successfully as partial; authentication,
version, conflict, protocol, capacity, publication, unresolved loss and timeout
failures remain failures. Preserve durable partial progress, name affected peers
and suggest another explicit run for changed topology; never start a repair timer.

#### Traversal validation and latency acceptance

Acceptance requires validated native relay outcomes for shared-outcome
scheduling and capped-discovery results.
The pinned [Iroh relay actor](../vendor/iroh/src/socket/transports/relay/actor.rs)
keeps unsolicited EndpointGone advisory. Surface only negotiated token-correlated
DialStatus observations as described above, with relay identity, source connection
generation and target. A previous connection or attempt cannot supply a current
negative, and the legacy advisory event cannot satisfy this acceptance gate.
Treat it as route absence, not cancellation of a working direct connection.
Retain normal verification and authentication. If a reliable observation cannot
be obtained, keep the outcome unknown and use the ordinary timeout/alternative
path rather than inventing negative evidence.

Validate the adapter with an absent destination, a working relay target,
a simultaneous direct success, source reconnect, stale/delayed EndpointGone and
relay congestion. Demonstrate prompt absent-route completion without changing
reachable-peer coverage or adding relay polling, enumeration APIs or keepalives.
Reconnect the target during an active dial and deliver an old disconnect notice
after that reconnect; neither event may supply a current negative. Test wrong
target/token/relay/generation replies and cancellation at subscription capacity.
Exercise both an older relay without extension acceptance and a negotiated
relay that sends no DialStatus: each retains the ordinary dial deadline and
unknown observation, while a current matching not-registered reply resolves
only its own attempt. Keep direct-success and reachable-target coverage matched
across these fallback cases.
Require matched small-mesh latency/traffic baselines for ready-first collection,
healthy connection reuse and shared relay scheduling. Connection reuse measurements cover phases of one job, never a
warmed job connection from another sync. Record whether Iroh endpoint path state
was cold or warm and report those measurements separately; a run with warm
Iroh paths is not a cold-path baseline.
Validate job-wide outcome reuse, delegated discovery, final reconciliation and
pressure-induced reopening against that baseline. Credit shared-outcome latency
gains only with validated native negative evidence; a larger protocol cannot
mask a broken transport observation. The normal binary advertises only profile 2 and always
applies the twenty-second discovery cap. Require the complete coverage, security
and lifecycle suite for that path; retain no rollout gate or recursive profile-1
implementation.

Test a mixed profile-1/profile-2 cohort: every observed profile-1 peer must appear
by endpoint ID with the update instruction in sync and checked-status human/JSON
results. Both commands return a real version failure, without record exchange,
recursive fallback or offline/partial-success masking. Ordinary idle status must
not retain the preceding command's compatibility results; fresh checked status
must detect an updated peer without a daemon restart or cached classification.

Validate overlapping L1/expansion, staggered attempts, transient final deltas,
group commits, page prefetch/end flags and byte-window fetching independently
against the baseline. Preserve FIFO valid-prefix behavior and final coverage
for runs not cut short. Preserve the fixed twenty-second cap, two-millisecond group
flush timer, 750-millisecond stagger, byte window and delta count/byte bounds
documented here, and rerun design-target matched-coverage measurements when
changing them. These
are internal constants, not runtime knobs. Do not change publication thresholds,
secure-forget or transfer-slot limits to obtain a timing win.

Use a fake monotonic clock and scripted asymmetric connectivity. Assert at most
two staggered assignments per missing target with unknown relay state and eight
contacts job-wide. For a correlated Gone target, verify twenty reached sources'
direct routes settle in three batches of at most eight, without stagger, while
unknown targets retain the two-assignment rule. Verify no-address sources return
NoDirectRoute without dialing, sleeping-peer acceptance at 5/10/20 devices exits
successfully as offline-only partial well inside the cap, and admitted mid-round
transport loss without correlated absence fails the command. Check route
deduplication, fair consideration of every eligible route and exactly one winner
under simultaneous accepts. Delay a preferred source and verify a second starts
after the fixed 0.5–1 second delay without waiting for the first timeout. An older
or fourth gateway stays eligible until discovery expiry; never omit an eligible gateway before then. Verify that expansion
starts on first-contact failure during a deliberately slow L1 transfer, and
new island originals/tombstones reach earlier peers in final redistribution.

Exercise 5, 10 and 20 devices across a few networks as the primary latency
workloads. Check correctness at 256 devices, including a chain longer than 16
edges, fan-out under the membership-scaled connection limit, four jobs, idle
eviction and pressure-induced reopening. Assert `C(1) = 32`, `C(5) = 32`,
`C(10) = 44`, `C(20) = 84` and `C(256) = 1028`. Exercise concurrent inbound and
outbound reservations, probes and losing duplicates against the same counter;
never over-admit or exceed the ceiling. Verify that unsigned/proposed or stale
membership cannot raise the target, verified growth unblocks queued work and
verified shrink drains idle connections while a progressing authorized transfer
finishes. Keep handshake concurrency at eight and transfers at two per direction.
A single 256-member star and four concurrent wide trees must not churn healthy
connections merely because their retained count exceeds 32. Force actual pressure
at `C(N)` to validate eviction/reopening, rather than retaining an obsolete
32-connection test limit or adding a runtime cap override.

Retain an idle role through a healthy transfer elsewhere lasting longer than
60 seconds; verify successful same-run continuation without an expiry timer.
Exercise all four metadata entries, admission of a fifth run, deterministic idle
entry eviction and typed capacity when all roles own active work. Evicted roles
return expired/unknown, cannot replay sources or revive phases, and never become
offline or complete. Verify explicit terminal/cancel, revocation and shutdown
reclaim roles; stalled orphans remain within the count bound, without original
bodies, live job connections or terminal-result reuse in another job. Release an idle edge after its final downward reopen, then finalize
or cancel it using exactly one additional control-only reopen. Verify that this
allowance cannot request inventories or records, repeat terminal work, revive a
finished job or extend any deadline, including after discovery-cap expiry. Include cliques, stars, islands, directed paths and parent loss.
Keep corpus and measurement artifacts out of commits.
Run local relay-backed native acceptance with `npm --prefix relay run
test:native` inside `nix develop`; keep cold/warm product timing under
`test:sync-bench` separate from the correctness suite.

For non-negotiating relays, verify ordinary timeout/unresolved behavior.
For correlated replies, verify prompt relay-confirmed-gone handling and one shared map
keyed only by relay/target on each node, with exactly one configured relay per
node. Reject multi-relay configuration instead of testing relay selection.
A direct success is not relay health and relay success to B says nothing about
C. Test stale/delayed EndpointGone, reconnect, congestion, source failure and
bounded evidence invalidation. Direct discovery to a deeper island must survive
relay absence. Do not add relay polling, enumeration APIs or keepalives.
Verify that fresh verified target presence permits exactly one relay-route
retry, including a previously tried route, only while discovery remains open.
Repeated target events and source-only reconnect must not grant another retry.
Ordinary exhaustion remains quiescent until an eligible event or actual failed
branch requires work; cancellation, finalization and discovery-cap expiry close
the window permanently and must not grant or restart a retry. Direct-route
deduplication remains unchanged, and invalidated stale absence is unresolved
when no retry is permitted.

Verify that the discovery-cap clock does not start during membership refresh
or restart speculative dial deadlines. Expire discovery during dials,
admission, expansion and final-round recovery route search. Only pending/new
contacts stop: admitted L1 transfers, a committing/filling group pair, publication,
final delta/full distribution and acks must continue beyond discovery expiry
under unchanged progress deadlines. Verify that a slow healthy admitted transfer
finishes and a stalled one still fails normally. Partial output names unreached
peers after admitted work settles. Unknown timeouts do not become offline
proof. User cancellation retains valid-prefix commit/publication cleanup and
cannot require an offline parent's ack or let a losing dial cancel the winner.

Inject parent loss during final collection and distribution. Fence old work,
recover through an untried healthy parent in the same final round using full
inventories, and redistribute newly recovered contributions to peers whose prior
ack did not cover them. Check failed fencing, no alternate route and discovery
expiry:
all retain durable progress and return explicit incomplete-subtree results.
A replacement admitted before discovery expiry must finish even if its transfer
outlasts the cap.
There is no elective route optimization or new root job during recovery.

Test bounded/chunked delta versus full original inventories for identical IDs,
hashes, tombstones, final bytes and acknowledgements. Include overlapping L1
baselines, local untracked mutation, shared list/count/byte overflow, restart, mismatch,
missing originals, recovery and lost chunks. Every uncertain case falls back to
full authoritative reconciliation; a true immutable conflict remains an error.
Shared-list overflow and restart must force full inventories for every branch,
including branches with a partly sent delta; branch-local doubt need not disable
other branches. An empty/shortened list never certifies completeness by itself.

Test page-sized Needs against per-entry decisions with conflicts at the start,
middle and end; commit the valid prefix before returning its error. Exercise
one prefetched next page, early error, ordered end-of-list validation and exact
256-entry last pages with no empty follow-up request. Check staged/published
tombstones and duplicates, source changes, understated/stale sizes, malformed
framing, tiny items and one valid item larger than the byte window. Assert byte,
metadata/request-count and decoded-memory bounds, FIFO retirement and no later
Apply after an earlier failure. Tombstone processing precedes record prefetch.

Check group count/bytes/time flushes, sparse arrivals, timer firing during a
slow network read, error/Finish/cancellation boundaries and commit failure.
Assert that group commit/flush does not stop fetch credits when the filling group
has capacity, only Finish boundaries pause credits, and at most one committing
plus one filling group exists. Discovery expiry
does not interrupt either group.
Crash between staging and group commit and between commit and publication:
recover committed staging and refetch at most two uncommitted groups per stream
idempotently. No count, ack or accepted result precedes commit. No SQLite write
transaction/store lane is held while waiting for a timer or network. Preserve
FULL durability and unchanged publication boundaries.

Forget a record while it is prefetched, decoded, buffered, in a pending group,
in a delta or mid-source-chunk. Drop/invalidate every local body copy before
forget acknowledgement and prevent late replies from repopulating it. Exercise
revocation and cancellation under a full byte window and slow Apply/Finish;
every credited response has capacity and idle waits do not introduce false
publication timeouts. Existing byte preservation, conflicts, publication recovery
and query independence remain required.

Test cached current correlated replies before verified adoption, replies arriving
after adoption, and replies racing the subscription boundary; each assigned
contact consumes its valid evidence once. Reconnect the observer before
adoption and before draining a buffered reply, and verify stale generations are
unresolved. A discarded dial forwards no evidence to its fresh verified attempt.
Learn a direct address during readiness, force a concrete non-timeout direct
failure, and verify the contact reports an actual direct attempt; later admission
through another source must not erase that failure.

Test speculative transport dials during refresh success/denial, verified-cache
fallback, identity/service change and reset/leave. No admission or original byte
exchange precedes the verified membership gate. Verify request-scoped HTTP clients
without pooling or cross-request reuse, and no cached signatures,
responses or mixed sync/mutation timeout policy. Run consecutive syncs and
assert fresh peer sessions, empty reachability/offline/version/outcome maps and
no last-job status. Verify independent jobs cannot adopt each other's warm
connections or terminal results. Durable originals and membership still survive.
Allow retained Iroh path state across those runs and concurrent jobs, but verify
that it never skips or shortens a contact attempt, supplies absence evidence or
changes peer eligibility. Do not require per-job isolation of Iroh path state.
For status checks, verify a fast peer's result streams before a sleeping peer's
timeout, with bounded parallelism and honest final unknowns. Retain presentation
cleanup and the single final machine-readable result.

Measure passive endpoint startup separately from each sync job. Record whether
each run has cold or warm Iroh endpoint path state and report the two separately.
Do not use a warm-path run as the cold baseline. Every run uses fresh job
connections, reachability/outcome maps and status, independently of retained
passive Iroh paths. On independent
stores, measure unchanged, small-change and initial sync with all peers online,
one sleeping peer and a two-island gateway topology; include loopback, delayed
relay-only and relay-unavailable direct paths. Record first useful transfer,
first completed L1 round, total completion, coordinator/readiness overlap,
contact/admission counts and time, store queue crossings, Needs transactions,
Get request/reply gaps, commit/fsync frequency, group flush latency, publication
time, delta hit/fallback rates, peak byte-window buffers, bytes and final coverage.
Use injected network delay to establish whether byte-window fetching and next-page
prefetch reduce RTT stalls; report throughput and first-item latency separately.
Compare with the pre-change recursive baseline on identical topology and data;
keep that baseline outside the product source. Assert fake-clock
per-attempt deadline bounds exactly; report real elapsed distributions and scheduling overhead
rather than claiming exact wall-clock deadlines from a single run. A no-change
run must neither scan/annotate unchanged bodies nor download a model. Do not
claim WAN or full-corpus validation from local fixtures.

Compare uncapped/completed runs at identical final coverage and workload.
Report discovery-cap partial runs separately,
with named missing peers, coverage at discovery expiry and final coverage after
admitted work settles. Measure discovery duration and total admitted-transfer
completion separately; a short discovery cap is not a command-time bound or
faster full convergence.
Measure the deliberate cap latency/coverage tradeoff without hiding skipped
reachable peers in aggregate timing numbers. Split traffic into original bodies,
inventory IDs/hashes, membership proofs, job control and relay framing; include
session handshakes and source-side store reads. Measure all-online no-change
latency for regressions from extra root coordination and pressure-induced reopened sessions. Record
root bandwidth and CPU separately: a shallow tree can shorten paths while making
the initiator a bottleneck. Measure peak combined connections, idle transport memory,
retained role count and pressure-induced handshakes under `C(N)`, including four
jobs at the 256-member ceiling. Compare the former 32-connection baseline at
matched final coverage; report fewer evictions/reopens as measurements only when
observed, not as proof of faster full convergence. Connection-cap arithmetic is
a resource bound, not a latency or traffic measurement. Require source/target/route deduplication, conclusive
outcome reuse through the `(relay, target)` map at L2 and beyond, and elimination of per-level scans in addition to any elapsed improvement. Measure
purely direct adversarial topology separately from the common relay-connected
case; do not claim a linear total-contact bound for both. Do not promise lower
total traffic than a single-pass recursive run without those measurements.
Optional coordinator refresh remains a separate possible five-second stall;
transport startup may overlap it under the explicit startup gate above, but
coordinator latency still gates cohort selection and original exchange. Keep
measurements and corpus/model artifacts out of commits.

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

Only approved, non-self endpoint IDs with mDNS discovery TXT profile version 1
and nonzero ports enter Linn's address lookup, with at most 32 IPv4 addresses
per approved endpoint. The mesh ALPN/job profile remains 2. Prioritize loopback within that cap. Re-browse cached records when
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
Do not pin the serialized store lane on delivery of metadata-only replies,
including membership KnownEvent/proof decisions, inventories, Needs and Finish.
Their consumers can remain unpolled while the same control owner awaits another
store operation. Release their delivery wait after committing the operation and
queuing its reply. Retain the existing original-valued Get and authorized chunk
delivery barriers; metadata completion is not original-byte delivery.
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
admitted reconciliation retains its existing progress deadlines.
The twenty-second discovery cap starts after membership refresh and only bounds finding/waiting on
peers. Its expiry never cancels progressing admitted transfer or publication;
explicit cancellation and actual failure follow durable-prefix cleanup. Network backpressure can delay
forget within the bounded chunk-write lease.

Reserve fair execution for local commands. A failed peer does not stop
unrelated peers or local ingest/search. A partially completed exchange remains
retry-safe; publish successfully accepted batches before reporting failure.
Classify auth, version, conflict, publication, timeout, unresolved loss and certified offline failures
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
only on human-terminal stdout under LLP 0009#presentation-policy; plain
stdout remains the raw token. JSON remains one
typed result. Notices and instructions go to stderr. Peer IDs identify
account-owned store instances, not machines.

Sync progress uses stderr and the shared presentation/cleanup rules in
LLP 0009#transient-progress. Peer collection/distribution counts include this peer
and show cohort denominators. Include the round/level, direct-round completion,
reached count, pending assignments and the `discovery_expired` window state; distinguish prior-round distribution from
final-round coverage. Publish snapshots after each assignment outcome and durably committed peer
transfer, including distribution to this peer. Label received items “changes
applied here” to distinguish them from outgoing edits. Completion appears once
on stdout; JSON retains the structured report without a second human completion summary. Applied-change
counts describe this store's records and deletions, not mesh-wide transfers.
Zero local changes must not be described as a mesh-wide no-op. Incomplete text
reports name missing peer phases, source-specific failures, unattempted peers,
discovery-cap expiry, typed failure categories
and recovery actions,
and explain durable partial progress. Membership-operation results explain the
outcome, affected peer/invitation where available and whether a first sync ran.
When no first-sync result exists, suggest `linn sync` without claiming that records
have never synced; the result describes this command, not transfer history.

`sync:status` remains local by default, explicitly marking remote reachability
unknown. Optional `--check` tests transport connectivity to known remote peers,
with at most eight concurrent checks and a ten-second probe deadline. Emit a
typed progress result for each completed peer immediately over local IPC; render
human results as they arrive instead of withholding them behind the slowest peer.
Retain a final aggregate result when all checks finish or the deadline expires;
mark uncompleted peers unknown. Follow LLP 0009's stderr/progress cleanup and
single final JSON-result rules, without duplicating completed lines on terminal
stdout. Queued or unfinished peers remain unknown; a connection failure is reported as unreachable
at check time. This is an authenticated endpoint transport snapshot, not proof
of membership authorization or successful record exchange. No records transfer,
no membership refresh is requested and no mesh job begins. Reachability results
are returned only for checked status, as an optional JSON `reachability` map;
ordinary status JSON is unchanged. The additional local check-status request is
introduced in application compatibility 6 and retained in compatibility 7;
checked status does not negotiate a mesh job or transfer records.
Checked status does negotiate authenticated transport/ALPN compatibility and
reports identified profile-1 peers as named `update linn` version failures, not
as offline checks. Its aggregate command fails if any such peer is observed.
Ordinary local status can show version failures only from an active job; after
a job ends it reports remote compatibility/reachability unknown until a fresh
check. Never cache a prior sync's failure to supply later idle status. Preserve
that distinction in human and JSON output.

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
  admitted, returns a partial report (successful exit for unavailable-only peers,
otherwise failure) and can be retried with
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

`sync:status` reports enrolled membership, local transport readiness, active manual job outcomes only (no retained last-job status), local bound addresses, relay/discovery readiness and
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
recovery. Native clients send WebSocket control health pings after 30 seconds
without an incoming relay message. Periodic checks allow 15 seconds for the
matching Pong; immediate network-change checks retain Iroh's adaptive RTT-based
deadline. Blocked relay writes retain their independent 15-second timeout.
Silent idle connection loss can therefore take
up to roughly 45 seconds to detect; active peer QUIC deadlines remain separate.
Repeated failed relay connections use exponential backoff starting at one second
and growing to a nominal 60 seconds, plus additive jitter up to another 60 seconds.
Queued outbound relay traffic and detected or explicit network-change checks can
interrupt this wait after the one-second minimum pause between failed attempts.
Wakeups coalesce and do not reset the exponential schedule; a full outbound queue
also signals demand before waiting for capacity. Priority route queries and
cancellation remain responsive, and no additional relay is configured. Loss of an
established connection retains its immediate first reconnect. Keep healthy relay
connections open for incoming remote jobs. mDNS/direct local transport continues
independently; discovery hints alone never suppress relay presence. During a quiet
outage, incoming remote reachability can take up to roughly two minutes to recover
if no local demand or network-change signal arrives.
Do not suppress health checks or invent replies. Native endpoints configure the
single service relay without QUIC address discovery, HTTPS latency probes or
captive-portal checks. The Worker has no QUIC address-discovery listener, and a
single relay needs no latency ranking. The pinned Iroh patch selects the sole
configured relay when HTTPS probes are disabled; the existing WebSocket handshake,
authentication, connection-status reporting, reconnect backoff and control health
checks establish actual reachability. Multi-relay selection is outside the
permanent one-relay-per-node design. Direct peer IP transports remain enabled.
The relay retains `/ping` for older clients. The relay sends no periodic
idle application pings. It sends random application Ping checkpoints when the
unconfirmed window reaches 64 KiB; matching application Pongs alone release the
checkpoint's earlier bytes. After a matching Pong, immediately checkpoint any
remaining unconfirmed window of at least 64 KiB. Before dropping a datagram that
would exceed the window, ensure a checkpoint is outstanding. This also recovers
older attachments with a full window and no outstanding checkpoint; recovery
must not depend on another successful data send or periodic idle alarm.
The 64 KiB threshold retains headroom for in-flight data. Local 8 MiB echo
spot checks with a 100 ms round-trip TCP delay compared 16, 64 and 128 KiB:
64 KiB reduced checkpoint replies with similar elapsed time; 128 KiB further
reduced replies but was slower. The 256 KiB cap and 30-second outstanding
checkpoint timeout remain unchanged. These are local transport measurements,
not production billing or WAN validation.
Control Pongs cannot release this window. Arm an alarm no later than the earliest
pending handshake or outstanding checkpoint deadline, retaining an already-earlier
alarm. Acknowledgements and successful authentication do not move or delete it.
When it fires, reconcile current socket deadlines and rearm only if any remain;
a stale earlier alarm must never send idle pings or close a not-yet-expired socket.
This allows one trailing wake after traffic, not a recurring idle loop. Closure
can remove an alarm when no deadlines remain. Deadline computation and alarm
updates share a storage transaction, and hibernation does not postpone deadlines. Older idle `pingAt`
attachment fields are ignored. Hibernatable attachments preserve identity,
deadlines and budgets. These limits bound application queues, not all platform buffering
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

Received Apply items may commit to the private incoming table in bounded groups
within one publication batch, under the count/bytes/time and prefix rules above.
Commit before Finish, acceptance acknowledgement, received count or error return.
A staged-but-uncommitted item is not accepted; a crash requires refetch of at
most two uncommitted groups per receiving stream (one committing and one
filling), using idempotent originals. Replica
inventory/get/needs include these accepted originals, preserving idempotence,
conflict detection and tombstone precedence before publication. Ordinary queries
continue using the preceding complete publication. Mesh transfers retain their 256-item/4 MiB publication
bound. Finish consumes durable staging in pages of at most 256 items or a 4 MiB
prefix, allowing one independently bounded item to exceed that byte target.
Validated file imports perform one bulk publication rather than repeatedly
rebuilding the whole store for each input page. Additional working memory for
pending groups and staging is bounded; durable accepted originals awaiting publication remain
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
Iroh relay datagrams and local IPC. Mesh profile 2 uses length-prefixed bounded
JSON on authenticated QUIC streams with a 16 MiB frame limit. Retain inventory,
get and finish requests on source streams. Inventory requests carry the item
class and exclusive cursor, and Get requests carry the item ID. Keep Needs and
grouped Apply as
typed local store operations. Remove the generic replica-request dispatcher and
its unused wire Needs/Apply/Entries/Needed/Applied forms. The public stdio
protocol and its handshake are removed. Negotiate the mesh ALPN before exchanging data.

Every session negotiates compatibility before original-data transfer. The Hello
includes the caller's locally verified membership head. An equal head lets the
receiver authorize the endpoint against its current local state and return
Authorized immediately; the caller still checks local authorization. Otherwise
Ack requests the full verified proof exchange. A claimed head never grants
membership. This negotiation adds no Worker request.

Profile-2 inventory pages contain at most 256 IDs/hashes with encoded item byte
sizes, ID-ordered with an exclusive keyset cursor and explicit end-of-list flag.
One next page may be prefetched under the bounds above. Tombstones precede records; send only missing bodies. A conflicting
hash fails without transferring replacement bytes. Transmit one envelope frame
at a time; the byte-budgeted FIFO pipeline above overlaps bounded reception,
ordered Apply and grouped staging commits. The mesh adapter revalidates and leases each bounded chunk under the
protocol resolutions below. Send original frames only through registered-buffer
delivery; metadata framing and control actors retain no store handle. Release
before peer application acknowledgement
waits. Oversized items fail rather than truncate. Malformed, oversized or
truncated frames and untyped replies fail explicitly. Unexpected EOF fails the
exchange; explicit finish and failure cleanup publish accepted pending items.
Inventories are bounded database reads, never permanent in-memory corpus
copies. The mesh transport does not raise the record/frame limit or change item
hashing implicitly.
Reset a failed replica stream with its typed application category without
closing an otherwise healthy job-control connection. A control/connection
failure still fences its owned work. Profile-2 delta inventory requests carry
class and exclusive cursor; delta entry replies carry exact IDs/hashes and an
end flag. An explicit unavailable reply selects full original inventories, never
an empty successful delta. Ordinary Get/Needs/Apply validation and Finish remain
authoritative in either mode.

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
`authentication`, `version`, `conflict`, `publication`, `timeout`, `offline`, `unresolved`,
`protocol`, `cancelled`, `capacity` or `hop_limit`. Failures identify their source
category. Preserve that category through local RPC and delegated reports without parsing
error message text. Schema/ALPN mismatches are version failures,
signature/membership denials are authentication failures, immutable-content or
signed-history forks are conflicts, and store/publication failures retain their
publication category. No-progress deadlines are timeouts. Sync discovery marks
a route offline only with current correlated relay absence; unconfirmed contact
timeouts and discovery-cap omissions remain unresolved.

Mesh profile 2 retains QUIC application codes `0x4c00` through `0x4c09` for
the existing categories and uses `0x4c0a` for `unresolved`. Only the scheduler
certifies `offline`; transport receipt of an offline application code remains
unresolved until independently certified. A failing control stream is reset
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
rejecting duplicate properties and unpaired Unicode surrogates in keys or
string values before canonicalization) and uses lowercase hex keys/hashes/signatures,
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

Manual job profile 2 retains Begin/Accepted for registration/status and adds
Assign/Contacted for bounded target delegation, Reconcile/Collected and
Distribute/Distributed for the two data rounds, and Finish/Completed for terminal
completion. Use the existing Progress heartbeat and owned-stream cancellation.
Forward changed application-progress revisions promptly toward the root, coalesced
to at most one activity update per 100 milliseconds per control stream. Keep the
ordinary ten-second heartbeat for parent-to-child progress; do not broadcast
every root revision promptly to all retained siblings. An unchanged revision
never renews a no-progress deadline. Do not accumulate a ten-second forwarding delay at each tree level,
which would falsely time out progressing deep branches. Parent-to-child progress
must not be echoed as new child activity.
Do not add an assignment acknowledgement solely to mirror local queue ownership;
the initiator owns the slot until Contacted, timeout or cancellation.

Begin binds the signed run identity, initiator, frozen head/cohort, protocol
version and maximum forwarding depth. An assignment carries a run-scoped ID,
source, target, route policy (direct-only, relay-only or direct/relay), parent
path and per-attempt duration. Record the actual routes attempted so a mixed
attempt cannot be scheduled again under another policy. The initiator records its
ownership before sending it. Contacted returns that ID, actual direct/relay path evidence and an authenticated
admission result or typed failure. Preserve explicit relay target absence as a
route outcome distinct from timeout; identify the relay service and observing
source so job-wide reuse is scoped correctly. A source returns NoDirectRoute
for a direct-only assignment with no configured/known direct address, without
dialing. Combined relay failures carry whether a direct route was actually
eligible. Traversal snapshots retain the distinct no-known-direct source/target
pairs, bounded by the same `N * (N - 1)` ledger, and validate both IDs against
the cohort. Authenticate every link and accept scheduling
commands only from the registered parent in this signed job. As with existing
trusted-member forwarding, an intermediate is trusted to carry the scheduler's
instructions; do not add per-assignment root signatures or cryptographic path
attestations. An authorized peer can already initiate its own explicit job.
Validate cohort membership, expected source/parent roles and hop bounds, reject
conflicting ID reuse, and return cached results for duplicate commands without
starting duplicate work. The signed job remains the authority boundary; compact
scheduling messages do not relax current membership or source authorization.

Reconcile identifies the round number, receiving peer's expected child roles
and whether upward collection is required. A branch reusing L1's contribution
acknowledges that reuse without opening another upward inventory; it still
participates in final distribution. Parent commands are authorized by the
registered tree. Rounds cannot overlap or replay earlier item sources.
In [source authorization](../src/mesh/peer/source.rs), bind each source stream
to its control-operation ID, round, authenticated reader and currently recorded
connection. Install downward source authority while the
immediate parent holds the child edge lease; keep that lease through the durable
child reply rather than exposing an unpinned source in a separate exchange.
Carry the operation ID and delta/full mode in the replica-stream opener. Permit
one delta stream and at most one full-inventory fallback under that operation;
a full fallback atomically cancels the old delta stream and its queued reads,
including when its FIN/reset has not arrived. Never permit a second delta,
second full fallback, another connection, or an earlier round to reuse the grant.
Revoke source authority on operation cancellation/completion, parent loss,
recovery, terminal cleanup or role eviction. Fence queued Get/chunk work and
late store replies, erase registered source buffers, and leave receiving valid
prefix commits under the existing publication/cancellation contract.
Collected acknowledges durable upward merge for that round. Distributed
acknowledges durable downward publication and child completion for that round.
In [final reconciliation](../src/mesh/peer/coordinated/reconciliation.rs), Finish
fixes final coverage and prevents new assignments/rounds. Require each retained
child's terminal acknowledgement before acknowledging its parent. A lost terminal
acknowledgement removes that target's and its ancestors' final coverage and
remains unresolved, including after report normalization; it is not relay absence.
Completed returns the final report; early round completion is never terminal job completion.
After writing a terminal reply, finish the control writer in order with a QUIC
FIN before dropping its polling tasks. Do not reset a buffered final reply by
aborting a newly started progress write during teardown.
Bound all sets by the frozen 256-peer cohort, paths by at most cohort size minus one edges, assignments by
distinct source-target-route entries by the finite cohort/relay ledger, data
rounds by two (L1 and final), and messages by the existing 512 KiB control-frame limit.
When no new committed contribution or uncovered peer remains, reuse the completed
L1 coverage. Never bypass pending
publication or acknowledge coverage merely because the participant set matches.

A job starts direct contact/reconciliation with overlapping assigned expansion
and upward contributions, then moves to final redistribution if needed,
finalization and terminal complete/failed/cancelled.
Within a round retain collecting, collected and distributing transfer substates;
initial L1 work overlaps explicitly assigned discovery and upward contributions.
Final redistribution follows completed initial baselines and ordinary discovery
becoming quiescent, with bounded failed-branch repair inside that final round.
Keep the recovery window open until finalization, cancellation or discovery-cap
expiry; exhaustion alone closes no recovery window. Recovery retains the same
finite route ledger, one relay-refresh allowance and original cap clock. It does
not restart ordinary discovery or authorize another sweep of already tried routes.
Do not retain the profile-1 assumption that Collected/Distributing happen only
once in a run. Keep the daemon's four-job limit, reserved local-control slot,
bounded active-run entries with no terminal retention after result delivery.
The signed depth bound is cohort size minus one; a chain must not be cut at
16 edges when more stable reachable members remain.

Use mesh ALPN/job profile 2 for these incompatible semantics and reject profile
1 with a typed version failure; do not silently fall back to recursive fan-out.
Report profile-1 incompatibility as a real typed version failure in sync results
and fresh `sync:status --check` results. Name every affected peer endpoint ID and
give the actionable instruction `update linn on peer <peer-id>, then retry`.
Include the affected IDs and update action in machine-readable diagnostics as
well as human output. Do not collapse it into offline, unknown reachability or
partial-success outcomes, and continue checking other peers so the result names
all observed profile-1 peers rather than only the first. Do not exchange
records, delegate jobs or fall back to recursive fan-out with these peers.
Unknown protocol versions receive an equally explicit unsupported-version update
error without inventing a profile number from an ambiguous transport timeout.

Use local RPC compatibility 7 for the structured round/assignment progress,
including no-known-direct-route source/target pairs, and
expanded partial-result reasons. Signed membership and original-item formats
remain unchanged. The specified level protocol supersedes the recursive
profile-1 traversal. [The peer implementation](../src/mesh/peer.rs) and its
job/control/progress types implement profile 2 unconditionally; retain old
profile-1 behavior only as an external compatibility-rejection fixture.

## Bounded forwarding and duplicate control

Sign the frozen cohort size minus one as the maximum forwarding depth of each
profile-2 job, bounded by 255 edges. Paths start at
the initiator, contain unique cohort endpoints and end at the authenticated
sender. Check current membership as well as the signed frozen head. Register
before accepting assignments or serving sources. Compare duplicate run IDs by
immutable signed bytes, independent of arrival path; reject conflicting reuse.
Retain a healthy first accepted parent; only confirmed failed-branch recovery
can replace it. Skip ancestors and never wait recursively on a duplicate caller. Duplicate acceptance reports the existing parent/phase so
the initiator can account for the existing route without claiming a second one.
If registration survives a lost acceptance, repeat only idempotent control on
the existing connection; it does not authorize another target dial or a second
parent. If that connection is lost with work pending, report uncertain coverage and
stop its work. An acknowledged idle release follows the bounded reopening rule
above; it neither retries discovery nor changes the registered parent.

Preserve the typed terminal fault of each live control session even if it had
no pending request when it failed. Subsequent queued requests return that fault;
a protocol/authentication/publication failure must not become a generic offline
or timeout result merely because the control actor has already stopped. Discard
the fault with that job-owned session.

Track assignment and round deduplication within the bounded run registry.
Repeated controls return cached/in-progress status for their own ID and round;
they never restart discovery, refresh a deadline, reapply a completed phase or
open a source outside its current authorized role. Reject a future/out-of-order
phase with a typed protocol failure. Keep independent control polling active
while work runs. Loss of an active parent session cancels its delegated assignments and transfers,
releases leases and repairs accepted publication. Expired assignments and
terminal runs cannot be revived by delayed frames. After owner shutdown all
bookkeeping is discarded; original-item reconciliation remains idempotent, but
an old round cannot resume without registering a fresh explicit job.
