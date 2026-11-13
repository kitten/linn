# Synchronization

`mod.rs` coordinates pull-all-before-push exchanges over the typed LocalReplica
interface. Library callers use Replica; the CLI sends one Sync operation
to daemon-owned workers in `runtime/coordinator.rs`. Their internal replica
requests keep peer acknowledgement waits outside the local store dispatcher.
Outgoing items retain a delivery lease through the SSH write, ordering forget
after completed or failed transmission. `protocol.rs` owns bounded,
versioned stdio messages; `ssh.rs` validates origins and manages SSH processes.
Original-record merging and remote configuration live in `storage/sync.rs`.

[LLP 0006](../../llp/0006-sync.explainer.md) specifies the protocol and recovery.
