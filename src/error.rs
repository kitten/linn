#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Porter(#[from] crate::nlp::porter::Error),
    #[error(transparent)]
    Sql(#[from] libsql::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}
pub type Result<T> = std::result::Result<T, Error>;
