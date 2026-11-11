//! @ref LLP 0001#schema-versions-and-migrations
use crate::error::{Error, Result};
use libsql::{Connection, TransactionBehavior};

const MIGRATIONS: &[&str] = &[
    include_str!("0001-core.sql"),
    include_str!("0002-writer.sql"),
    include_str!("0003-derived-schema.sql"),
    include_str!("0004-sync.sql"),
    include_str!("0005-writer-filters.sql"),
];

async fn version(db: &Connection) -> Result<usize> {
    let mut rows = db.query("PRAGMA user_version", ()).await?;
    let value = rows
        .next()
        .await?
        .ok_or_else(|| Error::Invalid("missing schema version".into()))?
        .get::<i64>(0)?;
    usize::try_from(value).map_err(|_| Error::Invalid("invalid schema version".into()))
}

fn supported(version: usize, latest: usize) -> Result<()> {
    if version > latest {
        return Err(Error::Invalid(format!(
            "store schema {version} is newer than this binary supports ({latest})"
        )));
    }
    Ok(())
}

pub(super) async fn apply(db: &Connection) -> Result<()> {
    apply_steps(db, MIGRATIONS).await
}

async fn apply_steps(db: &Connection, steps: &[&str]) -> Result<()> {
    let current = version(db).await?;
    supported(current, steps.len())?;
    if current == steps.len() {
        return Ok(());
    }
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .await?;
    let result = async {
        // Another opener may have completed migration while we waited for the lock.
        let current = version(&tx).await?;
        supported(current, steps.len())?;
        for (index, sql) in steps.iter().enumerate().skip(current) {
            tx.execute_batch(sql).await?;
            tx.execute_batch(&format!("PRAGMA user_version={}", index + 1))
                .await?;
        }
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            tx.commit().await?;
            Ok(())
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn originals(db: &Connection) -> Vec<(String, Vec<u8>, Vec<u8>, String, String)> {
        let mut rows = db
            .query(
                "SELECT id,body,envelope,body_hash,envelope_hash FROM records ORDER BY id",
                (),
            )
            .await
            .unwrap();
        let mut result = Vec::new();
        while let Some(row) = rows.next().await.unwrap() {
            result.push((
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
                row.get(3).unwrap(),
                row.get(4).unwrap(),
            ));
        }
        result
    }

    async fn fixture(db: &Connection) {
        let body = b"Immutable original body.";
        let envelope =
            br#"{"id":"legacy","body":"Immutable original body.","writer":"legacy@example.org"}"#;
        db.execute(
            "INSERT INTO records VALUES('legacy',NULL,'org/repo','accepted',?,?,?,?)",
            libsql::params![
                body.to_vec(),
                crate::storage::record::hash(body),
                envelope.to_vec(),
                crate::storage::record::hash(envelope)
            ],
        )
        .await
        .unwrap();
        db.execute("INSERT INTO tombstones VALUES('forgotten')", ())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn upgrades_preserve_originals_and_current_schema_is_a_noop() {
        for old in [1, 2, 3, 4] {
            let dir = tempfile::tempdir().unwrap();
            let database = libsql::Builder::new_local(dir.path().join("upgrade.db"))
                .build()
                .await
                .unwrap();
            let db = database.connect().unwrap();
            apply_steps(&db, &MIGRATIONS[..old]).await.unwrap();
            fixture(&db).await;
            let before = originals(&db).await;
            apply(&db).await.unwrap();
            assert_eq!(version(&db).await.unwrap(), MIGRATIONS.len());
            assert_eq!(originals(&db).await, before);
            let mut rows = db.query("SELECT id FROM tombstones", ()).await.unwrap();
            assert_eq!(
                rows.next()
                    .await
                    .unwrap()
                    .unwrap()
                    .get::<String>(0)
                    .unwrap(),
                "forgotten"
            );
            drop(rows);
            let mut rows = db.query("SELECT writer FROM records", ()).await.unwrap();
            assert_eq!(
                rows.next()
                    .await
                    .unwrap()
                    .unwrap()
                    .get::<String>(0)
                    .unwrap(),
                "legacy@example.org"
            );
            drop(rows);
            let mut rows = db.query("PRAGMA schema_version", ()).await.unwrap();
            let schema: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
            drop(rows);
            apply(&db).await.unwrap();
            let mut rows = db.query("PRAGMA schema_version", ()).await.unwrap();
            assert_eq!(
                rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
                schema
            );
        }
    }

    #[tokio::test]
    async fn failure_rolls_back_ddl_and_version_then_retry_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let database = libsql::Builder::new_local(dir.path().join("rollback.db"))
            .build()
            .await
            .unwrap();
        let db = database.connect().unwrap();
        apply_steps(&db, &MIGRATIONS[..1]).await.unwrap();
        fixture(&db).await;
        let before = originals(&db).await;
        let bad = [
            MIGRATIONS[0],
            MIGRATIONS[1],
            "CREATE TABLE partial_migration(id INTEGER); INSERT INTO missing_table VALUES(1);",
        ];
        assert!(apply_steps(&db, &bad).await.is_err());
        assert_eq!(version(&db).await.unwrap(), 1);
        assert_eq!(originals(&db).await, before);
        let mut rows = db
            .query(
                "SELECT count(*) FROM sqlite_schema WHERE name='partial_migration'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
        drop(rows);
        assert!(db.query("SELECT writer FROM records", ()).await.is_err());
        apply(&db).await.unwrap();
        assert_eq!(version(&db).await.unwrap(), MIGRATIONS.len());
        assert_eq!(originals(&db).await, before);
    }

    #[tokio::test]
    async fn newer_schema_is_rejected_without_changes() {
        let dir = tempfile::tempdir().unwrap();
        let database = libsql::Builder::new_local(dir.path().join("future.db"))
            .build()
            .await
            .unwrap();
        let db = database.connect().unwrap();
        db.execute_batch("CREATE TABLE future_only(value TEXT); INSERT INTO future_only VALUES('untouched'); PRAGMA user_version=999;").await.unwrap();
        assert!(apply(&db).await.unwrap_err().to_string().contains("newer"));
        assert_eq!(version(&db).await.unwrap(), 999);
        let mut rows = db.query("SELECT value FROM future_only", ()).await.unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "untouched"
        );
    }

    #[test]
    fn concurrent_initialization_and_upgrades_are_serialized() {
        for old in [0, 1, 2, 3, 4] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("concurrent.db");
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let database = libsql::Builder::new_local(&path).build().await.unwrap();
                    let db = database.connect().unwrap();
                    apply_steps(&db, &MIGRATIONS[..old]).await.unwrap();
                });
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
            let workers: Vec<_> = (0..2)
                .map(|_| {
                    let path = path.clone();
                    let barrier = barrier.clone();
                    std::thread::spawn(move || {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async {
                                let database =
                                    libsql::Builder::new_local(path).build().await.unwrap();
                                let db = database.connect().unwrap();
                                db.execute_batch("PRAGMA busy_timeout=5000;").await.unwrap();
                                barrier.wait();
                                apply(&db).await.unwrap();
                                assert_eq!(version(&db).await.unwrap(), MIGRATIONS.len());
                            });
                    })
                })
                .collect();
            for worker in workers {
                worker.join().unwrap();
            }
        }
    }
}
