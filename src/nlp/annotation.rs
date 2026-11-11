//! Shared annotation decoder and runtime declarations used by the asset compiler.
#[path = "filter/mod.rs"]
pub(super) mod filter;
#[path = "rule/mod.rs"]
pub(super) mod rule;
#[path = "tokenizer.rs"]
pub(super) mod tokenizer;
#[path = "types.rs"]
pub(super) mod types;
#[path = "utils/mod.rs"]
pub(super) mod utils;

pub use tokenizer::Tokenizer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] bincode::Error),
}
