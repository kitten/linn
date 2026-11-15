# Replica protocol

`protocol.rs` defines inventory, get, needs, apply and publication operations used
by authenticated mesh transport. Immutable merging lives in `storage/sync.rs`;
network jobs and delivery leases live in `mesh/`. See [LLP 0008](../../llp/0008-mesh-synchronization.explainer.md).
