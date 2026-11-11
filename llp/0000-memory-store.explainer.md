# LLP 0000: Memory store boundaries

**Type:** Explainer\
**Status:** Active\
**Systems:** Store, Retrieval, CLI\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0003, LLP 0004, LLP 0005, LLP 0006

## Boundaries

The LLP corpus is the specification. Linn is a private local memory store, read on demand through one native CLI. File and stdin ingest preserve original text. One local libSQL file holds bodies, immutable envelopes, sections, FTS, and vectors. The static model is embedded at build time: ingest and search require neither network nor another runtime.

There is no background service, GitHub or session acquisition, or reciprocal rank fusion. See LLP 0001 for data, LLP 0002 for search, LLP 0004 for operation, LLP 0005 for scoring, and LLP 0006 for synchronization.

Each account owns a local store file. Explicit SSH sync exchanges immutable
records and permanent tombstones between trusted accounts; it never shares the
live database or runs automatically during reads and writes. Optional repository
selection controls exchange, not SSH authorization. Writer email is unverified
attribution, not authentication. Tombstones prevent resurrection on every replica
that receives them; removing a remote does not erase copies already shared.

The POS relation selector additionally embeds the pinned linguistic
resource described in LLP 0004. It adds no semantic embedding model or product
runtime. Selected source relations are derived same-file data under LLP 0001;
the fixed ranking algorithm and adoption of source changes remain governed by LLP 0002 and LLP 0005.

## Implementation layout

Code is grouped by domain under `src/`: `storage` owns persistence and original
byte spans, `retrieval` owns query execution and ranking, `language` owns embedded
resources and retrieval-specific annotations, `nlp` owns linguistic analysis,
`sync` owns SSH exchange, and `cli` owns command execution and presentation.
The [source map](../src/README.md) locates their modules. The library root retains
existing import paths as re-exports; this organization changes no store format,
algorithm, or command contract.
