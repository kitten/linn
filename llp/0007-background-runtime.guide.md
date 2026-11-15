# LLP 0007: Background runtime

**Type:** Guide\
**Status:** Active\
**Systems:** API, CLI, Runtime, Store, Sync\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0004, LLP 0005, LLP 0008

## Intent and status

An automatically started, detached local Linn process retains initialized
models and reusable store resources across CLI invocations. The CLI, runtime and
library callers share a typed application API. Keep one native Rust product binary and local-file libSQL as
the only persistent store.

The Unix implementation follows this contract. Native Windows delivery is
specified separately below and remains unimplemented; no Windows support is
claimed. LLP 0000 and LLP 0004 include this local daemon; LLP 0008 owns replication
and mesh synchronization.
There is no foreground `linn serve` command, manual startup requirement or
supervisor configuration to install.

Mesh synchronization extends this runtime under LLP 0008. The owner maintains
passive peer transport and schedules no automatic record exchange.
Mesh synchronization uses explicit `linn sync` jobs and `linn sync:pair`
enrollment. Owners may maintain passive transport readiness and serve those jobs;
local writes, timers and reconnects do not start record synchronization.
Pairing succeeds after verified admission is stored locally; passive transport
startup is scheduled independently and cannot turn that admission into an error.
It waits for neither peer acknowledgements nor a sync job (LLP 0008).
Automatic record propagation is not enabled. Retrieval algorithms, arithmetic, thresholds, card order, original-text
ingest and accepted model identities remain unchanged under LLP 0001, LLP 0002
and LLP 0005. Runtime changes do not reopen closed retrieval studies.

## Application boundary

The library application layer owns reference resolution, cursors, lifecycle
clocks, input provenance and partial-commit errors. [CLI execution](../src/cli/mod.rs),
[bundles](../src/cli/bundle.rs), [listing](../src/cli/listing.rs) and
[mesh commands](../src/cli/mesh.rs) prepare typed requests and present results.

The application layer accepts typed requests and explicit caller context, and
returns typed results or errors. It owns operation semantics and uses storage,
retrieval and sync domains. It must not depend on Clap, terminal rendering,
process-global cwd, inherited client environment, or socket framing. Direct
library execution and IPC dispatch call the same operations; no second command
implementation is maintained for the background process.

The CLI owns argument parsing, store/configuration resolution, Git-derived caller
scope, local input/output files and stdin, presentation, and exit-code mapping.
Resolve these per invocation using LLP 0004 precedence, including XDG validation.
Send original input bytes and explicit provenance rather than asking the server
to resolve client-relative paths. Keep operation-specific scope filters separate
from caller scope. Capture a fresh operation clock for each request; preserve
cursor-pinned context on pagination rather than using daemon startup time.

For add, use a typed Note/File input task, not client-constructed record kind or
source JSON. The client snapshots file/stdin bytes and captures absolute file
path and native repository root for a File task. The daemon derives relative
source path, input kind and source metadata. File source references reuse the
body SHA-256 under LLP 0001; no Git subprocess, cwd or environment payload is
needed. It does not
re-read the live file, which may change while the request is queued. Keep caller
repository discovery and file configuration lookup in the invoking process;
moving them to a resident process would require recreating caller cwd/environment
semantics. Never mutate process-global Git search paths or environment to do so.
Store/config paths, local input/output I/O and presentation remain client tasks.

Mesh request validation is shared application behavior: check credential content
and size, IDs and incompatible pairing modes before store/network mutation,
including requests from non-CLI callers. The CLI owns descriptor/privacy checks
for bootstrap files; it sends validated bytes rather than a file path.

The daemon allocates sync run IDs and sends typed admission/progress events on
the existing request connection. Use a two-event queue, dropping intermediate
sync snapshots when full; admission is queued before snapshots and drained before
the final result. Event writes have the ordinary response deadline. A slow or
closed client cannot create an unbounded queue or block the store lane on progress.
Clients render events and do not poll status to follow their own command.
Mesh metadata replies drain outside the store lane, retaining bounded delivery
waits for shutdown. Original-data replies and peer chunks retain their existing
serialized delivery barriers.

The [shared reference encoder](../src/reference.rs) constructs canonical fetch identifiers for cards and pointers;
search references retain the 12-character hash prefix and exact byte spans.
CLI adapters render these identifiers without reconstructing their syntax.

Results carry data, not preformatted terminal text. Text and JSON adapters
preserve existing output, diagnostics and exit codes. Structured errors retain
whether original data committed, whether publication failed, and the applicable
recovery instruction. Streaming adapters preserve original bytes and existing
bundle validation; malformed or incomplete imports must not become partial
imports merely because input now crosses IPC.

## Selected platform stack

