//! Candidate record reads and clock-dependent corpus totals.
//! @ref LLP 0001#on-demand-source-and-structural-reads
use super::*;

fn pointer_query(related: bool) -> String {
    let selection = if related {
        "SELECT * FROM (SELECT *,row_number() OVER (PARTITION BY record ORDER BY matched DESC,heading_only DESC,length,start) AS priority FROM eligible) WHERE priority=1 ORDER BY matched DESC,heading_only DESC,length,record,start LIMIT 2"
    } else {
        "SELECT * FROM eligible ORDER BY matched DESC,length,record,start LIMIT 3"
    };
    // Callers supply the existing deduplicated content words.
    let sql = format!(
        "WITH probes AS (SELECT DISTINCT value AS term FROM json_each(?1)),
         hits AS (
           SELECT w.section,count(*) AS matched,
                  sum(CASE WHEN (w.flags&1)=0 THEN 1 ELSE 0 END) AS heading_only,
                  json_group_array(p.term) AS terms
           FROM probes p CROSS JOIN nav_lexemes n ON n.surface=p.term
           CROSS JOIN nav_section_words w ON w.word=n.id
           WHERE ?2 OR (w.flags&1)!=0 GROUP BY w.section HAVING count(*)>=?3
         ), eligible AS (
           SELECT s.record,s.data,h.matched,h.heading_only,h.terms,
                  json_extract(CAST(s.data AS TEXT),'$.span.start') AS start,
                  json_extract(CAST(s.data AS TEXT),'$.span.end')-json_extract(CAST(s.data AS TEXT),'$.span.start') AS length
           FROM hits h JOIN nav_sections s ON s.id=h.section
           JOIN record_read_index r ON r.record=s.record
           WHERE (r.eligible&2)!=0 AND (r.expires='' OR r.expires>?4)
             AND length>0 AND (NOT ?2 OR (s.own_nonempty!=0 AND h.heading_only>0))
             AND s.record NOT IN (SELECT value FROM json_each(?5))
         ) {selection}"
    );
    sql
}

