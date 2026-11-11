# Local retrieval evaluation

[LLP 0005](../llp/0005-evaluation.explainer.md) defines input, scoring and measured floors.
Capture supplied JSONL files without committing them:

```sh
nix develop -c cargo run -p linn-eval -- capture <source-directory> q4
nix develop -c cargo run --release -p linn-eval -- --pages evals/output/dev.jsonl
nix develop -c cargo run --release -p linn-eval -- --corpus evals/corpus/h2 --mode all
```

Create evals/output before exporting. Default corpus is evals/corpus/q4;
--corpus or LINN_EVAL_CORPUS overrides it. The product always uses shared heading
trees and POS selection; evaluation does not configure an alternative retrieval
algorithm.

`--store` reuses a local store; omit it for a fresh full ingest. `--pages` exports
ordered first-call record ids per wording. `--compare` checks against another
export; `--compare-pages <actual> --compare <expected>` compares two exports
without opening a store. `--compare-vectors` accepts a little-endian float32
block matrix in section-id order. `--rebuild-vectors` rebuilds derived vectors.
`--case`, `--query`, and `--scope` support focused local diagnostics.
Both comparison modes reject duplicate, missing, extra, or malformed page keys
and compare the full ordered id arrays. Reuse `--store` only with the corpus
that built it; omit it for a fresh independent ingest.

Do not commit corpus, weights, or outputs. Workspace tests construct their own
small records and cases; a missing captured corpus never fails cargo test.

Neutral fixed-loop gate tables (no model agent or CLI driving):

```sh
nix develop -c cargo run --release -p linn-eval -- --neutral
nix develop -c cargo run --release -p linn-eval -- --neutral --corpus evals/corpus/h2 --mode all
nix develop -c cargo run --release -p linn-eval -- --neutral --corpus evals/corpus/scale-q4
nix develop -c cargo run --release -p linn-eval -- --neutral --corpus evals/corpus/scale-h2 --mode all
```

Capture scale corpora with `capture <source-directory> scale-q4` and `scale-h2`.
Only records.jsonl and cases.jsonl are copied. These commands consume their body
bytes; they never open source_path or enrich author terms. Neutral grading's
protocol and unchanged thresholds are in LLP 0005. Gates may fail: this is a
measurement table, not an all-pass assertion. G4 forget uses a scratch store.
The default command still prints the unchanged two-line retrieval measurement.

The accepted combination has completed q4, h2 and scale-q4 neutral tables with
failing gates; scale-h2 neutral was canceled. See FINDINGS.md for the dated
measurements and distinguish them from tables for earlier binaries. Save full gate tables
under ignored evals/output (for example, redirect stdout to q4-neutral.md,
h2-neutral.md, scale-q4-neutral.md, and scale-h2-neutral.md). Fixtures and an
empty `--case` selection do not substitute for those runs. A focused case must
exist in the chosen corpus and pass the selected split. Diagnostics can still
run when no cases are selected; see LLP 0005 for the exact scope and cost rules.

## Saved-page and historical hybrid comparisons

```sh
nix develop -c cargo run --release -p linn-eval --example page_changes -- \
  <corpus-directory> <before-pages.jsonl> <after-pages.jsonl>
nix develop -c cargo run --release -p linn-eval --example hybrid_compare -- \
  <captured-corpus-directory> <readonly-archive-directory> <new-output-directory>
```

`page_changes` reads two native `--pages` exports with identical case/wording
keys and reports ordered-page changes and paired gold/cross-scope gains/losses.
It runs no searches. These first-call exports cannot score empty-correct or
section containment; use the full evaluator for those measures.

`hybrid_compare` builds fresh Linn stores for the archived 1,243- and 90-document
populations and compares against saved hybrid predictions. It verifies original
body bytes, prepared queries, page keys and historical gold counts. It never
runs or downloads a hybrid model. The archive must contain `docs-q4/`,
`docs-sub/`, `queries-sub.json`, `queries-s5.json`, `hits-q4-hybrid-sub.jsonl`,
`hits-q4-hybrid-s5.jsonl` and `hits-sub.jsonl`. Bodies are ingested from captured
JSONL; archived Markdown is used only to verify the matched population. Keep the
archive and output under ignored local directories. Existing comparison stores
are refused. Corpus-free scoring fixtures run with the workspace tests.

