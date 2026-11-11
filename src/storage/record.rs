//! Immutable inputs and local eligibility projections.
//! @ref LLP 0001#original-bytes
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Close {
    pub target: String,
    pub reason: String,
    #[serde(default)]
    pub replacement: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub body: String,
    #[serde(skip)]
    pub(crate) stored_body: Option<StoredBody>,
    #[serde(skip)]
    pub(crate) projected_eligible: Option<bool>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default = "accepted")]
    pub state: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub close: Option<Close>,
    #[serde(default)]
    pub stale_after: Option<String>,
    // Preserve absent legacy fields separately from an explicit unknown writer.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "writer"
    )]
    pub writer: Option<Option<String>>,
    #[serde(default)]
    pub source: serde_json::Value,
    #[serde(default)]
    pub created: String,
}

/// Metadata for an original body that has not been read into this Store instance.
#[derive(Clone, Debug)]
pub(crate) struct StoredBody {
    pub hash: String,
    pub bytes: usize,
    pub nonblank: bool,
}

fn writer<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

fn accepted() -> String {
    "accepted".into()
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl Record {
    pub fn body_hash(&self) -> String {
        self.stored_body
            .as_ref()
            .map_or_else(|| hash(self.body.as_bytes()), |body| body.hash.clone())
    }
    pub fn body_len(&self) -> usize {
        self.stored_body
            .as_ref()
            .map_or(self.body.len(), |body| body.bytes)
    }
    pub fn body_nonblank(&self) -> bool {
        self.stored_body
            .as_ref()
            .map_or_else(|| !self.body.trim().is_empty(), |body| body.nonblank)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Validity {
    closed: BTreeSet<String>,
    superseded: BTreeSet<String>,
}

impl Validity {
    pub fn project(records: &[Record]) -> Self {
        let mut value = Self::default();
        for record in records.iter().filter(|r| r.state == "accepted") {
            if let Some(close) = &record.close {
                value.closed.insert(close.target.clone());
            } else if let Some(parent) = &record.parent {
                value.superseded.insert(parent.clone());
            }
        }
        value
    }

    pub fn eligible(&self, record: &Record, now: &str) -> bool {
        record.projected_eligible.unwrap_or_else(|| {
            record.state == "accepted"
                && record.close.is_none()
                && !self.closed.contains(&record.id)
                && !self.superseded.contains(&record.id)
        }) && record
            .stale_after
            .as_deref()
            .is_none_or(|stale| stale.is_empty() || stale > now)
            && record.body_len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: &str) -> Record {
        serde_json::from_value(serde_json::json!({"id":id,"body":" "})).unwrap()
    }
    #[test]
    fn accepted_children_and_closes_project_without_scope_filtering() {
        let mut parent = record("parent");
        parent.scope = Some("other/repo".into());
        let mut child = record("child");
        child.parent = Some("parent".into());
        let mut close = record("close");
        close.close = Some(Close {
            target: "child".into(),
            reason: "expire".into(),
            replacement: None,
        });
        let records = vec![parent, child, close];
        let validity = Validity::project(&records);
        assert!(
            records
                .iter()
                .all(|r| !validity.eligible(r, "2026-11-05T00:00:00Z"))
        );
        let untouched = record("untouched");
        assert!(validity.eligible(&untouched, "2026-11-05T00:00:00Z"));
    }
    #[test]
    fn candidate_child_does_not_hide_parent_and_expiry_is_textual() {
        let parent = record("parent");
        let mut child = record("child");
        child.parent = Some("parent".into());
        child.state = "candidate".into();
        let validity = Validity::project(&[parent.clone(), child]);
        assert!(validity.eligible(&parent, "2026-11-05T00:00:00Z"));
        let mut expired = parent;
        expired.stale_after = Some("2026-11-05T00:00:00Z".into());
        assert!(!validity.eligible(&expired, "2026-11-05T00:00:00Z"));
    }
}

#[cfg(test)]
mod attribution_tests {
    use super::*;
    #[test]
    fn missing_writer_stays_missing_and_legacy_attribution_roundtrips() {
        let new: Record =
            serde_json::from_value(serde_json::json!({"id":"n","body":"original"})).unwrap();
        assert!(serde_json::to_value(&new).unwrap().get("writer").is_none());
        let legacy: Record = serde_json::from_value(
            serde_json::json!({"id":"n","body":"original","writer":"agent"}),
        )
        .unwrap();
        assert_eq!(serde_json::to_value(&legacy).unwrap()["writer"], "agent");
        let unknown: Record =
            serde_json::from_value(serde_json::json!({"id":"n","body":"original","writer":null}))
                .unwrap();
        assert_eq!(unknown.writer, Some(None));
        assert_eq!(
            serde_json::to_value(&unknown).unwrap().get("writer"),
            Some(&serde_json::Value::Null)
        );
    }
}

#[cfg(test)]
mod writer_migration_tests {
    use crate::storage::Store;

    #[tokio::test]
    async fn writer_column_migration_preserves_legacy_envelopes_and_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let records = [
            serde_json::json!({"id":"absent","body":"Original legacy bytes."}),
            serde_json::json!({"id":"known","body":"Another legacy note.","writer":"operator"}),
        ]
        .into_iter()
        .map(|r| serde_json::from_value(r).unwrap())
        .collect();
        let store = Store::build(&path, records).await.unwrap();
        let mut rows = store
            .db
            .query(
                "SELECT id,envelope,envelope_hash FROM records ORDER BY id",
                (),
            )
            .await
            .unwrap();
        let mut before = Vec::new();
        while let Some(row) = rows.next().await.unwrap() {
            before.push((
                row.get::<String>(0).unwrap(),
                row.get::<Vec<u8>>(1).unwrap(),
                row.get::<String>(2).unwrap(),
            ));
        }
        drop(rows);
        store
            .db
            .execute_batch("DROP INDEX records_writer_created; DROP INDEX records_writer_expiry; ALTER TABLE records DROP COLUMN writer; DROP TABLE sync_remotes; DROP TABLE sync_state; DROP INDEX sync_scope_records; PRAGMA user_version=1;")
            .await
            .unwrap();
        drop(store);
        let mut reopened = Store::open(&path).await.unwrap();
        for rebuild in [false, true] {
            if rebuild {
                reopened.rebuild().await.unwrap();
            }
            for (id, envelope, hash) in &before {
                let mut rows = reopened
                    .db
                    .query(
                        "SELECT writer,envelope,envelope_hash FROM records WHERE id=?",
                        [id.as_str()],
                    )
                    .await
                    .unwrap();
                let row = rows.next().await.unwrap().unwrap();
                assert_eq!(
                    row.get::<Option<String>>(0).unwrap().as_deref(),
                    if id == "known" {
                        Some("operator")
                    } else {
                        None
                    }
                );
                assert_eq!(&row.get::<Vec<u8>>(1).unwrap(), envelope);
                assert_eq!(&row.get::<String>(2).unwrap(), hash);
            }
        }
    }
}
