# Command-line interface

`mod.rs` owns Clap parsing, client-side input preparation and text/JSON rendering.
`bundle.rs` handles export destinations; `listing.rs` and `sync.rs` own arguments
and the SSH stdio adapter. `paths.rs` captures native caller Git provenance and
absolute file-input context. It spawns no Git command for CLI operations.
Typed execution lives in `application`; `runtime` connects to the detached owner.
The CLI never opens or creates a store, validates content, derives labels or
coordinates SSH sync. It transports original bytes, caller metadata and replies.
The application derives file source paths and source references from a typed
input task and the uploaded snapshot. New file source references reuse the body
SHA-256; no Git subprocess, checkout filters or environment payload is needed.
Existing immutable source references remain unchanged.
`main.rs` handles the internal bootstrap before creating Tokio, then runs the CLI.

[LLP 0004](../../llp/0004-runtime-and-evaluation.guide.md) defines command contracts;
[LLP 0007](../../llp/0007-background-runtime.guide.md) specifies the local runtime.