Use `interprocess` with its Tokio adapter for reconnectable local IPC. The
implemented backends target Linux and macOS; its Windows backend is the selected
transport for the subsequent native Windows delivery. Use `daemonize-next` for Unix detachment in a fresh internal
bootstrap. Use `windows-sys` for Windows process creation, token/ACL operations
and file identity. Use `directories::BaseDirs::data_local_dir` for the Windows
local known folder, and std file locks for ownership. Keep these behind small
runtime modules; application requests and execution remain platform-independent.
Commit resolved dependency versions in Cargo.lock when implementing. The inspected
releases are interprocess 2.4.8, daemonize-next 0.1.0 and directories 6.0.0.
The Unix binary adds only interprocess and daemonize-next plus libc. Tokio is
pinned at least 1.54: interprocess 2.4.8 uses AsyncFd::register_with_interest,
which does not compile with the former 1.53.1 lockfile entry.

### Rationale and rejected alternatives

Reuse daemonize-next instead of implementing the Unix double-fork sequence from
scratch. It supplies fork/setsid/fork, standard-stream setup and propagation of
post-fork setup failures to the launcher. It refuses detected multithreaded fork
by default; Linn also launches it in a fresh bootstrap before Tokio, libSQL and
model initialization. Do not opt into allow_threads. Retain Linn's own bounded
readiness handshake: successful detachment is not application readiness.

Do not use original daemonize 0.5: RustSec RUSTSEC-2025-0069 marks it unmaintained.
Do not use daemonizable: its same-binary double-fork and typed parent/child channel
are Unix-only, and its documented descriptor/macOS caveats do not eliminate
Linn's reconnectable listener and platform adapter requirements. Do not use
running-process or processkit: their detached spawn APIs abstract Windows flags
and stdio, but their documented Unix contracts use setsid without Linn's
second-fork guarantee, and neither supplies Linn's ownership/readiness contract.
A general process-management framework adds surface without removing that work.

Interprocess supplies Unix-domain sockets and Windows named pipes without adding
application framing. Its Windows listener extension accepts an owned security
descriptor; the inspected pipe backend sets PIPE_REJECT_REMOTE_CLIENTS by default.
Use these facilities instead of reimplementing sockets/pipes. Set the account DACL
explicitly. Disable automatic name reclamation and try_overwrite; Linn removes
stale socket files only under its own ownership lock. Windows portable peer
credentials expose a PID, not SID, so validate process account identity with the
Windows token adapter. Unix peer checks use the available effective UID. Linux applies the atomic socket
mode option. macOS does not support that interprocess option: bind only within
the validated 0700 directory, then chmod the pathname to 0600 before accepting
clients. The private parent protects the bind-to-chmod interval.

Do not add fs4 solely for locking: std File::try_lock has been stable since Rust
1.89, and the inspected flake toolchain is 1.97.1. Do not add dunce or same-file:
canonical paths remain native, and existing-file identity comes from Unix dev/ino
or Windows FILE_ID_INFO through the platform adapter. Dunce simplifies path
syntax but cannot establish identity. Do not use fs-mistrust for Windows privacy:
it explicitly accepts Windows owners/permissions without inspecting their ACLs.
Use windows-sys for account-restricted creation and validation of directories,
lock files and store files. Pipe security descriptors do not secure those files.

These dependencies cover mechanisms, not application ownership, publication,
mutation retry or peer authorization. Those remain explicit Linn decisions below.

## Local protocol and ownership

Use pathname Unix-domain sockets on Unix and local named pipes on Windows; no
TCP listener. Keep local IPC separate from the peer protocol in LLP 0008. Encode
versioned typed request/reply envelopes as length-prefixed JSON, with a 32-bit
big-endian length and a maximum 16 MiB frame. Each connection carries one operation with request ID 1. Transfer invocation and
reply JSON and large inputs/outputs as bounded chunks with explicit completion,
backpressure and cancellation. The
frame limit is not a whole-record limit. Reuse serde types for direct and IPC
adapters; never serialize Clap structures or executable command strings.

Stream JSON envelopes in the existing 64 KiB chunks without temporary files.
Encode every message through a two-slot bounded channel and a blocking serde
worker. Do not probe size by serializing on the IPC thread: escaping one large
string can scan its entire contents before a capped writer fills. Decode a
single-chunk message directly in bounded memory; larger messages use the same
bounded-channel bridge to a blocking reader. Never accumulate the complete
encoded message. Channel closure cancels those workers when transport is dropped.
Retain frame/chunk limits, explicit End, request/reply typing, input deadlines and
response-delivery barriers. JSON is the format, not a reason to spool to disk.
Only Add/Import carry input bytes; ordinary queries, controls and internal replica
steps pass no input handle and create no empty temporary input files.

