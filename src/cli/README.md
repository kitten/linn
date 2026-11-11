# Command-line interface

`mod.rs` handles Clap arguments, command execution, cursors, input provenance,
and text/JSON presentation. `bundle.rs` handles file export/import;
`listing.rs` handles metadata browsing; `sync.rs` handles sync commands. `paths.rs` resolves the store location. Retrieval
and persistence remain in their library domains; `main.rs` is the thin entry point.

[LLP 0004](../../llp/0004-runtime-and-evaluation.guide.md) defines the command
surface and packaging contract.
