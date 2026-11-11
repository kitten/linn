//! Embedded linguistic annotation, independent of Linn storage and retrieval.
//!
//! The compatibility decoder reads the pinned English resource. Runtime output
//! preserves original token spans, alternative tags, and statistical chunks.
//! @ref LLP 0002#pos-relation-selection
mod annotation;
pub mod porter;

pub use annotation::{Error, Tokenizer};
use annotation::{filter, rule, tokenizer, types, utils};
