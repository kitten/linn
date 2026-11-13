//! Lossless local path transport; UTF-8 paths retain the lifecycle wire format.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(untagged)]
enum Encoded {
    Text(String),
    Native(OsString),
}
impl Encoded {
    fn path(self) -> PathBuf {
        match self {
            Self::Text(path) => path.into(),
            Self::Native(path) => path.into(),
        }
    }
}
struct Borrowed<'a>(&'a Path);
impl Serialize for Borrowed<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0.to_str() {
            Some(text) => text.serialize(serializer),
            None => self.0.as_os_str().serialize(serializer),
        }
    }
}
pub fn serialize<S: Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    Borrowed(path).serialize(serializer)
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PathBuf, D::Error> {
    Ok(Encoded::deserialize(deserializer)?.path())
}
pub mod optional {
    use super::*;
    pub fn serialize<S: Serializer>(
        path: &Option<PathBuf>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        path.as_deref().map(Borrowed).serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<PathBuf>, D::Error> {
        Ok(Option::<Encoded>::deserialize(deserializer)?.map(Encoded::path))
    }
}
