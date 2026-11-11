# Synchronization

`mod.rs` coordinates pull-all-before-push exchanges. `protocol.rs` owns bounded,
versioned stdio messages; `ssh.rs` validates origins and manages SSH processes.
Original-record merging and remote configuration live in `storage/sync.rs`.

[LLP 0006](../../llp/0006-sync.explainer.md) specifies the protocol and recovery.
