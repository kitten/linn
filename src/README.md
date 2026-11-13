# Native Linn implementation

The CLI and eval harness share this library. Code is grouped by responsibility:

- [storage/](storage/README.md): immutable records, section cuts, libSQL indexes,
  publication, and on-demand reads.
- [retrieval/](retrieval/README.md): query interpretation, lexical pages, ordered
  fallbacks, and exact ranking operations.
- [language/](language/README.md): embedded model encoding and POS annotations
  derived from original text.
- [nlp/](nlp/README.md): embedded linguistic annotation and lookup-ready assets,
  independent of storage and ranking.
- [sync/](sync/README.md): SSH transport, bounded exchange, and mesh coordination.
- `application/`: typed operations/input tasks, caller context, source metadata, results and cursors.
- [runtime/](runtime/README.md): detached bootstrap, ownership, framed IPC, a dedicated store thread and SSH coordination workers.
- `diagnostics.rs`: task-local output controls and captured diagnostics.
- [cli/](cli/README.md): argument parsing, caller context, byte transport and presentation.

`main.rs` only runs the CLI and reports errors. `error.rs` defines shared typed
errors. `lib.rs` exposes the domains and retains the earlier library import paths
as re-exports for callers. Internal code uses domain paths.

The [LLP map](../ARCHITECTURE.md) leads to the specification. These directory
guides locate code; they do not define another algorithm or persistence contract.
