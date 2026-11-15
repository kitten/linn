//! Shared publication and secure-forget primitives.
//! @ref LLP 0001#native-persistence
use super::*;

/// Internal helpers join the owning publication. Any helper error propagates
/// to that owner, whose transaction guard rolls back the complete replacement.
pub(crate) struct Transaction {
    root: Option<libsql::Transaction>,
    db: Connection,
}
impl std::ops::Deref for Transaction {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.db
    }
}
impl Transaction {
    pub(crate) async fn commit(self) -> Result<()> {
        if let Some(root) = self.root {
            root.commit().await?;
        }
        Ok(())
    }
    pub(crate) async fn rollback(self) -> Result<()> {
        if let Some(root) = self.root {
            root.rollback().await?;
        }
        Ok(())
    }
}
pub(super) async fn transaction(
    db: &Connection,
    behavior: libsql::TransactionBehavior,
) -> Result<Transaction> {
    let root = if db.is_autocommit() {
        Some(db.transaction_with_behavior(behavior).await?)
    } else {
        None
    };
    Ok(Transaction {
        root,
        db: db.clone(),
    })
}

pub(super) async fn enable_wal(db: &Connection) -> Result<()> {
    let mut rows = db.query("PRAGMA journal_mode=WAL", ()).await?;
    if rows.next().await?.unwrap().get::<String>(0)? != "wal" {
        return Err(Error::Invalid("store requires WAL journaling".into()));
    }
    drop(rows);
    // Keep FULL durability. Short reader snapshots allow automatic checkpoints;
    // truncate reusable WAL space when it exceeds the ordinary checkpoint size.
    db.execute_batch("PRAGMA synchronous=FULL; PRAGMA wal_autocheckpoint=256; PRAGMA journal_size_limit=4194304;").await?;
    let mut rows = db
        .query("SELECT 1 FROM sync_state WHERE key='erase-checkpoint'", ())
        .await?;
    let pending = rows.next().await?.is_some();
    drop(rows);
    if pending {
        checkpoint_erasure(db).await?;
    }
    Ok(())
}

pub(super) async fn settle_wal(db: &Connection) -> Result<()> {
    // Do not wait for readers owned by another process. A later publication or
    // close retries; our short daemon snapshots are drained by the commit gate.
    db.busy_timeout(std::time::Duration::ZERO)?;
    let checkpoint = db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").await;
    db.busy_timeout(std::time::Duration::from_secs(5))?;
    checkpoint?;
    Ok(())
}

pub(super) async fn checkpoint_erasure(db: &Connection) -> Result<()> {
    let mut rows = db.query("PRAGMA wal_checkpoint(TRUNCATE)", ()).await?;
    let busy = rows.next().await?.unwrap().get::<i64>(0)? != 0;
    drop(rows);
    if busy {
        return Err(Error::Invalid(
            "forget committed, but an external reader prevented WAL erasure; close it and retry"
                .into(),
        ));
    }
    db.execute("DELETE FROM sync_state WHERE key='erase-checkpoint'", ())
        .await?;
    Ok(())
}

#[derive(Clone)]
pub(crate) struct LiveLinks {
    pub publication: Vec<u8>,
    pub links: Arc<HashMap<String, HashSet<String>>>,
}
pub(crate) struct Publication {
    gate: tokio::sync::RwLock<Option<LiveLinks>>,
    reader: std::sync::Mutex<Option<Store>>,
}
impl std::ops::Deref for Publication {
    type Target = tokio::sync::RwLock<Option<LiveLinks>>;
    fn deref(&self) -> &Self::Target {
        &self.gate
    }
}
pub(crate) type ReadBarrier = Arc<Publication>;
pub(crate) fn read_barrier() -> ReadBarrier {
    Arc::new(Publication {
        gate: tokio::sync::RwLock::new(None),
        reader: std::sync::Mutex::new(None),
    })
}
impl Publication {
    pub(crate) fn take_reader(&self) -> Option<Store> {
        self.reader.lock().unwrap().take()
    }
    pub(crate) fn park_reader(&self, mut store: Option<Store>) {
        if let Some(store) = &mut store {
            store.read_barrier = None;
        }
        *self.reader.lock().unwrap() = store;
    }
    pub(super) fn clear_reader(&self) {
        self.reader.lock().unwrap().take();
    }
}

