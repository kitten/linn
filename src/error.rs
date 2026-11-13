#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Application(#[from] ApplicationError),
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
    #[error(
        "store schema {found} is incompatible with this binary (supports {supported}); use a matching or newer binary"
    )]
    Schema { found: usize, supported: usize },
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    Invalid,
    Io,
    Store,
    Schema,
    Annotation,
}
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error("{message}")]
pub struct ApplicationError {
    pub code: ErrorCode,
    pub diagnostics: Vec<String>,
    pub message: String,
    pub mutation_may_have_committed: bool,
}
impl ApplicationError {
    pub fn new(error: Error, mutating: bool) -> Self {
        let code = match error {
            Error::Invalid(_) | Error::Json(_) => ErrorCode::Invalid,
            Error::Io(_) => ErrorCode::Io,
            Error::Sql(_) => ErrorCode::Store,
            Error::Schema { .. } => ErrorCode::Schema,
            Error::Porter(_) => ErrorCode::Annotation,
            Error::Application(error) => return error,
        };
        Self {
            diagnostics: Vec::new(),
            code,
            message: error.to_string(),
            mutation_may_have_committed: mutating,
        }
    }
}
