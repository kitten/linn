//! Shared publication and secure-forget primitives.
//! @ref LLP 0001#native-persistence
use super::*;

pub(super) async fn erase_record(db: &Connection, id: &str) -> Result<()> {
    db.execute("INSERT OR IGNORE INTO tombstones VALUES(?)", [id])
        .await?;
    db.execute("DELETE FROM sections WHERE record=?", [id])
        .await?;
    db.execute("DELETE FROM records WHERE id=?", [id]).await?;
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
    pub(crate) fn ensure_usable(&self) -> Result<()> {
        if self.write_failed {
            return Err(Error::Invalid(
                "a write failed on this store handle; discard it and reopen the store".into(),
            ));
        }
        Ok(())
    }
    pub(super) async fn complete_publication(&self) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute("DELETE FROM sync_state WHERE key='pending'", ())
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(super) fn write_outcome(&mut self, result: Result<()>) -> Result<()> {
        result.map_err(|e| {
            self.write_failed = true;
            Error::Invalid(format!("{e}; write may have committed: reopen the store, and run index:rebuild if publication is incomplete"))
        })
    }
}