pub(super) async fn erase_record(db: &Connection, id: &str) -> Result<()> {
    db.execute("INSERT OR IGNORE INTO tombstones VALUES(?)", [id])
        .await?;
    db.execute("DELETE FROM sections WHERE record=?", [id])
        .await?;
    db.execute("DELETE FROM records WHERE id=?", [id]).await?;
    db.execute("DELETE FROM sync_incoming WHERE id=?", [id])
        .await?;
    db.execute(
        "INSERT OR REPLACE INTO sync_state VALUES('erase-checkpoint',1)",
        (),
    )
    .await?;
    let mut rows = db
        .query("SELECT EXISTS(SELECT 1 FROM sections)", ())
        .await?;
    let nonempty = rows.next().await?.unwrap().get::<i64>(0)? != 0;
    drop(rows);
    if !nonempty {
        db.execute("INSERT INTO fts(fts) VALUES('delete-all')", ())
            .await?;
    }
    db.execute("INSERT INTO fts(fts) VALUES('optimize')", ())
        .await?;
    Ok(())
}

impl Store {
    pub(super) async fn reject_staged_id(&self, id: &str) -> Result<()> {
        let mut rows = self
            .db
            .query(
                "SELECT json_extract(item,'$.envelope') IS NULL FROM sync_incoming WHERE id=?",
                [id],
            )
            .await?;
        if let Some(row) = rows.next().await? {
            return Err(Error::Invalid(if row.get::<i64>(0)? != 0 {
                "record is forgotten".into()
            } else {
                format!("record {id} already accepted by sync")
            }));
        }
        Ok(())
    }

    pub(crate) fn restore_live_links(&mut self, live: &LiveLinks) {
        if live.publication == self.structure_stamp {
            self.links = live.links.clone();
        }
    }

    pub(crate) async fn publication_transaction(&self) -> Result<Transaction> {
        transaction(&self.db, libsql::TransactionBehavior::Immediate).await
    }