Keep the local schema in [runtime protocol](../src/runtime/protocol.rs), separate
from byte framing and from the peer exchange schema. A connection progresses through
Hello/Accepted, Invoke(1), streamed Invocation JSON, streamed input bytes, then
Reply(1) with streamed reply JSON and optional output, or a typed Failure.
Each stream ends explicitly with End. Unexpected frames, IDs, malformed JSON,
unknown envelope fields and incomplete streams fail the exchange. The shared
application Operation and Value enums define payloads; the client checks the
operation/result pairing, including individual replica replies and hello version.
Only Export returns an output stream, and its stream is required. Keep mutation
classification exhaustive so a new operation cannot silently default to a read.

Failures preserve their error category, diagnostics and mutation uncertainty.
An explicit Rejected response means not executed; preserve its conservative
false mutation indicator. Any other incomplete or invalid application reply to
a mutation is uncertain, including JSON/schema errors and mismatched result
types, not only socket I/O errors. Never replay it automatically. Keep the small
existing lifecycle handshake/control wire representation stable across builds;
application schema changes remain protected by the explicit compatibility number.

Use one daemon per account and canonical store path. Resolve symlinks, including
aliases to a missing final file, without opening or creating the database.
Native-canonicalize the existing prefix to preserve filesystem spelling,
including case aliases on macOS. Hash account identity plus the resolved native
path with SHA-256; encode the first
128 bits as a 26-character base32 endpoint key. Compare full routing identities
at handshake, not just hashes; a collision fails explicitly. The client may
create transport coordination directories and files, but only the daemon opens,
creates, migrates, reads, writes and closes the database. A missing destination
remains missing until an application operation needs it; invalid imports are
rejected before creation. The daemon binds physical file identity on observing
or creating the store and checks it around operations. Replacement or removal
fails explicitly and retires the stale owner. Reject hard-linked SQLite stores;
symlink aliases resolve to the same routing path. Windows delivery must resolve
native path aliases and casing through the OS, including missing-file routing,
without creating the database in the client.

Use the shared [native path codec](../src/native_path.rs) for every local path in
identity, startup, endpoint, handshake and file-input payloads. UTF-8 paths retain
their string encoding (and existing routing keys); other paths use serde's native
OsString encoding, Unix bytes or Windows wide units, without lossy conversion.
Reject a foreign platform's native representation. Store files themselves must
have UTF-8 canonical paths because libSQL requires them; return an ordinary typed
error before creating coordination files or starting a daemon for an unsupported
store path. This backend restriction does not apply to source/runtime paths.
Record source-path text keeps its existing display conversion under LLP 0001.

### Stable owner directory

Keep startup and ownership locks in an account-private
`.linn-runtime-<uid>/` child of the canonical database parent on Unix. Windows
uses `.linn-runtime-<account-hash>/` at the same location. These are coordination
sidecars, like SQLite's own lock/journal sidecars, not another persistent store.
They contain no originals, embeddings or corpus indexes. Keep sockets elsewhere
as specified below. Do not apply cache/temp cleanup to the ownership directory.

Native Nix sandbox tests cannot write to the passwd account home, and service
accounts can lack a writable home entirely. The canonical store parent is already the required
location for writable SQLite sidecars. Store-local ownership remains independent
of HOME, XDG_RUNTIME_DIR and executable version, and different stores remain
isolated without global writable account state. Reject hard-linked stores: SQLite
journals and coordination cannot safely identify one database through different
parent directories. Symlink aliases resolve to the canonical target before lookup.
Require local filesystem locking and private permissions; fail if the anchor
cannot be established. Nix checks supply a private XDG_RUNTIME_DIR inside the
build sandbox for sockets; this changes locations only, never algorithms.

Create `<key>.start.lock` and `<key>.owner.lock`, mode 0600 on Unix and an
account-restricted DACL on Windows. Directories use mode 0700 or the corresponding
Windows ACL. Open without following symlinks/reparse points and validate opened
handles. Never unlink or replace lock files during normal cleanup. Unlocked files
are reusable, regardless of recorded PID. Daemon ownership lasts until its handle
closes; do not inherit it into child processes.

Before a daemon has bound a physical database file, hold the parent ownership
directory's shared `create.lock` around routing validation and the first
application operation. Recheck native path identity after acquiring it. This
serializes creation when differently cased missing basenames could become the
same file on a case-insensitive volume; the losing alias fails explicitly before
execution and retires, rather than opening a second store handle. Existing
stores retain independent per-store execution. Never unlink this lock either.

A separate `<key>.endpoint` file contains only bounded endpoint/store
identity information, atomically replaced after the daemon acquires its lifetime
lock. Keep metadata separate from the locked file: Windows file locks can prevent
nonholders from reading locked ranges. Apply the same private permissions to this
metadata. Clients read it under startup coordination and authenticate the live
endpoint; its contents alone never prove ownership. A client whose
XDG_RUNTIME_DIR differs reconnects to the already published endpoint rather than
launching a second owner. If the owner lock is held but the endpoint is missing
or inaccessible, return an actionable error; do not start another daemon. Reject
owner-directory replacement detected by handle/path identity checks before
serving work. Same-account adversarial filesystem mutation is outside the privacy
threat model; accidental directory loss still fails closed.

