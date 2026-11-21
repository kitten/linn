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
#[derive(Clone, Debug)]
pub struct InventoryEntry {
    pub entry: Entry,
    pub frame_bytes: u64,
    pub envelope_bytes: u64,
}
impl Serialize for InventoryEntry {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut row = serializer.serialize_tuple(4)?;
        row.serialize_element(&self.entry.id)?;
        row.serialize_element(&self.entry.hash)?;
        row.serialize_element(&self.frame_bytes)?;
        row.serialize_element(&self.envelope_bytes)?;
        row.end()
    }
}
impl<'de> Deserialize<'de> for InventoryEntry {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let (id, hash, frame_bytes, envelope_bytes) =
            <(String, Option<String>, u64, u64)>::deserialize(deserializer)?;
        Ok(Self {
            entry: Entry { id, hash },
            frame_bytes,
            envelope_bytes,
        })
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryPage {
    #[serde(deserialize_with = "deserialize_inventory_entries")]
    pub entries: Vec<InventoryEntry>,
    pub done: bool,
}
pub(crate) fn deserialize_inventory_entries<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Vec<InventoryEntry>, D::Error> {
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = Vec<InventoryEntry>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("at most 256 inventory entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while entries.len() < PAGE {
                let Some(entry) = sequence.next_element()? else {
                    return Ok(entries);
                };
                entries.push(entry);
            }
            if sequence.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("oversized inventory page"));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Entries)
}
pub struct Prefix<T> {
    pub decisions: Vec<T>,
    pub error: Option<Error>,
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
    pub async fn inventory(
        &self,
        tombstones: bool,
        after: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Vec<Entry>> {
        Ok(self
            .inventory_page(tombstones, after, scope, None)
            .await?
            .entries
            .into_iter()
            .map(|entry| entry.entry)
            .collect())
    }
    pub async fn inventory_page(
        &self,
        tombstones: bool,
        after: Option<&str>,
        scope: Option<&str>,
        cutoff: Option<u64>,
    ) -> Result<InventoryPage> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let after = after.unwrap_or("");
        let cutoff = cutoff
            .map(i64::try_from)
            .transpose()
            .map_err(|_| Error::Invalid("invalid snapshot cutoff".into()))?;
        let mut rows = if tombstones {
            tx.query("SELECT id,hash,frame_bytes,envelope_bytes FROM sync_sizes WHERE hash IS NULL AND id>? AND (? IS NULL OR ordinal<=?) ORDER BY id LIMIT 257", params![after,cutoff,cutoff]).await?
        } else if let Some(scope) = scope {
            tx.query("SELECT id,hash,frame_bytes,envelope_bytes FROM sync_sizes WHERE hash IS NOT NULL AND id>? AND scope=? COLLATE NOCASE AND (? IS NULL OR ordinal<=?) ORDER BY id LIMIT 257", params![after,scope,cutoff,cutoff]).await?
        } else {
            tx.query("SELECT id,hash,frame_bytes,envelope_bytes FROM sync_sizes WHERE hash IS NOT NULL AND id>? AND (? IS NULL OR ordinal<=?) ORDER BY id LIMIT 257", params![after,cutoff,cutoff]).await?
        };
        let mut out = Vec::new();
        while let Some(r) = rows.next().await? {
            out.push(InventoryEntry {
                entry: Entry {
                    id: r.get(0)?,
                    hash: r.get(1)?,
                },
                frame_bytes: r
                    .get::<i64>(2)?
                    .try_into()
                    .map_err(|_| Error::Invalid("invalid inventory frame byte size".into()))?,
                envelope_bytes: r
                    .get::<i64>(3)?
                    .try_into()
                    .map_err(|_| Error::Invalid("invalid inventory envelope byte size".into()))?,
            });
        }
        let done = out.len() <= PAGE;
        out.truncate(PAGE);
        Ok(InventoryPage { entries: out, done })
    }
    pub(crate) async fn source_snapshot(&self) -> Result<u64> {
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT coalesce(max(ordinal),0) FROM sync_sizes", ())
            .await?;
        rows.next()
            .await?
            .unwrap()
            .get::<i64>(0)?
            .try_into()
            .map_err(|_| Error::Invalid("invalid source snapshot ordinal".into()))
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
    pub async fn sizes(&self, entries: Vec<Entry>) -> Result<Vec<InventoryEntry>> {
        if entries.len() > PAGE {
            return Err(Error::Invalid("oversized inventory".into()));
        }
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let statement = tx
            .prepare("SELECT frame_bytes,envelope_bytes FROM sync_sizes WHERE id=?")
            .await?;
        let mut sized = Vec::with_capacity(entries.len());
        for entry in entries {
            statement.reset();
            let mut rows = statement.query([entry.id.as_str()]).await?;
            let row = rows
                .next()
                .await?
                .ok_or_else(|| Error::Invalid("missing item size".into()))?;
            sized.push(InventoryEntry {
                entry,
                frame_bytes: row
                    .get::<i64>(0)?
                    .try_into()
                    .map_err(|_| Error::Invalid("invalid frame size".into()))?,
                envelope_bytes: row
                    .get::<i64>(1)?
                    .try_into()
                    .map_err(|_| Error::Invalid("invalid envelope size".into()))?,
            });
        }
        Ok(sized)
    }
    pub async fn needs(&self, entry: &Entry) -> Result<bool> {
        let mut prefix = self.needs_page(std::slice::from_ref(entry)).await?;
        match prefix.error {
            Some(error) => Err(error),
            None => Ok(prefix.decisions.remove(0)),
        }
    }
    pub async fn needs_page(&self, entries: &[Entry]) -> Result<Prefix<bool>> {
        if entries.len() > PAGE {
            return Err(Error::Invalid("oversized inventory".into()));
        }
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let statement = tx.prepare("SELECT hash FROM sync_sizes WHERE id=?").await?;
        let mut prefix = Prefix {
            decisions: Vec::with_capacity(entries.len()),
            error: None,
        };
        for entry in entries {
            statement.reset();
            let decision = async {
                let mut rows = statement.query([entry.id.as_str()]).await?;
                match rows.next().await? {
                    None => Ok(true),
                    Some(row) => {
                        let hash = row.get::<Option<String>>(0)?;
                        if hash.is_none() || hash == entry.hash {
                            Ok(false)
                        } else if entry.hash.is_none() {
                            Ok(true)
                        } else {
                            Err(crate::mesh::failure::Kind::Conflict
                                .error(format!("immutable record conflict: {}", entry.id)))
                        }
                    }
                }
            }
            .await;
            match decision {
                Ok(needed) => prefix.decisions.push(needed),
                Err(error) => {
                    prefix.error = Some(error);
                    break;
                }
            }
        }
        Ok(prefix)
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
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let changed = self.apply_staged(&tx, item, scope).await?;
        tx.commit().await?;
        Ok(changed)
    }
    pub async fn apply_group(&self, items: &[Item], scope: Option<&str>) -> Result<Prefix<bool>> {
        if items.len() > PAGE {
            return Err(Error::Invalid("oversized Apply group".into()));
        }
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut prefix = Prefix {
            decisions: Vec::with_capacity(items.len()),
            error: None,
        };
        for item in items {
            match self.apply_staged(&tx, item, scope).await {
                Ok(changed) => prefix.decisions.push(changed),
                Err(error) => {
                    prefix.error = Some(error);
                    break;
                }
            }
        }
        tx.commit()
            .await
            .map_err(crate::mesh::failure::Kind::publication)?;
        Ok(prefix)
    }
    async fn apply_staged(
        &self,
        tx: &Connection,
        item: &Item,
        scope: Option<&str>,
    ) -> Result<bool> {
        item.validate(scope)?;
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
        if force {
            tx.execute_batch(
                "INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE true ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes; DELETE FROM sync_sizes WHERE NOT EXISTS(SELECT 1 FROM sync_size_source s WHERE s.id=sync_sizes.id);",
            )
            .await?;
        }
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
            for item in items {
                let applied = self.apply_published(&item, None).await?;
                // Preserve the ID's ordinal through staging -> publication.
                // Rollback restores both representations on a later failure.
                tx.execute("DELETE FROM sync_incoming WHERE id=?", [item.id.as_str()])
                    .await?;
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
    fn item(id: &str) -> Item {
        let envelope = serde_json::json!({"id":id,"body":"Original bytes."}).to_string();
        Item {
            id: id.into(),
            envelope_hash: Some(hash(envelope.as_bytes())),
            body_hash: Some(hash(b"Original bytes.")),
            envelope: Some(envelope),
        }
    }

    #[test]
    #[ignore = "child process for the grouped staging crash-recovery test"]
    fn grouped_staging_crash_child() {
        let path = std::env::var_os("LINN_GROUP_CRASH_STORE").expect("crash fixture path");
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let replica = Replica::open(Path::new(&path)).await.unwrap();
                let committed: Vec<_> = (0..32).map(|n| item(&format!("crash-{n:03}"))).collect();
                assert!(
                    replica
                        .apply_group(&committed, None)
                        .await
                        .unwrap()
                        .error
                        .is_none()
                );
                let tx = replica
                    .transaction(TransactionBehavior::Immediate)
                    .await
                    .unwrap();
                for n in 32..64 {
                    replica
                        .apply_staged(&tx, &item(&format!("crash-{n:03}")), None)
                        .await
                        .unwrap();
                }
                let _filling: Vec<_> = (64..96).map(|n| item(&format!("crash-{n:03}"))).collect();
                // No destructors or graceful SQLite close: the second group is
                // staged inside its transaction and the third exists only in RAM.
                unsafe {
                    libc::kill(libc::getpid(), libc::SIGKILL);
                }
                unreachable!("SIGKILL must terminate the crash fixture");
            });
    }

    #[tokio::test]
    async fn process_crash_recovers_committed_staging_and_refetches_two_groups_idempotently() {
        use std::os::unix::process::ExitStatusExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("crash.db");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "storage::sync::schema_tests::grouped_staging_crash_child",
                "--ignored",
            ])
            .env("LINN_GROUP_CRASH_STORE", &path)
            .output()
            .unwrap();
        assert_eq!(
            output.status.signal(),
            Some(libc::SIGKILL),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let replica = Replica::open(&path).await.unwrap();
        for n in 0..96 {
            let actual = replica.get(&format!("crash-{n:03}"), None).await.unwrap();
            if n < 32 {
                assert_eq!(
                    actual.unwrap().envelope,
                    item(&format!("crash-{n:03}")).envelope
                );
            } else {
                assert!(actual.is_none());
            }
        }
        let originals: Vec<_> = (0..96).map(|n| item(&format!("crash-{n:03}"))).collect();
        let entries: Vec<_> = originals
            .iter()
            .map(|item| Entry {
                id: item.id.clone(),
                hash: item.envelope_hash.clone(),
            })
            .collect();
        let needed = replica.needs_page(&entries).await.unwrap();
        assert!(needed.error.is_none());
        assert_eq!(&needed.decisions[..32], &[false; 32]);
        let missing: Vec<_> = originals
            .iter()
            .zip(needed.decisions)
            .filter_map(|(item, needed)| needed.then_some(item.clone()))
            .collect();
        assert_eq!(missing.len(), 64);
        for group in missing.chunks(32) {
            let prefix = replica.apply_group(group, None).await.unwrap();
            assert!(prefix.error.is_none());
            assert_eq!(prefix.decisions, [true; 32]);
        }
        for group in originals.chunks(32) {
            let prefix = replica.apply_group(group, None).await.unwrap();
            assert!(prefix.error.is_none());
            assert_eq!(prefix.decisions, [false; 32]);
        }
        replica.finish().await.unwrap();
        for n in 0..96 {
            assert_eq!(
                replica
                    .get(&format!("crash-{n:03}"), None)
                    .await
                    .unwrap()
                    .unwrap()
                    .envelope,
                item(&format!("crash-{n:03}")).envelope
            );
        }
    }