The comparison reports gold-any/first, original-query MRR@5, cross-scope and
empty-correct, plus paired outcomes. It cannot compare section or neutral gates:
the historical predictions contain only ranked record ids. This compares
recorded predictions on matched slices; it is neither a fresh external-system
run nor a full-corpus comparison. Its resident query timings exclude ingest and
CLI startup.

## Synthetic six-document retrieval

The ignored `evals/corpus/synthetic-six-docs/` contains a one-time conversion of
a published external retrieval fixture (source revision
`86d2365d17fc682e491c2c53d1e4a884aee8682a`): six unchanged synthetic Markdown
bodies and 49 document-relevance cases (24 standard, 25 deep-research).
The BM25 tests repeat the same 24 standard questions, and the older CLI harness
repeats 18 of them; neither is counted twice. Fine-tuning query-expansion evals
have no document qrels and are not included.

```sh
nix develop -c cargo run --release -p linn-eval -- \
  --document-benchmark --corpus evals/corpus/synthetic-six-docs --mode all
```

This reports first-call Hit@1/3/5 and MRR@5, overall and by the original suite
and difficulty. It runs each original question once, with no intent hints,
explanatory notes, query expansions or alternative queries supplied to retrieval.
Use `--pages <output-file>` for the existing strict page comparison flow.
The normal two-line scorer remains available on this corpus too; its section,
no-gold and cross-scope denominators are zero because upstream supplies no such
judgments. The document benchmark makes that limited evaluation purpose explicit.

The data are independent of Linn's cases, not unbiased: they are synthetic,
small, openly published, and some questions explicitly target hybrid search.
Qrels name one expected document rather than exhaustive judgments of every
possible answer. No questions were selected by Linn performance, no document was
rewritten, and no source intent, notes, synthetic keywords or artificial titles
were indexed. All original questions were retained without grouping different
questions into alt_queries or inventing negative or section labels.

Conversion resolved each expected filename substring uniquely, assigned stable
opaque ids, kept null scope and accepted document state, and derived display
labels only from each original first heading. Body and question strings are
preserved. The ignored PROVENANCE.json records the source revision, per-file
SHA-256 hashes, exact filename-to-id mapping, counts, and preparation rules;
LICENSE preserves the upstream notice. Only the final JSONL and this provenance
are retained locally. There is no raw source snapshot, intermediate dataset,
dynamic converter, or external-system dependency in the harness. These data were not scored while
being prepared; this adds no new measured floor or adoption claim.

## Lookup optimization checks

Compare two fresh ingestion results exactly with
`nix develop -c cargo run --release -p linn-eval --example ingest_parity --
<before-store> <after-store>`. This reads both stores without mutation and checks
block vectors, fallback metadata/vector bytes, and source POS annotations in
stable row order. Publication generations are excluded; model identities, row
ids, original spans and all numerical payload bytes must match.

To measure repeated vector bytes in an existing copied store without changing
it, run `nix develop -c cargo run --release -p linn-eval --example
vector_duplicates -- evals/output/scale-copy.db --kind sentences`.
The diagnostic groups complete float32 payloads by SHA-256 and estimates the
size of a vector dictionary plus row ordinals. It does not implement compression
or change the numerical population; reported time includes hashing.

To compare candidate-only vector scores bit for bit with the full matrix on a
populated copied store (including its native-index upgrade):

```sh
nix develop -c env LINN_VECTOR_PARITY_STORE=evals/output/scale-copy.db \
  cargo test --lib candidate_vector_corpus_bits -- --ignored --nocapture
```

This diagnostic checks two candidate subsets and four query vectors. Normal
tests need no corpus and cover contiguous ids and deletion gaps.

