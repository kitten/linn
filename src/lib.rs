//! Linn's native store, retrieval algorithm, and command-line interface.
pub mod cli;
pub mod error;
pub mod language;
pub mod nlp;
pub mod retrieval;
pub mod storage;
pub mod sync;

// Preserve existing library imports while the implementation lives in domain modules.
pub use cli::paths;
pub use language::embedding as embed;
pub use retrieval::{lexical as search, query};
pub use storage::{self as store, record, sections};

pub mod diagnostics;
