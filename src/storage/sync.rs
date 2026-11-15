//! @ref LLP 0008#replication-and-publication
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
    pub(super) db: Connection,
    pub(super) path: PathBuf,
    pub(crate) read_barrier: Option<super::publication::ReadBarrier>,
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
            "PRAGMA busy_timeout=5000; PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-2000; PRAGMA mmap_size=0;",
        )
        .await?;
        migrations::apply(&db).await?;
        super::publication::enable_wal(&db).await?;
        let mut rows = db.query("SELECT NOT EXISTS(SELECT 1 FROM records) AND NOT EXISTS(SELECT 1 FROM projections WHERE key='structure') AND NOT EXISTS(SELECT 1 FROM sync_state WHERE key='pending')", ()).await?;
        let initialize = rows.next().await?.unwrap().get::<i64>(0)? != 0;
        drop(rows);
        if initialize {
            Store::open(path).await?;
        }
        Ok(Self {
            _database: database,
            db,
            read_barrier: None,
            path: path.to_owned(),
        })
    }
    pub(super) async fn transaction(
        &self,
        behavior: TransactionBehavior,
    ) -> Result<super::publication::Transaction> {
        let tx = super::publication::transaction(&self.db, behavior).await?;
        migrations::check(&tx).await?;
        Ok(tx)
    }
    pub(crate) async fn inventory_budget(&self) -> Result<[u64; 2]> {
        let mut rows = self
            .db
            .query(
                "SELECT (SELECT count(*) FROM records)+(SELECT count(*) FROM sync_incoming WHERE json_extract(item,'$.envelope') IS NOT NULL),(SELECT count(*) FROM tombstones)+(SELECT count(*) FROM sync_incoming WHERE json_extract(item,'$.envelope') IS NULL)",
                (),
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("missing inventory counts".into()))?;
        Ok([row.get::<i64>(0)? as u64, row.get::<i64>(1)? as u64])
    }
    pub async fn inventory(
        &self,
        tombstones: bool,
        after: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Vec<Entry>> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let after = after.unwrap_or("");
        let sql = if tombstones {
            "SELECT id,NULL FROM (SELECT id FROM tombstones UNION SELECT id FROM sync_incoming WHERE json_extract(item,'$.envelope') IS NULL) WHERE id>? ORDER BY id LIMIT 256"
        } else {
            "SELECT id,hash FROM (SELECT id,envelope_hash AS hash,original_scope AS scope FROM records WHERE NOT EXISTS(SELECT 1 FROM sync_incoming s WHERE s.id=records.id) UNION ALL SELECT id,json_extract(item,'$.envelope_hash'),json_extract(json_extract(item,'$.envelope'),'$.scope') FROM sync_incoming WHERE json_extract(item,'$.envelope') IS NOT NULL AND NOT EXISTS(SELECT 1 FROM tombstones t WHERE t.id=sync_incoming.id)) WHERE id>? AND (? IS NULL OR scope=? COLLATE NOCASE) ORDER BY id LIMIT 256"
        };
        let mut rows = if tombstones {
            tx.query(sql, [after]).await?
        } else {
            tx.query(sql, params![after, scope, scope]).await?
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
        let mut rows = tx.query("SELECT item FROM sync_incoming WHERE id=? AND NOT EXISTS(SELECT 1 FROM tombstones WHERE id=?)", params![id,id]).await?;
        if let Some(row) = rows.next().await? {
            let item: Item = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
            let record = item.validate(None)?;
            if scope.is_some_and(|s| {
                record.as_ref().is_some_and(|r| {
                    r.scope
                        .as_deref()
                        .is_none_or(|v| !v.eq_ignore_ascii_case(s))
                })
            }) {
                return Ok(None);
            }
            return Ok(Some(item));
        }
        drop(rows);
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
        let mut rows = tx.query("SELECT NULL AS hash,0 AS priority FROM tombstones WHERE id=? UNION ALL SELECT json_extract(item,'$.envelope_hash'),1 FROM sync_incoming WHERE id=? UNION ALL SELECT envelope_hash,2 FROM records WHERE id=? ORDER BY priority LIMIT 1", params![entry.id.as_str(),entry.id.as_str(),entry.id.as_str()]).await?;
        match rows.next().await? {
            None => Ok(true),
            Some(row) => {
                let hash = row.get::<Option<String>>(0)?;
                if hash.is_none() {
                    Ok(false)
                } else if entry.hash.is_none() {
                    Ok(true)
                } else if hash == entry.hash {
                    Ok(false)
                } else {
                    Err(crate::mesh::failure::Kind::Conflict
                        .error(format!("immutable record conflict: {}", entry.id)))
                }
            }
        }
    }
    async fn apply_published(&self, item: &Item, scope: Option<&str>) -> Result<bool> {
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
                    Err(crate::mesh::failure::Kind::Conflict
                        .error(format!("immutable record conflict: {}", item.id)))
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
    pub async fn apply(&self, item: &Item, scope: Option<&str>) -> Result<bool> {
        item.validate(scope)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query(
                "SELECT item FROM sync_incoming WHERE id=?",
                [item.id.as_str()],
            )
            .await?;
        let staged = rows
            .next()
            .await?
            .map(|r| r.get::<Vec<u8>>(0))
            .transpose()?;
        drop(rows);
        if let Some(bytes) = staged {
            let previous: Item = serde_json::from_slice(&bytes)?;
            if previous.envelope.is_none() || previous.envelope_hash == item.envelope_hash {
                return Ok(false);
            }
            if item.envelope.is_some() {
                return Err(crate::mesh::failure::Kind::Conflict
                    .error(format!("immutable record conflict: {}", item.id)));
            }
        }
        // Published tombstones always win; published identical records are no-ops.
        let mut rows = tx
            .query("SELECT 1 FROM tombstones WHERE id=?", [item.id.as_str()])
            .await?;
        if rows.next().await?.is_some() {
            return Ok(false);
        }
        drop(rows);
        if item.envelope.is_some() {
            let mut rows = tx
                .query(
                    "SELECT envelope_hash FROM records WHERE id=?",
                    [item.id.as_str()],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                return if Some(row.get::<String>(0)?) == item.envelope_hash {
                    Ok(false)
                } else {
                    Err(crate::mesh::failure::Kind::Conflict
                        .error(format!("immutable record conflict: {}", item.id)))
                };
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO sync_incoming VALUES(?,?)",
            params![item.id.as_str(), serde_json::to_vec(item)?],
        )
        .await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn finish(&self) -> Result<()> {
        self.publish(false).await
    }
    pub(crate) async fn rebuild(&self) -> Result<()> {
        self.publish(true).await
    }
    async fn publish(&self, force: bool) -> Result<()> {
        // Acquire the writer before opening the reconstruction handle, so all
        // original changes and every derived publication share one transaction.
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx.query("SELECT EXISTS(SELECT 1 FROM sync_state WHERE key='pending'),EXISTS(SELECT 1 FROM sync_state WHERE key='erase-checkpoint')", ()).await?;
        let row = rows.next().await?.unwrap();
        let pending = row.get::<i64>(0)? != 0;
        let mut erased = pending || row.get::<i64>(1)? != 0;
        drop(row);
        drop(rows);
        let mut changed = false;
        loop {
            let mut rows = tx
                .query("SELECT item FROM sync_incoming ORDER BY id LIMIT 256", ())
                .await?;
            let mut items = Vec::new();
            let mut bytes = 0;
            while let Some(row) = rows.next().await? {
                let encoded = row.get::<Vec<u8>>(0)?;
                bytes += encoded.len();
                items.push(serde_json::from_slice::<Item>(&encoded)?);
                if bytes >= 4 * 1024 * 1024 {
                    break;
                }
            }
            drop(rows);
            if items.is_empty() {
                break;
            }
            // Reuse staging pages inside the transaction. Rollback restores
            // accepted originals if any merge or publication step fails.
            for item in &items {
                tx.execute("DELETE FROM sync_incoming WHERE id=?", [item.id.as_str()])
                    .await?;
            }
            for item in items {
                let applied = self.apply_published(&item, None).await?;
                // A discarded staged original may already have a tombstone.
                erased |= item.envelope.is_none() || !applied;
                changed |= applied;
            }
        }
        if erased {
            tx.execute(
                "INSERT OR REPLACE INTO sync_state VALUES('erase-checkpoint',1)",
                (),
            )
            .await?;
        }
        if !force && !changed && !pending && !erased {
            tx.rollback().await?;
            return super::publication::settle_wal(&self.db).await;
        }
        if force || changed || pending {
            if let Some(barrier) = &self.read_barrier {
                let _publication = barrier.write().await;
                barrier.clear_reader();
            }
            let database = Builder::new_local(&self.path).build().await?;
            let mut store = Store::open_connection(database, self.db.clone(), true).await?;
            store.rebuild().await?;
        }
        let mut readers = if let Some(barrier) = &self.read_barrier {
            Some(barrier.write().await)
        } else {
            None
        };
        tx.commit().await?;
        if let Some(barrier) = &self.read_barrier {
            barrier.clear_reader();
        }
        if (force || changed || pending)
            && let Some(readers) = readers.as_mut()
        {
            **readers = None;
        }
        if erased {
            super::publication::checkpoint_erasure(&self.db).await?;
        } else {
            super::publication::settle_wal(&self.db).await?;
        }
        self.db.execute_batch("PRAGMA shrink_memory").await?;
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
        for result in [
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
        let mut rows = writer
            .query(
                "SELECT (SELECT count(*) FROM tombstones),(SELECT value FROM future_only)",
                (),
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<i64>(0).unwrap(), 0);
        assert_eq!(row.get::<String>(1).unwrap(), "untouched");
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
            let result = worker.join().unwrap();
            assert!(matches!(result, Err(Error::Schema { .. })), "{result:?}");
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
