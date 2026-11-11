# Language resources

- `embedding.rs`: the embedded static model, token averaging, and normalization.
- `pos.rs`: adapts `nlp` annotations to query/source relation analysis,
  and same-file POS index publication and lookup.

Both resources are compiled into the binary. POS works with the original units
and occurrence evidence in `retrieval::ordered`; its source relations are
published through `storage::Store`.
The independent linguistic runtime lives in [nlp/](../nlp/README.md).

[LLP 0001](../../llp/0001-storage-and-sections.explainer.md) defines publication;
[LLP 0002](../../llp/0002-retrieval-parity.explainer.md) defines how the results
participate in retrieval.
