//! Embedded linguistic annotation, independent of Linn storage and retrieval.
//!
//! The runtime reads the prepared pinned English asset. Runtime output
//! preserves original token spans, alternative tags, and statistical chunks.
//! @ref LLP 0002#pos-relation-selection
mod annotation;
pub mod porter;

pub use annotation::{Error, Tokenizer};