### Socket locations

Select the socket directory for a new owner in this order:

- Unix, including macOS: honor a nonempty XDG_RUNTIME_DIR and use its `linn/`
  child. Require an absolute, account-owned local directory with mode 0700 and
  socket support. Reject unsafe or relative explicit values. This is stricter
  than XDG's relative-path ignore policy.
- Linux with XDG_RUNTIME_DIR unset/empty: atomically create and validate
  `/tmp/linn-<uid>/` with mode 0700 and diagnose use of the fallback. Do not guess
  `/run/user/<uid>` when a headless account has no login-managed runtime directory.
- macOS with XDG_RUNTIME_DIR unset/empty: use
  `confstr(_CS_DARWIN_USER_TEMP_DIR)` plus a private `linn/` child. Do not rely on
  caller TMPDIR or a desktop login. Fail clearly if the native directory is
  unavailable. Darwin temporary storage does not promise XDG session lifetime.
- Windows: use `\\.\pipe\linn-<account-hash>-<store-hash>`; there is no socket
  file. The database parent holds the stable coordination sidecars. Restrict the pipe
  to the account SID, reject remote clients and validate the server's account.

For a new Unix owner use `<key>.sock` at mode 0600. Check the full encoded path
against the platform sun_path capacity; report an actionable path-length error
rather than silently changing roots. The hash only shortens the basename. Do not
scan fallback locations on connection failure. The stable endpoint metadata is the sole
endpoint discovery source once ownership exists. If temporary cleanup removes a
live socket, the owner retires and releases its lock before a replacement starts.

Read XDG_RUNTIME_DIR explicitly on macOS: dirs/directories runtime_dir returns
None there and would otherwise miss an explicit override. Keep the established
Unix store/configuration precedence in LLP 0004 unchanged.

## Automatic detached lifecycle

The CLI first attempts the authenticated published endpoint. If unavailable,
acquire the per-store startup lock, recheck owner state and launch only when no
owner exists. Hold the startup lock until readiness or startup failure. The child
acquires the lifetime owner lock before opening libSQL. Help/version and
`daemon:status` do not start a daemon. Status reports running state and the
authenticated peer’s PID for inspection; a stored PID never authorizes signals. `daemon:stop` drains accepted work and
releases ownership; the next ordinary operation starts it again. Keep the daemon
resident until stopped, upgraded or terminated; there is no idle-exit policy.

