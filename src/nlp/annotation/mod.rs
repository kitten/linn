//! Linguistic annotation shared by the runtime and build-time resource compiler.
pub(super) mod chunker;
pub(super) mod dictionary;
pub(super) mod multiword;
pub(super) mod regex;
pub(super) mod rules;
pub(super) mod serialization;
pub(super) mod tokenizer;
pub(super) mod types;

pub use tokenizer::Tokenizer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported text NLP engine in rule {0}")]
    UnsupportedTextEngine(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] bincode::Error),
}
