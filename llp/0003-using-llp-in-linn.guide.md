# LLP 0003: Using LLP in Linn

**Type:** Guide\
**Status:** Active\
**Systems:** LLP, ContributorWorkflow\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0004

## Workflow

The LLPs are the specification. Read the relevant LLP before implementing a subsystem.
When durable intent changes, update its governing LLP in the same change. Add a
numbered document only for independently useful cross-file intent. Use filenames
`NNNN-slug.explainer.md` or `NNNN-slug.guide.md` and the metadata fields above.
ARCHITECTURE.md is the short map; AGENTS.md is the contributor enforcement layer.

## References

Use sparse module references or narrow invariant references, for example
`@ref LLP 0001#byte-spans`. Do not mechanically annotate obvious code. Anchors
are stable interfaces. LLPs may cite only other LLPs and artifacts present in
this repository, never external resources or machine-local research documents.

Run `nix develop -c python3 scripts/check_llp_refs.py` to validate references.