To profile opening an existing store without reading corpus JSONL:

```sh
nix develop -c env LINN_STARTUP_TIMINGS=1 cargo run --release -p linn-eval \
  --example startup_check -- <copied-store> --query 'example query'
nix develop -c cargo run --release -p linn-eval --example startup_check -- \
  <copied-store> --raw
```

Raw mode reports libSQL preparation, row stepping and value extraction apart
from Rust search reconstruction. Compare its default connection settings with
`--cache-kib 262144 --mmap-mib 2048`; actual PRAGMA values are printed on stderr.
These diagnostic options do not change the product's connection defaults.
The query mode emits complete calls, so its timing output should stay private
with the copied store under ignored evals/output.

To probe materialized-view support in the pinned engine without touching a file:

```sh
nix develop -c cargo run --release -p linn-eval --example startup_check -- \
  :memory: --probe-materialized-views
```

Time an older store's first projection build separately from subsequent opens.
The first open materializes indexed structural projections in the same database;
later opens read metadata and query postings on demand. Vectors and the FTS
vocabulary are deferred until needed, so keep first-query timings as well as the
open timer. Never report only the open timer as end-to-end CLI latency.

For a fixed sample of fresh CLI invocations, use:

```sh
nix develop -c cargo run --release -p linn-eval --example cli_latency -- \
  --binary <release-linn> --store <copied-store> --corpus <corpus-directory> \
  --output evals/output/cli-latency --per-group 2
```

Each process uses an original dev query and the real CLI clock/caller scope.
The sample spans six case groups, and `--case <id>` adds a focused dev case.
Use `--only-cases` with `--case` to run only named dev cases.
`--compare-binary <saved-release-linn>` compares semantic CLI output, excluding
the clock-bearing cursor. Exit 1 is an honest miss, not an execution error.
This is neither the frozen retrieval measurement nor the neutral latency gate.

`cli_latency_compare` alternates two saved product binaries over a stratified
sample and checks their complete JSON responses and exit codes. Supply `--before`,
`--after`, `--corpus`, `--pages` and `--store`; optional `--before-store` uses a
separate identical copy for the baseline, avoiding alternating schema migrations
in the measured invocations. It performs an unmeasured warm-up pair, followed by
`--rounds` measured rounds and a diagnostic round. Keep other heavy work idle for
final latency comparisons.
Selected cases outside the CLI query grammar are excluded and listed in
`excluded_cli_queries`; the store's full retrieval evaluation still includes them.

Writers now prepare query-independent fallback arrays. Older stores prepare
them once when missing; subsequent searches load only the stage they need.
Inspect their storage cost with `startup_check <store> --artifact-stats` and
report migration/write time separately from CLI measurements.

Add `--timings evals/output/query-times.jsonl` to a normal retrieval run to record
store-open/build time separately from each agent invocation. Stderr reports
median, p95 and maximum query milliseconds; the raw JSONL includes first-use
cache costs. This runs the same queries and preserves the two-line score output.
It is not the neutral per-returned-call latency gate, and it does not include
corpus JSONL loading in the store timer.

For a copied store and two saved `linn-eval` executables, run:

```sh
nix develop -c cargo run --release -p linn-eval --example lookup_check -- \
  <corpus-directory> <copied-store> <before-binary> <after-binary> <output-directory>
```

The utility checks identical ordered row ids from indexed SQL and the in-memory
record lookup, then times 100 passes over at most 128 records. It also compares
complete call JSON on up to 24 evenly spaced original queries from the dev split,
including later calls, spans, excerpts, hints, and related pointers. It saves both
exports and fails on a difference. The two executables must accept `linn-eval`'s
ordinary `--query`, `--scope`, `--corpus`, and `--store` interface. Whole-command
timings include startup and store open; they are not the neutral latency gate.
The helper opens the supplied store and may install its derived SQL index: use
a copy under ignored evals/output. Keep full-corpus ordered page comparisons and
the two-line scores as separate checks; this fixed sample is not full parity.
