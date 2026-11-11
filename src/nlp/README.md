# Linguistic annotation runtime

This domain owns linguistic annotation independently of storage and retrieval.
`language/pos.rs` adapts its tokens, tags and chunks to Linn's relation selector.

- `porter.rs`: direct access to the two pinned libSQL Porter tokenizers, with
  thread-owned native handles and no document-store access.
- `annotation.rs`: shared annotation declarations for runtime and the build-time
  asset compiler; the compiler does not depend on the Porter adapter.
- `tokenizer.rs`: source spans, sentence splitting, and ordered rule application.
- `tokenizer/tag.rs`: dictionary FST lookup and flat word/tag arrays. The upstream
  wire dictionary is converted once at build time, not reconstructed at startup.
- `tokenizer/chunk.rs`: the pinned statistical chunker, precompiled immutable
  feature lookups, and its exact arithmetic.
- `tokenizer/multiword.rs`: source-written multiword dictionary entries.
- `rule/`: disambiguation, group captures, exclusions and matching order.
- `types.rs`, `filter/`, `utils/`: annotation coordinates and matching primitives.

`build.rs` verifies the pinned English input and embeds a lookup-ready asset.
No runtime file, download, grammar-correction engine, XML compiler or alternate
NLP backend is required. Source derived from nlprule retains its attribution and
licenses in [LICENSE](LICENSE). The unchanged crate is used only by the eval package
for differential analysis, not by the product or its build.

[LLP 0002](../../llp/0002-retrieval-parity.explainer.md) governs linguistic
semantics. `evals/examples/pos_engine_parity.rs` compares complete annotations
and exercises the compiled asset as well as the compatibility decoder.
