//! @ref LLP 0006#merge-and-publication
use super::{
    Store, migrations,
    record::{Record, hash},
};
use crate::error::{Error, Result};
use libsql::{Builder, Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PAGE: usize = 256;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub origin: String,
    pub scope: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub hash: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    /// None is a permanent tombstone; otherwise the exact serialized envelope.
    pub envelope: Option<String>,
    pub envelope_hash: Option<String>,
    pub body_hash: Option<String>,
}
impl Item {
    pub(crate) fn validate(&self, scope: Option<&str>) -> Result<Option<Record>> {
        if self.id.is_empty() {
            return Err(Error::Invalid("empty sync record ID".into()));
        }
        let record = if let Some(envelope) = &self.envelope {
            let r: Record = serde_json::from_str(envelope)?;
            if r.id != self.id
                || Some(hash(envelope.as_bytes())) != self.envelope_hash
                || Some(r.body_hash()) != self.body_hash
            {
                return Err(Error::Invalid("sync envelope ID/hash mismatch".into()));
            }
            if scope.is_some_and(|s| r.scope.as_ref().is_none_or(|v| !v.eq_ignore_ascii_case(s))) {
                return Err(Error::Invalid("sync record outside selected scope".into()));
            }
            Some(r)
        } else {
            if self.body_hash.is_some() || self.envelope_hash.is_some() {
                return Err(Error::Invalid("invalid tombstone".into()));
            }
            None
        };
        Ok(record)
    }
}
pub struct Replica {
    _database: libsql::Database,
    db: Connection,
    path: PathBuf,
}
impl Replica {
    pub async fn open(path: &Path) -> Result<Self> {
        use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        let database = Builder::new_local(path).build().await?;
        let db = database.connect()?;
        db.execute_batch(
            "PRAGMA busy_timeout=5000; PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON;",
        )
        .await?;
        migrations::apply(&db).await?;
        Ok(Self {
            _database: database,
            db,
            path: path.to_owned(),
        })
    }
    async fn transaction(&self, behavior: TransactionBehavior) -> Result<libsql::Transaction> {
        let tx = self.db.transaction_with_behavior(behavior).await?;
        migrations::check(&tx).await?;
        Ok(tx)
    }
    pub async fn remotes(&self) -> Result<Vec<Remote>> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query(
                "SELECT name,origin,scope FROM sync_remotes ORDER BY name",
                (),
            )
            .await?;
        let mut out = Vec::new();
        while let Some(r) = rows.next().await? {
            out.push(Remote {
                name: r.get(0)?,
                origin: r.get(1)?,
                scope: r.get(2)?,
            });
        }
        Ok(out)
    }
    pub async fn pair(&self, remote: &Remote) -> Result<()> {
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let inserted = tx
            .execute(
                "INSERT INTO sync_remotes VALUES(?,?,?) ON CONFLICT(name) DO NOTHING",
                params![
                    remote.name.clone(),
                    remote.origin.clone(),
                    remote.scope.clone()
                ],
            )
            .await?;
        if inserted == 0 {
            return Err(Error::Invalid(format!(
                "sync remote {} already exists; remove it before pairing again",
                remote.name
            )));
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn remove(&self, name: &str) -> Result<()> {
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        if tx
            .execute("DELETE FROM sync_remotes WHERE name=?", [name])
            .await?
            == 0
        {
            return Err(Error::Invalid(format!("unknown sync remote: {name}")));
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn inventory(
        &self,
        tombstones: bool,
        after: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Vec<Entry>> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let after = after.unwrap_or("");
        let mut rows = if tombstones {
            tx.query(
                "SELECT id,NULL FROM tombstones WHERE id>? ORDER BY id LIMIT 256",
                [after],
            )
            .await?
        } else if let Some(scope) = scope {
            tx.query("SELECT id,envelope_hash FROM records WHERE original_scope=? COLLATE NOCASE AND id>? ORDER BY id LIMIT 256", params![scope,after]).await?
        } else {
            tx.query(
                "SELECT id,envelope_hash FROM records WHERE id>? ORDER BY id LIMIT 256",
                [after],
            )
            .await?
        };
        let mut out = Vec::new();
        while let Some(r) = rows.next().await? {
            out.push(Entry {
                id: r.get(0)?,
                hash: r.get(1)?,
            });
        }
        Ok(out)
    }
    pub async fn get(&self, id: &str, scope: Option<&str>) -> Result<Option<Item>> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT id FROM tombstones WHERE id=?", [id])
            .await?;
        if rows.next().await?.is_some() {
            return Ok(Some(Item {
                id: id.into(),
                envelope: None,
                envelope_hash: None,
                body_hash: None,
            }));
        }
        let mut rows=tx.query("SELECT envelope,envelope_hash,body_hash FROM records WHERE id=? AND (? IS NULL OR original_scope=? COLLATE NOCASE)",params![id,scope,scope]).await?;
        rows.next()
            .await?
            .map(|r| {
                Ok(Item {
                    id: id.into(),
                    envelope: Some(
                        String::from_utf8(r.get::<Vec<u8>>(0)?)
                            .map_err(|_| Error::Invalid("non-UTF-8 envelope".into()))?,
                    ),
                    envelope_hash: Some(r.get(1)?),
                    body_hash: Some(r.get(2)?),
                })
            })
            .transpose()
    }
    pub async fn needs(&self, entry: &Entry) -> Result<bool> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT 1 FROM tombstones WHERE id=?", [entry.id.as_str()])
            .await?;
        if rows.next().await?.is_some() {
            return Ok(false);
        }
        if entry.hash.is_none() {
            return Ok(true);
        }
        let mut rows = tx
            .query(
                "SELECT envelope_hash FROM records WHERE id=?",
                [entry.id.as_str()],
            )
            .await?;
        match rows.next().await? {
            None => Ok(true),
            Some(row) if Some(row.get::<String>(0)?) == entry.hash => Ok(false),
            Some(_) => Err(Error::Invalid(format!(
                "immutable record conflict: {}",
                entry.id
            ))),
        }
    }
    pub async fn apply(&self, item: &Item, scope: Option<&str>) -> Result<bool> {
        let record = item.validate(scope)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query("SELECT 1 FROM tombstones WHERE id=?", [item.id.as_str()])
            .await?;
        let forgotten = rows.next().await?.is_some();
        drop(rows);
        if forgotten {
            tx.rollback().await?;
            return Ok(false);
        }
        if let Some(r) = record {
            let mut rows = tx
                .query(
                    "SELECT envelope_hash FROM records WHERE id=?",
                    [item.id.as_str()],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                let same = Some(row.get::<String>(0)?) == item.envelope_hash;
                drop(rows);
                tx.rollback().await?;
                return if same {
                    Ok(false)
                } else {
                    Err(Error::Invalid(format!(
                        "immutable record conflict: {}",
                        item.id
                    )))
                };
            }
            drop(rows);
            tx.execute(
                "INSERT INTO records VALUES(?,NULL,?,?,?,?,?,?)",
                params![
                    r.id,
                    r.scope,
                    r.state,
                    r.body.as_bytes().to_vec(),
                    item.body_hash.clone(),
                    item.envelope.as_ref().unwrap().as_bytes().to_vec(),
                    item.envelope_hash.clone()
                ],
            )
            .await?;
        } else {
            super::publication::erase_record(&tx, &item.id).await?;
        }
        tx.execute("INSERT OR REPLACE INTO sync_state VALUES('pending',1)", ())
            .await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn finish(&self) -> Result<()> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT 1 FROM sync_state WHERE key='pending'", ())
            .await?;
        let pending = rows.next().await?.is_some();
        drop(rows);
        tx.rollback().await?;
        if pending {
            let mut store = Store::open_for_rebuild(&self.path).await?;
            store.rebuild().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod schema_tests {
    use super::*;
    #[tokio::test]
    async fn resident_handles_refuse_future_schema_without_changing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("future.db");
        let mut store = Store::open(&path).await.unwrap();
        let replica = Replica::open(&path).await.unwrap();
        let database = Builder::new_local(&path).build().await.unwrap();
        let writer = database.connect().unwrap();
        writer.execute_batch(&format!("BEGIN IMMEDIATE; CREATE TABLE future_only(value TEXT); INSERT INTO future_only VALUES('untouched'); PRAGMA user_version={}; COMMIT;", super::super::SCHEMA_VERSION + 1)).await.unwrap();
        let remote = Remote {
            name: "blocked".into(),
            origin: "ssh://example.invalid".into(),
            scope: None,
        };
        for result in [
            replica.pair(&remote).await,
            replica.remove("blocked").await,
            replica
                .apply(
                    &Item {
                        id: "blocked".into(),
                        envelope: None,
                        envelope_hash: None,
                        body_hash: None,
                    },
                    None,
                )
                .await
                .map(|_| ()),
            replica.finish().await,
            replica.remotes().await.map(|_| ()),
            replica.inventory(false, None, None).await.map(|_| ()),
            replica.get("blocked", None).await.map(|_| ()),
            replica
                .needs(&Entry {
                    id: "blocked".into(),
                    hash: None,
                })
                .await
                .map(|_| ()),
            store.integrity().await,
            store.forget("blocked").await,
        ] {
            let error = crate::error::ApplicationError::new(result.unwrap_err(), true);
            assert!(matches!(error.code, crate::error::ErrorCode::Schema));
        }
        let mut rows = writer.query("SELECT (SELECT count(*) FROM sync_remotes),(SELECT count(*) FROM tombstones),(SELECT value FROM future_only)", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<i64>(0).unwrap(), 0);
        assert_eq!(row.get::<i64>(1).unwrap(), 0);
        assert_eq!(row.get::<String>(2).unwrap(), "untouched");
        assert!(Store::open(&path).await.is_err());
        assert!(Replica::open(&path).await.is_err());
    }

    #[test]
    fn queued_writer_checks_schema_after_acquiring_sqlite_write_lock() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("race.db");
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (go_tx, go_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn({
            let path = path.clone();
            move || {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let replica = Replica::open(&path).await.unwrap();
                    ready_tx.send(()).unwrap();
                    go_rx.recv().unwrap();
                    replica
                        .apply(
                            &Item {
                                id: "blocked".into(),
                                envelope: None,
                                envelope_hash: None,
                                body_hash: None,
                            },
                            None,
                        )
                        .await
                })
            }
        });
        ready_rx.recv().unwrap();
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let database = Builder::new_local(&path).build().await.unwrap();
            let db = database.connect().unwrap();
            let migration = db
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .await
                .unwrap();
            migration
                .execute_batch(&format!(
                    "CREATE TABLE future_only(value TEXT); PRAGMA user_version={};",
                    super::super::SCHEMA_VERSION + 1
                ))
                .await
                .unwrap();
            go_tx.send(()).unwrap();
            migration.commit().await.unwrap();
            assert!(matches!(worker.join().unwrap(), Err(Error::Schema { .. })));
            let mut rows = db
                .query("SELECT count(*) FROM tombstones", ())
                .await
                .unwrap();
            assert_eq!(
                rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
                0
            );
        });
    }
}
