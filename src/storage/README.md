# Storage

`mod.rs` owns `Store`: the local libSQL connection, immutable records, derived
index publication, lifecycle operations, and indexed reads. Retrieval and POS
modules add methods to this same store; they do not own another database.

- `publication.rs`: publication completion, failed-handle refusal, and shared secure erasure.
- `catalog.rs`: candidate headers, lifecycle lookups and indexed clock-dependent totals.
- `record.rs`: record envelopes, hashes, and lifecycle eligibility.
- `bundle.rs`: snapshot export selection and validated original-data bundles.
- `sync.rs`: immutable replica merges and store-local remote configuration.
- `migrations/`: numbered SQL schema upgrades, their transaction runner and exact-schema fences for resident connections.
- `sections.rs`: original-byte section, block, and hint spans.

[LLP 0001](../../llp/0001-storage-and-sections.explainer.md) governs this domain.