Use an internal same-executable bootstrap entry point, not a public foreground
serve command. On Unix launch a fresh process, run daemonize-next before the
async runtime, then start the application only in the detached grandchild. Use
umask 077, working directory /, and null standard streams. Close inherited
nonstandard descriptors in the fresh bootstrap before initializing application
resources, using close_range on Linux with enumeration of `/proc/self/fd` if
that syscall fails;
Darwin uses libproc PROC_PIDLISTFDS to close the descriptors actually open,
without scanning every unused number up to RLIMIT_NOFILE. Validate the complete
list before closing; if enumeration fails or is incomplete, fail startup with
an actionable error rather than falling back to an unbounded descriptor-limit
sweep. The supported macOS targets provide this linked API; no old-OS fallback
is required. The [runtime source map](../src/runtime/README.md#descriptor-cleanup-references)
records pinned zmx, tmux, OpenSSH and Apple references used for this decision.
Enumeration and cleanup occur only in the fresh,
single-threaded bootstrap, without concurrent descriptor opens. No inherited database/model or
worker-thread state crosses a fork.

On Windows use windows-sys CreateProcessW with an explicit absolute executable
path, DETACHED_PROCESS and CREATE_NEW_PROCESS_GROUP. Do not add CREATE_NO_WINDOW
or CREATE_NEW_CONSOLE. Disable handle inheritance; the child opens its own NUL
standard streams and readiness connection. This avoids reliance on nightly std
handle-inheritance controls. Use Windows argument encoding, never cmd.exe or
PowerShell to launch Linn itself. Keep process handles long enough to detect
startup exit, then close them. Do not request job breakaway: respect external
job containment. Surviving terminal closure is supported where the host permits
it; logout, job termination and machine shutdown are not promised survivable.

Use a private one-shot interprocess endpoint with an unpredictable startup token
for readiness and errors on both platforms, instead of inheriting a pipe. The
child connects after detachment, reports setup errors, or publishes its ordinary
endpoint and reports ready. Wait at most 30 seconds for lock acquisition/startup;
on timeout report the phase and observed owner state. Do not kill a PID from a
file. An owner completing startup after its initiating client exits remains
connectable by subsequent clients. Application readiness requires accepting
requests, not corpus preloading. Healthy opens remain scalar-only and models
retain their existing lazy initialization semantics.

Use the manual application compatibility number in
[runtime protocol](../src/runtime/protocol.rs), combined with the exact supported
SQL schema version derived from the migration list. Both must match for an
application handshake. Do not compute compatibility from source hashes, compiler
flags, Cargo profiles or executable identity. Different debug/release builds and
compatible implementations may share an owner.

Bump the application number when changing request/reply layouts, operation or
error semantics, input provenance, delivery/mutation guarantees or runtime/store
assumptions in a way an existing peer cannot honor. A schema change is fenced by
the migration-derived schema number; bump the application number too if the API
contract changes independently. Compatible implementation fixes, CLI presentation
changes and performance-only changes retain the number. Review protocol changes
against this rule; no build system can infer that compatibility decision.

Authenticate peer account and compare lifecycle protocol, application compatibility,
schema capability and full store identity before operations. Reject incompatible
application/schema pairs with an instruction to run daemon:stop, then retry.
Keep lifecycle framing at version 1 and accept status/stop independently of the
application/schema pair. Missing application compatibility or schema defaults to
zero, rejecting application work from older clients while preserving their
lifecycle control. Retain the old string `build` fields in handshake and endpoint
metadata as empty legacy placeholders so older status/stop clients can parse
them; never consult them for compatibility. Schema capability advertises binary
support, not the on-disk version: supported older stores still migrate on open.
Do not automatically alternate daemons between old/new clients. Keep lifecycle
status/stop messages in a small versioned control envelope independent of the
application request schema. Never route to a different store on mismatch.

Keep socket names and both ownership locks independent of build, protocol and
schema versions. A versioned socket alone cannot fence a database, and separate
versioned ownership would permit competing daemons. The stable owner is the
cut-off: an incompatible client must stop it through the compatible control
protocol before its own executable starts an owner. Never silently replace it.
Keep the application number alongside schema compatibility: application/RPC and
runtime semantics can change without a SQL migration. The existing exact-schema
snapshot/write guards continue to fence stale resident handles independently of
the handshake.

Serialize store operations. Admit at most 64 application connections before
receiving their invocation or body; reject excess application handshakes with a
busy/not-executed error. Keep the store queue bounded at 64 and network workers
bounded at 64 as well. Bound all accepted connections, including incomplete
handshakes and lifecycle requests, at 128; close excess connections before
allocating another task. Lifecycle requests bypass application admission while
remaining subject to the overall connection bound. Request slots last until the
connection finishes, not merely until its input reaches the queue. Bound
connection/handshake and idle input-frame waits to 30 seconds. Accepted long-running
rebuild/sync work has no arbitrary execution timeout. Disconnect cancels work
only before mutation begins; after mutation begins, finish its durable publication
or retain the documented pending marker. Do not automatically replay mutations
after lost replies; report uncertain completion. Retry a read only before any
response is delivered, using a new consistent snapshot and clock as applicable.

## Windows delivery boundary

Ship Linux and macOS first. The shared API/protocol and selected IPC stack support
a Windows adapter, but native Windows product support is a separate delivery
with these fixed requirements:

- Replace unconditional Unix filesystem imports in
  [store opening](../src/storage/mod.rs) and
  [sync storage](../src/storage/sync.rs) with platform-private creation helpers.
  Windows uses account-restricted ACLs and validates existing files/directories.
- Windows store and config defaults are `<LocalAppData>/Linn/store.db` and
  `<LocalAppData>/Linn/config.toml`. Preserve explicit flag, LINN_STORE and config
  precedence; validate supplied XDG values as in LLP 0004, without using XDG
  defaults for Windows. Update LLP 0004 with these defaults when implementing.
- Resolve routing aliases without client database creation, and use daemon-side
  native file identity checks for drive casing, junctions, symlinks and
  extended-length paths. Unicode and space-containing paths remain supported;
  unsafe runtime reparse points fail rather than being followed.
- Git provenance uses bundled git2; there is no Git CLI runtime dependency.
  Peer transport is governed by LLP 0008. Do not add an OpenSSH installation or
  remote-shell requirement to native Windows delivery.
- Add native Windows MSVC build/test/release coverage beside the existing
  [CI](../.github/workflows/ci.yml) and
  [release workflow](../.github/workflows/release.yml). Package the same embedded
  model/POS assets and notices with linn.exe. Build vendored libSQL with its
  required native toolchain and extend release manifests for Windows artifacts.
- Run direct/IPC parity and lifecycle tests natively, including ACL rejection,
  named-pipe busy/startup behavior, job containment, handle leaks, process death,
  file sharing, close-before-replace/delete and publication/forget recovery.
  No model, arithmetic, database-format or retrieval substitution is permitted.

Cross-compilation or crate platform support alone does not satisfy these release
requirements. No foreground server workflow, administrator rights, service
installation is required for local Windows use.

## Concurrency and scheduling

Keep one store dispatcher on a dedicated daemon thread with its own current-thread
Tokio runtime. A private dispatcher object owns application and ownership state;
validate the known bound socket path directly around operations, without rereading
endpoint discovery JSON. Endpoint metadata remains client discovery data. Start
network workers directly in a JoinSet blocking task; no extra local-task wrapper
or LocalSet is needed in the store runtime. Construct, use and drop Application, Store and Replica handles on
that thread. The IPC runtime accepts connections, frames input/output and answers
status independently. The vendored [local libSQL adapter](../vendor/libsql/src/local/impls.rs)
has async signatures but executes synchronous native calls; native model/linguistic work also need not yield.
Placing those calls on the IPC thread would stall handshakes, overload rejection,
disconnect detection and status during an ordinary query or SQLite busy wait.
Transport JSON framing/base64 processing still consumes IPC work;
this is scheduling isolation, not a hard real-time responsiveness guarantee.

| Work | Current concurrency decision |
|---|---|
| Different stores | Independent owners and dispatchers; may execute in parallel. |
| Handshakes, input transfer, daemon lifecycle status | Overlap store execution within connection/admission bounds. Lifecycle status does not touch the database. |
| Search and search:more | One at a time per store, including other reads. They mutate query clocks, candidate caches and temporary SQL workspaces. |
| Show, outline, list and export | Execute in the same serialized store lane. Complete original-data response transmission before the next store operation. |
| Add, close, forget, import and rebuild | Exclusive store execution through publication and response completion. No partially executed mutation is replayed. |
| Integrity check | Serialized; its implementation takes an immediate transaction, so a nonmutating API label does not make it a parallel reader. |
| Sync | Network workers may overlap each other and ordinary requests. Each local replica step is serialized. A whole exchange is not one atomic store transaction. |
| Stop | Stops admission, drains accepted store work and network workers, then closes handles and releases ownership. |

Execute complete uploads in the order they enter the external FIFO; neither
socket arrival nor handshake admission determines execution order. Internal
replica requests have their own bounded FIFO; choose
between ready external/internal work without a priority that can indefinitely
exclude peer progress. Do not promise a total ordering across independent sync
sessions. Ordinary reads may fail with pending-publication recovery instructions
between a sync apply and finish; they must not observe partial derived state.

The response-delivery acknowledgement belongs to the transport task. That task
has one 30-second response-write timeout and resolves or drops its acknowledgement
only after the response future finishes or is cancelled. For ordinary application
replies the dispatcher waits for that acknowledgement without a second competing timer: otherwise its timer
could release a later forget while the earlier transport is still writing.
A stalled ordinary response therefore delays subsequent store operations until
completion, transport cancellation or disconnect, while status remains available.
Mesh metadata/progress delivery drains separately as described above.

Mesh Get steps read original bytes on the store lane. Before each outbound
chunk of at most 64 KiB, a separate step revalidates identity/hash, tombstone and
peer authority and grants a delivery lease. Hold it through that write attempt,
then release on progress or cancellation before reacquiring for another chunk
or waiting for a peer acknowledgement. Cancel/reset partial frames before lease
release on failure. A stalled chunk can delay forget until the idle-write
deadline; peer acknowledgement waits do not occupy the lane. LLP 0008 defines
the exact transfer and cancellation contract. Mesh status reads store metadata
through the dispatcher; it contacts no peer and starts no record job.

Each store uses one serialized execution lane. Cloning a libSQL Connection
shares its underlying connection and does not create independent transactions,
query workspaces or mutable caches. Immutable model resources can be shared.

Reader initialization must not race migration or derived-state publication.
Every read must pin and validate the publication/pending state in its own
snapshot, preserve live-versus-reopened lifecycle behavior, and invalidate all
publication-bound state after writes. Keep mutations exclusive; drain active
read execution and response transmission before acknowledging forget, rebuilding,
retiring an owner or stopping. Consecutive read-response streaming could overlap
under that same barrier, but is serialized here. Do not switch journal mode just
to add concurrency. Any later reader-pool delivery must measure throughput,
tail latency, per-reader memory and writer delay on representative stores and
verify exact ordered page parity under LLP 0005 before adoption.

## Store publication and synchronization

Serialize store operations in one dispatcher. Avoid holding a read
snapshot across requests. Every request checks publication identity and pending
publication in the same snapshot as its reads, as required by LLP 0001. Search
and pagination hold one checked source snapshot through filter-reference
resolution, retrieval, pointer-block loading and result construction; the shared
agent implementation does not nest a second transaction. Direct agent and list
callers also validate exact schema, pending publication and source data version
inside their own snapshots. Acquire and verify the Store once, then hold that
snapshot around the store operation; snapshot ownership does not reacquire the
Store through a second application layer. External writers remain possible,
including older binaries and independently initiated replica sessions.

Retain only the active query and caller scope's query memos. A query/scope change
clears coverage, ranking and confidence memos, plus stemming and
document-frequency misses. Immutable FTS-hit and candidate-feature memos may span
queries, and eligibility-filtered FTS hits may span queries at the same clock,
under fixed soft budgets checked at query boundaries: at most 256 entries,
64 KiB of key payload and 65,536 allocated ID slots per map. Drop an over-budget
map as a whole, including its allocation; do not introduce an LRU framework or
configuration knob. A single active query may exceed the budget before the next
boundary. These checks inspect memo metadata, not corpus rows or posting elements.
Retain positive stemming results for known corpus words and positive document frequencies for the current
clock, bounded by the publication's vocabulary. A clock change clears eligibility
hits and all statistics together with clock-dependent ranking state. Replaying an
older cursor recomputes those exact values at its pinned clock. Keep immutable
models, publication-bound encodings and positive candidate/posting data resident.
Negative surface/navigation/section and vocabulary probes belong to the active
query, and basename misses are not retained. Remove previous-query negative
entries by their recorded probe keys; do not traverse corpus-wide positive maps
at query startup. This bounds retention by active query work and publication data,
without a cache knob or change to retrieval arithmetic.

On a stale handle, discard publication-bound caches and reopen before starting
an operation. Preserve process-wide immutable model resources. Do not retry
partially executed writes. Pending publication blocks ordinary reads and leaves
explicit sync/rebuild recovery available. Recovery dispatch must not depend on
successfully opening an ordinary healthy Store first. Failed rebuild/forget
handles remain unusable under LLP 0001.

Enforce LLP 0001's exact-schema fence inside SQLite read snapshots and write
transactions, including replica operations and startup publication writes.
A schema failure closes cached application handles. The daemon finishes the
failure response, closes admission, rejects queued application/internal work
without touching the store, drains mesh workers and releases ownership. It must
not reopen or rebuild a newer schema. SQLite transaction locking orders a
migration against an in-flight write; a waiting writer checks the schema only
after acquiring its write lock. A prior read snapshot cannot upgrade to a writer
after a conflicting WAL migration. These guards use the database's schema version, without a schema sidecar.

This fencing applies to binaries that implement it. It cannot retrofit an
already-running older binary that lacks transaction guards or ownership locks;
stop such legacy processes before the first incompatible migration.

Keep current live-instance versus reopened-store lifecycle/link behavior exactly
as specified in LLP 0001. Test both: residency must not accidentally change the
existing forget-until-rebuild rules. Clear the specified forgotten-record caches;
never serve stale bodies or queued results after a forget has completed. Acknowledge successful mutations only after publication completes. Serialize
ordinary reply delivery before the next operation, so a completed forget cannot be
followed by transmission of an earlier queued response. A disconnected client
is skipped before execution when detected; executing mutations still complete
their durable publication or preserve the pending marker. Preserve transactional secure deletion and tombstone precedence;
Residency adds no stronger memory-erasure guarantee than the existing contract.

Route peer synchronization and membership operations through the same typed
application API. Network coordination runs outside the store dispatcher, using
the typed internal mesh-store queue. Keep the public runtime API at typed
operations and lifecycle control; do not add a second public replica-over-IPC
convenience adapter solely for tests. The CLI requests work from the owner;
it performs no peer exchange. Network workers send inventory/get/needs/apply/
finish steps through the internal channel, keeping handshakes and peer response
waits outside the store lane. Only outgoing original-data writes retain a
delivery lease. Simultaneous peer exchanges must not wait on requests queued
behind their own network acknowledgement. Shutdown drains accepted mesh workers
and their internal store steps before closing the owner.

Keep one lazily opened Replica in the application alongside its resident Store.
Read-only replica steps reuse it without discarding the Store; mutating replica
requests invalidate the Store before execution. Errors discard both handles.
Every replica step retains its exact in-transaction schema checks, and runtime
physical-identity checks remain around every operation. Both handles execute
only on the serialized store lane; this is not a reader pool. Move an owned
incoming Item through the authenticated mesh Apply step rather than cloning its envelope.
Preserve outbound delivery leases through completed/failed writes and release
before acknowledgement waits. Imports, local writes and synced writes use one
mutation/publication boundary. Do not retain every transferred body in memory.

LLP 0008 owns explicit sync-job admission, bounded retries, peer authorization
failures, shutdown and publication availability. Persistent enrollment and
transport readiness never schedule record
exchange on a local write, timer or reconnect. The existing owner remains the
only store owner while serving authenticated jobs initiated by another member.
The obsolete subprocess coordinator, stdio adapter and remote configuration are
removed. Local ownership, serialization, compatibility, mutation and publication
contracts in this guide remain required.

## Implementation layout

[Application operations](../src/application/mod.rs),
[typed input preparation](../src/application/input.rs) and
[list cursor handling](../src/application/listing.rs) define the typed API.
[CLI parsing/presentation](../src/cli/mod.rs) prepares caller context and bytes.
[Runtime coordination](../src/runtime/mod.rs),
[store dispatcher](../src/runtime/dispatcher.rs),
[mesh network workers](../src/runtime/mesh.rs),
[Unix platform operations](../src/runtime/platform.rs) and
[local RPC schema](../src/runtime/protocol.rs) with
[bounded framing](../src/runtime/wire.rs) provide transport and lifecycle.
[Request-local diagnostics](../src/diagnostics.rs) preserve explicit timing/trace
controls without inheriting a previous client's flags or changing algorithms.
Storage numerical-array, candidate-vector and block-matrix timings use this same
transport rather than the detached process's null stderr.
Mesh workers use explicit enrolled service/identity state, never a previous
CLI caller's environment. The application API carries no subprocess environment
snapshot or SSH stderr stream. Ordinary reads and writes transmit no subprocess
environment. File source
references reuse the original body hash and require no subprocess or fallback
chain.
Direct library callers use Application or the existing storage/retrieval API;
evaluations continue to use the shared library without starting a daemon.

Local typed errors preserve category, diagnostics and an explicit conservative
mutation-may-have-committed indicator. Lost mutation responses are never replayed.
The daemon validates complete imports once after transport, before creating a
missing destination file or merging. It also derives default record labels from
the original bytes and supplied input filename. The CLI supplies raw bytes and
caller metadata; it performs no content parsing or application validation. Exports are staged before
client output-file creation. Original-data staging for add/import/export uses
anonymous temporary files in the private ownership directory where application
snapshot/validation and complete-output semantics require it. IPC JSON itself is
streamed and never spooled; ordinary query paths use no temporary input files.
These are not permanent corpus copies or whole-corpus caches. The fresh bootstrap sets its
own TMPDIR to that validated directory before Tokio starts, so removal of the
initiating shell’s temporary directory does not break later requests. This is a
fixed runtime resource path, not an ingestion/retrieval setting.

## Verification and completion

Corpus-free integration tests use temporary stores and private runtime
directories, real detached children and bounded cleanup. Cover concurrent cold
starts, stale endpoints, startup failure, parent/harness EOF, stop/restart,
crash recovery, upgrade/version mismatch, store aliases/replacement, multiple
stores, unsafe endpoints, malformed/truncated frames and stream cancellation.
Include long Unix socket paths, runtime-root changes, temporary-directory cleanup,
lock-file replacement and Linux headless fallback. On native Windows cover pipe
ACLs/remote rejection, SID separation, busy-pipe startup, parent/job lifetime,
handle leaks and process death releasing locks; do not infer these from Unix tests.
Cover mixed simultaneous writes/queries/readback, overload before input upload,
control during native SQLite lock waits, stalled output delivery and disconnect,
and overlapping mesh jobs using the same owner.
On case-insensitive macOS volumes, cover native spelling of existing paths and
concurrent first operations through differently cased missing basenames. Test
mismatched reply types/IDs, unexpected or truncated output, malformed reply JSON
and typed failure diagnostics/uncertainty without executing a real mutation.
Verify lost mutation replies are never blindly replayed and shutdown does not
abandon a committed-but-unpublished write without recovery state.

Compare direct and IPC operations for text/JSON output, stderr and exit status,
original-byte ingest/export, reference and cursor handling, caller scope from
different repositories, per-request expiry, writes, import, sync, rebuild and
forget. Exercise external mutations, failed publication, stale caches and
same-process versus reopened lifecycle behavior. Exercise replica sessions
against compatible peers without exposing local IPC framing on the network.

On representative curated and scale stores, compare ordered first-page IDs and
full page content with the accepted dev baseline. Counts alone are insufficient.
Follow LLP 0005 for any required holdout confirmation; do not retune ranking or
reopen not-adopted studies. Report daemon cold startup/store opening separately
from first request, warm end-to-end CLI latency and resident execution latency.
Include lexical and fallback queries, repeated and varied queries, idle/peak
memory, post-write reopen cost and concurrent-client queue behavior. Measure
fresh-process POS/model initialization rather than treating historical timings
as a present speedup. Report cache conditions and separate workloads accurately;
a small fixture is not full neutral validation.

Native macOS verification is recorded in evals/FINDINGS.md. Curated/scale
corpus verification and native Linux/Windows runs are not implied by a small
real-store sample.

Run the repository checks through the flake: cargo fmt, workspace clippy with
all targets/features and warnings denied, workspace tests with all features,
nix build, and scripts/check_llp_refs.py, as prescribed by AGENTS.md. Completion
requires automatic background operation with unchanged ordinary CLI contracts,
verified publication/forget correctness, documented protocol/lifecycle behavior,
and measured latency and memory tradeoffs. No foreground service workflow or
automatic mesh synchronization is a prerequisite or part of this runtime baseline.
