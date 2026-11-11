# Linn contributor rules

The LLP corpus is the specification. Read the governing LLP before implementation.
Preserve measured retrieval behavior, thresholds, and card order.
evals/FINDINGS.md is the closed-measurement log, not the specification; a
not-adopted entry is closed and must not be rerun as if it were new.
Adopt retrieval changes only under LLP 0005's dev-selection and one-shot h2
confirmation rule; neutral gates and hybrid comparisons never authorize ranker
retuning or additional models.

Preserve the accepted source-evidence and BM25L combination specified in LLP
0002, including the measured h2 tradeoffs in LLP 0005. This recorded acceptance
does not relax the adoption rule for future changes.

The recorded POS adjustment study uses q4 and h2 after its first confirmation
failed. Those runs are tuning measurements, not independent holdout confirmation;
preserve scoring and record every tested configuration.

Isolated replacement-encoder research is authorized only when a candidate embeds
in the native binary and is similarly fast. Measure those
constraints and full retrieval outcomes; keep accepted defaults, original-text
ingest and scoring unchanged until the ordinary adoption requirements pass.
This permits a replacement, not an additional model or a reranker.

- Keep one native Rust product binary. An eval package may share its library in a Cargo workspace.
- Use local-file libSQL as the only persistent store. Preserve original bodies and immutable envelopes; rebuild only derived state.
- Search is the two lexical matchers, tier-preserving interleave, heading fills,
  and ordered fallback procedure in LLP 0002. Do not introduce RRF or a reranker.
- Store scope as null; retain original scope separately for the within-tier penalty.
- Keep ordinary queries driven by indexed candidates and reusable derived work.
  Performance changes preserve arithmetic and result order; measure store opening
  separately from resident query latency and verify page parity.
- Use libSQL's existing term indexes and token counts on demand; do not rebuild
  corpus-wide frequency and position maps at startup or add a parallel data file.
- Healthy startup reads scalar publication metadata only; keep record headers,
  ordinals and eligibility data candidate-local rather than building full-store maps.
- Maintain structural ingestion indexes in the same libSQL file on writes.
  Fetch candidate bodies, sections, blocks, trees and references by indexed keys;
  unloaded data must never change eligibility, corpus totals or ordering.
  Read indexed candidates on demand; preserve publication, invalidation and
  secure-forget rules in LLP 0001.
- Prepare query-independent fallback encodings on writes; load exact stored
  float32 arrays on demand, guarded by model and publication identity. Never
  substitute embeddings, quantization or numerical kernels during a performance-only change.
  The accepted encoder and its fixed arithmetic are specified in LLP 0002.
- Ingestion and retrieval algorithms are fixed, with no runtime flags, environment
  variables or configuration knobs. POS is always enabled at weight 0.75, and
  heading-tree membership is always shared. Diagnostic output controls do not
  change algorithms; store location, presentation and query filters remain CLI inputs.
- Writers supply text, not author evidence, terms, or section_terms. Leave legacy
  author fields unread; do not add a column or an enrichment step for them.
- Documents must ingest on their own. Do not manufacture keyword lists, labels,
  summaries, or other content surrogates to improve retrieval or evaluation.
  Index the original content using only the specified derived indexing and model.
- Sections and blocks are derived byte spans, never independent records. Labels,
  filenames, and ancestor heading paths do not become lexical body evidence.
- New CLI records are accepted immediately; corrections use a child or a close
  envelope. Preserve accepted-state and lifecycle filtering for stored records.
- Cards are leads to original bytes; hints and related pointers are not answers.
  Preserve an honest miss instead of adding unmeasured retrieval behavior.
- Embed the local static model at build time. Runtime ingest and search must not download or require a second runtime.
- Keep linguistic annotation in `src/nlp` independent of storage and ranking;
  compile its pinned data into the binary and preserve exact annotation semantics.
- Keep parsing, execution and presentation separate, with typed errors and a thin Clap-derived main. Results go to stdout; diagnostics go to stderr.
- Resolve paths by explicit flag, environment, configuration, then XDG default.
  Reject relative XDG values. Search must not depend on cwd.
- Read the governing LLP before changing a subsystem. Update durable decisions
  in its LLP in the same change; use sparse `@ref LLP NNNN#anchor` comments.
- LLPs cite only files in this repository and other LLPs. ARCHITECTURE.md is a map.
- Run development tools through the flake:
  `nix develop -c cargo fmt`,
  `nix develop -c cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `nix develop -c cargo test --workspace --all-features`, `nix build`, and
  `nix develop -c python3 scripts/check_llp_refs.py`.
- Dev retrieval and first-page ids must match the measured page before freezing and
  scoring holdout. Counts alone do not establish parity. Do not commit corpus,
  model weights, or evaluation output.
- The neutral fixed-loop gate table is a separate measurement, not permission to
  retune the store. Keep corpus-free scorer tests; do not claim full neutral
  validation from an empty selection or a small fixture run.
