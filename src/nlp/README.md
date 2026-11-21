# Linguistic annotation runtime

This domain owns linguistic annotation independently of storage and retrieval.
`language/pos.rs` adapts its tokens, tags and chunks to Linn's relation selector.

- `mod.rs`: public `Tokenizer` and `Error` exports plus the Porter adapter.
- `porter.rs`: the two pinned libSQL Porter tokenizers, with thread-owned native
  handles and no document-store access.
- `compiler.rs`: build-only legacy bincode decoding and conversion to the runtime
  asset. It shares `annotation/` without compiling the Porter adapter.
- `annotation/mod.rs`: the shared annotation module root and typed errors.
- `annotation/tokenizer.rs`: source spans, sentence splitting and the ordered
  annotation pipeline.
- `annotation/dictionary.rs`: word/POS identities, dictionary FST lookup and flat
  word/tag arrays. The legacy dictionary is converted once at build time.
- `annotation/chunker.rs`: statistical model execution, precompiled feature
  lookups and fixed arithmetic.
- `annotation/multiword.rs`: token-aligned dictionary phrase annotations.
- `annotation/types.rs`: coordinates, words, tokens and sentences.
- `annotation/rules/`: `pattern.rs` owns the serialized pattern schema and
  captures; `matcher.rs` owns ordered match iteration and exclusions;
  `actions.rs` applies disambiguation;
  `mod.rs` coordinates rule ownership, partial-POS filters and mutation.
- `annotation/regex.rs`: the fixed-resource regex adapter. Pattern lookup maps
  serialize canonically in `annotation/rules/pattern.rs`.

The runtime and compiler use the same annotation tree with explicit internal
paths. The compiler is outside that tree, so ordinary loading cannot decode the
legacy resource or depend on build-only wire types.

`build.rs` verifies the pinned English input and embeds a lookup-ready asset.
No runtime file, download, grammar-correction engine, XML compiler or alternate
NLP backend is required. The linguistic data retain their separate notices in
[LICENSE](LICENSE). The unchanged nlprule crate is used only by the
eval package for differential analysis, not by the product or its build.

[LLP 0002](../../llp/0002-retrieval-parity.explainer.md) governs linguistic
semantics. `evals/examples/pos_engine_parity.rs` compares complete annotations
and exercises the build-only compatibility decoder and deterministic asset writer.
The runtime omits upstream rule examples, no-op rules and the unused text engine.
It retains original rule order and diagnostic names. Regex matching uses the
regular engine, with an equivalent translation of the pinned URL pattern; it
does not accept arbitrary legacy lookaround expressions.
