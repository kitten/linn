# Retrieval

- `query.rs`: query parsing, filters, normalization, and content terms.
- `lexical.rs`: the two matchers, fusion, card assembly, hints, and navigation.
- `fallback.rs`: the ordered empty-page procedure and its derived encodings.
- `ordered.rs`: POS-selected occurrence evidence used by lexical scoring.
- `confidence.rs`: confidence features and the fixed open gate.
- `hit_order.rs`: posting iteration order.
- `selection.rs`: compatible top-k partition ordering.
- `similarity.rs`: exact floating-point vector and matrix kernels.

These modules operate on `storage::Store`. Language resources live in
`language`; retrieval chooses records and returns their original bytes.
[LLP 0002](../../llp/0002-retrieval-parity.explainer.md) governs the algorithm,
including numerical and tie ordering.