    pub(super) async fn commit_publication(
        &self,
        tx: Transaction,
        result: Result<()>,
        erased: bool,
    ) -> Result<()> {
        match result {
            Ok(()) => {
                let mut readers = if let Some(barrier) = &self.read_barrier {
                    Some(barrier.write().await)
                } else {
                    None
                };
                tx.commit().await?;
                if let Some(barrier) = &self.read_barrier {
                    barrier.clear_reader();
                }
                if let Some(readers) = readers.as_mut() {
                    **readers = erased.then(|| LiveLinks {
                        publication: self.structure_stamp.clone(),
                        links: self.links.clone(),
                    });
                }
                if self.db.is_autocommit() {
                    if erased {
                        checkpoint_erasure(&self.db).await?;
                    } else {
                        settle_wal(&self.db).await?;
                    }
                }
                Ok(())
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
    pub(crate) fn ensure_usable(&self) -> Result<()> {
        if self.write_failed {
            return Err(Error::Invalid(
                "a write failed on this store handle; discard it and reopen the store".into(),
            ));
        }
        Ok(())
    }
    pub(super) async fn complete_publication(&self) -> Result<()> {
        let tx = self.publication_transaction().await?;
        self.verify_source_version(&tx).await?;
        tx.execute("DELETE FROM sync_state WHERE key='pending'", ())
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(super) fn write_outcome(&mut self, result: Result<()>) -> Result<()> {
        result.map_err(|e| {
            self.write_failed = true;
            let mut error = crate::error::ApplicationError::new(e, true);
            error.message = format!("{}; write may have committed: reopen the store, and run index:rebuild if publication is incomplete", error.message);
            error.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: &str, body: &str) -> Record {
        serde_json::from_value(serde_json::json!({"id":id,"body":body})).unwrap()
    }
    fn item(record: &Record) -> sync::Item {
        let envelope = serde_json::to_string(record).unwrap();
        sync::Item {
            id: record.id.clone(),
            envelope_hash: Some(hash(envelope.as_bytes())),
            body_hash: Some(record.body_hash()),
            envelope: Some(envelope),
        }
    }
    async fn count(db: &Connection, table: &str) -> i64 {
        db.query(&format!("SELECT count(*) FROM {table}"), ())
            .await
            .unwrap()
            .next()
            .await
            .unwrap()
            .unwrap()
            .get(0)
            .unwrap()
    }
    #[tokio::test]
    async fn joined_commits_do_not_publish_and_owner_drop_rolls_back_every_helper() {
        let database = Builder::new_local(":memory:").build().await.unwrap();
        let db = database.connect().unwrap();
        db.execute("CREATE TABLE sample(id INTEGER)", ())
            .await
            .unwrap();
        let outer = transaction(&db, libsql::TransactionBehavior::Immediate)
            .await
            .unwrap();
        outer
            .execute("INSERT INTO sample VALUES(1)", ())
            .await
            .unwrap();
        let nested = transaction(&db, libsql::TransactionBehavior::Immediate)
            .await
            .unwrap();
        nested
            .execute("INSERT INTO sample VALUES(2)", ())
            .await
            .unwrap();
        nested.commit().await.unwrap();
        assert!(!db.is_autocommit());
        assert_eq!(count(&db, "sample").await, 2);
        drop(outer);
        assert_eq!(count(&db, "sample").await, 0);
    }
    #[tokio::test]
    async fn staging_and_failed_publication_preserve_complete_reader_snapshots() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        drop(
            Store::build(&path, vec![record("old", "Existing routing evidence.")])
                .await
                .unwrap(),
        );
        let replica = sync::Replica::open(&path).await.unwrap();
        replica
            .apply(&item(&record("new", "New compiler evidence.")), None)
            .await
            .unwrap();
        drop(replica);
        let mut reader = Store::open(&path).await.unwrap();
        assert_eq!(count(&reader.db, "records").await, 1);
        assert!(
            reader
                .add(record("new", "Conflicting local bytes."))
                .await
                .is_err()
        );
        assert!(
            Store::build(&path, vec![record("new", "Conflicting local bytes.")])
                .await
                .is_err()
        );
        let replica = sync::Replica::open(&path).await.unwrap();
        replica.db.execute_batch("CREATE TRIGGER reject_catalog BEFORE INSERT ON record_read_index BEGIN SELECT RAISE(ABORT,'injected publication failure'); END;").await.unwrap();
        assert!(replica.finish().await.is_err());
        assert_eq!(count(&reader.db, "records").await, 1);
        assert_eq!(count(&reader.db, "sync_incoming").await, 1);
        assert!(!reader.publication_current().await.unwrap());
        replica
            .db
            .execute("DROP TRIGGER reject_catalog", ())
            .await
            .unwrap();
        reader = Store::open(&path).await.unwrap();
        let snapshot = reader.application_snapshot().await.unwrap();
        replica.finish().await.unwrap();
        assert_eq!(count(&reader.db, "records").await, 1);
        assert_eq!(count(&reader.db, "record_read_index").await, 1);
        snapshot.rollback().await.unwrap();
        assert!(!reader.publication_current().await.unwrap());
        let reader = Store::open(&path).await.unwrap();
        assert_eq!(count(&reader.db, "records").await, 2);
        assert_eq!(count(&reader.db, "record_read_index").await, 2);
        assert_eq!(count(&reader.db, "sync_incoming").await, 0);
        reader.integrity().await.unwrap();
    }
    #[tokio::test]
    async fn finish_retries_a_committed_erasure_after_an_external_reader_releases_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        drop(
            Store::build(&path, vec![record("old", "Old original bytes.")])
                .await
                .unwrap(),
        );
        let reader = Store::open(&path).await.unwrap();
        let snapshot = reader.application_snapshot().await.unwrap();
        let replica = sync::Replica::open(&path).await.unwrap();
        replica.db.busy_timeout(std::time::Duration::ZERO).unwrap();
        replica
            .apply(
                &sync::Item {
                    id: "old".into(),
                    envelope: None,
                    envelope_hash: None,
                    body_hash: None,
                },
                None,
            )
            .await
            .unwrap();
        assert!(
            replica
                .finish()
                .await
                .unwrap_err()
                .to_string()
                .contains("WAL erasure")
        );
        assert_eq!(count(&reader.db, "records").await, 1);
        snapshot.rollback().await.unwrap();
        replica.finish().await.unwrap();
        assert_eq!(count(&replica.db, "records").await, 0);
        assert_eq!(count(&replica.db, "sync_incoming").await, 0);
        assert_eq!(count(&replica.db, "sync_state").await, 0);
    }

    #[tokio::test]
    async fn publication_discards_parked_reader_bytes_before_forget_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let mut writer = Store::build(&path, vec![record("old", "Original bytes to forget.")])
            .await
            .unwrap();
        let mut reader = Store::open(&path).await.unwrap();
        reader.materialize_records().await.unwrap();
        reader.load_records(["old".to_owned()]).await.unwrap();
        assert_eq!(reader.records["old"].body, "Original bytes to forget.");
        let barrier = read_barrier();
        barrier.park_reader(Some(reader));
        writer.read_barrier = Some(barrier.clone());
        writer.forget("old").await.unwrap();
        assert!(barrier.take_reader().is_none());
    }

    #[tokio::test]
    async fn replayed_staging_is_drained_and_already_forgotten_bytes_are_erased() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let original = record("live", "Existing immutable original.");
        drop(Store::build(&path, vec![original.clone()]).await.unwrap());
        let mut writer = Store::open(&path).await.unwrap();
        writer.forget("forgotten").await.unwrap();
        drop(writer);
        let secret = "DISCARDED_STAGING_SECRET_6b438adb";
        let replica = sync::Replica::open(&path).await.unwrap();
        for item in [item(&original), item(&record("forgotten", secret))] {
            replica
                .db
                .execute(
                    "INSERT INTO sync_incoming VALUES(?,?)",
                    libsql::params![item.id.as_str(), serde_json::to_vec(&item).unwrap()],
                )
                .await
                .unwrap();
        }
        replica.finish().await.unwrap();
        assert_eq!(count(&replica.db, "sync_incoming").await, 0);
        assert_eq!(count(&replica.db, "records").await, 1);
        for suffix in ["", "-wal"] {
            let bytes = std::fs::read(format!("{}{suffix}", path.display())).unwrap_or_default();
            assert!(
                !bytes
                    .windows(secret.len())
                    .any(|window| window == secret.as_bytes())
            );
        }
        replica.finish().await.unwrap();
        Store::open(&path).await.unwrap().integrity().await.unwrap();
    }

    #[tokio::test]
    async fn failed_add_is_atomic_and_forget_erases_originals_from_wal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let secret = "UNIQUE_SECURE_ERASURE_8a428df6 original body bytes";
        let store = Store::build(&path, vec![record("secret", secret)])
            .await
            .unwrap();
        store.db.execute_batch("CREATE TRIGGER reject_catalog BEFORE INSERT ON record_read_index BEGIN SELECT RAISE(ABORT,'injected publication failure'); END;").await.unwrap();
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        assert!(
            store
                .add(record("failed", "Must never become visible."))
                .await
                .is_err()
        );
        assert_eq!(count(&store.db, "records").await, 1);
        store
            .db
            .execute("DROP TRIGGER reject_catalog", ())
            .await
            .unwrap();
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        store.forget("secret").await.unwrap();
        for suffix in ["", "-wal"] {
            let bytes = std::fs::read(format!("{}{suffix}", path.display())).unwrap_or_default();
            assert!(
                !bytes
                    .windows(secret.len())
                    .any(|window| window == secret.as_bytes()),
                "original bytes remain in {suffix}"
            );
        }
        store.integrity().await.unwrap();
    }
}
