//! @ref LLP 0006#file-export-and-import
use super::{
    Store,
    sync::{Item, Replica},
};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

const MAX_LINE: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub struct Selection {
    pub scope: Option<String>,
    pub kind: Option<String>,
    pub since: Option<String>,
    pub ids: Vec<String>,
    pub all_states: bool,
    pub now: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Frame {
    Item { item: Item },
    End { count: u64, sha256: String },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    format: String,
    version: u32,
}

fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

fn line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    let mut out = Vec::new();
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            return if out.is_empty() {
                Ok(None)
            } else {
                Err(invalid("truncated export bundle"))
            };
        }
        let end = bytes.iter().position(|&b| b == b'\n').map(|i| i + 1);
        let n = end.unwrap_or(bytes.len());
        if out.len() + n > MAX_LINE {
            return Err(invalid("bundle line exceeds 16 MiB"));
        }
        out.extend_from_slice(&bytes[..n]);
        reader.consume(n);
        if end.is_some() {
            return Ok(Some(out));
        }
    }
}

fn encoded(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_LINE {
        return Err(invalid("bundle line exceeds 16 MiB"));
    }
    Ok(bytes)
}

impl Store {
    pub async fn export_bundle(
        &self,
        selection: &Selection,
        writer: &mut impl Write,
    ) -> Result<u64> {
        self.ensure_usable()?;
        let tx = self.db.transaction().await?;
        let result = async {
            // Pin the publication snapshot before reading selection metadata.
            let mut rows = tx.query("SELECT value FROM projections WHERE key='structure'", ()).await?;
            if rows.next().await?.map(|r| r.get::<Vec<u8>>(0)).transpose()?.as_deref() != Some(self.structure_stamp.as_slice()) {
                return Err(invalid("store changed; retry export"));
            }
            drop(rows);
            let mut pending = tx.query("SELECT 1 FROM sync_state WHERE key='pending'", ()).await?;
            if pending.next().await?.is_some() { return Err(invalid("index publication incomplete; run linn index:rebuild")); }
            drop(pending);
            tx.execute_batch("CREATE TEMP TABLE IF NOT EXISTS export_ids(id TEXT PRIMARY KEY); DELETE FROM export_ids; CREATE TEMP TABLE IF NOT EXISTS export_selected(id TEXT PRIMARY KEY); DELETE FROM export_selected;").await?;
            for id in &selection.ids { tx.execute("INSERT OR IGNORE INTO export_ids VALUES(?)", [id.as_str()]).await?; }
            let mut sql = String::from("INSERT INTO export_selected SELECT r.id FROM records r JOIN record_headers h ON h.record=r.id JOIN record_read_index i ON i.record=r.id WHERE 1");
            let mut values: Vec<libsql::Value> = Vec::new();
            if !selection.all_states {
                sql.push_str(" AND (i.eligible&1)!=0 AND (i.expires='' OR i.expires>?)");
                values.push(selection.now.clone().into());
            }
            for (predicate, value) in [
                ("r.original_scope=? COLLATE NOCASE", &selection.scope),
                ("json_extract(CAST(h.data AS TEXT),'$.kind')=?", &selection.kind),
                ("json_extract(CAST(h.data AS TEXT),'$.created')>=?", &selection.since),
            ] {
                if let Some(value) = value { sql.push_str(" AND "); sql.push_str(predicate); values.push(value.clone().into()); }
            }
            if !selection.ids.is_empty() { sql.push_str(" AND r.id IN (SELECT id FROM export_ids)"); }
            tx.execute(&sql, values).await?;
            // Selected history must carry an actual suppressing witness, not synthetic state.
            let mut hidden = tx.query(
                "SELECT s.id FROM export_selected s JOIN record_read_index i ON i.record=s.id JOIN record_headers h ON h.record=s.id
                 WHERE (i.eligible&1)=0 AND json_extract(CAST(h.data AS TEXT),'$.state')='accepted'
                 AND json_extract(CAST(h.data AS TEXT),'$.close') IS NULL AND h.body_bytes>0
                 AND NOT EXISTS (SELECT 1 FROM export_selected w JOIN record_headers wh ON wh.record=w.id
                   WHERE json_extract(CAST(wh.data AS TEXT),'$.state')='accepted'
                   AND (json_extract(CAST(wh.data AS TEXT),'$.close.target')=s.id
                     OR (json_extract(CAST(wh.data AS TEXT),'$.close') IS NULL AND json_extract(CAST(wh.data AS TEXT),'$.parent')=s.id))) LIMIT 1", ()).await?;
            if let Some(row) = hidden.next().await? {
                return Err(Error::Invalid(format!("history selection omits lifecycle witness for {}; broaden selection, or index:rebuild if its witness was forgotten", row.get::<String>(0)?)));
            }
            drop(hidden);
            writer.write_all(&encoded(&Header { format: "linn-export".into(), version: 1 })?)?;
            let mut count = 0;
            let mut digest = Sha256::new();
            let metadata_filters = selection.scope.is_some() || selection.kind.is_some() || selection.since.is_some();
            if selection.all_states && !metadata_filters {
                let sql = if selection.ids.is_empty() { "SELECT id FROM tombstones ORDER BY id" } else { "SELECT id FROM tombstones WHERE id IN (SELECT id FROM export_ids) ORDER BY id" };
                let mut rows = tx.query(sql, ()).await?;
                while let Some(row) = rows.next().await? {
                    emit(writer, &mut digest, &mut count, Item { id: row.get(0)?, envelope: None, envelope_hash: None, body_hash: None })?;
                }
            }
            let mut rows = tx.query("SELECT r.id,r.envelope,r.envelope_hash,r.body_hash FROM export_selected s JOIN records r ON r.id=s.id ORDER BY s.id", ()).await?;
            while let Some(row) = rows.next().await? {
                let item = Item {
                    id: row.get(0)?,
                    envelope: Some(String::from_utf8(row.get(1)?).map_err(|_| invalid("non-UTF-8 envelope"))?),
                    envelope_hash: Some(row.get(2)?),
                    body_hash: Some(row.get(3)?),
                };
                item.validate(None)?;
                emit(writer, &mut digest, &mut count, item)?;
            }
            writer.write_all(&encoded(&Frame::End { count, sha256: format!("{:x}", digest.finalize()) })?)?;
            Ok(count)
        }.await;
        tx.rollback().await?;
        result
    }

    pub(crate) async fn export_reference(
        &mut self,
        reference: &str,
        scope: Option<&str>,
    ) -> Result<String> {
        let mut rows = self
            .db
            .query("SELECT id FROM tombstones WHERE id=?", [reference])
            .await?;
        if rows.next().await?.is_some() {
            return Ok(reference.into());
        }
        drop(rows);
        Ok(self.resolve_metadata(reference, scope).await?.0.id.clone())
    }
}

fn emit(writer: &mut impl Write, digest: &mut Sha256, count: &mut u64, item: Item) -> Result<()> {
    let bytes = encoded(&Frame::Item { item })?;
    digest.update(&bytes);
    writer.write_all(&bytes)?;
    *count += 1;
    Ok(())
}

/// Validate the entire bundle before merging any items; stage bounded lines on disk.
pub fn stage(reader: &mut impl BufRead) -> Result<std::fs::File> {
    let mut spool = tempfile::tempfile()?;
    let first = line(reader)?.ok_or_else(|| invalid("missing bundle header"))?;
    let header: Header = serde_json::from_slice(&first)?;
    if header.format != "linn-export" || header.version != 1 {
        return Err(invalid("unsupported export bundle"));
    }
    let mut count = 0;
    let mut digest = Sha256::new();
    loop {
        let bytes = line(reader)?.ok_or_else(|| invalid("missing bundle end"))?;
        match serde_json::from_slice::<Frame>(&bytes)? {
            Frame::Item { item } => {
                item.validate(None)?;
                digest.update(&bytes);
                count += 1;
                spool.write_all(&bytes)?;
            }
            Frame::End {
                count: expected,
                sha256,
            } => {
                if count != expected || format!("{:x}", digest.finalize()) != sha256 {
                    return Err(invalid("bundle count/digest mismatch"));
                }
                if line(reader)?.is_some() {
                    return Err(invalid("trailing bundle content"));
                }
                spool.seek(SeekFrom::Start(0))?;
                return Ok(spool);
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Imported {
    pub changed: u64,
    pub unchanged: u64,
}

pub async fn import_bundle(replica: &Replica, staged: std::fs::File) -> Result<Imported> {
    let mut reader = BufReader::new(staged);
    let result: Result<Imported> = async {
        let mut counts = Imported {
            changed: 0,
            unchanged: 0,
        };
        while let Some(bytes) = line(&mut reader)? {
            if let Frame::Item { item } = serde_json::from_slice(&bytes)? {
                if replica.apply(&item, None).await? {
                    counts.changed += 1;
                } else {
                    counts.unchanged += 1;
                }
            }
        }
        Ok(counts)
    }
    .await;
    let publication = replica.finish().await;
    match (result, publication) {
        (Ok(counts), Ok(())) => Ok(counts),
        (Err(error), Ok(())) => Err(Error::Invalid(format!(
            "import stopped; earlier items may have committed; retry safely: {error}"
        ))),
        (result, Err(error)) => Err(Error::Invalid(format!(
            "import publication failed; run linn index:rebuild: {error}; merge: {}",
            result.err().map_or("completed".into(), |e| e.to_string())
        ))),
    }
}