    #[tokio::test]
    async fn grouped_staging_commits_only_the_valid_prefix_and_replays_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("groups.db");
        let replica = Replica::open(&path).await.unwrap();
        let mut invalid = item("b");
        invalid.envelope_hash = Some("wrong".into());
        let prefix = replica
            .apply_group(&[item("a"), invalid, item("c")], None)
            .await
            .unwrap();
        assert_eq!(prefix.decisions, [true]);
        assert!(prefix.error.is_some());
        drop(replica);
        let replica = Replica::open(&path).await.unwrap();
        assert_eq!(
            replica.get("a", None).await.unwrap().unwrap().envelope,
            item("a").envelope
        );
        assert!(replica.get("b", None).await.unwrap().is_none());
        assert!(replica.get("c", None).await.unwrap().is_none());
        let prefix = replica
            .apply_group(&[item("a"), item("c")], None)
            .await
            .unwrap();
        assert_eq!(prefix.decisions, [false, true]);
        assert!(prefix.error.is_none());
        let decisions = replica
            .needs_page(&[
                Entry {
                    id: "missing".into(),
                    hash: Some("new".into()),
                },
                Entry {
                    id: "a".into(),
                    hash: Some("conflicting".into()),
                },
                Entry {
                    id: "later".into(),
                    hash: None,
                },
            ])
            .await
            .unwrap();
        assert_eq!(decisions.decisions, [true]);
        assert_eq!(
            crate::mesh::failure::Kind::of(&decisions.error.unwrap()),
            crate::mesh::failure::Kind::Conflict
        );
        replica
            .db
            .execute("INSERT INTO tombstones VALUES('a')", ())
            .await
            .unwrap();
        let prefix = replica.apply_group(&[item("a")], None).await.unwrap();
        assert_eq!(prefix.decisions, [false]);
        assert!(prefix.error.is_none());
        replica.finish().await.unwrap();
        assert!(
            replica
                .get("a", None)
                .await
                .unwrap()
                .unwrap()
                .envelope
                .is_none()
        );
    }

    #[tokio::test]
    async fn sized_inventory_marks_a_full_last_page_without_an_empty_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let replica = Replica::open(&dir.path().join("pages.db")).await.unwrap();
        let tx = replica.db.transaction().await.unwrap();
        for index in 0..PAGE {
            tx.execute("INSERT INTO tombstones VALUES(?)", [format!("{index:04}")])
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
        let page = replica
            .inventory_page(true, None, None, None)
            .await
            .unwrap();
        assert_eq!(page.entries.len(), PAGE);
        assert!(page.done);
        assert!(
            page.entries
                .iter()
                .all(|entry| entry.frame_bytes > 4 && entry.envelope_bytes == 0)
        );
        replica
            .db
            .execute("INSERT INTO tombstones VALUES('last')", ())
            .await
            .unwrap();
        let page = replica
            .inventory_page(true, None, None, None)
            .await
            .unwrap();
        assert!(!page.done);
        let page = replica
            .inventory_page(
                true,
                Some(&page.entries.last().unwrap().entry.id),
                None,
                None,
            )
            .await
            .unwrap();
        assert!(page.done);
        assert_eq!(page.entries[0].entry.id, "last");
    }

    #[tokio::test]
    async fn group_sql_errors_preserve_prefix_and_commit_failures_accept_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("commit-errors.db");
        let replica = Replica::open(&path).await.unwrap();
        replica.db.execute_batch("CREATE TRIGGER reject_second BEFORE INSERT ON sync_incoming WHEN NEW.id='b' BEGIN SELECT RAISE(ABORT,'injected statement failure'); END;").await.unwrap();
        let prefix = replica
            .apply_group(&[item("a"), item("b"), item("c")], None)
            .await
            .unwrap();
        assert_eq!(prefix.decisions, [true]);
        assert!(prefix.error.is_some());
        replica.db.execute_batch("DROP TRIGGER reject_second; CREATE TRIGGER rollback_second BEFORE INSERT ON sync_incoming WHEN NEW.id='e' BEGIN SELECT RAISE(ROLLBACK,'injected rollback'); END;").await.unwrap();
        let failure = replica
            .apply_group(&[item("d"), item("e")], None)
            .await
            .err()
            .unwrap();
        assert_eq!(
            crate::mesh::failure::Kind::of(&failure),
            crate::mesh::failure::Kind::Publication
        );
        drop(replica);
        let replica = Replica::open(&path).await.unwrap();
        assert!(replica.get("a", None).await.unwrap().is_some());
        for id in ["b", "c", "d", "e"] {
            assert!(replica.get(id, None).await.unwrap().is_none(), "{id}");
        }
        let mut rows = replica.db.query("PRAGMA synchronous", ()).await.unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            2
        );
    }

    #[tokio::test]
    async fn source_cutoff_survives_publication_rebuild_and_earlier_insertions() {
        let dir = tempfile::tempdir().unwrap();
        let replica = Replica::open(&dir.path().join("snapshot.db"))
            .await
            .unwrap();
        replica.apply(&item("z-original"), None).await.unwrap();
        let cutoff = replica.source_snapshot().await.unwrap();
        for number in 0..PAGE + 1 {
            replica
                .apply(&item(&format!("a-later-{number:04}")), None)
                .await
                .unwrap();
        }
        for rebuild in [false, true] {
            if rebuild {
                replica.rebuild().await.unwrap();
            } else {
                replica.finish().await.unwrap();
            }
            let page = replica
                .inventory_page(false, None, None, Some(cutoff))
                .await
                .unwrap();
            assert!(page.done);
            assert_eq!(page.entries.len(), 1);
            assert_eq!(page.entries[0].entry.id, "z-original");
        }
        let tombstone = Item {
            id: "z-original".into(),
            envelope: None,
            envelope_hash: None,
            body_hash: None,
        };
        replica.apply(&tombstone, None).await.unwrap();
        let page = replica
            .inventory_page(true, None, None, Some(cutoff))
            .await
            .unwrap();
        assert_eq!(page.entries[0].entry.id, "z-original");
        assert!(
            replica
                .inventory_page(false, None, None, Some(cutoff))
                .await
                .unwrap()
                .entries
                .is_empty()
        );
    }
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
