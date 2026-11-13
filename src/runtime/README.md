# Local runtime

`mod.rs` owns authenticated IPC, bounded admission, launch/readiness and reply
transport. `dispatcher.rs` owns the dedicated serialized store thread;
`coordinator.rs` runs SSH workers and sends local replica steps to that thread.
`platform.rs` implements Unix paths, private coordination files and detachment.
The private dispatcher retains ownership checks and its bound socket path;
endpoint metadata is read by clients for discovery, not by each store operation.
`protocol.rs` defines local RPC envelopes and operation/reply validation;
`wire.rs` frames and streams JSON through bounded memory/channels, with no JSON
spooling files. Encoders and large-message decoders use cancellable blocking
serde workers; single-chunk decoding stays in bounded memory. Ordinary queries carry no input file. The application owns the shared request,
result and error types. The client treats malformed or incomplete mutation
responses as uncertain completion, while preserving explicit server failures.
Application handshakes require matching manual application compatibility and
supported SQL schema versions; control
messages remain compatible across those identities. Endpoint names and ownership
locks stay version independent. A schema error fences application work and retires
the owner after draining replies/workers, as specified in LLP 0007.
[LLP 0007](../../llp/0007-background-runtime.guide.md)
is the governing contract.

## Descriptor cleanup references

Reviewed 2026-11-13, including the following pinned primary sources:

- [zmx daemonization, f5d604e](https://github.com/neurosnap/zmx/blob/f5d604e417d439f95b8a5f8ca927b8e5b0f694b9/src/daemonize.zig#L168):
  fork/setsid, null standard streams and an allowlist while closing descriptors
  3 through 63. Its subsequent forkpty starts the shell; that terminal/PTY
  lifecycle differs from Linn's detached server grandchild and fresh bootstrap. Its bounded cleanup would not cover arbitrary inherited descriptors.
- [tmux closefrom, 6fa1a88](https://github.com/tmux/tmux/blob/6fa1a88934274db237391b350c7058b92c50e52e/compat/closefrom.c#L87)
  and [OpenSSH closefrom, 268bdcb](https://github.com/openssh/openssh-portable/blob/268bdcb0c80876d97abdc74a1ca8992a1fdf7652/openbsd-compat/bsd-closefrom.c#L85):
  macOS uses the two-call libproc descriptor-list operation and closes only open
  entries. Both retain a sweep fallback. Both carry the same family of
  closefrom compatibility code and corroborate the native enumeration pattern.
- [Apple libproc declarations](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/libsyscall/wrappers/libproc/libproc.h#L90)
  declare proc_pidinfo available since macOS 10.5. The kernel's
  [descriptor-list implementation](https://github.com/apple-oss-distributions/xnu/blob/f6217f891ac0bb64f3d375211650a4c1ff8ca1ea/bsd/kern/proc_info.c#L485)
  reports required bytes with spare capacity, then returns the bytes populated.
  Apple labels these process-inspection interfaces private. Linn validates bounds,
  entry alignment and completeness before closing anything, and fails startup
  when enumeration fails.
- [close_fds 0.3.2](https://docs.rs/close_fds/0.3.2/close_fds/): broader portable
  and post-fork, async-signal-safe descriptor handling. Its pre-exec macOS example
  sets close-on-exec rather than closing directly. Linn instead cleans up after
  entering a fresh executable and before its daemon forks or creates threads;
  it does not need that broader post-fork abstraction here.

The original macOS descriptor-limit sweep was complete only within its limit and
cost proportional to that limit, including unused entries. On the measured host,
that meant roughly one million close calls. LLP 0007 now selects native enumeration
and explicit startup failure on enumeration errors; it keeps no macOS sweep
fallback. Runtime tests cover descriptor 4096 above a child soft limit of 128 and verify
inherited pipe EOF while the daemon remains alive.

On Linux, a failed `close_range` uses `/proc/self/fd` enumeration with explicit
startup failure if enumeration fails. Both platform lists include descriptors
above a subsequently lowered soft limit; no descriptor-limit sweep is used.