impl Store {
    pub(crate) async fn indexed_pointer_sections(
        &self,
        words: &[String],
        related: bool,
        excluded: &[String],
    ) -> Result<Option<Vec<(String, Section, Vec<String>)>>> {
        if !self.structure_lazy {
            return Ok(None);
        }
        if words.len() < 2 {
            return Ok(Some(Vec::new()));
        }
        let sql = pointer_query(related);
        let mut rows = self
            .db
            .query(
                &sql,
                params![
                    serde_json::to_string(words)?,
                    i64::from(related),
                    2.max(words.len().div_ceil(2)) as i64,
                    self.query_clock.as_str(),
                    serde_json::to_string(excluded)?,
                ],
            )
            .await?;
        let mut selected = Vec::new();
        while let Some(row) = rows.next().await? {
            let terms: HashSet<String> = serde_json::from_str(&row.get::<String>(4)?)?;
            selected.push((
                row.get(0)?,
                serde_json::from_slice(&row.get::<Vec<u8>>(1)?)?,
                words
                    .iter()
                    .filter(|word| terms.contains(*word))
                    .cloned()
                    .collect(),
            ));
        }
        Ok(Some(selected))
    }
    pub(super) async fn open_catalog(&mut self) -> Result<bool> {
        let mut rows = self.db.query("SELECT c.value,s.value FROM projections c JOIN projections s ON s.key='structure' WHERE c.key='record-read-v1'", ()).await?;
        let Some(row) = rows.next().await? else {
            return Ok(false);
        };
        let Ok((format, generation, records, blocks)) =
            serde_json::from_slice::<(u8, Vec<u8>, usize, usize)>(&row.get::<Vec<u8>>(0)?)
        else {
            return Ok(false);
        };
        if format != 2 {
            return Ok(false);
        }
        let structure: Vec<u8> = row.get(1)?;
        if generation != structure
            || !structure.starts_with(&[structure_version().as_slice(), b":"].concat())
        {
            return Ok(false);
        }
        self.structure_stamp = structure;
        self.record_count = records;
        self.block_count = blocks;
        self.records_lazy = true;
        self.structure_lazy = true;
        Ok(true)
    }
    pub(super) async fn publish_catalog(&mut self) -> Result<()> {
        self.materialize_records().await?;
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        let mut rows = tx
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await?;
        if rows
            .next()
            .await?
            .map(|r| r.get::<Vec<u8>>(0))
            .transpose()?
            .as_ref()
            != Some(&self.structure_stamp)
        {
            return Err(Error::Invalid(
                "store changed before record-index publication; reopen".into(),
            ));
        }
        drop(rows);
        tx.execute_batch("DELETE FROM record_read_index; DELETE FROM record_expiry_totals;")
            .await?;
        let insert = tx
            .prepare("INSERT INTO record_read_index VALUES(?,?,?,?)")
            .await?;
        let mut expiry = BTreeMap::<String, (usize, usize)>::new();
        expiry.insert(String::new(), (0, 0));
        for (ordinal, (id, record)) in self.records.iter().enumerate() {
            let eligible = self.validity.eligible(record, "");
            let expires = record.stale_after.as_deref().unwrap_or("");
            insert
                .execute(params![
                    id.as_str(),
                    ordinal as i64,
                    i64::from(eligible) | (i64::from(eligible && record.body_nonblank()) << 1),
                    expires
                ])
                .await?;
            insert.reset();
            if eligible {
                let totals = expiry.entry(expires.into()).or_default();
                totals.0 += self.lookups[id].count;
                totals.1 += self.lookups[id].length;
            }
        }
        let insert = tx
            .prepare("INSERT INTO record_expiry_totals VALUES(?,?,?)")
            .await?;
        // Empty expiry is immortal; all other rows are suffix sums including it.
        let mut suffix = expiry[""];
        insert
            .execute(params!["", suffix.0 as i64, suffix.1 as i64])
            .await?;
        insert.reset();
        for (expires, (blocks, length)) in expiry.iter().rev().filter(|(key, _)| !key.is_empty()) {
            suffix.0 += blocks;
            suffix.1 += length;
            insert
                .execute(params![expires.as_str(), suffix.0 as i64, suffix.1 as i64])
                .await?;
            insert.reset();
        }
        self.record_count = self.records.len();
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('record-read-v1',?)",
            [serde_json::to_vec(&(
                2u8,
                &self.structure_stamp,
                self.record_count,
                self.block_count,
            ))?],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    fn mean_identity(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&(
            &self.structure_stamp,
            crate::language::embedding::ARTIFACT_MODEL,
        ))?)
    }
    pub(crate) async fn fallback_reads_ready(&self) -> Result<bool> {
        let mut rows = self
            .db
            .query(
                "SELECT value FROM projections WHERE key='fallback-read-v1'",
                (),
            )
            .await?;
        Ok(rows
            .next()
            .await?
            .map(|r| r.get::<Vec<u8>>(0))
            .transpose()?
            .as_ref()
            == Some(&self.mean_identity()?))
    }
    pub(crate) async fn publish_fallback_reads(
        &self,
        means: &[(usize, Vec<f32>)],
        proposals: &[String],
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        let mut rows = tx
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await?;
        if rows
            .next()
            .await?
            .map(|r| r.get::<Vec<u8>>(0))
            .transpose()?
            .as_ref()
            != Some(&self.structure_stamp)
        {
            return Err(Error::Invalid(
                "store changed before section-mean publication; reopen".into(),
            ));
        }
        drop(rows);
        tx.execute("DELETE FROM section_means", ()).await?;
        let insert = tx
            .prepare("INSERT INTO section_means VALUES(?,?,?)")
            .await?;
        for (ordinal, (block, vector)) in means.iter().enumerate() {
            let bytes: Vec<u8> = vector.iter().flat_map(|v| v.to_le_bytes()).collect();
            insert
                .execute(params![*block as i64, ordinal as i64, bytes])
                .await?;
            insert.reset();
        }
        tx.execute("DELETE FROM beam_terms", ()).await?;
        let insert = tx.prepare("INSERT INTO beam_terms VALUES(?,?)").await?;
        for (ordinal, term) in proposals.iter().enumerate() {
            insert
                .execute(params![term.as_str(), ordinal as i64])
                .await?;
            insert.reset();
        }
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('fallback-read-v1',?)",
            [self.mean_identity()?],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn candidate_beam_terms(
        &self,
        words: &[String],
    ) -> Result<HashMap<String, usize>> {
        let mut rows = self.db.query("SELECT term,ordinal FROM beam_terms WHERE term IN (SELECT value FROM json_each(?))", [serde_json::to_string(words)?]).await?;
        let mut terms = HashMap::new();
        while let Some(row) = rows.next().await? {
            terms.insert(row.get(0)?, row.get::<i64>(1)? as usize);
        }
        Ok(terms)
    }
    pub(crate) async fn candidate_section_means(
        &self,
        blocks: &HashSet<usize>,
    ) -> Result<Vec<(usize, Vec<f32>)>> {
        let mut rows = self.db.query("SELECT block,emb FROM section_means WHERE block IN (SELECT value FROM json_each(?)) ORDER BY ordinal", [serde_json::to_string(blocks)?]).await?;
        let mut means = Vec::new();
        while let Some(row) = rows.next().await? {
            let bytes: Vec<u8> = row.get(1)?;
            if bytes.len() != 512 * 4 {
                return Err(Error::Invalid("invalid section-mean dimensions".into()));
            }
            means.push((
                row.get::<i64>(0)? as usize,
                bytes
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect(),
            ));
        }
        Ok(means)
    }
    pub fn record_count(&self) -> usize {
        self.record_count
    }
    pub async fn contains_record(&self, id: &str) -> Result<bool> {
        self.ensure_usable()?;
        let mut rows = self
            .db
            .query("SELECT 1 FROM records WHERE id=?", [id])
            .await?;
        Ok(rows.next().await?.is_some())
    }
    pub(crate) async fn load_record_headers(
        &mut self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<()> {
        if !self.records_lazy {
            return Ok(());
        }
        let mut ids: Vec<_> = ids
            .into_iter()
            .filter(|id| !self.records.contains_key(id))
            .collect();
        ids.sort();
        ids.dedup();
        for chunk in ids.chunks(128) {
            let mut rows = self.db.query("SELECT h.data,h.body_bytes,h.body_nonblank,h.body_hash,c.ordinal,c.eligible,t.blocks,t.length FROM record_headers h JOIN record_read_index c ON c.record=h.record JOIN record_index_totals t ON t.record=h.record WHERE h.record IN (SELECT value FROM json_each(?)) ORDER BY h.record", [serde_json::to_string(chunk)?]).await?;
            while let Some(row) = rows.next().await? {
                let mut record: Record = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
                record.stored_body = Some(record::StoredBody {
                    bytes: row.get::<i64>(1)? as usize,
                    nonblank: row.get::<i64>(2)? != 0,
                    hash: row.get(3)?,
                });
                record.projected_eligible = Some(row.get::<i64>(5)? != 0);
                let ordinal = row.get::<i64>(4)? as usize;
                let lookup = self.lookups.entry(record.id.clone()).or_default();
                lookup.count = row.get::<i64>(6)? as usize;
                lookup.length = row.get::<i64>(7)? as usize;
                self.indexes.records.insert(ordinal, record.id.clone());
                self.indexes
                    .record_positions
                    .insert(record.id.clone(), ordinal);
                self.sections.entry(record.id.clone()).or_default();
                self.records.insert(record.id.clone(), record);
            }
        }
        Ok(())
    }
    pub(super) async fn materialize_records(&mut self) -> Result<()> {
        if !self.records_lazy {
            return Ok(());
        }
        let mut rows = self
            .db
            .query("SELECT record FROM record_headers ORDER BY record", ())
            .await?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            ids.push(row.get::<String>(0)?);
        }
        self.load_record_headers(ids).await?;
        let mut rows = self
            .db
            .query("SELECT value FROM projections WHERE key='validity'", ())
            .await?;
        if let Some(row) = rows.next().await? {
            self.validity = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
        }
        for record in self.records.values_mut() {
            record.projected_eligible = None;
        }
        self.records_lazy = false;
        Ok(())
    }
    pub(crate) async fn last_nonempty_record(
        &mut self,
        excluded: &HashSet<String>,
    ) -> Result<Option<usize>> {
        let mut rows = self.db.query("SELECT t.record FROM record_index_totals t JOIN record_read_index r ON r.record=t.record WHERE t.blocks>0 AND (r.eligible&1)!=0 AND (r.expires='' OR r.expires>?2) AND t.record NOT IN (SELECT value FROM json_each(?1)) ORDER BY t.record DESC LIMIT 1", params![serde_json::to_string(excluded)?, self.query_clock.as_str()]).await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let id = row.get::<String>(0)?;
        drop(rows);
        self.load_record_headers([id.clone()]).await?;
        Ok(self.indexes.record_positions.get(&id).copied())
    }
    pub(crate) async fn lifecycle_references(&mut self, target: &str) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for field in ["parent", "close.target"] {
            let mut rows = self.db.query(&format!("SELECT record FROM record_headers WHERE json_extract(CAST(data AS TEXT),'$.{field}')=? ORDER BY record"), [target]).await?;
            while let Some(row) = rows.next().await? {
                ids.push(row.get::<String>(0)?);
            }
        }
        ids.sort();
        ids.dedup();
        self.load_record_headers(ids.iter().cloned()).await?;
        self.indexes.lifecycle.insert(target.into(), ids.clone());
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn pointer_selection_seeks_existing_posting_and_record_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("plans.db")).await.unwrap();
        for related in [false, true] {
            let mut rows = store
                .db
                .query(
                    &format!("EXPLAIN QUERY PLAN {}", pointer_query(related)),
                    params!["[\"alpha\",\"beta\"]", i64::from(related), 2, NOW, "[]"],
                )
                .await
                .unwrap();
            let mut plan = Vec::new();
            while let Some(row) = rows.next().await.unwrap() {
                plan.push(row.get::<String>(3).unwrap());
            }
            eprintln!("related={related}: {plan:?}");
            for seek in [
                "SEARCH n USING",
                "SEARCH w USING PRIMARY KEY (word=?)",
                "SEARCH s USING INTEGER PRIMARY KEY",
                "SEARCH r USING PRIMARY KEY (record=?)",
            ] {
                assert!(
                    plan.iter().any(|line| line.contains(seek)),
                    "{seek}: {plan:?}"
                );
            }
            assert!(
                !plan.iter().any(|line| line == "SCAN s"
                    || line == "SCAN r"
                    || line == "SCAN w"
                    || line == "SCAN n"),
                "{plan:?}"
            );
        }
    }
    #[tokio::test]
    async fn legacy_numerical_upgrade_prepares_compact_headers_first() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        let record: Record = serde_json::from_value(
            serde_json::json!({"id":"r", "body":"# Heading\nalpha-beta original text."}),
        )
        .unwrap();
        let hash = record.body_hash();
        let built = Store::build(&path, vec![record]).await.unwrap();
        built.db.execute_batch("DELETE FROM projections WHERE key IN ('record-headers-v1','record-read-v1','fallback-read-v1'); DELETE FROM record_headers; DELETE FROM fallback_artifacts; DELETE FROM fallback_vectors;").await.unwrap();
        drop(built);
        let mut loaded = Store::open(&path).await.unwrap();
        loaded.load_record_blocks("r").await.unwrap();
        assert!(!loaded.rows.is_empty());
        assert!(loaded.rows.values().all(|b| b.hash == hash));
        assert_eq!(
            loaded.resolve_loaded("r", None).await.unwrap().0.body,
            "# Heading\nalpha-beta original text."
        );
        assert!(loaded.fallback_artifacts_ready().await.unwrap());
    }
    #[tokio::test]
    async fn healthy_open_and_clock_statistics_do_not_load_record_lists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clock.db");
        let records = vec![
            serde_json::json!({"id":"alive","body":"alpha original text"}),
            serde_json::json!({"id":"empty-expiry","body":"alpha original text","stale_after":""}),
            serde_json::json!({"id":"spaces","body":"   "}),
            serde_json::json!({"id":"early","body":"alpha original text","stale_after":"2026-11-05T00:00:00Z"}),
            serde_json::json!({"id":"late","body":"alpha original text","stale_after":"2027"}),
            serde_json::json!({"id":"candidate","body":"alpha original text","state":"candidate"}),
            serde_json::json!({"id":"closed","body":"alpha original text"}),
            serde_json::json!({"id":"close","body":"","close":{"target":"closed","reason":"expire"}}),
        ].into_iter().map(|v| serde_json::from_value(v).unwrap()).collect();
        let full = Store::build(&path, records).await.unwrap();
        let mut lazy = Store::open(&path).await.unwrap();
        assert_eq!(lazy.record_count(), full.records.len());
        assert_eq!(lazy.block_count(), full.block_count());
        assert!(lazy.records.is_empty() && lazy.lookups.is_empty() && lazy.sections.is_empty());
        assert!(lazy.indexes.records.is_empty() && lazy.indexes.record_positions.is_empty());
        let missing = query::Query::parse("zzzzunattested").unwrap();
        let clocks = ["", "2026", NOW, "2026-11-06", "2027", "z"];
        for now in clocks {
            let expected: Vec<_> = full
                .rows
                .values()
                .filter(|b| full.validity.eligible(&full.records[&b.record], now))
                .collect();
            let stats = lazy.scope_stats(now, &missing).await.unwrap();
            assert_eq!(stats.n, expected.len() as f64);
            let avg = expected.iter().map(|b| b.length).sum::<usize>() as f64
                / (expected.len() as f64).max(1.);
            assert_eq!(stats.avg.to_bits(), avg.to_bits());
            assert_eq!(stats.df["zzzzunattested"], 0.);
            assert!(lazy.records.is_empty() && lazy.lookups.is_empty());
        }
        let alpha = query::Query::parse("alpha").unwrap();
        for now in clocks {
            let stats = lazy.scope_stats(now, &alpha).await.unwrap();
            let df = full
                .rows
                .values()
                .filter(|b| {
                    full.validity.eligible(&full.records[&b.record], now)
                        && b.text.contains("alpha")
                })
                .count();
            assert_eq!(stats.df["alpha"], df as f64);
        }
        lazy.resolve_loaded("closed", None).await.unwrap();
        assert!(!lazy.hint_eligible("closed"));
        let mut plan = lazy.db.query("EXPLAIN QUERY PLAN SELECT blocks,length FROM record_expiry_totals WHERE expires>? ORDER BY expires LIMIT 1", [NOW]).await.unwrap();
        let mut search = false;
        while let Some(row) = plan.next().await.unwrap() {
            let detail = row.get::<String>(3).unwrap();
            assert!(!detail.contains("SCAN"), "{detail}");
            search |= detail.contains("SEARCH");
        }
        assert!(search);
        let mut fresh = Store::open(&path).await.unwrap();
        fresh
            .load_vocabulary_terms(vec!["alpha".into(), "zzzzunattested".into()])
            .await
            .unwrap();
        assert_eq!(fresh.vocab, HashSet::from(["alpha".into()]));
        assert_eq!(fresh.vocabulary_probes.len(), 2);
        assert!(fresh.records.is_empty());
    }
}
