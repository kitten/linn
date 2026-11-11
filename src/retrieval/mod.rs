//! Query interpretation, lexical pages, and ordered fallback retrieval.
//! @ref LLP 0002#lexical-evidence
pub(crate) mod confidence;
pub(crate) mod fallback;
pub(crate) mod hit_order;
pub mod lexical;
pub(crate) mod ordered;
pub mod query;
pub(crate) mod selection;
pub(crate) mod similarity;
