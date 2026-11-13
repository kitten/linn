//! Typed file provenance; source references reuse the original body hash.
//! @ref LLP 0007#application-boundary
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputTask {
    Note { source_path: Option<String> },
    File(FileInput),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileInput {
    #[serde(with = "crate::native_path")]
    pub path: PathBuf,
    #[serde(default, with = "crate::native_path::optional")]
    pub repository: Option<PathBuf>,
}
pub(super) struct Prepared {
    pub kind: &'static str,
    pub path: Option<String>,
}
pub(super) fn prepare(task: &InputTask) -> Result<Prepared> {
    let (kind, path) = match task {
        InputTask::Note { source_path } => ("note", source_path.clone()),
        InputTask::File(file) => {
            if !file.path.is_absolute()
                || file.repository.as_ref().is_some_and(|p| !p.is_absolute())
            {
                return Err(Error::Invalid(
                    "file input context must use absolute paths".into(),
                ));
            }
            let path = file
                .repository
                .as_ref()
                .and_then(|root| file.path.strip_prefix(root).ok())
                .unwrap_or(&file.path)
                .to_string_lossy()
                .into_owned();
            ("doc", Some(path))
        }
    };
    Ok(Prepared { kind, path })
}
impl Prepared {
    pub fn source(&self, body_hash: &str) -> serde_json::Value {
        match self.kind {
            "doc" => serde_json::json!({"type":"repo-file","ref":body_hash}),
            _ => serde_json::json!({"type":"derived-from-thread","ref":null}),
        }
    }
}
