//! @ref LLP 0001#original-bytes
pub mod bundle;
mod catalog;
pub mod listing;
mod migrations;
mod publication;
pub mod record;
pub mod sections;
pub mod sync;
use crate::{
    error::{Error, Result},
    language::embedding::Encoder,
    retrieval::query,
    storage::record::{Record, Validity, hash},
    storage::sections::Section,
};
use libsql::{Builder, Connection, Database, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

struct StartupTiming {
    enabled: bool,
    last: Instant,
}
impl StartupTiming {
    fn new() -> Self {
        Self {
            enabled: crate::diagnostics::enabled("LINN_STARTUP_TIMINGS"),
            last: Instant::now(),
        }
    }
    fn mark(&mut self, phase: &str) {
        let elapsed = self.last.elapsed();
        self.last = Instant::now();
        if self.enabled {
            crate::diagnostics::emit(format!(
                "linn startup: {}",
                serde_json::json!({"phase":phase,"ms":elapsed.as_secs_f64()*1000.})
            ));
        }
    }
}

pub const NOW: &str = "2026-11-05T00:00:00Z";
pub const SCHEMA_VERSION: usize = migrations::VERSION;
pub fn current_time() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds")
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp")
}
pub(crate) struct BeamCache {
    pub proposals: Vec<String>,
    pub matrix: crate::retrieval::similarity::Matrix,
}
pub(crate) type SentenceCache = Arc<(Vec<usize>, crate::retrieval::similarity::Matrix)>;
pub(crate) struct BlockMatrix {
    pub ids: Vec<usize>,
    pub matrix: crate::retrieval::similarity::Matrix,
    pub records: Vec<usize>,
    pub eligible: Vec<bool>,
    pub expires: Vec<String>,
}
#[derive(Clone)]
pub(crate) struct Statistics {
    pub n: f64,
    pub avg: f64,
    pub df: HashMap<String, f64>,
}
#[derive(Default)]
pub(crate) struct SectionTokens {
    pub own: HashSet<String>,
    pub extra: HashSet<String>,
    pub own_nonempty: bool,
}
#[derive(Default)]
pub(crate) struct RecordLookup {
    pub rows: Vec<usize>,
    pub words: HashSet<String>,
    pub sections: BTreeMap<usize, SectionTokens>,
    pub length: usize,
    pub count: usize,
}
#[derive(Default)]
pub(crate) struct Indexes {
    pub lifecycle: HashMap<String, Vec<String>>,
    pub sections: BTreeMap<usize, (String, usize)>,
    pub own: HashMap<String, Vec<usize>>,
    pub related: HashMap<String, Vec<usize>>,
    pub direct: HashMap<String, Vec<usize>>,
    pub deep: HashMap<String, Vec<usize>>,
    pub records: BTreeMap<usize, String>,
    pub record_words: HashMap<String, Vec<usize>>,
    pub block_surface: HashMap<String, Vec<usize>>,
    pub record_positions: HashMap<String, usize>,
}

pub(crate) fn posting_candidates<'a>(
    postings: &HashMap<String, Vec<usize>>,
    words: impl Iterator<Item = &'a str>,
    need: usize,
) -> Vec<usize> {
    let mut counts = HashMap::<usize, usize>::new();
    let mut seen = HashSet::new();
    for word in words.filter(|word| seen.insert(*word)) {
        for &id in postings.get(word).into_iter().flatten() {
            *counts.entry(id).or_default() += 1;
        }
    }
    let mut ids: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count >= need)
        .map(|(id, _)| id)
        .collect();
    ids.sort_unstable();
    ids
}
#[derive(Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub record: String,
    pub hash: String,
    pub ordinal: usize,
    pub heading: String,
    pub path: Vec<String>,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub words: HashSet<String>,
    pub surface: HashSet<String>,
    pub tf: HashMap<String, [f64; 3]>,
    pub length: usize,
    pub body_positions: HashMap<String, Vec<usize>>,
    pub stem_positions: HashMap<String, Vec<usize>>,
}
#[derive(Clone)]
pub struct Tree {
    pub record: String,
    pub words: HashSet<String>,
    pub rows: Vec<usize>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Card {
    pub record: String,
    pub heading: String,
    pub span: [usize; 2],
    pub section: usize,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetch: Option<String>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Pointer {
    pub record: String,
    pub label: String,
    pub heading: String,
    pub span: [usize; 2],
    pub matched: Vec<String>,
    pub missing: Vec<String>,
    pub excerpt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Call {
    pub cards: Vec<Card>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<Pointer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<Pointer>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Context {
    pub scope: Option<String>,
    pub cwd: String,
    pub now: String,
    pub session: String,
    pub calls_left: usize,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            scope: None,
            cwd: String::new(),
            now: current_time(),
            session: String::new(),
            calls_left: 3,
        }
    }
}
pub struct Store {
    pub(crate) query_clock: String,
    memo_query: Option<(String, Option<String>)>,
    porter_probes: HashSet<String>,
    stat_misses: HashSet<String>,
    _database: Database,
    pub db: Connection,
    pub records: BTreeMap<String, Record>,
    pub rows: BTreeMap<usize, Block>,
    pub sections: BTreeMap<String, Vec<Section>>,
    pub(crate) lookups: HashMap<String, RecordLookup>,
    pub(crate) indexes: Indexes,
    pub validity: Validity,
    pub vectors: HashMap<usize, Vec<f32>>,
    pub df: HashMap<String, usize>,
    pub vocab: HashSet<String>,
    pub word_order: Vec<String>,
    pub direct: BTreeMap<usize, Tree>,
    pub deep: BTreeMap<usize, Tree>,
    pub links: HashMap<String, HashSet<String>>,
    pub basenames: HashMap<String, HashSet<String>>,
    encoder: Option<Arc<Encoder>>,
    pub hits: HashMap<(bool, String), HashSet<usize>>,
    eligible_hits: HashMap<(bool, String, String), HashSet<usize>>,
    pub porter_cache: HashMap<String, String>,
    pub(crate) form_cache: Option<Arc<crate::retrieval::fallback::Forms>>,
    pub(crate) beam_cache: Option<Arc<BeamCache>>,
    pub(crate) vector_matrix: Option<Arc<BlockMatrix>>,
    pub(crate) sentence_cache: Option<SentenceCache>,
    pub(crate) stats: HashMap<String, Arc<Statistics>>,
    pub(crate) coverage:
        HashMap<(String, Option<String>, bool), Arc<crate::retrieval::lexical::Coverage>>,
    pub(crate) rankings:
        HashMap<(String, Option<String>, String, bool), Vec<crate::retrieval::lexical::Rank>>,
    pub confidences: HashMap<(String, Option<String>, String), Option<f64>>,
    loaded_terms: HashSet<String>,
    loaded_stems: HashSet<String>,
    candidate_terms: HashMap<(bool, bool, Vec<String>), HashSet<usize>>,
    structure_lazy: bool,
    records_lazy: bool,
    record_count: usize,
    block_count: usize,
    loaded_blocks: HashSet<usize>,
    loaded_block_headers: HashSet<usize>,
    loaded_sections: HashSet<String>,
    navigation_words: HashSet<String>,
    surface_words: HashSet<String>,
    section_words: HashSet<String>,
    tree_rows: HashSet<(bool, usize)>,
    vectors_loaded: bool,
    vocabulary_loaded: bool,
    vocabulary_probes: HashSet<String>,
    pub(crate) structure_stamp: Vec<u8>,
    source_version: i64,
    write_failed: bool,
}

// Soft memo budgets checked between queries, independent of corpus size.
fn memo_over_budget(entries: impl Iterator<Item = (usize, usize)>) -> bool {
    let (mut count, mut keys, mut slots) = (0usize, 0usize, 0usize);
    for (key_bytes, id_slots) in entries {
        count += 1;
        keys = keys.saturating_add(key_bytes);
        slots = slots.saturating_add(id_slots);
        if count > 256 || keys > 64 * 1024 || slots > 64 * 1024 {
            return true;
        }
    }
    false
}

const SCRATCH: &str = r#"
CREATE TEMP TABLE feature_candidates(id INTEGER PRIMARY KEY);
CREATE VIRTUAL TABLE temp.fts_terms USING fts5vocab(main,fts,'row');
"#;

impl Store {
    pub async fn open(path: &Path) -> Result<Self> {
        Self::open_index(path, false).await
    }
    /// Open an older embedding projection only for an explicit complete rebuild.
    pub(crate) async fn open_for_rebuild(path: &Path) -> Result<Self> {
        Self::open_index(path, true).await
    }
    async fn open_index(path: &Path, rebuilding: bool) -> Result<Self> {
        let mut timing = StartupTiming::new();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
        }
        use std::os::unix::fs::OpenOptionsExt;
        if !path.exists() {
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
        }
        let database = Builder::new_local(path).build().await?;
        let db = database.connect()?;
        db.execute_batch("PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-65536; PRAGMA mmap_size=268435456;").await?;
        migrations::apply(&db).await?;
        let model = db.transaction().await?;
        migrations::check(&model).await?;
        model.execute_batch(SCRATCH).await?;
        // @ref LLP 0001#embedding-publication
        let mut identity = model
            .query(
                "SELECT value FROM projections WHERE key='block-model-v1'",
                (),
            )
            .await?;
        let identity = identity
            .next()
            .await?
            .map(|row| row.get::<Vec<u8>>(0))
            .transpose()?;
        if identity.as_deref() != Some(crate::language::embedding::ARTIFACT_MODEL.as_bytes()) {
            let mut records = model
                .query("SELECT EXISTS(SELECT 1 FROM records)", ())
                .await?;
            let populated = records.next().await?.unwrap().get::<i64>(0)? != 0;
            drop(records);
            if populated && !rebuilding {
                return Err(Error::Invalid(
                    "embedding index needs `linn index:rebuild` with this binary".into(),
                ));
            }
            if !populated {
                model
                    .execute(
                        "INSERT OR REPLACE INTO projections VALUES('block-model-v1',?)",
                        [crate::language::embedding::ARTIFACT_MODEL.as_bytes()],
                    )
                    .await?;
            }
        }
        model.commit().await?;
        timing.mark("database-open-schema-scratch");
        let mut st = Self {
            query_clock: current_time(),
            memo_query: None,
            porter_probes: HashSet::new(),
            stat_misses: HashSet::new(),
            _database: database,
            db,
            records: BTreeMap::new(),
            rows: BTreeMap::new(),
            sections: BTreeMap::new(),
            lookups: HashMap::new(),
            indexes: Indexes::default(),
            validity: Validity::default(),
            vectors: HashMap::new(),
            df: HashMap::new(),
            vocab: HashSet::new(),
            word_order: Vec::new(),
            direct: BTreeMap::new(),
            deep: BTreeMap::new(),
            links: HashMap::new(),
            basenames: HashMap::new(),
            encoder: None,
            hits: HashMap::new(),
            eligible_hits: HashMap::new(),
            porter_cache: HashMap::new(),
            form_cache: None,
            beam_cache: None,
            vector_matrix: None,
            sentence_cache: None,
            stats: HashMap::new(),
            coverage: HashMap::new(),
            rankings: HashMap::new(),
            confidences: HashMap::new(),
            loaded_terms: HashSet::new(),
            loaded_stems: HashSet::new(),
            candidate_terms: HashMap::new(),
            structure_lazy: false,
            records_lazy: false,
            record_count: 0,
            block_count: 0,
            loaded_blocks: HashSet::new(),
            loaded_block_headers: HashSet::new(),
            loaded_sections: HashSet::new(),
            navigation_words: HashSet::new(),
            surface_words: HashSet::new(),
            section_words: HashSet::new(),
            tree_rows: HashSet::new(),
            vectors_loaded: false,
            vocabulary_loaded: false,
            vocabulary_probes: HashSet::new(),
            structure_stamp: Vec::new(),
            source_version: 0,
            write_failed: false,
        };
        let (snapshot, source_version) = Self::source_snapshot(&st.db, rebuilding).await?;
        st.source_version = source_version;
        if st.open_catalog().await? {
            timing.mark("record-catalog-open");
            snapshot.rollback().await?;
            st.prepare_fallback_artifacts().await?;
            timing.mark("fallback-artifacts-verify-or-build");
            st.prepare_pos_pair_lookup().await?;
            timing.mark("pos-pair-lookup-verify-or-build");
            return Ok(st);
        }
        let mut header_marker = st
            .db
            .query(
                "SELECT 1 FROM projections WHERE key='record-headers-v1'",
                (),
            )
            .await?;
        let headers_ready = header_marker.next().await?.is_some();
        drop(header_marker);
        if headers_ready {
            let mut rows = st.db.query("SELECT data,body_bytes,body_nonblank,body_hash FROM record_headers ORDER BY record", ()).await?;
            while let Some(row) = rows.next().await? {
                let mut record: Record = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
                record.stored_body = Some(crate::storage::record::StoredBody {
                    bytes: row.get::<i64>(1)? as usize,
                    nonblank: row.get::<i64>(2)? != 0,
                    hash: row.get(3)?,
                });
                st.records.insert(record.id.clone(), record);
            }
        } else {
            let mut result = st
                .db
                .query("SELECT envelope FROM records ORDER BY id", ())
                .await?;
            while let Some(row) = result.next().await? {
                let bytes = row.get::<Vec<u8>>(0)?;
                let rec: Record = serde_json::from_slice(&bytes)?;
                st.records.insert(rec.id.clone(), rec);
            }
        }
        timing.mark("records-read-decode");
        let mut projection = st
            .db
            .query("SELECT value FROM projections WHERE key='validity'", ())
            .await?;
        if let Some(row) = projection.next().await? {
            st.validity = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
        } else {
            st.validity = Validity::project(&st.records.values().cloned().collect::<Vec<_>>());
        }
        drop(projection);
        timing.mark("validity");
        let mut ready = st
            .db
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await?;
        if let Some(row) = ready.next().await? {
            st.structure_stamp = row.get(0)?;
        }
        drop(ready);
        let version = structure_version();
        let projected = st.structure_stamp == version
            || st
                .structure_stamp
                .starts_with(&[version.as_slice(), b":"].concat());
        if projected {
            st.load_structure().await?;
            timing.mark("structural-metadata-read");
        } else {
            st.load_records(st.records.keys().cloned().collect::<Vec<_>>())
                .await?;
            st.make_rows();
            timing.mark("parse-blocks-tokenize");
            let mut by_span: HashMap<_, _> = std::mem::take(&mut st.rows)
                .into_values()
                .map(|b| ((b.record.clone(), b.start, b.end), b))
                .collect();
            // Forget leaves gaps in persistent row ids. Reopen must retain them.
            let mut stored = st
                .db
                .query("SELECT id,record,start,end FROM sections ORDER BY id", ())
                .await?;
            let mut indexed = BTreeMap::new();
            while let Some(row) = stored.next().await? {
                let id = row.get::<i64>(0)? as usize;
                let record = row.get::<String>(1)?;
                let start = row.get::<i64>(2)? as usize;
                let end = row.get::<i64>(3)? as usize;
                if let Some(mut block) = by_span.remove(&(record, start, end)) {
                    block.id = id;
                    indexed.insert(id, block);
                }
            }
            st.rows = indexed;
            st.block_count = st.rows.len();
            timing.mark("section-ids-read-map");
        }
        timing.mark("vectors-deferred");
        st.statistics_timed(&mut timing).await?;
        if projected {
            snapshot.rollback().await?;
            if st.structure_stamp == structure_version() {
                let stamp = structure_stamp();
                let tx = st
                    .db
                    .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
                    .await?;
                st.verify_source_version(&tx).await?;
                let changed = tx
                    .execute(
                        "UPDATE projections SET value=? WHERE key='structure' AND value=?",
                        params![stamp.clone(), st.structure_stamp.clone()],
                    )
                    .await?;
                if changed != 1 {
                    return Err(Error::Invalid(
                        "store changed during open; reopen it".into(),
                    ));
                }
                tx.commit().await?;
                st.structure_stamp = stamp;
            }
        } else {
            st.navigation_timed(&mut timing);
            snapshot.rollback().await?;
            st.persist_structure().await?;
            st.publish_pos().await?;
            timing.mark("structural-projection-write");
        }
        st.prepare_record_headers().await?;
        st.prepare_fallback_artifacts().await?;
        timing.mark("fallback-artifacts-verify-or-build");
        st.prepare_record_totals().await?;
        st.prepare_basename_index().await?;
        st.prepare_pos_pair_lookup().await?;
        timing.mark("pos-pair-lookup-verify-or-build");
        st.publish_catalog().await?;
        Ok(st)
    }
    // @ref LLP 0006#merge-and-publication
    async fn source_snapshot(
        db: &Connection,
        rebuilding: bool,
    ) -> Result<(libsql::Transaction, i64)> {
        let snapshot = db.transaction().await?;
        migrations::check(&snapshot).await?;
        // This read pins the snapshot before recording its version or reading indexes.
        let mut pending = snapshot
            .query("SELECT 1 FROM sync_state WHERE key='pending'", ())
            .await?;
        if pending.next().await?.is_some() && !rebuilding {
            return Err(Error::Invalid(
                "index publication incomplete; run linn index:rebuild or linn sync".into(),
            ));
        }
        drop(pending);
        let mut version = snapshot.query("PRAGMA data_version", ()).await?;
        let source_version = version.next().await?.unwrap().get(0)?;
        drop(version);
        Ok((snapshot, source_version))
    }
    pub fn encoder(&mut self) -> Result<Arc<Encoder>> {
        if self.encoder.is_none() {
            self.encoder = Some(crate::language::embedding::shared_encoder()?);
        }
        Ok(self.encoder.as_ref().unwrap().clone())
    }
    pub async fn load_vectors(&mut self) -> Result<()> {
        self.ensure_usable()?;
        if self.vectors_loaded {
            return Ok(());
        }
        let mut vectors = HashMap::new();
        let mut result = self
            .db
            .query("SELECT id,emb FROM vectors ORDER BY id", ())
            .await?;
        while let Some(row) = result.next().await? {
            let bytes = row.get::<Vec<u8>>(1)?;
            let vector = bytes
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            vectors.insert(row.get::<i64>(0)? as usize, vector);
        }
        self.vectors = vectors;
        self.vectors_loaded = true;
        Ok(())
    }
    #[cfg(test)]
    pub(crate) async fn load_vocabulary(&mut self) -> Result<()> {
        if self.vocabulary_loaded {
            return Ok(());
        }
        let mut vocabulary = HashSet::new();
        let mut result = self.db.query("SELECT term FROM temp.fts_terms", ()).await?;
        while let Some(row) = result.next().await? {
            vocabulary.insert(row.get(0)?);
        }
        self.vocab = vocabulary;
        self.vocabulary_loaded = true;
        Ok(())
    }
    pub(crate) async fn load_vocabulary_terms(&mut self, words: Vec<String>) -> Result<()> {
        if self.vocabulary_loaded {
            return Ok(());
        }
        let words: Vec<_> = words
            .into_iter()
            .filter(|word| !self.vocabulary_probes.contains(word) && !self.vocab.contains(word))
            .collect();
        for chunk in words.chunks(128) {
            let mut rows = self.db.query(
                "SELECT term FROM temp.fts_terms WHERE term IN (SELECT value FROM json_each(?))",
                [serde_json::to_string(chunk)?],
            ).await?;
            while let Some(row) = rows.next().await? {
                self.vocab.insert(row.get(0)?);
            }
            self.vocabulary_probes.extend(chunk.iter().cloned());
        }
        Ok(())
    }
    pub(crate) async fn fallback_artifact<T: serde::de::DeserializeOwned>(
        &self,
        kind: &str,
    ) -> Result<Option<(T, crate::retrieval::similarity::Matrix)>> {
        let started = Instant::now();
        let Some((metadata, rows)) = self.fallback_metadata(kind).await? else {
            return Ok(None);
        };
        let matrix = self.fallback_matrix(kind, rows).await?;
        if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
            crate::diagnostics::emit(format!(
                "linn query: {}",
                serde_json::json!({"phase":"numerical-array-read","kind":kind,"rows":rows,"ms":started.elapsed().as_secs_f64()*1000.})
            ));
        }
        Ok(Some((metadata, matrix)))
    }
    pub(crate) async fn fallback_metadata<T: serde::de::DeserializeOwned>(
        &self,
        kind: &str,
    ) -> Result<Option<(T, usize)>> {
        let mut result=self.db.query("SELECT metadata,rows FROM fallback_artifacts WHERE kind=? AND generation=? AND model=?",params![kind,self.structure_stamp.clone(),crate::language::embedding::ARTIFACT_MODEL]).await?;
        let Some(row) = result.next().await? else {
            return Ok(None);
        };
        let metadata = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
        let rows = usize::try_from(row.get::<i64>(1)?)
            .map_err(|_| Error::Invalid("invalid numerical projection length".into()))?;
        Ok(Some((metadata, rows)))
    }
    pub(crate) async fn fallback_matrix(
        &self,
        kind: &str,
        rows: usize,
    ) -> Result<crate::retrieval::similarity::Matrix> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(
                rows.checked_mul(512)
                    .ok_or_else(|| Error::Invalid("oversized numerical projection".into()))?,
            )
            .map_err(|e| Error::Invalid(format!("numerical projection allocation: {e}")))?;
        let mut result = self
            .db
            .query(
                "SELECT data FROM fallback_vectors WHERE kind=? ORDER BY part",
                [kind],
            )
            .await?;
        while let Some(row) = result.next().await? {
            let bytes = row.get::<Vec<u8>>(0)?;
            if bytes.len() % 4 != 0 {
                return Err(Error::Invalid("invalid numerical projection bytes".into()));
            }
            values.extend(
                bytes
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap())),
            );
        }
        crate::retrieval::similarity::Matrix::from_values(values, rows)
            .ok_or_else(|| Error::Invalid("invalid numerical projection matrix".into()))
    }
    pub(crate) async fn fallback_artifacts_ready(&self) -> Result<bool> {
        let mut result=self.db.query("SELECT count(*) FROM fallback_artifacts WHERE generation=? AND model=? AND kind IN ('forms','beam',?)",params![self.structure_stamp.clone(),crate::language::embedding::ARTIFACT_MODEL,crate::retrieval::fallback::SENTENCE_DICTIONARY]).await?;
        Ok(result.next().await?.unwrap().get::<i64>(0)? == 3)
    }
    pub(crate) async fn legacy_fallback_artifacts_ready(&self) -> Result<bool> {
        let mut result=self.db.query("SELECT count(*) FROM fallback_artifacts WHERE generation=? AND model=? AND kind IN ('forms','beam','sentences')",params![self.structure_stamp.clone(),crate::language::embedding::ARTIFACT_MODEL]).await?;
        Ok(result.next().await?.unwrap().get::<i64>(0)? == 3)
    }
    pub(crate) async fn publish_fallback_artifacts(
        &self,
        artifacts: &[(&str, Vec<u8>, &crate::retrieval::similarity::Matrix)],
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        let mut ready = tx
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await?;
        if ready
            .next()
            .await?
            .map(|r| r.get::<Vec<u8>>(0))
            .transpose()?
            .as_ref()
            != Some(&self.structure_stamp)
        {
            return Err(Error::Invalid(
                "store changed; reopen it before publishing numerical projections".into(),
            ));
        }
        drop(ready);
        let manifest = tx
            .prepare("INSERT OR REPLACE INTO fallback_artifacts VALUES(?,?,?,?,?)")
            .await?;
        let chunk = tx
            .prepare("INSERT INTO fallback_vectors VALUES(?,?,?)")
            .await?;
        for (kind, metadata, matrix) in artifacts {
            if *kind == crate::retrieval::fallback::SENTENCE_DICTIONARY {
                tx.execute("DELETE FROM fallback_vectors WHERE kind='sentences'", ())
                    .await?;
                tx.execute("DELETE FROM fallback_artifacts WHERE kind='sentences'", ())
                    .await?;
            }
            tx.execute("DELETE FROM fallback_vectors WHERE kind=?", [*kind])
                .await?;
            manifest
                .execute(params![
                    *kind,
                    self.structure_stamp.clone(),
                    crate::language::embedding::ARTIFACT_MODEL,
                    metadata.clone(),
                    matrix.row_count() as i64
                ])
                .await?;
            manifest.reset();
            for (part, values) in matrix.chunks().enumerate() {
                let bytes: Vec<_> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
                chunk.execute(params![*kind, part as i64, bytes]).await?;
                chunk.reset();
            }
        }
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn selected_vectors(&self, ids: &[usize]) -> Result<HashMap<usize, Vec<f32>>> {
        let mut vectors = HashMap::new();
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        if let Some(matrix) = &self.vector_matrix {
            for id in ids {
                let row = matrix
                    .ids
                    .binary_search(&id)
                    .map_err(|_| Error::Invalid("missing selected block vector".into()))?;
                vectors.insert(id, matrix.matrix.row(row).to_vec());
            }
            return Ok(vectors);
        }
        for chunk in ids.chunks(256) {
            let mut rows = self.db.query(
                "SELECT id,emb FROM vectors WHERE id IN (SELECT value FROM json_each(?)) ORDER BY id",
                [serde_json::to_string(chunk)?],
            ).await?;
            while let Some(row) = rows.next().await? {
                let bytes = row.get::<Vec<u8>>(1)?;
                if bytes.len() != 512 * 4 {
                    return Err(Error::Invalid("invalid block vector size".into()));
                }
                vectors.insert(
                    row.get::<i64>(0)? as usize,
                    bytes
                        .chunks_exact(4)
                        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                        .collect(),
                );
            }
        }
        if vectors.len() != ids.len() {
            return Err(Error::Invalid("missing selected block vector".into()));
        }
        Ok(vectors)
    }
    // @ref LLP 0001#publication-and-indexed-reads
    pub(crate) async fn candidate_vector_scores(
        &self,
        ids: &[usize],
        query: &[f32],
    ) -> Result<Vec<(usize, f32)>> {
        if let Some(cache) = &self.vector_matrix {
            let scores = cache.matrix.multiply(query);
            return Ok(ids
                .iter()
                .filter_map(|id| {
                    cache
                        .ids
                        .binary_search(id)
                        .ok()
                        .map(|row| (*id, scores[row]))
                })
                .collect());
        }
        let started = Instant::now();
        let mut bounds = self
            .db
            .query(
                "SELECT (SELECT count(*) FROM vectors INDEXED BY vectors_order),(SELECT id FROM vectors ORDER BY id LIMIT 1),(SELECT id FROM vectors ORDER BY id DESC LIMIT 1)",
                (),
            )
            .await?;
        let bounds = bounds.next().await?.unwrap();
        let count = bounds.get::<i64>(0)? as usize;
        if count == 0 || ids.is_empty() {
            return Ok(vec![]);
        }
        let first = bounds.get::<i64>(1)?;
        let last = bounds.get::<i64>(2)?;
        let contiguous = last - first + 1 == count as i64;
        let sql = if contiguous {
            "SELECT id,id-?2,emb FROM vectors WHERE id IN (SELECT value FROM json_each(?1)) ORDER BY id"
        } else {
            "WITH positions AS (SELECT id,row_number() OVER (ORDER BY id)-1 AS position FROM vectors INDEXED BY vectors_order) SELECT p.id,p.position,v.emb FROM positions p JOIN vectors v ON v.id=p.id WHERE p.id IN (SELECT value FROM json_each(?1)) AND ?2 IS NOT NULL ORDER BY p.id"
        };
        let mut rows = self
            .db
            .query(sql, params![serde_json::to_string(ids)?, first])
            .await?;
        // Keep the original kernel shape and row positions. Unselected outputs
        // are discarded; zero padding avoids reading their vector payloads.
        let size = count
            .checked_mul(512)
            .ok_or_else(|| Error::Invalid("oversized block matrix".into()))?;
        let mut values = vec![0.; size];
        let mut selected = Vec::new();
        while let Some(row) = rows.next().await? {
            let id = row.get::<i64>(0)? as usize;
            let position = row.get::<i64>(1)? as usize;
            let bytes = row.get::<Vec<u8>>(2)?;
            if bytes.len() != 512 * 4 || position >= count {
                return Err(Error::Invalid("invalid selected block vector".into()));
            }
            for (out, bytes) in values[position * 512..(position + 1) * 512]
                .iter_mut()
                .zip(bytes.chunks_exact(4))
            {
                *out = f32::from_le_bytes(bytes.try_into().unwrap());
            }
            selected.push((id, position));
        }
        let matrix = crate::retrieval::similarity::Matrix::from_values(values, count).unwrap();
        let scores = matrix.multiply(query);
        if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
            crate::diagnostics::emit(format!(
                "linn query: {}",
                serde_json::json!({"phase":"candidate-vector-score","rows":count,"selected":selected.len(),"ms":started.elapsed().as_secs_f64()*1000.})
            ));
        }
        Ok(selected
            .into_iter()
            .map(|(id, position)| (id, scores[position]))
            .collect())
    }
    pub(crate) async fn block_matrix(&mut self) -> Result<Arc<BlockMatrix>> {
        if let Some(cache) = &self.vector_matrix {
            return Ok(cache.clone());
        }
        let started = Instant::now();
        let mut ids = Vec::with_capacity(self.block_count);
        let mut records = Vec::with_capacity(self.block_count);
        let mut eligible = Vec::with_capacity(self.block_count);
        let mut expires = Vec::with_capacity(self.block_count);
        let mut values = Vec::new();
        values
            .try_reserve_exact(
                self.block_count
                    .checked_mul(512)
                    .ok_or_else(|| Error::Invalid("oversized block matrix".into()))?,
            )
            .map_err(|e| Error::Invalid(format!("block matrix allocation: {e}")))?;
        if self.vectors_loaded && !self.records_lazy {
            self.load_block_headers(self.vectors.keys().copied().collect::<Vec<_>>())
                .await?;
            let mut ordered: Vec<_> = self.vectors.keys().copied().collect();
            ordered.sort_unstable();
            for id in ordered {
                ids.push(id);
                records.push(self.indexes.record_positions[&self.rows[&id].record]);
                let record = &self.records[&self.rows[&id].record];
                eligible.push(self.tree_eligible(record));
                expires.push(record.stale_after.clone().unwrap_or_default());
                values.extend_from_slice(&self.vectors[&id]);
            }
        } else {
            let mut result = self
                .db
                .query(
                    "SELECT v.id,v.emb,c.ordinal,(c.eligible & 2)!=0,c.expires FROM vectors v JOIN record_read_index c ON c.record=v.record ORDER BY v.id",
                    (),
                )
                .await?;
            while let Some(row) = result.next().await? {
                let id = row.get::<i64>(0)? as usize;
                let bytes = row.get::<Vec<u8>>(1)?;
                if bytes.len() != 512 * 4 {
                    return Err(Error::Invalid("invalid block vector size".into()));
                }
                ids.push(id);
                records.push(row.get::<i64>(2)? as usize);
                eligible.push(row.get::<i64>(3)? != 0);
                expires.push(row.get::<String>(4)?);
                values.extend(
                    bytes
                        .chunks_exact(4)
                        .map(|b| f32::from_le_bytes(b.try_into().unwrap())),
                );
            }
        }
        let matrix = crate::retrieval::similarity::Matrix::from_values(values, ids.len())
            .ok_or_else(|| Error::Invalid("invalid block matrix".into()))?;
        let cache = Arc::new(BlockMatrix {
            ids,
            matrix,
            records,
            eligible,
            expires,
        });
        if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
            crate::diagnostics::emit(format!(
                "linn query: {}",
                serde_json::json!({"phase":"block-matrix-read","rows":cache.ids.len(),"ms":started.elapsed().as_secs_f64()*1000.})
            ));
        }
        self.vector_matrix = Some(cache.clone());
        Ok(cache)
    }
    async fn persist_structure(&mut self) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute_batch("DELETE FROM nav_block_words; DELETE FROM nav_section_words; DELETE FROM nav_surface; DELETE FROM nav_sections; DELETE FROM nav_blocks; DELETE FROM nav_tree_rows; DELETE FROM nav_trees; DELETE FROM nav_links; DELETE FROM nav_lexemes; DELETE FROM record_index_totals; DELETE FROM fallback_vectors; DELETE FROM fallback_artifacts;").await?;
        let mut lexemes = HashMap::<&str, i64>::new();
        for word in &self.word_order {
            let id = lexemes.len() as i64;
            lexemes.entry(word).or_insert(id);
        }
        for lookup in self.lookups.values() {
            for section in lookup.sections.values() {
                for word in section.own.union(&section.extra) {
                    let id = lexemes.len() as i64;
                    lexemes.entry(word).or_insert(id);
                }
            }
        }
        let insert = tx.prepare("INSERT INTO nav_lexemes VALUES(?,?,?)").await?;
        for (&word, &id) in &lexemes {
            insert.execute(params![id, word, query::norm(word)]).await?;
            insert.reset();
        }
        let insert = tx.prepare("INSERT INTO nav_surface VALUES(?,?,?)").await?;
        for (order, word) in self.word_order.iter().enumerate() {
            insert
                .execute(params![
                    lexemes[word.as_str()],
                    self.df[word] as i64,
                    order as i64
                ])
                .await?;
            insert.reset();
        }
        let block = tx
            .prepare("INSERT INTO nav_blocks VALUES(?,?,?,?,?,?,?)")
            .await?;
        let word = tx
            .prepare("INSERT INTO nav_block_words VALUES(?,?)")
            .await?;
        for b in self.rows.values() {
            block
                .execute(params![
                    b.id as i64,
                    b.record.as_str(),
                    b.ordinal as i64,
                    b.heading.as_str(),
                    serde_json::to_vec(&b.path)?,
                    b.start as i64,
                    b.end as i64
                ])
                .await?;
            block.reset();
            for surface in &b.surface {
                word.execute(params![lexemes[surface.as_str()], b.id as i64])
                    .await?;
                word.reset();
            }
        }
        let section = tx
            .prepare("INSERT INTO nav_sections VALUES(?,?,?,?,?)")
            .await?;
        let word = tx
            .prepare("INSERT INTO nav_section_words VALUES(?,?,?)")
            .await?;
        for (&id, (rid, ordinal)) in &self.indexes.sections {
            let tokens = &self.lookups[rid].sections[ordinal];
            section
                .execute(params![
                    id as i64,
                    rid.as_str(),
                    *ordinal as i64,
                    serde_json::to_vec(&self.sections[rid][*ordinal])?,
                    i64::from(tokens.own_nonempty)
                ])
                .await?;
            section.reset();
            for token in tokens.own.union(&tokens.extra) {
                let flags = i64::from(tokens.own.contains(token))
                    + 2 * i64::from(tokens.extra.contains(token));
                word.execute(params![lexemes[token.as_str()], id as i64, flags])
                    .await?;
                word.reset();
            }
        }
        let tree = tx.prepare("INSERT INTO nav_trees VALUES(?,?,?)").await?;
        let row = tx
            .prepare("INSERT INTO nav_tree_rows VALUES(?,?,?)")
            .await?;
        for (kind, trees) in [(0i64, &self.direct), (1, &self.deep)] {
            for (&id, t) in trees {
                tree.execute(params![kind, id as i64, t.record.as_str()])
                    .await?;
                tree.reset();
                for &sid in &t.rows {
                    row.execute(params![kind, id as i64, sid as i64]).await?;
                    row.reset();
                }
            }
        }
        let link = tx.prepare("INSERT INTO nav_links VALUES(?,?)").await?;
        for (source, targets) in &self.links {
            for target in targets {
                link.execute(params![source.as_str(), target.as_str()])
                    .await?;
                link.reset();
            }
        }
        let stamp = structure_stamp();
        self.write_record_totals(&tx, &stamp).await?;
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('structure',?)",
            [stamp.clone()],
        )
        .await?;
        tx.commit().await?;
        self.structure_stamp = stamp;
        self.prepare_basename_index().await?;
        self.prepare_record_headers().await?;
        Ok(())
    }
    // @ref LLP 0007#store-publication-and-synchronization
    pub(crate) async fn application_snapshot(&self) -> Result<libsql::Transaction> {
        self.ensure_usable()?;
        let (snapshot, version) = Self::source_snapshot(&self.db, false).await?;
        if version != self.source_version {
            return Err(Error::Invalid("store changed; reopen it".into()));
        }
        Ok(snapshot)
    }
    pub(crate) async fn verify_source_version(&self, db: &Connection) -> Result<()> {
        migrations::check(db).await?;
        let mut version = db.query("PRAGMA data_version", ()).await?;
        if version.next().await?.unwrap().get::<i64>(0)? != self.source_version {
            return Err(Error::Invalid(
                "store changed during reconstruction; reopen it".into(),
            ));
        }
        Ok(())
    }
    async fn load_structure(&mut self) -> Result<()> {
        self.structure_lazy = true;
        self.loaded_block_headers.clear();
        self.navigation_words.clear();
        self.surface_words.clear();
        self.section_words.clear();
        self.tree_rows.clear();
        self.lookups = self
            .records
            .keys()
            .map(|r| (r.clone(), RecordLookup::default()))
            .collect();
        self.indexes = Indexes::default();
        self.indexes.records = self.records.keys().cloned().enumerate().collect();
        self.indexes.record_positions = self
            .indexes
            .records
            .iter()
            .map(|(i, r)| (r.clone(), *i))
            .collect();
        self.sections = self.records.keys().map(|r| (r.clone(), vec![])).collect();
        self.rows.clear();
        self.direct.clear();
        self.deep.clear();
        self.links.clear();
        self.basenames.clear();
        self.loaded_blocks.clear();
        self.loaded_sections.clear();
        self.block_count = 0;
        let mut marker = self
            .db
            .query(
                "SELECT value FROM projections WHERE key='record-totals-v1'",
                (),
            )
            .await?;
        let ready = match marker.next().await? {
            Some(row) => row.get::<Vec<u8>>(0)? == self.structure_stamp,
            None => false,
        };
        drop(marker);
        if ready {
            let mut rows = self
                .db
                .query(
                    "SELECT record,blocks,length FROM record_index_totals ORDER BY record",
                    (),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                let rid = row.get::<String>(0)?;
                let lookup = self
                    .lookups
                    .get_mut(&rid)
                    .ok_or_else(|| Error::Invalid("unknown count record".into()))?;
                lookup.count = row.get::<i64>(1)? as usize;
                lookup.length = row.get::<i64>(2)? as usize;
                self.block_count += lookup.count;
            }
        } else {
            // One-time migration: decode the native FTS lengths, preserving integer totals.
            let mut rows = self.db.query("SELECT b.record,d.sz FROM nav_blocks b JOIN fts_docsize d ON d.id=b.id ORDER BY b.id", ()).await?;
            while let Some(row) = rows.next().await? {
                let rid = row.get::<String>(0)?;
                let lookup = self.lookups.get_mut(&rid).unwrap();
                lookup.count += 1;
                lookup.length += column_lengths(&row.get::<Vec<u8>>(1)?)?;
                self.block_count += 1;
            }
        }
        Ok(())
    }
    // @ref LLP 0001#on-demand-source-and-structural-reads
    pub(crate) async fn load_records(
        &mut self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<()> {
        let ids: Vec<_> = ids.into_iter().collect();
        self.load_record_headers(ids.iter().cloned()).await?;
        let mut ids: Vec<_> = ids
            .into_iter()
            .filter(|id| {
                self.records
                    .get(id)
                    .is_some_and(|r| r.stored_body.is_some())
            })
            .collect();
        ids.sort();
        ids.dedup();
        for chunk in ids.chunks(128) {
            let mut rows = self.db.query("SELECT id,body FROM records WHERE id IN (SELECT value FROM json_each(?)) ORDER BY id", [serde_json::to_string(chunk)?]).await?;
            while let Some(row) = rows.next().await? {
                let id = row.get::<String>(0)?;
                let record = self.records.get_mut(&id).unwrap();
                record.body = String::from_utf8(row.get::<Vec<u8>>(1)?)
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                if record.body.len() != record.body_len() {
                    return Err(Error::Invalid("record body length changed; reopen".into()));
                }
                record.stored_body = None;
            }
            if chunk
                .iter()
                .any(|id| self.records[id].stored_body.is_some())
            {
                return Err(Error::Invalid("record body missing; reopen".into()));
            }
        }
        Ok(())
    }
    async fn prepare_record_headers(&mut self) -> Result<()> {
        let mut rows = self
            .db
            .query(
                "SELECT 1 FROM projections WHERE key='record-headers-v1'",
                (),
            )
            .await?;
        if rows.next().await?.is_some() {
            return Ok(());
        }
        drop(rows);
        self.load_records(self.records.keys().cloned().collect::<Vec<_>>())
            .await?;
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute("DELETE FROM record_headers", ()).await?;
        let statement = tx
            .prepare("INSERT INTO record_headers VALUES(?,?,?,?,?)")
            .await?;
        for record in self.records.values() {
            let mut header = record.clone();
            header.body.clear();
            statement
                .execute(params![
                    record.id.as_str(),
                    serde_json::to_vec(&header)?,
                    record.body.len() as i64,
                    i64::from(record.body_nonblank()),
                    record.body_hash()
                ])
                .await?;
            statement.reset();
        }
        drop(statement);
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('record-headers-v1',x'01')",
            (),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn resolve_metadata(
        &mut self,
        reference: &str,
        scope: Option<&str>,
    ) -> Result<(&Record, [usize; 2])> {
        self.load_record_headers([reference.to_owned()]).await?;
        let (id, span) = if let Some(record) = self.records.get(reference) {
            (record.id.clone(), [0, record.body_len()])
        } else {
            let prefix = reference.split(':').next().unwrap_or("");
            if prefix.len() < 8 || !prefix.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(Error::Invalid(format!("unknown reference: {reference}")));
            }
            let mut rows = self
                .db
                .query(
                    "SELECT id FROM records WHERE body_hash>=? AND body_hash<? ORDER BY id",
                    params![prefix, format!("{prefix}g")],
                )
                .await?;
            let mut ids = Vec::new();
            while let Some(row) = rows.next().await? {
                ids.push(row.get::<String>(0)?);
            }
            self.load_record_headers(ids.iter().cloned()).await?;
            let candidates = ids.iter().filter_map(|id| self.records.get(id)).collect();
            let (record, span) = self.resolve_candidates(reference, scope, candidates)?;
            (record.id.clone(), span)
        };
        Ok((&self.records[&id], span))
    }
    pub async fn resolve_loaded(
        &mut self,
        reference: &str,
        scope: Option<&str>,
    ) -> Result<(&Record, [usize; 2])> {
        let (record, span) = self.resolve_metadata(reference, scope).await?;
        let id = record.id.clone();
        self.load_records([id.clone()]).await?;
        Ok((&self.records[&id], span))
    }
    pub(crate) async fn display_span(
        &self,
        rid: &str,
        span: [usize; 2],
        all: bool,
    ) -> Result<(String, usize)> {
        let record = &self.records[rid];
        if record.stored_body.is_none() {
            let text = String::from_utf8_lossy(&record.body.as_bytes()[span[0]..span[1]]);
            return Ok((
                if all {
                    text.to_string()
                } else {
                    text.chars().take(1500).collect()
                },
                text.chars().count(),
            ));
        }
        let len = span[1] - span[0];
        let mut rows = self.db.query(
            "SELECT substr(body,?2,?3),length(CAST(substr(body,?2,?4) AS TEXT)),substr(body,?5,1),instr(substr(body,?2,?4),x'00') FROM records WHERE id=?1",
            params![rid,span[0] as i64+1,if all { len } else { len.min(6000) } as i64,len as i64,span[1] as i64+1],
        ).await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("record body missing; reopen".into()))?;
        let bytes: Vec<u8> = row.get(0)?;
        let next: Vec<u8> = row.get(2)?;
        let continuation = |b: &u8| b & 0xc0 == 0x80;
        if row.get::<i64>(3)? != 0
            || bytes.first().is_some_and(continuation)
            || next.first().is_some_and(continuation)
        {
            // Arbitrary byte references may cut a scalar. Preserve Rust's lossy decoding.
            let mut rows = self
                .db
                .query(
                    "SELECT substr(body,?,?) FROM records WHERE id=?",
                    params![span[0] as i64 + 1, len as i64, rid],
                )
                .await?;
            let bytes = rows.next().await?.unwrap().get::<Vec<u8>>(0)?;
            let text = String::from_utf8_lossy(&bytes);
            return Ok((
                if all {
                    text.to_string()
                } else {
                    text.chars().take(1500).collect()
                },
                text.chars().count(),
            ));
        }
        let count = row.get::<i64>(1)? as usize;
        let text = String::from_utf8_lossy(&bytes);
        Ok((
            if all {
                text.to_string()
            } else {
                text.chars().take(1500).collect()
            },
            count,
        ))
    }
    pub fn block_count(&self) -> usize {
        self.block_count
    }
    async fn write_record_totals(&self, db: &Connection, stamp: &[u8]) -> Result<()> {
        db.execute("DELETE FROM record_index_totals", ()).await?;
        let statement = db
            .prepare("INSERT INTO record_index_totals VALUES(?,?,?)")
            .await?;
        for (rid, lookup) in &self.lookups {
            statement
                .execute(params![
                    rid.as_str(),
                    lookup.count as i64,
                    lookup.length as i64
                ])
                .await?;
            statement.reset();
        }
        db.execute(
            "INSERT OR REPLACE INTO projections VALUES('record-totals-v1',?)",
            [stamp.to_vec()],
        )
        .await?;
        Ok(())
    }
    async fn prepare_record_totals(&self) -> Result<()> {
        let mut rows = self
            .db
            .query(
                "SELECT value FROM projections WHERE key='record-totals-v1'",
                (),
            )
            .await?;
        if let Some(row) = rows.next().await?
            && row.get::<Vec<u8>>(0)? == self.structure_stamp
        {
            return Ok(());
        }
        drop(rows);
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        self.write_record_totals(&tx, &self.structure_stamp).await?;
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn load_block_headers(
        &mut self,
        ids: impl IntoIterator<Item = usize>,
    ) -> Result<()> {
        self.load_block_metadata(ids, true).await
    }
    async fn load_block_core(&mut self, ids: impl IntoIterator<Item = usize>) -> Result<()> {
        self.load_block_metadata(ids, false).await
    }
    async fn load_block_metadata(
        &mut self,
        ids: impl IntoIterator<Item = usize>,
        full: bool,
    ) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        let mut ids: Vec<_> = ids
            .into_iter()
            .filter(|id| {
                if full {
                    !self.loaded_block_headers.contains(id)
                } else {
                    !self.rows.contains_key(id)
                }
            })
            .collect();
        ids.sort_unstable();
        ids.dedup();
        for chunk in ids.chunks(256) {
            let presentation = if full {
                "b.heading,b.path"
            } else {
                "'',x'5b5d'"
            };
            let mut rows = self.db.query(&format!("SELECT b.id,b.record,b.ordinal,b.start,b.end,d.sz,r.body_hash,{presentation} FROM nav_blocks b JOIN fts_docsize d ON d.id=b.id JOIN record_headers r ON r.record=b.record WHERE b.id IN (SELECT value FROM json_each(?)) ORDER BY b.id"), [serde_json::to_string(chunk)?]).await?;
            let mut blocks = Vec::with_capacity(chunk.len());
            while let Some(row) = rows.next().await? {
                blocks.push(Block {
                    id: row.get::<i64>(0)? as usize,
                    record: row.get(1)?,
                    ordinal: row.get::<i64>(2)? as usize,
                    start: row.get::<i64>(3)? as usize,
                    end: row.get::<i64>(4)? as usize,
                    length: column_lengths(&row.get::<Vec<u8>>(5)?)?,
                    hash: row.get(6)?,
                    heading: row.get(7)?,
                    path: serde_json::from_slice(&row.get::<Vec<u8>>(8)?)?,
                    text: String::new(),
                    words: HashSet::new(),
                    surface: HashSet::new(),
                    tf: HashMap::new(),
                    body_positions: HashMap::new(),
                    stem_positions: HashMap::new(),
                });
            }
            drop(rows);
            self.load_record_headers(blocks.iter().map(|b| b.record.clone()))
                .await?;
            for mut block in blocks {
                if full {
                    self.loaded_block_headers.insert(block.id);
                }
                if let Some(existing) = self.rows.get_mut(&block.id) {
                    existing.heading = block.heading;
                    existing.path = block.path;
                    continue;
                }
                for (word, ids) in &self.indexes.block_surface {
                    if ids.binary_search(&block.id).is_ok() {
                        block.surface.insert(word.clone());
                    }
                }
                let lookup = self.lookups.get_mut(&block.record).unwrap();
                let at = lookup.rows.binary_search(&block.id).unwrap_err();
                lookup.rows.insert(at, block.id);
                self.rows.insert(block.id, block);
            }
            if chunk.iter().any(|id| !self.rows.contains_key(id)) {
                return Err(Error::Invalid("missing indexed block".into()));
            }
        }
        Ok(())
    }
    pub async fn load_record_blocks(&mut self, rid: &str) -> Result<()> {
        self.ensure_usable()?;
        if !self.structure_lazy {
            return Ok(());
        }
        self.load_record_block_headers(rid).await?;
        let ids: Vec<_> = self.record_blocks(rid).map(|b| b.id).collect();
        self.load_blocks(ids).await
    }
    pub(crate) async fn first_record_block(&mut self, rid: &str) -> Result<Option<usize>> {
        if !self.structure_lazy {
            return Ok(self
                .record_blocks(rid)
                .min_by_key(|b| b.start)
                .map(|b| b.id));
        }
        let mut rows = self
            .db
            .query(
                "SELECT id FROM nav_blocks WHERE record=? ORDER BY start,id LIMIT 1",
                [rid],
            )
            .await?;
        let id = rows
            .next()
            .await?
            .map(|r| r.get::<i64>(0))
            .transpose()?
            .map(|id| id as usize);
        self.load_block_headers(id).await?;
        Ok(id)
    }
    pub(crate) async fn load_record_block_headers(&mut self, rid: &str) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        self.load_record_headers([rid.to_owned()]).await?;
        if self
            .lookups
            .get(rid)
            .is_some_and(|r| r.rows.len() == r.count)
        {
            return Ok(());
        }
        let mut rows = self
            .db
            .query(
                "SELECT id FROM nav_blocks WHERE record=? ORDER BY id",
                [rid],
            )
            .await?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            ids.push(row.get::<i64>(0)? as usize);
        }
        self.load_block_headers(ids).await
    }
    async fn prepare_basename_index(&self) -> Result<()> {
        let mut rows = self
            .db
            .query("SELECT value FROM projections WHERE key='basenames-v1'", ())
            .await?;
        if let Some(row) = rows.next().await?
            && row.get::<Vec<u8>>(0)? == self.structure_stamp
        {
            return Ok(());
        }
        drop(rows);
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute("DELETE FROM record_basenames", ()).await?;
        let statement = tx
            .prepare("INSERT INTO record_basenames VALUES(?,?)")
            .await?;
        for record in self.records.values() {
            if let Some(path) = &record.source_path {
                let name = path.rsplit('/').next().unwrap_or("").to_lowercase();
                if !name.is_empty() {
                    statement.execute(params![name, record.id.as_str()]).await?;
                    statement.reset();
                }
            }
        }
        drop(statement);
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('basenames-v1',?)",
            [self.structure_stamp.clone()],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub(crate) async fn load_basenames(&mut self, names: &[String]) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        for name in names {
            if self.basenames.contains_key(name) {
                continue;
            }
            let mut rows = self
                .db
                .query(
                    "SELECT record FROM record_basenames WHERE name=? ORDER BY record",
                    [name.as_str()],
                )
                .await?;
            let mut records = HashSet::new();
            while let Some(row) = rows.next().await? {
                records.insert(row.get(0)?);
            }
            self.load_record_headers(records.iter().cloned()).await?;
            if !records.is_empty() {
                self.basenames.insert(name.clone(), records);
            }
        }
        Ok(())
    }
    /// Fetch immutable block presentation only for requested persisted ids.
    pub(crate) async fn load_blocks(&mut self, ids: impl IntoIterator<Item = usize>) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        let mut ids: Vec<_> = ids.into_iter().collect();
        self.load_block_headers(ids.iter().copied()).await?;
        ids.retain(|id| !self.loaded_blocks.contains(id));
        ids.sort_unstable();
        ids.dedup();
        for chunk in ids.chunks(256) {
            let encoded = serde_json::to_string(chunk)?;
            let mut result = self.db.query(
                "SELECT b.id,substr(r.body,b.start+1,b.end-b.start) FROM nav_blocks b JOIN records r ON r.id=b.record WHERE b.id IN (SELECT value FROM json_each(?)) ORDER BY b.id",
                [encoded],
            ).await?;
            while let Some(row) = result.next().await? {
                let id = row.get::<i64>(0)? as usize;
                let b = self
                    .rows
                    .get_mut(&id)
                    .ok_or_else(|| Error::Invalid("unknown projected block".into()))?;
                b.text = String::from_utf8(row.get::<Vec<u8>>(1)?)
                    .map_err(|_| Error::Invalid("invalid UTF-8 block body".into()))?;
                if b.text.len() != b.end - b.start {
                    return Err(Error::Invalid("invalid projected block span".into()));
                }
                self.loaded_blocks.insert(id);
            }
            if chunk.iter().any(|id| !self.loaded_blocks.contains(id)) {
                return Err(Error::Invalid(
                    "missing projected block; reopen store".into(),
                ));
            }
        }
        Ok(())
    }
    pub(crate) async fn load_pointer_blocks(&mut self, pointers: &[(String, usize)]) -> Result<()> {
        if !self.structure_lazy || pointers.is_empty() {
            return Ok(());
        }
        let mut rows = self.db.query(
            "SELECT DISTINCT (SELECT b.id FROM nav_blocks b WHERE b.record=json_extract(p.value,'$[0]') AND b.start<=json_extract(p.value,'$[1]') AND b.end>json_extract(p.value,'$[1]') ORDER BY b.id LIMIT 1) FROM json_each(?) p",
            [serde_json::to_string(pointers)?],
        ).await?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            if let Some(id) = row.get::<Option<i64>>(0)? {
                ids.push(id as usize);
            }
        }
        self.load_block_headers(ids).await
    }
    pub(crate) async fn load_record_sections(&mut self, rid: &str) -> Result<()> {
        self.load_record_headers([rid.to_owned()]).await?;
        if !self.structure_lazy || self.loaded_sections.contains(rid) {
            return Ok(());
        }
        let mut result = self
            .db
            .query(
                "SELECT data FROM nav_sections WHERE record=? ORDER BY ordinal",
                [rid],
            )
            .await?;
        let mut sections = Vec::new();
        while let Some(row) = result.next().await? {
            sections.push(serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?);
        }
        self.sections.insert(rid.to_owned(), sections);
        self.loaded_sections.insert(rid.to_owned());
        Ok(())
    }
    pub(crate) async fn linked_neighbors(
        &self,
        have: &HashSet<String>,
    ) -> Result<HashMap<String, HashSet<String>>> {
        let mut neighbors = HashMap::<String, HashSet<String>>::new();
        if !self.structure_lazy {
            // A live forget intentionally retains its previous resolved links.
            for rid in have {
                for target in self.links.get(rid).into_iter().flatten() {
                    neighbors
                        .entry(target.clone())
                        .or_default()
                        .insert(format!("linked-from:{rid}"));
                }
                for other in self.records.values().filter(|r| !have.contains(&r.id)) {
                    if self
                        .links
                        .get(&other.id)
                        .is_some_and(|links| links.contains(rid))
                    {
                        neighbors
                            .entry(other.id.clone())
                            .or_default()
                            .insert(format!("backlink:{rid}"));
                    }
                    if other.source_path.is_some()
                        && other.source_path == self.records[rid].source_path
                    {
                        neighbors
                            .entry(other.id.clone())
                            .or_default()
                            .insert(format!("evidence:{rid}"));
                    }
                }
            }
            return Ok(neighbors);
        }
        let encoded = serde_json::to_string(have)?;
        let mut rows = self.db.query(
            "WITH have(id) AS (SELECT value FROM json_each(?)) SELECT l.target,'linked-from:'||h.id FROM have h JOIN nav_links l ON l.source=h.id UNION ALL SELECT l.source,'backlink:'||h.id FROM have h JOIN nav_links l ON l.target=h.id WHERE l.source NOT IN (SELECT id FROM have) UNION ALL SELECT other.id,'evidence:'||h.id FROM have h JOIN record_headers r ON r.record=h.id JOIN records other ON json_extract(CAST(other.envelope AS TEXT),'$.source_path')=json_extract(CAST(r.data AS TEXT),'$.source_path') WHERE other.id NOT IN (SELECT id FROM have)",
            [encoded],
        ).await?;
        while let Some(row) = rows.next().await? {
            neighbors
                .entry(row.get(0)?)
                .or_default()
                .insert(row.get(1)?);
        }
        Ok(neighbors)
    }
    pub async fn materialize_structure(&mut self) -> Result<()> {
        self.ensure_usable()?;
        self.materialize_records().await?;
        self.load_records(self.records.keys().cloned().collect::<Vec<_>>())
            .await?;
        if !self.structure_lazy {
            return Ok(());
        }
        if self.structure_lazy {
            let mut rows = self
                .db
                .query(
                    "SELECT id,record,ordinal,own_nonempty FROM nav_sections ORDER BY id",
                    (),
                )
                .await?;
            while let Some(row) = rows.next().await? {
                let id = row.get::<i64>(0)? as usize;
                let rid = row.get::<String>(1)?;
                let ordinal = row.get::<i64>(2)? as usize;
                self.indexes.sections.insert(id, (rid.clone(), ordinal));
                self.lookups
                    .get_mut(&rid)
                    .unwrap()
                    .sections
                    .entry(ordinal)
                    .or_insert(SectionTokens {
                        own_nonempty: row.get::<i64>(3)? != 0,
                        ..SectionTokens::default()
                    });
            }
            let names: Vec<_> = self
                .records
                .values()
                .filter_map(|r| r.source_path.as_ref())
                .map(|path| path.rsplit('/').next().unwrap_or("").to_lowercase())
                .filter(|name| !name.is_empty())
                .collect();
            self.load_basenames(&names).await?;
        }

        if self.structure_lazy {
            let mut rows = self
                .db
                .query("SELECT kind,id,record FROM nav_trees ORDER BY kind,id", ())
                .await?;
            while let Some(row) = rows.next().await? {
                let kind = row.get::<i64>(0)?;
                let id = row.get::<i64>(1)? as usize;
                let tree = Tree {
                    record: row.get(2)?,
                    words: HashSet::new(),
                    rows: vec![],
                };
                if kind == 0 {
                    self.direct.entry(id).or_insert(tree);
                } else {
                    self.deep.entry(id).or_insert(tree);
                }
            }
        }

        self.links = self
            .records
            .keys()
            .map(|r| (r.clone(), HashSet::new()))
            .collect();
        let mut result = self
            .db
            .query("SELECT source,target FROM nav_links", ())
            .await?;
        while let Some(row) = result.next().await? {
            self.links
                .entry(row.get(0)?)
                .or_default()
                .insert(row.get(1)?);
        }

        if self.structure_lazy {
            let mut rows = self
                .db
                .query("SELECT id FROM nav_blocks ORDER BY id", ())
                .await?;
            let mut ids = Vec::new();
            while let Some(row) = rows.next().await? {
                ids.push(row.get::<i64>(0)? as usize);
            }
            self.load_blocks(ids).await?;
        }
        for rid in self.records.keys().cloned().collect::<Vec<_>>() {
            self.load_record_sections(&rid).await?;
        }
        Ok(())
    }
    pub(crate) async fn load_surface(&mut self, words: &[String]) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        let missing: Vec<_> = words
            .iter()
            .filter(|w| {
                !self.surface_words.contains(*w) && !self.indexes.block_surface.contains_key(*w)
            })
            .cloned()
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        let mut result = self.db.query(
            "SELECT n.surface,s.df,(SELECT json_group_array(block ORDER BY block) FROM nav_block_words WHERE word=n.id) FROM nav_lexemes n JOIN nav_surface s ON s.word=n.id WHERE n.surface IN (SELECT value FROM json_each(?))",
            [serde_json::to_string(&missing)?],
        ).await?;
        for word in &missing {
            self.indexes.block_surface.insert(word.clone(), Vec::new());
        }
        while let Some(row) = result.next().await? {
            let word: String = row.get(0)?;
            let ids: Vec<usize> = serde_json::from_str(&row.get::<String>(2)?)?;
            self.df.insert(word.clone(), row.get::<i64>(1)? as usize);
            for id in &ids {
                if let Some(block) = self.rows.get_mut(id) {
                    block.surface.insert(word.clone());
                }
            }
            self.indexes.block_surface.insert(word, ids);
        }
        self.surface_words.extend(missing);
        Ok(())
    }
    // @ref LLP 0001#persistent-structural-projections
    pub(crate) async fn load_surface_dictionary(&mut self) -> Result<()> {
        if !self.structure_lazy || !self.word_order.is_empty() {
            return Ok(());
        }
        let mut result = self.db.query("SELECT n.surface,s.df FROM nav_surface s JOIN nav_lexemes n ON n.id=s.word ORDER BY s.ord", ()).await?;
        while let Some(row) = result.next().await? {
            let word = row.get::<String>(0)?;
            self.word_order.push(word.clone());
            self.df.insert(word, row.get::<i64>(1)? as usize);
        }
        Ok(())
    }
    pub(crate) fn indexed_navigation(&self, q: &query::Query) -> bool {
        self.structure_lazy && q.filters.is_empty()
    }
    async fn navigation_hard_words_exist(&self, q: &query::Query) -> Result<bool> {
        if q.hard_words.is_empty() {
            return Ok(true);
        }
        let mut rows = self.db.query(
            "SELECT NOT EXISTS (SELECT 1 FROM json_each(?) h WHERE NOT EXISTS (SELECT 1 FROM nav_lexemes n WHERE n.norm=h.value))",
            [serde_json::to_string(&q.hard_words)?],
        ).await?;
        Ok(rows.next().await?.unwrap().get::<i64>(0)? != 0)
    }
    pub(crate) async fn load_search_navigation(&mut self, q: &query::Query) -> Result<()> {
        if !self.structure_lazy || !q.filters.is_empty() {
            return self.load_navigation(q).await;
        }
        // These trees are query-local winners, never a complete structural cache.
        self.direct.clear();
        self.deep.clear();
        self.indexes.direct.clear();
        self.indexes.deep.clear();
        self.navigation_words.clear();
        self.tree_rows.clear();
        if !self.navigation_hard_words_exist(q).await? {
            return Ok(());
        }
        let mut rows = self.db.query(
            "WITH words AS (
                SELECT n.norm,w.block FROM nav_lexemes n
                JOIN nav_block_words w ON w.word=n.id
                WHERE n.norm IN (SELECT value FROM json_each(?1))
                   OR n.norm IN (SELECT value FROM json_each(?2))
            ), matches AS (
                SELECT t.kind,t.tree,w.block,w.norm FROM words w
                JOIN nav_tree_rows t ON t.block=w.block
            ), tree_words AS (
                SELECT DISTINCT kind,tree,norm FROM matches
            ), qualified AS (
                SELECT kind,tree,json_group_array(norm) words FROM tree_words
                GROUP BY kind,tree
                HAVING count(*) FILTER (WHERE norm IN (SELECT value FROM json_each(?1)))>=?3
                   AND count(*) FILTER (WHERE norm IN (SELECT value FROM json_each(?2)))=?4
            ), block_counts AS (
                SELECT kind,tree,block,count(DISTINCT norm) hits,
                       json_group_array(DISTINCT norm) words FROM matches
                WHERE norm IN (SELECT value FROM json_each(?1))
                GROUP BY kind,tree,block
            ), winners AS (
                SELECT b.kind,b.tree,q.words,b.block,b.words block_words,
                       row_number() OVER (PARTITION BY b.kind,b.tree ORDER BY b.hits DESC,b.block) choice
                FROM block_counts b JOIN qualified q ON q.kind=b.kind AND q.tree=b.tree
            )
            SELECT w.kind,w.tree,n.record,w.words,w.block,w.block_words
            FROM winners w JOIN nav_trees n ON n.kind=w.kind AND n.id=w.tree
            JOIN record_read_index r ON r.record=n.record
            WHERE choice=1 AND (r.eligible&2)!=0 AND (r.expires='' OR r.expires>?5) ORDER BY w.kind,w.tree",
            params![serde_json::to_string(&q.words)?,serde_json::to_string(&q.hard_words)?,q.need.max(1) as i64,q.hard_words.iter().collect::<HashSet<_>>().len() as i64,self.query_clock.as_str()],
        ).await?;
        let mut selected = Vec::new();
        while let Some(row) = rows.next().await? {
            selected.push((
                row.get::<i64>(0)? != 0,
                row.get::<i64>(1)? as usize,
                row.get::<String>(2)?,
                serde_json::from_str::<HashSet<String>>(&row.get::<String>(3)?)?,
                row.get::<i64>(4)? as usize,
                serde_json::from_str::<HashSet<String>>(&row.get::<String>(5)?)?,
            ));
        }
        self.load_block_core(selected.iter().map(|r| r.4)).await?;
        for (deep, id, record, words, block, block_words) in selected {
            self.rows.get_mut(&block).unwrap().words.extend(block_words);
            let postings = if deep {
                &mut self.indexes.deep
            } else {
                &mut self.indexes.direct
            };
            for word in &words {
                postings.entry(word.clone()).or_default().push(id);
            }
            let trees = if deep {
                &mut self.deep
            } else {
                &mut self.direct
            };
            trees.insert(
                id,
                Tree {
                    record,
                    words,
                    rows: vec![block],
                },
            );
        }
        Ok(())
    }
    pub(crate) async fn selected_record_fill(
        &mut self,
        q: &query::Query,
        excluded: &HashSet<String>,
    ) -> Result<Option<usize>> {
        if !self.navigation_hard_words_exist(q).await? {
            return Ok(None);
        }
        let mut rows = self.db.query(
            "WITH words AS (SELECT DISTINCT n.norm,b.record FROM nav_lexemes n JOIN nav_block_words w ON w.word=n.id JOIN nav_blocks b ON b.id=w.block WHERE n.norm IN (SELECT value FROM json_each(?1)) OR n.norm IN (SELECT value FROM json_each(?2))), candidates AS (SELECT record,count(*) FILTER (WHERE norm IN (SELECT value FROM json_each(?1))) hits FROM words WHERE record NOT IN (SELECT value FROM json_each(?3)) GROUP BY record HAVING hits>=?4 AND count(*) FILTER (WHERE norm IN (SELECT value FROM json_each(?2)))=?5) SELECT c.record FROM candidates c JOIN record_read_index r ON r.record=c.record WHERE (r.eligible&1)!=0 AND (r.expires='' OR r.expires>?6) ORDER BY hits DESC,c.record DESC LIMIT 1",
            params![serde_json::to_string(&q.words)?,serde_json::to_string(&q.hard_words)?,serde_json::to_string(excluded)?,q.need.saturating_sub(1).max(1) as i64,q.hard_words.iter().collect::<HashSet<_>>().len() as i64,self.query_clock.as_str()],
        ).await?;
        let record = if let Some(row) = rows.next().await? {
            Some(row.get::<String>(0)?)
        } else if q.need <= 1 && q.hard_words.is_empty() {
            self.last_nonempty_record(excluded)
                .await?
                .map(|id| self.indexes.records[&id].clone())
        } else {
            None
        };
        let Some(record) = record else {
            return Ok(None);
        };
        let mut rows = self.db.query(
            "WITH hits AS (SELECT w.block,count(DISTINCT n.norm) hits FROM nav_lexemes n JOIN nav_block_words w ON w.word=n.id JOIN nav_blocks b ON b.id=w.block WHERE n.norm IN (SELECT value FROM json_each(?)) AND b.record=? GROUP BY w.block) SELECT b.id FROM nav_blocks b LEFT JOIN hits h ON h.block=b.id WHERE b.record=? ORDER BY coalesce(h.hits,0) DESC,b.id LIMIT 1",
            params![serde_json::to_string(&q.words)?,record.as_str(),record.as_str()],
        ).await?;
        Ok(rows
            .next()
            .await?
            .map(|r| r.get::<i64>(0))
            .transpose()?
            .map(|id| id as usize))
    }
    pub(crate) async fn load_navigation(&mut self, q: &query::Query) -> Result<()> {
        for (kind, source) in &q.filters {
            if kind == "replaces" {
                self.lifecycle_references(source).await?;
            }
        }
        if !self.structure_lazy {
            return Ok(());
        }
        self.load_record_headers(
            q.filters
                .iter()
                .filter(|(kind, _)| kind == "linked-from" || kind == "version-of")
                .map(|(_, id)| id.clone()),
        )
        .await?;
        for (_, source) in q.filters.iter().filter(|(kind, _)| kind == "linked-from") {
            if self.links.contains_key(source) {
                continue;
            }
            let mut result = self
                .db
                .query(
                    "SELECT target FROM nav_links WHERE source=?",
                    [source.as_str()],
                )
                .await?;
            let mut targets = HashSet::new();
            while let Some(row) = result.next().await? {
                targets.insert(row.get::<String>(0)?);
            }
            self.links.insert(source.clone(), targets);
        }
        let missing: HashSet<_> = q
            .words
            .iter()
            .chain(&q.hard_words)
            .filter(|w| {
                !self.navigation_words.contains(*w) && !self.indexes.record_words.contains_key(*w)
            })
            .cloned()
            .collect();
        if !missing.is_empty() {
            let encoded = serde_json::to_string(&missing)?;
            let mut result = self.db.query(
                "WITH matches AS (SELECT n.norm,json_group_array(DISTINCT w.block ORDER BY w.block) blocks,json_group_array(DISTINCT b.record) records FROM nav_lexemes n JOIN nav_block_words w ON w.word=n.id JOIN nav_blocks b ON b.id=w.block WHERE n.norm IN (SELECT value FROM json_each(?)) GROUP BY n.norm) SELECT norm,blocks,(SELECT json_group_array(ordinal ORDER BY ordinal) FROM record_read_index WHERE record IN (SELECT value FROM json_each(matches.records))) FROM matches ORDER BY norm",
                [encoded.clone()],
            ).await?;
            let mut groups = Vec::new();
            while let Some(row) = result.next().await? {
                groups.push((
                    row.get::<String>(0)?,
                    serde_json::from_str::<Vec<usize>>(&row.get::<String>(1)?)?,
                    serde_json::from_str::<Vec<usize>>(&row.get::<String>(2)?)?,
                ));
            }
            self.load_block_headers(groups.iter().flat_map(|g| g.1.iter().copied()))
                .await?;
            for word in &missing {
                self.indexes.record_words.insert(word.clone(), Vec::new());
                self.indexes.direct.insert(word.clone(), Vec::new());
                self.indexes.deep.insert(word.clone(), Vec::new());
            }
            for (word, blocks, records) in groups {
                self.indexes.record_words.insert(word.clone(), records);
                for id in blocks {
                    let block = self.rows.get_mut(&id).unwrap();
                    block.words.insert(word.clone());
                    self.lookups
                        .get_mut(&block.record)
                        .unwrap()
                        .words
                        .insert(word.clone());
                }
            }
            let mut result = self.db.query(
                "SELECT DISTINCT n.norm,t.kind,t.tree,nr.record FROM nav_lexemes n JOIN nav_block_words b ON b.word=n.id JOIN nav_tree_rows t ON t.block=b.block JOIN nav_trees nr ON nr.kind=t.kind AND nr.id=t.tree WHERE n.norm IN (SELECT value FROM json_each(?)) ORDER BY n.norm,t.kind,t.tree",
                [encoded],
            ).await?;
            while let Some(row) = result.next().await? {
                let word: String = row.get(0)?;
                let deep = row.get::<i64>(1)? != 0;
                let id = row.get::<i64>(2)? as usize;
                let trees = if deep {
                    &mut self.deep
                } else {
                    &mut self.direct
                };
                trees
                    .entry(id)
                    .or_insert(Tree {
                        record: row.get(3)?,
                        words: HashSet::new(),
                        rows: Vec::new(),
                    })
                    .words
                    .insert(word.clone());
                let postings = if deep {
                    &mut self.indexes.deep
                } else {
                    &mut self.indexes.direct
                };
                postings.get_mut(&word).unwrap().push(id);
            }
            self.navigation_words.extend(missing);
        }
        let mut requested = Vec::new();
        for deep in [false, true] {
            let postings = if deep {
                &self.indexes.deep
            } else {
                &self.indexes.direct
            };
            requested.extend(
                posting_candidates(postings, q.words.iter().map(String::as_str), q.need)
                    .into_iter()
                    .filter(|id| !self.tree_rows.contains(&(deep, *id)))
                    .map(|id| (i64::from(deep), id)),
            );
        }
        if !requested.is_empty() {
            let mut result = self.db.query(
                "SELECT t.kind,t.tree,t.block FROM json_each(?) q JOIN nav_tree_rows t ON t.kind=json_extract(q.value,'$[0]') AND t.tree=json_extract(q.value,'$[1]') ORDER BY t.kind,t.tree,t.block",
                [serde_json::to_string(&requested)?],
            ).await?;
            let mut blocks = Vec::new();
            while let Some(row) = result.next().await? {
                let deep = row.get::<i64>(0)? != 0;
                let id = row.get::<i64>(1)? as usize;
                let block = row.get::<i64>(2)? as usize;
                let trees = if deep {
                    &mut self.deep
                } else {
                    &mut self.direct
                };
                trees.get_mut(&id).unwrap().rows.push(block);
                blocks.push(block);
            }
            self.load_block_headers(blocks).await?;
            self.tree_rows
                .extend(requested.into_iter().map(|(kind, id)| (kind != 0, id)));
        }
        Ok(())
    }
    pub(crate) async fn load_section_words(&mut self, words: &[String]) -> Result<()> {
        if !self.structure_lazy {
            return Ok(());
        }
        for word in words {
            if self.section_words.contains(word) || self.indexes.related.contains_key(word) {
                continue;
            }
            let mut result = self.db.query("SELECT s.section,s.flags,m.record,m.ordinal,m.own_nonempty FROM nav_lexemes n JOIN nav_section_words s ON s.word=n.id JOIN nav_sections m ON m.id=s.section WHERE n.surface=? ORDER BY s.section",[word.as_str()]).await?;
            let mut own = Vec::new();
            let mut related = Vec::new();
            while let Some(row) = result.next().await? {
                let id = row.get::<i64>(0)? as usize;
                let flags = row.get::<i64>(1)?;
                let rid = row.get::<String>(2)?;
                let ordinal = row.get::<i64>(3)? as usize;
                self.indexes.sections.insert(id, (rid.clone(), ordinal));
                let tokens = self
                    .lookups
                    .entry(rid.clone())
                    .or_default()
                    .sections
                    .entry(ordinal)
                    .or_insert(SectionTokens {
                        own_nonempty: row.get::<i64>(4)? != 0,
                        ..SectionTokens::default()
                    });
                if flags & 1 != 0 {
                    tokens.own.insert(word.clone());
                    own.push(id);
                }
                if flags & 2 != 0 {
                    tokens.extra.insert(word.clone());
                }
                related.push(id);
            }
            self.indexes.own.insert(word.clone(), own);
            self.indexes.related.insert(word.clone(), related);
            self.section_words.insert(word.clone());
        }
        Ok(())
    }
    fn materialize_block_words(&mut self) {
        if !self.structure_lazy {
            return;
        }
        for block in self.rows.values_mut() {
            let words = query::surface(&format!("{} {}", block.heading, block.text));
            block.words = words.iter().map(|w| query::norm(w)).collect();
            block.surface = words.into_iter().collect();
        }
        self.structure_lazy = false;
    }
    pub async fn build(path: &Path, records: Vec<Record>) -> Result<Self> {
        let mut st = Self::open(path).await?;
        let mut forgotten = HashSet::new();
        let mut tombstones = st.db.query("SELECT id FROM tombstones", ()).await?;
        while let Some(row) = tombstones.next().await? {
            forgotten.insert(row.get::<String>(0)?);
        }
        for r in records {
            if !forgotten.contains(&r.id) {
                st.records.insert(r.id.clone(), r);
            }
        }
        st.rebuild().await?;
        Ok(st)
    }
    pub async fn add(&mut self, r: Record) -> Result<()> {
        self.ensure_usable()?;
        if self.records.contains_key(&r.id) || self.contains_record(&r.id).await? {
            return Err(Error::Invalid(format!("record {} already exists", r.id)));
        }
        let mut t = self
            .db
            .query("SELECT 1 FROM tombstones WHERE id=?", [r.id.clone()])
            .await?;
        if t.next().await?.is_some() {
            return Err(Error::Invalid("record is forgotten".into()));
        }
        self.records.insert(r.id.clone(), r);
        self.rebuild().await
    }
    fn make_rows(&mut self) {
        self.structure_lazy = false;
        self.rows.clear();
        self.sections.clear();
        let mut id = 1;
        for r in self.records.values() {
            let sections = sections::parse_sections(&r.body);
            let bh = r.body_hash();
            for s in &sections {
                for span in sections::block_spans(&r.body, s.span.clone(), 1400) {
                    let text = r.body[span.clone()].to_owned();
                    let blob = format!("{} {}", s.heading, text);
                    let surface = query::surface(&blob);
                    self.rows.insert(
                        id,
                        Block {
                            id,
                            record: r.id.clone(),
                            hash: bh.clone(),
                            ordinal: s.ordinal,
                            heading: s.heading.clone(),
                            path: s.path.clone(),
                            start: span.start,
                            end: span.end,
                            text,
                            words: surface.iter().map(|word| query::norm(word)).collect(),
                            surface: surface.into_iter().collect(),
                            tf: HashMap::new(),
                            length: 0,
                            body_positions: HashMap::new(),
                            stem_positions: HashMap::new(),
                        },
                    );
                    id += 1;
                }
            }
            self.sections.insert(r.id.clone(), sections);
        }
    }
    pub async fn rebuild(&mut self) -> Result<()> {
        let previous = self.begin_write_cache().await?;
        let result = self.rebuild_inner().await;
        let result = self.finish_write_cache(previous, result).await;
        self.write_outcome(result)
    }
    // @ref LLP 0001#connection-policy
    async fn begin_write_cache(&self) -> Result<i64> {
        self.ensure_usable()?;
        let mut rows = self.db.query("PRAGMA cache_size", ()).await?;
        let previous = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("missing pager cache setting".into()))?
            .get::<i64>(0)?;
        drop(rows);
        self.db.execute_batch("PRAGMA cache_size=-32768").await?;
        Ok(previous)
    }
    async fn finish_write_cache(&self, previous: i64, result: Result<()>) -> Result<()> {
        let restored = self
            .db
            .execute_batch(&format!(
                "PRAGMA cache_size={previous}; PRAGMA shrink_memory;"
            ))
            .await;
        match (result, restored) {
            (result, Ok(_)) => result,
            (Ok(()), Err(error)) => Err(Error::Invalid(format!(
                "write completed, but restoring the pager cache failed: {error}"
            ))),
            (Err(write), Err(restore)) => Err(Error::Invalid(format!(
                "{write}; restoring the pager cache also failed: {restore}"
            ))),
        }
    }
    // @ref LLP 0001#original-bytes
    async fn validate_rebuild_records(&mut self) -> Result<()> {
        let mut rows = self
            .db
            .query("SELECT envelope FROM records ORDER BY id", ())
            .await?;
        while let Some(row) = rows.next().await? {
            let original: Record = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
            if let Some(candidate) = self.records.get(&original.id)
                && serde_json::to_vec(candidate)? != serde_json::to_vec(&original)?
            {
                return Err(Error::Invalid(format!(
                    "immutable record conflict: {}",
                    original.id
                )));
            }
            // Original bytes remain authoritative even if a caller removed a resident entry.
            self.records.insert(original.id.clone(), original);
        }
        Ok(())
    }
    async fn rebuild_inner(&mut self) -> Result<()> {
        self.materialize_records().await?;
        self.load_records(self.records.keys().cloned().collect::<Vec<_>>())
            .await?;
        self.validate_rebuild_records().await?;
        self.make_rows();
        self.block_count = self.rows.len();
        self.validity = Validity::project(&self.records.values().cloned().collect::<Vec<_>>());
        self.vectors.clear();
        self.vectors_loaded = true;
        let mut missing = Vec::new();
        let mut ids = Vec::new();
        for (&sid, b) in &self.rows {
            ids.push(sid);
            missing.push(b.text.clone());
        }
        if !missing.is_empty() {
            let vectors = self.encoder()?.batch(&missing);
            for (id, v) in ids.into_iter().zip(vectors) {
                self.vectors.insert(id, v);
            }
        }
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute("DELETE FROM sections", ()).await?;

        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('validity',?)",
            [serde_json::to_vec(&self.validity)?],
        )
        .await?;
        for r in self.records.values() {
            let envelope = serde_json::to_vec(r)?;
            tx.execute(
                "INSERT INTO records VALUES(?,NULL,?,?,?,?,?,?) ON CONFLICT(id) DO NOTHING",
                params![
                    r.id.clone(),
                    r.scope.clone(),
                    r.state.clone(),
                    r.body.as_bytes().to_vec(),
                    r.body_hash(),
                    envelope.clone(),
                    hash(&envelope)
                ],
            )
            .await?;
        }
        for b in self.rows.values() {
            let section = &self.sections[&b.record][b.ordinal];
            let heading = if b.start == section.span.start {
                b.heading.as_str()
            } else {
                ""
            };
            tx.execute(
                "INSERT INTO sections VALUES(?,?,?,?,?,?,?)",
                params![
                    b.id as i64,
                    b.record.clone(),
                    heading,
                    b.text.clone(),
                    sections::pieces(&b.text),
                    b.start as i64,
                    b.end as i64
                ],
            )
            .await?;
            tx.execute(
                "INSERT INTO stem(rowid,text) VALUES(?,?)",
                params![b.id as i64, format!("{heading}\n{}", b.text)],
            )
            .await?;
            let bytes: Vec<u8> = self.vectors[&b.id]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            tx.execute(
                "INSERT INTO vectors VALUES(?,?,?)",
                params![b.id as i64, b.record.clone(), bytes],
            )
            .await?;
        }
        tx.execute("INSERT INTO fts(fts) VALUES('optimize')", ())
            .await?;
        tx.execute(
            "INSERT OR REPLACE INTO projections VALUES('block-model-v1',?)",
            [crate::language::embedding::ARTIFACT_MODEL.as_bytes()],
        )
        .await?;
        tx.execute("INSERT OR REPLACE INTO sync_state VALUES('pending',1)", ())
            .await?;
        tx.commit().await?;
        self.hits.clear();
        self.eligible_hits.clear();
        self.rankings.clear();
        self.confidences.clear();
        self.form_cache = None;
        self.beam_cache = None;
        self.statistics().await?;
        self.navigation();
        self.persist_structure().await?;
        self.prepare_fallback_artifacts().await?;
        self.publish_pos().await?;
        self.publish_catalog().await?;
        self.complete_publication().await
    }
    async fn statistics(&mut self) -> Result<()> {
        self.statistics_timed(&mut StartupTiming::new()).await
    }
    async fn statistics_timed(&mut self, timing: &mut StartupTiming) -> Result<()> {
        self.memo_query = None;
        self.porter_cache.clear();
        self.porter_probes.clear();
        self.stat_misses.clear();
        self.coverage.clear();
        self.stats.clear();
        self.sentence_cache = None;
        self.vector_matrix = None;
        self.df.clear();
        self.word_order.clear();
        self.vocab.clear();
        self.vocabulary_probes.clear();
        self.vocabulary_loaded = false;
        self.loaded_terms.clear();
        self.candidate_terms.clear();
        self.loaded_stems.clear();
        for b in self.rows.values_mut() {
            b.tf.clear();
            b.length = 0;
            b.body_positions.clear();
            b.stem_positions.clear();
        }
        timing.mark("fts-vocabulary-deferred");
        if self.structure_lazy {
            timing.mark("fts-column-lengths-deferred");
            return Ok(());
        }
        let mut result = self.db.query("SELECT id,sz FROM fts_docsize", ()).await?;
        while let Some(row) = result.next().await? {
            if let Some(b) = self.rows.get_mut(&(row.get::<i64>(0)? as usize)) {
                b.length = column_lengths(&row.get::<Vec<u8>>(1)?)?;
            }
        }
        timing.mark("fts-column-lengths");
        if !self.structure_lazy {
            for b in self.rows.values() {
                let mut seen = HashSet::new();
                for w in query::surface(&format!("{} {}", b.heading, b.text)) {
                    if seen.insert(w.clone()) {
                        if !self.df.contains_key(&w) {
                            self.word_order.push(w.clone());
                        }
                        *self.df.entry(w).or_default() += 1;
                    }
                }
            }
        }
        timing.mark(if self.structure_lazy {
            "surface-df-deferred"
        } else {
            "surface-df-tokenize"
        });
        Ok(())
    }
    #[cfg(test)]
    pub(crate) async fn load_terms(
        &mut self,
        words: impl IntoIterator<Item = String>,
        stem: bool,
    ) -> Result<()> {
        self.read_term_features(words, stem, true, None).await
    }
    pub(crate) async fn load_candidate_terms(
        &mut self,
        words: impl IntoIterator<Item = String>,
        stem: bool,
        ids: &[usize],
    ) -> Result<()> {
        self.read_term_features(words, stem, true, Some(ids)).await
    }
    pub(crate) async fn load_candidate_frequencies(
        &mut self,
        words: impl IntoIterator<Item = String>,
        ids: &[usize],
    ) -> Result<()> {
        self.read_term_features(words, false, false, Some(ids))
            .await
    }
    async fn read_term_features(
        &mut self,
        words: impl IntoIterator<Item = String>,
        stem: bool,
        with_positions: bool,
        ids: Option<&[usize]>,
    ) -> Result<()> {
        let mut words: Vec<_> = words.into_iter().collect();
        words.sort();
        words.dedup();
        let key = (stem, with_positions, words.clone());
        let missing = ids.map(|ids| {
            ids.iter()
                .copied()
                .filter(|id| {
                    !self
                        .candidate_terms
                        .get(&key)
                        .is_some_and(|seen| seen.contains(id))
                })
                .collect::<Vec<_>>()
        });
        if missing.as_ref().is_some_and(Vec::is_empty) {
            return Ok(());
        }
        if let Some(ids) = &missing {
            self.load_block_core(ids.iter().copied()).await?;
            self.db
                .execute("DELETE FROM temp.feature_candidates", ())
                .await?;
            self.db
                .execute(
                    "INSERT OR IGNORE INTO temp.feature_candidates SELECT value FROM json_each(?)",
                    [serde_json::to_string(ids)?],
                )
                .await?;
        }
        for word in words {
            if if stem {
                self.loaded_stems.contains(&word)
            } else {
                self.loaded_terms.contains(&word)
            } {
                continue;
            }
            let table = if stem { "stem_vocab" } else { "vocab" };
            let positions = if !with_positions {
                "'[]'"
            } else if stem {
                "json_group_array(offset ORDER BY offset)"
            } else {
                "json_group_array(offset ORDER BY offset) FILTER (WHERE col='body')"
            };
            let filter = if missing.is_some() {
                " AND doc IN (SELECT id FROM temp.feature_candidates)"
            } else {
                ""
            };
            let sql = format!(
                "SELECT doc,col,count(*),{positions} FROM {table} WHERE term=?{filter} GROUP BY doc,col"
            );
            let mut result = self.db.query(&sql, [word.clone()]).await?;
            let mut occurrences = Vec::new();
            while let Some(row) = result.next().await? {
                occurrences.push((
                    row.get::<i64>(0)? as usize,
                    row.get::<String>(1)?,
                    row.get::<i64>(2)? as f64,
                    serde_json::from_str::<Vec<usize>>(&row.get::<String>(3)?)?,
                ));
            }
            if missing.is_none() {
                self.load_block_headers(occurrences.iter().map(|r| r.0))
                    .await?;
            }
            for (id, col, count, offsets) in occurrences {
                if let Some(block) = self.rows.get_mut(&id) {
                    if stem {
                        block.stem_positions.insert(word.clone(), offsets);
                    } else {
                        if with_positions && col == "body" {
                            block.body_positions.insert(word.clone(), offsets);
                        }
                        let col = match col.as_str() {
                            "heading" => 0,
                            "body" => 1,
                            _ => 2,
                        };
                        block.tf.entry(word.clone()).or_default()[col] = count;
                    }
                }
            }
            if missing.is_none() {
                if stem {
                    self.loaded_stems.insert(word);
                } else {
                    self.loaded_terms.insert(word);
                }
            }
        }
        if let Some(ids) = missing {
            self.candidate_terms.entry(key).or_default().extend(ids);
        }
        Ok(())
    }
    pub(crate) fn record_eligible(&self, rid: &str) -> bool {
        self.records
            .get(rid)
            .is_some_and(|r| self.validity.eligible(r, &self.query_clock))
    }
    pub fn hint_eligible(&self, rid: &str) -> bool {
        self.records
            .get(rid)
            .is_some_and(|r| self.validity.eligible(r, &self.query_clock) && r.body_nonblank())
    }
    pub fn tree_eligible(&self, r: &Record) -> bool {
        self.validity.eligible(r, "") && r.body_nonblank()
    }
    // @ref LLP 0007#store-publication-and-synchronization
    pub(crate) fn begin_query(&mut self, query: &str, ctx: &Context) {
        if memo_over_budget(
            self.hits
                .iter()
                .map(|((_, text), ids)| (text.capacity(), ids.capacity())),
        ) {
            self.hits = HashMap::new();
        }
        if memo_over_budget(self.eligible_hits.iter().map(|((_, text, clock), ids)| {
            (
                text.capacity().saturating_add(clock.capacity()),
                ids.capacity(),
            )
        })) {
            self.eligible_hits = HashMap::new();
        }
        if memo_over_budget(self.candidate_terms.iter().map(|((_, _, words), ids)| {
            (
                words
                    .capacity()
                    .saturating_mul(std::mem::size_of::<String>())
                    .saturating_add(words.iter().map(String::capacity).sum::<usize>()),
                ids.capacity(),
            )
        })) {
            self.candidate_terms = HashMap::new();
        }
        if self
            .memo_query
            .as_ref()
            .is_none_or(|(q, scope)| q != query || scope != &ctx.scope)
        {
            for word in self.porter_probes.drain() {
                if !self.df.contains_key(&word) && !self.vocab.contains(&word) {
                    self.porter_cache.remove(&word);
                }
            }
            if let Some(stats) = self.stats.get_mut(&self.query_clock) {
                let stats = Arc::make_mut(stats);
                for word in self.stat_misses.drain() {
                    stats.df.remove(&word);
                }
            } else {
                self.stat_misses.clear();
            }
            self.coverage.clear();
            self.rankings.clear();
            self.confidences.clear();
            // Drop negative probes from the previous query without traversing
            // the publication's positive posting lists or materialized rows.
            for word in self.surface_words.drain() {
                if self
                    .indexes
                    .block_surface
                    .get(&word)
                    .is_some_and(Vec::is_empty)
                {
                    self.indexes.block_surface.remove(&word);
                }
            }
            for word in self.navigation_words.drain() {
                for postings in [
                    &mut self.indexes.record_words,
                    &mut self.indexes.direct,
                    &mut self.indexes.deep,
                ] {
                    if postings.get(&word).is_some_and(Vec::is_empty) {
                        postings.remove(&word);
                    }
                }
            }
            for word in self.section_words.drain() {
                for postings in [&mut self.indexes.own, &mut self.indexes.related] {
                    if postings.get(&word).is_some_and(Vec::is_empty) {
                        postings.remove(&word);
                    }
                }
            }
            self.vocabulary_probes.clear();
            self.memo_query = Some((query.into(), ctx.scope.clone()));
        }
        self.set_query_clock(&ctx.now);
    }
    pub(crate) fn set_query_clock(&mut self, now: &str) {
        if self.query_clock != now {
            self.query_clock = now.to_owned();
            self.eligible_hits.clear();
            self.stats.clear();
            self.stat_misses.clear();
            self.coverage.clear();
            self.rankings.clear();
            self.confidences.clear();
        }
    }
    // @ref LLP 0001#derived-lookups
    fn rebuild_lookups(&mut self) {
        self.lookups.clear();
        self.indexes = Indexes::default();
        for (rid, sections) in &self.sections {
            let record = &self.records[rid];
            let tokens = sections
                .iter()
                .map(|s| {
                    let own: HashSet<_> =
                        query::surface(&format!("{} {}", s.heading, &record.body[s.span.clone()]))
                            .into_iter()
                            .collect();
                    (
                        s.ordinal,
                        SectionTokens {
                            own_nonempty: !own.is_empty(),
                            own,
                            extra: query::surface(&format!(
                                "{} {}",
                                s.path.join(" "),
                                record.label
                            ))
                            .into_iter()
                            .collect(),
                        },
                    )
                })
                .collect();
            self.lookups.insert(
                rid.clone(),
                RecordLookup {
                    sections: tokens,
                    ..RecordLookup::default()
                },
            );
        }
        // The BTreeMap traversal preserves the original scan's row order, including gaps.
        for (&id, block) in &self.rows {
            let lookup = self.lookups.get_mut(&block.record).unwrap();
            lookup.rows.push(id);
            lookup.count += 1;
            lookup.words.extend(block.words.iter().cloned());
            lookup.length += block.length;
            for word in &block.surface {
                self.indexes
                    .block_surface
                    .entry(word.clone())
                    .or_default()
                    .push(id);
            }
        }
        for (rid, sections) in &self.sections {
            let lookup = &self.lookups[rid];
            for section in sections {
                let id = self.indexes.sections.len();
                self.indexes
                    .sections
                    .insert(id, (rid.clone(), section.ordinal));
                let tokens = &lookup.sections[&section.ordinal];
                for word in &tokens.own {
                    self.indexes.own.entry(word.clone()).or_default().push(id);
                }
                for word in tokens.own.union(&tokens.extra) {
                    self.indexes
                        .related
                        .entry(word.clone())
                        .or_default()
                        .push(id);
                }
            }
        }
        self.indexes.records = self.records.keys().cloned().enumerate().collect();
        self.indexes.record_positions = self
            .indexes
            .records
            .iter()
            .map(|(i, r)| (r.clone(), *i))
            .collect();
        for (&id, rid) in &self.indexes.records {
            let lookup = &self.lookups[rid];
            for word in &lookup.words {
                self.indexes
                    .record_words
                    .entry(word.clone())
                    .or_default()
                    .push(id);
            }
        }
    }
    /// Blocks of a record in ascending persisted id order, including gaps after forget.
    pub fn record_blocks(&self, rid: &str) -> impl Iterator<Item = &Block> {
        self.lookups
            .get(rid)
            .into_iter()
            .flat_map(|lookup| &lookup.rows)
            .map(|id| &self.rows[id])
    }
    pub(crate) fn rare_blocks(&self, words: &[String], threshold: f64) -> HashSet<usize> {
        words
            .iter()
            .filter(|word| self.idf(word) >= threshold)
            .flat_map(|word| {
                self.indexes
                    .block_surface
                    .get(word)
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect()
    }
    fn navigation(&mut self) {
        self.navigation_timed(&mut StartupTiming::new());
    }
    fn navigation_timed(&mut self, timing: &mut StartupTiming) {
        self.rebuild_lookups();
        timing.mark("derived-postings-tokenize");
        self.direct.clear();
        self.deep.clear();
        self.basenames.clear();
        self.links.clear();
        let link = regex::Regex::new(r"\]\(([^)]+)\)").unwrap();
        let mut by_path = HashMap::<&str, Vec<&str>>::new();
        for record in self.records.values() {
            if let Some(path) = record.source_path.as_deref() {
                by_path.entry(path).or_default().push(&record.id);
            }
        }
        for r in self.records.values() {
            if let Some(path) = &r.source_path {
                let name = path.rsplit('/').next().unwrap_or("").to_lowercase();
                if !name.is_empty() {
                    self.basenames.entry(name).or_default().insert(r.id.clone());
                }
            }
            let dests: Vec<_> = link
                .captures_iter(&r.body)
                .map(|c| c[1].to_owned())
                .collect();
            let dir = Path::new(r.source_path.as_deref().unwrap_or(""))
                .parent()
                .unwrap_or(Path::new(""));
            let paths: HashSet<_> = dests
                .iter()
                .map(|d| normalize_path(&dir.join(d.split('#').next().unwrap_or(""))))
                .collect();
            let targets = dests
                .iter()
                .filter(|id| self.records.contains_key(*id))
                .cloned()
                .chain(
                    paths
                        .iter()
                        .flat_map(|path| by_path.get(path.as_str()).into_iter().flatten())
                        .map(|id| (*id).to_owned()),
                )
                .collect();
            self.links.insert(r.id.clone(), targets);
            if !self.tree_eligible(r) {
                continue;
            }
            let rows: Vec<_> = self.lookups[&r.id]
                .rows
                .iter()
                .map(|id| &self.rows[id])
                .collect();
            let mut paths = Vec::new();
            let mut seen = HashSet::new();
            let mut descendants = HashMap::<&[String], Vec<&Block>>::new();
            let mut children = HashMap::<&[String], Vec<&Block>>::new();
            for b in &rows {
                if seen.insert(&b.path) {
                    paths.push(b.path.clone());
                }
                for length in 0..=b.path.len() {
                    descendants.entry(&b.path[..length]).or_default().push(b);
                    if b.path.len() <= length + 1 {
                        children.entry(&b.path[..length]).or_default().push(b);
                    }
                }
            }
            for path in paths {
                for deep in [false, true] {
                    let matching = if deep {
                        &descendants[&path[..]]
                    } else {
                        &children[&path[..]]
                    };
                    let tree = Tree {
                        record: r.id.clone(),
                        words: matching
                            .iter()
                            .flat_map(|b| b.words.iter().cloned())
                            .collect(),
                        rows: matching.iter().map(|b| b.id).collect(),
                    };
                    if deep {
                        self.deep.insert(self.deep.len(), tree);
                    } else {
                        self.direct.insert(self.direct.len(), tree);
                    }
                }
            }
        }
        for (trees, postings) in [
            (&self.direct, &mut self.indexes.direct),
            (&self.deep, &mut self.indexes.deep),
        ] {
            for (&id, tree) in trees {
                for word in &tree.words {
                    postings.entry(word.clone()).or_default().push(id);
                }
            }
        }
        timing.mark("links-trees-postings");
    }
    pub fn card(&self, id: usize, center: Option<&query::Query>) -> Card {
        let b = &self.rows[&id];
        let start = center
            .and_then(|q| {
                q.terms
                    .iter()
                    .filter_map(|t| {
                        regex::RegexBuilder::new(&regex::escape(&t.text))
                            .case_insensitive(true)
                            .build()
                            .ok()
                            .and_then(|re| re.find(&b.text).map(|m| m.start()))
                    })
                    .min()
            })
            .map(|p| b.text[..p].chars().count().saturating_sub(50))
            .unwrap_or(0);
        Card {
            record: b.record.clone(),
            heading: b.path.join(" > "),
            span: [b.start, b.end],
            section: id,
            text: b.text.chars().skip(start).take(180).collect(),
            labels: Vec::new(),
            fetch: center.map(|_| format!("{}:v1:{}:{}", b.hash, b.start, b.end)),
        }
    }
    pub(crate) async fn prepare_porter(
        &mut self,
        words: impl IntoIterator<Item = String>,
    ) -> Result<()> {
        let mut words: Vec<_> = words
            .into_iter()
            .map(|w| w.to_lowercase())
            .filter(|w| !self.porter_cache.contains_key(w))
            .collect();
        words.sort();
        words.dedup();
        if words.is_empty() {
            return Ok(());
        }
        for word in words {
            let stem = crate::nlp::porter::word_lowercased(&word)?;
            self.porter_probes.insert(word.clone());
            self.porter_cache.insert(word, stem);
        }
        Ok(())
    }
    pub async fn porter(&mut self, w: &str) -> Result<String> {
        let w = w.to_lowercase();
        if let Some(s) = self.porter_cache.get(&w) {
            return Ok(s.clone());
        }
        let stem = crate::nlp::porter::word_lowercased(&w)?;
        self.porter_probes.insert(w.clone());
        self.porter_cache.insert(w, stem.clone());
        Ok(stem)
    }
    pub(crate) async fn index_stems(&self, text: &str) -> Result<Vec<String>> {
        Ok(crate::nlp::porter::index_stems(text)?)
    }

    pub async fn fts(&mut self, stem: bool, expression: &str) -> Result<HashSet<usize>> {
        self.ensure_usable()?;
        let hits = self.fts_ids(stem, expression).await?;
        self.load_block_headers(hits.iter().copied()).await?;
        Ok(hits)
    }
    pub(crate) async fn eligible_fts_hits(
        &mut self,
        stem: bool,
        expression: &str,
        now: &str,
    ) -> Result<HashSet<usize>> {
        let key = (stem, expression.to_owned(), now.to_owned());
        if let Some(ids) = self.eligible_hits.get(&key) {
            return Ok(ids.clone());
        }
        let table = if stem { "stem" } else { "fts" };
        let mut rows=self.db.query(
            &format!("SELECT {table}.rowid FROM {table} JOIN nav_blocks b ON b.id={table}.rowid JOIN record_read_index c ON c.record=b.record WHERE {table} MATCH ? AND (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>?)"),
            params![expression,now],
        ).await?;
        let mut ids = HashSet::new();
        while let Some(row) = rows.next().await? {
            ids.insert(row.get::<i64>(0)? as usize);
        }
        self.eligible_hits.insert(key, ids.clone());
        Ok(ids)
    }
    pub(crate) async fn eligible_hit_ids(
        &self,
        ids: &HashSet<usize>,
        now: &str,
        records: Option<&HashSet<String>>,
    ) -> Result<HashSet<usize>> {
        if ids.is_empty() || records.is_some_and(HashSet::is_empty) {
            return Ok(HashSet::new());
        }
        let mut rows = self.db.query(
            "SELECT b.id FROM json_each(?1) x JOIN nav_blocks b ON b.id=x.value JOIN record_read_index c ON c.record=b.record WHERE (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>?2) AND (?3 IS NULL OR b.record IN (SELECT value FROM json_each(?3)))",
            params![serde_json::to_string(ids)?,now,records.map(serde_json::to_string).transpose()?],
        ).await?;
        let mut result = HashSet::new();
        while let Some(row) = rows.next().await? {
            result.insert(row.get::<i64>(0)? as usize);
        }
        Ok(result)
    }
    /// Apply writer selection in SQL before lexical block headers are materialized.
    pub(crate) async fn writer_fts_ids(
        &mut self,
        stem: bool,
        expression: &str,
        q: &query::Query,
    ) -> Result<HashSet<usize>> {
        let mut writers = q
            .filters
            .iter()
            .filter(|(k, _)| k == "writer")
            .map(|(_, v)| v.as_str());
        let Some(writer) = writers.next() else {
            return self.fts_ids(stem, expression).await;
        };
        if writers.any(|other| other != writer) {
            return Ok(HashSet::new());
        }
        let (sql, values) = writer_fts_statement(stem, expression, writer);
        let mut rows = self.db.query(&sql, values).await?;
        let mut hits = HashSet::new();
        while let Some(row) = rows.next().await? {
            hits.insert(row.get::<i64>(0)? as usize);
        }
        Ok(hits)
    }
    pub(crate) async fn fts_ids(&mut self, stem: bool, expression: &str) -> Result<HashSet<usize>> {
        let key = (stem, expression.to_owned());
        if let Some(hit) = self.hits.get(&key) {
            return Ok(hit.clone());
        }
        let table = if stem { "stem" } else { "fts" };
        let mut rows = self
            .db
            .query(
                &format!("SELECT rowid FROM {table} WHERE {table} MATCH ?"),
                [expression],
            )
            .await?;
        let mut hits = HashSet::new();
        while let Some(row) = rows.next().await? {
            hits.insert(row.get::<i64>(0)? as usize);
        }
        self.hits.insert(key, hits.clone());
        Ok(hits)
    }
    pub fn idf(&self, w: &str) -> f64 {
        (self.block_count as f64 / *self.df.get(w).unwrap_or(&1) as f64).ln()
    }
    pub(crate) async fn scope_stats(
        &mut self,
        now: &str,
        q: &query::Query,
    ) -> Result<Arc<Statistics>> {
        if !self.stats.contains_key(now) {
            let mut rows = self.db.query("SELECT blocks,length FROM record_expiry_totals WHERE expires>? ORDER BY expires LIMIT 1", [now]).await?;
            let totals = if let Some(row) = rows.next().await? {
                (row.get::<i64>(0)?, row.get::<i64>(1)?)
            } else {
                let mut rows = self
                    .db
                    .query(
                        "SELECT blocks,length FROM record_expiry_totals WHERE expires=''",
                        (),
                    )
                    .await?;
                let row = rows
                    .next()
                    .await?
                    .ok_or_else(|| Error::Invalid("missing corpus totals".into()))?;
                (row.get::<i64>(0)?, row.get::<i64>(1)?)
            };
            let n = totals.0 as f64;
            self.stats.insert(
                now.into(),
                Arc::new(Statistics {
                    n,
                    avg: totals.1 as f64 / n.max(1.),
                    df: HashMap::new(),
                }),
            );
        }
        let words: HashSet<_> = q
            .terms
            .iter()
            .flat_map(|term| query::surface(&term.text))
            .map(|word| query::norm(&word))
            .collect();
        for word in words {
            if self.stats[now].df.contains_key(&word) {
                continue;
            }
            // MATCH enumerates each matching block once, without visiting every
            // repeated occurrence. Noncanonical tokens retain literal vocabulary
            // lookup: re-tokenizing those could change the requested term.
            let canonical = crate::nlp::porter::exact_tokens(&word)? == [word.as_str()];
            let (sql, term) = if canonical {
                (
                    "SELECT count(*) FROM fts JOIN nav_blocks b ON b.id=fts.rowid JOIN record_read_index c ON c.record=b.record WHERE fts MATCH ? AND (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>?)",
                    format!("\"{}\"", word.replace('"', "\"\"")),
                )
            } else {
                (
                    "SELECT count(*) FROM (SELECT DISTINCT doc FROM vocab WHERE term=?) v JOIN nav_blocks b ON b.id=v.doc JOIN record_read_index c ON c.record=b.record WHERE (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>?)",
                    word.clone(),
                )
            };
            let mut rows = self.db.query(sql, params![term, now]).await?;
            let df = rows.next().await?.unwrap().get::<i64>(0)? as f64;
            if df == 0. {
                self.stat_misses.insert(word.clone());
            }
            Arc::make_mut(self.stats.get_mut(now).unwrap())
                .df
                .insert(word, df);
        }
        Ok(self.stats[now].clone())
    }
    pub async fn forget(&mut self, rid: &str) -> Result<()> {
        let previous = self.begin_write_cache().await?;
        let result = self.forget_inner(rid).await;
        let result = self.finish_write_cache(previous, result).await;
        self.write_outcome(result)
    }
    async fn forget_inner(&mut self, rid: &str) -> Result<()> {
        self.materialize_structure().await?;
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        publication::erase_record(&tx, rid).await?;
        tx.execute("INSERT OR REPLACE INTO sync_state VALUES('pending',1)", ())
            .await?;
        tx.commit().await?;
        self.materialize_block_words();
        self.records.remove(rid);
        self.rows.retain(|_, b| b.record != rid);
        self.block_count = self.rows.len();
        self.vectors.retain(|sid, _| self.rows.contains_key(sid));
        self.sections.remove(rid);
        self.hits.clear();
        self.eligible_hits.clear();
        self.form_cache = None;
        self.beam_cache = None;
        self.statistics().await?;
        let links = self.links.clone();
        self.navigation();
        self.persist_structure().await?;
        self.prepare_fallback_artifacts().await?;
        self.publish_pos().await?;
        self.publish_catalog().await?;
        self.links = links;
        self.complete_publication().await
    }
    pub async fn integrity(&self) -> Result<()> {
        let snapshot = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        migrations::check(&snapshot).await?;
        let mut rows = snapshot.query("PRAGMA quick_check", ()).await?;
        while let Some(row) = rows.next().await? {
            let result: String = row.get(0)?;
            if result != "ok" {
                return Err(Error::Invalid(format!("database integrity: {result}")));
            }
        }
        drop(rows);
        let mut rows = snapshot
            .query(
                "SELECT id,body,body_hash,envelope,envelope_hash FROM records ORDER BY id",
                (),
            )
            .await?;
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let body: Vec<u8> = row.get(1)?;
            let envelope: Vec<u8> = row.get(3)?;
            let record: Record = serde_json::from_slice(&envelope)?;
            if hash(&body) != row.get::<String>(2)?
                || hash(&envelope) != row.get::<String>(4)?
                || record.id != id
                || record.body.as_bytes() != body
            {
                return Err(Error::Invalid(format!("original record integrity: {id}")));
            }
        }
        drop(rows);
        let mut rows = snapshot.query("SELECT EXISTS(SELECT 1 FROM sections s LEFT JOIN vectors v ON v.id=s.id WHERE v.id IS NULL OR v.record IS NOT s.record OR v.emb IS NULL OR typeof(v.emb)!='blob' OR length(v.emb)!=2048) OR EXISTS(SELECT 1 FROM vectors v LEFT JOIN sections s ON s.id=v.id WHERE s.id IS NULL)", ()).await?;
        if rows.next().await?.unwrap().get::<i64>(0)? != 0 {
            return Err(Error::Invalid(
                "block/vector index coverage mismatch".into(),
            ));
        }
        drop(rows);
        self.db
            .execute("INSERT INTO fts(fts,rank) VALUES('integrity-check',1)", ())
            .await
            .map_err(|e| Error::Invalid(format!("exact index integrity: {e}")))?;
        self.db
            .execute("INSERT INTO stem(stem) VALUES('integrity-check')", ())
            .await?;
        snapshot.rollback().await?;
        Ok(())
    }
    pub fn resolve(&self, reference: &str, scope: Option<&str>) -> Result<(&Record, [usize; 2])> {
        self.ensure_usable()?;
        if let Some(r) = self.records.get(reference) {
            return Ok((r, [0, r.body_len()]));
        }
        let parts: Vec<_> = reference.split(':').collect();
        let prefix = parts[0];
        if prefix.len() < 8 || !prefix.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::Invalid(format!("unknown reference: {reference}")));
        }
        let candidates: Vec<_> = self
            .records
            .values()
            .filter(|r| r.body_hash().starts_with(prefix))
            .collect();
        self.resolve_candidates(reference, scope, candidates)
    }
    fn resolve_candidates<'a>(
        &'a self,
        reference: &str,
        scope: Option<&str>,
        mut candidates: Vec<&'a Record>,
    ) -> Result<(&'a Record, [usize; 2])> {
        let parts: Vec<_> = reference.split(':').collect();
        let hashes: HashSet<_> = candidates.iter().map(|r| r.body_hash()).collect();
        if hashes.len() > 1 {
            return Err(Error::Invalid(format!(
                "ambiguous hash: {}",
                candidates
                    .iter()
                    .map(|r| r.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        candidates.sort_by_key(|r| {
            (
                !self.hint_eligible(&r.id),
                r.scope.as_ref().map(|s| s.to_lowercase()) != scope.map(str::to_lowercase),
                r.id.clone(),
            )
        });
        let r = candidates
            .first()
            .ok_or_else(|| Error::Invalid(format!("unknown reference: {reference}")))?;
        let span = if parts.len() == 4 && parts[1] == "v1" {
            let a = parts[2]
                .parse::<usize>()
                .map_err(|_| Error::Invalid("invalid span".into()))?;
            let b = parts[3]
                .parse::<usize>()
                .map_err(|_| Error::Invalid("invalid span".into()))?;
            if a > b || b > r.body_len() {
                return Err(Error::Invalid("invalid byte span".into()));
            }
            [a, b]
        } else if parts.len() == 1 {
            [0, r.body_len()]
        } else {
            return Err(Error::Invalid("invalid fetch id".into()));
        };
        Ok((r, span))
    }
}
fn structure_stamp() -> Vec<u8> {
    let mut stamp = structure_version();
    stamp.extend_from_slice(
        format!(
            ":{}:{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
        .as_bytes(),
    );
    stamp
}
fn structure_version() -> Vec<u8> {
    b"navigation-v2-lifecycle".to_vec()
}
// FTS5 column sizes use SQLite's big-endian varints, one per indexed column.
fn column_lengths(bytes: &[u8]) -> Result<usize> {
    let mut offset = 0;
    let mut sizes = [0usize; 3];
    for size in &mut sizes {
        let mut value = 0u64;
        for index in 0..9 {
            let byte = *bytes
                .get(offset)
                .ok_or_else(|| Error::Invalid("truncated FTS column sizes".into()))?;
            offset += 1;
            if index == 8 {
                value = (value << 8) | byte as u64;
                break;
            }
            value = (value << 7) | (byte & 127) as u64;
            if byte < 128 {
                break;
            }
        }
        *size = usize::try_from(value)
            .map_err(|_| Error::Invalid("oversized FTS column size".into()))?;
    }
    if offset != bytes.len() {
        return Err(Error::Invalid("unexpected FTS column sizes".into()));
    }
    sizes[0]
        .checked_add(sizes[1])
        .ok_or_else(|| Error::Invalid("oversized FTS length".into()))
}
fn normalize_path(path: &Path) -> String {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => match out.components().next_back() {
                Some(std::path::Component::Normal(_)) => {
                    out.pop();
                }
                Some(std::path::Component::RootDir) => {}
                _ => out.push(".."),
            },
            std::path::Component::CurDir => {}
            c => out.push(c),
        }
    }
    out.to_string_lossy().into()
}

#[cfg(test)]
mod lookup_tests {
    use super::*;
    #[tokio::test]
    async fn write_cache_is_bounded_and_restored_after_success_and_failure() {
        async fn cache(store: &Store) -> i64 {
            store
                .db
                .query("PRAGMA cache_size", ())
                .await
                .unwrap()
                .next()
                .await
                .unwrap()
                .unwrap()
                .get(0)
                .unwrap()
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("write-cache.db");
        let records = ["a", "b"]
            .into_iter()
            .map(|id| {
                serde_json::from_value::<Record>(serde_json::json!({
                    "id":id,"body":"Original amber cobalt material."
                }))
                .unwrap()
            })
            .collect();
        let mut store = Store::build(&path, records).await.unwrap();
        store
            .db
            .execute_batch(
                "PRAGMA cache_size=-7777;
            CREATE TRIGGER check_write_cache BEFORE INSERT ON sections BEGIN
                SELECT CASE WHEN (SELECT cache_size FROM pragma_cache_size)!=-32768
                    THEN RAISE(ABORT,'wrong write cache') END;
            END;
            CREATE TRIGGER check_forget_cache BEFORE INSERT ON tombstones BEGIN
                SELECT CASE WHEN (SELECT cache_size FROM pragma_cache_size)!=-32768
                    THEN RAISE(ABORT,'wrong forget cache') END;
            END;",
            )
            .await
            .unwrap();
        store.rebuild().await.unwrap();
        assert_eq!(cache(&store).await, -7777);
        store.forget("a").await.unwrap();
        assert_eq!(cache(&store).await, -7777);
        store.integrity().await.unwrap();
        store
            .db
            .execute_batch(
                "CREATE TRIGGER abort_record_delete BEFORE DELETE ON sections BEGIN
            SELECT RAISE(ABORT,'injected write failure'); END;",
            )
            .await
            .unwrap();
        assert!(
            store
                .rebuild()
                .await
                .unwrap_err()
                .to_string()
                .contains("injected write failure")
        );
        assert_eq!(cache(&store).await, -7777);
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        let previous = cache(&store).await;
        assert!(
            store
                .forget("b")
                .await
                .unwrap_err()
                .to_string()
                .contains("injected write failure")
        );
        assert_eq!(cache(&store).await, previous);
        store.integrity().await.unwrap();
    }

    async fn check_features(store: &mut Store) {
        store.materialize_structure().await.unwrap();
        store.load_vocabulary().await.unwrap();
        let mut expected = store.rows.clone();
        for block in expected.values_mut() {
            block.tf.clear();
            block.body_positions.clear();
            block.stem_positions.clear();
            block.length = 0;
        }
        let mut result = store
            .db
            .query("SELECT term,doc,col,offset FROM vocab", ())
            .await
            .unwrap();
        while let Some(row) = result.next().await.unwrap() {
            let word = row.get::<String>(0).unwrap();
            let Some(block) = expected.get_mut(&(row.get::<i64>(1).unwrap() as usize)) else {
                continue;
            };
            let col = row.get::<String>(2).unwrap();
            if col == "body" {
                block
                    .body_positions
                    .entry(word.clone())
                    .or_default()
                    .push(row.get::<i64>(3).unwrap() as usize);
            }
            let col = match col.as_str() {
                "heading" => 0,
                "body" => 1,
                _ => 2,
            };
            block.tf.entry(word).or_default()[col] += 1.;
            if col < 2 {
                block.length += 1;
            }
        }
        let mut stems = HashSet::new();
        let mut result = store
            .db
            .query("SELECT term,doc,offset FROM stem_vocab", ())
            .await
            .unwrap();
        while let Some(row) = result.next().await.unwrap() {
            let word = row.get::<String>(0).unwrap();
            stems.insert(word.clone());
            if let Some(block) = expected.get_mut(&(row.get::<i64>(1).unwrap() as usize)) {
                block
                    .stem_positions
                    .entry(word)
                    .or_default()
                    .push(row.get::<i64>(2).unwrap() as usize);
            }
        }
        let words: Vec<_> = store.vocab.iter().cloned().collect();
        store.load_terms(words.clone(), false).await.unwrap();
        store.load_terms(words, false).await.unwrap(); // A repeated load must not double TF.
        store.load_terms(stems, true).await.unwrap();
        for (id, block) in expected {
            let actual = &store.rows[&id];
            assert_eq!(actual.length, block.length);
            assert_eq!(actual.tf, block.tf);
            assert_eq!(actual.body_positions, block.body_positions);
            assert_eq!(actual.stem_positions, block.stem_positions);
        }
    }

    #[tokio::test]
    async fn lazy_features_equal_instance_scans_across_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("features.db");
        let record = |id: &str| {
            serde_json::from_value::<Record>(serde_json::json!({"id":id,"state":"accepted","body":format!("# Heading\n{}\n## Heading\nfoo_bar foo-bar camelCase résumé", "alpha beta ".repeat(230))})).unwrap()
        };
        let mut store = Store::build(&path, vec![record("a"), record("z")])
            .await
            .unwrap();
        assert!(store.rows.values().all(|b| b.tf.is_empty()
            && b.body_positions.is_empty()
            && b.stem_positions.is_empty()));
        let mut full = Store::open(&path).await.unwrap();
        check_features(&mut full).await;
        let ctx = Context::default();
        for query in ["alpha beta", "foo_bar résumé", "camelCase alpha"] {
            let lazy = store.agent(query, &ctx).await.unwrap();
            let eager = full.agent(query, &ctx).await.unwrap();
            assert_eq!(
                serde_json::to_value(lazy).unwrap(),
                serde_json::to_value(eager).unwrap()
            );
        }
        let bits = |store: &Store| {
            store
                .confidences
                .iter()
                .map(|(k, v)| (k.clone(), v.map(f64::to_bits)))
                .collect::<HashMap<_, _>>()
        };
        assert_eq!(bits(&store), bits(&full));
        drop(full);
        check_features(&mut store).await;
        store.forget("a").await.unwrap();
        check_features(&mut store).await;
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        assert!(store.loaded_terms.is_empty());
        check_features(&mut store).await;
        store.add(record("new")).await.unwrap();
        check_features(&mut store).await;
        store.rebuild().await.unwrap();
        check_features(&mut store).await;
    }

    #[test]
    fn native_column_size_varints_are_bounded_and_exact() {
        assert_eq!(column_lengths(&[0, 0, 0]).unwrap(), 0);
        assert_eq!(column_lengths(&[0x81, 0, 1, 7]).unwrap(), 129);
        assert!(column_lengths(&[0x80]).is_err());
        assert!(column_lengths(&[0, 0, 0, 0]).is_err());
    }

    #[tokio::test]
    async fn block_matrix_keeps_only_record_ordinals_until_candidates_need_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("matrix.db");
        let records = ["a", "b"].into_iter().map(|id| {
            serde_json::from_value::<Record>(serde_json::json!({
                "id": id, "body": "# Heading\nOriginal alpha content.\n## Child\nOriginal beta content."
            })).unwrap()
        }).collect();
        let mut built = Store::build(&path, records).await.unwrap();
        for (name, expected) in [
            ("secure_delete", 1),
            ("foreign_keys", 1),
            ("busy_timeout", 5000),
            ("temp_store", 2),
            ("cache_size", -65536),
            ("mmap_size", 268435456),
        ] {
            let mut rows = built.db.query(&format!("PRAGMA {name}"), ()).await.unwrap();
            assert_eq!(
                rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
                expected,
                "{name}"
            );
        }
        let expected = built.block_matrix().await.unwrap();
        let mut reopened = Store::open(&path).await.unwrap();
        let selected = reopened.selected_vectors(&expected.ids).await.unwrap();
        assert!(reopened.vector_matrix.is_none());
        let actual = reopened.block_matrix().await.unwrap();
        assert!(reopened.rows.is_empty());
        assert!(reopened.loaded_blocks.is_empty());
        assert_eq!(actual.ids, expected.ids);
        assert_eq!(actual.records, expected.records);
        assert_eq!(
            selected,
            reopened.selected_vectors(&actual.ids).await.unwrap()
        );
        for (i, id) in actual.ids.iter().enumerate() {
            assert_eq!(
                selected[id].iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                expected
                    .matrix
                    .row(i)
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
        }
        for i in 0..actual.ids.len() {
            assert_eq!(
                actual
                    .matrix
                    .row(i)
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .matrix
                    .row(i)
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>()
            );
            let id = actual.ids[i];
            reopened.load_blocks([id]).await.unwrap();
            assert_eq!(
                reopened.rows[&id].record,
                reopened.indexes.records[&actual.records[i]]
            );
        }
        // The explicit vector diagnostic must remain usable before a matrix walk.
        let mut diagnostic = Store::open(&path).await.unwrap();
        diagnostic.load_vectors().await.unwrap();
        let matrix = diagnostic.block_matrix().await.unwrap();
        assert_eq!(matrix.ids, expected.ids);
        assert_eq!(matrix.records, expected.records);
    }

    #[tokio::test]
    async fn structural_projection_is_lazy_transactional_and_invalidated_by_legacy_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projection.db");
        let records = vec![serde_json::from_value::<Record>(serde_json::json!({"id":"r","state":"accepted","label":"ExtraLabel","source_path":"docs/shared.md","body":"# Root\nalpha beta foo-bar. café.\n## Child\ngamma delta.\n[link](shared.md)"})).unwrap()];
        let full = Store::build(&path, records).await.unwrap();
        let expected_order = full.word_order.clone();
        let expected_df = full.df.clone();
        drop(full);
        let mut store = Store::open(&path).await.unwrap();
        assert!(store.structure_lazy);
        assert!(store.loaded_blocks.is_empty());
        assert!(store.loaded_sections.is_empty());
        assert!(store.rows.is_empty());
        assert!(store.direct.is_empty() && store.deep.is_empty());
        assert!(store.indexes.sections.is_empty());
        assert!(
            store
                .records
                .values()
                .all(|r| r.stored_body.is_some() && r.body.is_empty())
        );
        assert!(store.records.is_empty());
        assert!(store.lookups.is_empty());
        assert!(store.indexes.records.is_empty());
        assert!(store.indexes.record_positions.is_empty());
        assert_eq!(store.record_count(), 1);
        store.load_record_headers(["r".into()]).await.unwrap();
        assert!(store.hint_eligible("r"));
        assert_eq!(store.block_count(), 2);
        assert!(
            store
                .rows
                .values()
                .all(|b| b.text.is_empty() && b.path.is_empty())
        );
        assert!(
            store
                .rows
                .values()
                .all(|b| b.words.is_empty() && b.surface.is_empty())
        );
        assert!(
            store
                .direct
                .values()
                .all(|t| t.rows.is_empty() && t.words.is_empty())
        );
        assert!(store.indexes.own.is_empty() && store.indexes.direct.is_empty());
        assert!(store.word_order.is_empty());
        assert!(store.df.is_empty());
        let hits = store.fts(false, "alpha").await.unwrap();
        assert!(!hits.is_empty());
        assert!(store.loaded_blocks.is_empty());
        assert!(store.records["r"].body.is_empty());
        let id = *hits.iter().next().unwrap();
        assert_eq!(store.rows[&id].heading, "Root");
        store.load_blocks([id]).await.unwrap();
        assert_eq!(store.rows[&id].text, "# Root\nalpha beta foo-bar. café.\n");
        assert!(store.records["r"].body.is_empty());
        store
            .agent("alpha beta", &Context::default())
            .await
            .unwrap();
        assert!(store.structure_lazy);
        assert!(!store.loaded_blocks.is_empty());
        assert!(!store.navigation_words.contains("gamma"));
        assert_eq!(store.df.get("alpha"), expected_df.get("alpha"));
        assert!(!store.df.contains_key("gamma"));
        store.load_surface_dictionary().await.unwrap();
        assert_eq!(store.word_order, expected_order);
        assert_eq!(store.df, expected_df);
        let mut plan = store.db.query("EXPLAIN QUERY PLAN SELECT b.block FROM nav_lexemes n JOIN nav_block_words b ON b.word=n.id WHERE n.surface=? ORDER BY b.block",["alpha"]).await.unwrap();
        let mut explanation = String::new();
        while let Some(row) = plan.next().await.unwrap() {
            explanation.push_str(&row.get::<String>(3).unwrap());
        }
        assert!(
            explanation.contains("SEARCH n") && explanation.contains("SEARCH b"),
            "{explanation}"
        );
        store
            .db
            .execute_batch("BEGIN; UPDATE records SET envelope=envelope; ROLLBACK;")
            .await
            .unwrap();
        let mut ready = store
            .db
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await
            .unwrap();
        assert!(ready.next().await.unwrap().is_some());
        drop(ready);
        store
            .db
            .execute("UPDATE records SET envelope=envelope", ())
            .await
            .unwrap();
        let mut ready = store
            .db
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await
            .unwrap();
        assert!(ready.next().await.unwrap().is_none());
        drop(ready);
        drop(plan);
        drop(store);
        let store = Store::open(&path).await.unwrap();
        assert!(!store.structure_lazy);
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        assert!(store.structure_lazy);
        assert!(store.loaded_blocks.is_empty());
        assert!(store.loaded_sections.is_empty());
        assert!(store.rows.is_empty());
        assert!(store.direct.is_empty() && store.deep.is_empty());
        assert!(store.indexes.sections.is_empty());
        assert!(
            store
                .records
                .values()
                .all(|r| r.stored_body.is_some() && r.body.is_empty())
        );
        assert!(store.records.is_empty());
        assert!(store.lookups.is_empty());
        assert!(store.indexes.records.is_empty());
        assert!(store.indexes.record_positions.is_empty());
        assert_eq!(store.record_count(), 1);
        store.load_record_headers(["r".into()]).await.unwrap();
        assert!(store.hint_eligible("r"));
        assert_eq!(store.block_count(), 2);
        assert!(
            store
                .rows
                .values()
                .all(|b| b.text.is_empty() && b.path.is_empty())
        );
        check_lookups(&mut store).await;
        store.forget("r").await.unwrap();
        let mut words = store
            .db
            .query("SELECT count(*) FROM nav_lexemes", ())
            .await
            .unwrap();
        assert_eq!(
            words.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn separate_publications_require_reopen_and_legacy_forget_erases_projections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("publication.db");
        let record=serde_json::from_value::<Record>(serde_json::json!({"id":"r","state":"accepted","body":"# Secret Heading\nprivate_unique_identifier alpha beta"})).unwrap();
        drop(Store::build(&path, vec![record]).await.unwrap());
        let mut stale = Store::open(&path).await.unwrap();
        assert!(stale.vectors.is_empty() && stale.vocab.is_empty());
        stale
            .agent("alpha beta", &Context::default())
            .await
            .unwrap();
        let mut writer = Store::open(&path).await.unwrap();
        writer.forget("r").await.unwrap();
        let error = stale
            .agent("alpha beta", &Context::default())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("reopen"));
        assert!(
            stale
                .rebuild()
                .await
                .unwrap_err()
                .to_string()
                .contains("reopen")
        );
        let mut remaining = writer
            .db
            .query("SELECT count(*) FROM records", ())
            .await
            .unwrap();
        assert_eq!(
            remaining
                .next()
                .await
                .unwrap()
                .unwrap()
                .get::<i64>(0)
                .unwrap(),
            0
        );
        drop(remaining);
        drop(stale);
        drop(writer);
        let record=serde_json::from_value::<Record>(serde_json::json!({"id":"new","state":"accepted","body":"# Private heading\nnew private text"})).unwrap();
        let mut writer = Store::open(&path).await.unwrap();
        writer.add(record).await.unwrap();
        writer
            .db
            .execute_batch("BEGIN; DELETE FROM records; ROLLBACK;")
            .await
            .unwrap();
        let mut count = writer
            .db
            .query("SELECT count(*) FROM nav_lexemes", ())
            .await
            .unwrap();
        assert!(count.next().await.unwrap().unwrap().get::<i64>(0).unwrap() > 0);
        drop(count);
        // A record DELETE from an older writer must erase derived content in its own transaction.
        writer.db.execute("DELETE FROM records", ()).await.unwrap();
        for table in [
            "nav_lexemes",
            "nav_blocks",
            "nav_sections",
            "nav_links",
            "fallback_artifacts",
            "fallback_vectors",
        ] {
            let mut count = writer
                .db
                .query(&format!("SELECT count(*) FROM {table}"), ())
                .await
                .unwrap();
            assert_eq!(
                count.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
                0
            );
        }
    }

    #[tokio::test]
    async fn numerical_projection_chunks_preserve_bits_and_model_generation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("numeric.db");
        let store = Store::open(&path).await.unwrap();
        let rows = 16385;
        let values: Vec<_> = (0..rows * 512)
            .map(|i| match i % 17 {
                0 => f32::from_bits(0x7fc12345),
                1 => -0.,
                _ => ((i % 8192) as f32).sin(),
            })
            .collect();
        let matrix = crate::retrieval::similarity::Matrix::from_values(values, rows).unwrap();
        store
            .publish_fallback_artifacts(&[(
                "roundtrip",
                serde_json::to_vec(&vec!["metadata"]).unwrap(),
                &matrix,
            )])
            .await
            .unwrap();
        let (metadata, loaded) = store
            .fallback_artifact::<Vec<String>>("roundtrip")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(metadata, vec!["metadata"]);
        assert!(
            matrix
                .chunks()
                .flatten()
                .map(|v| v.to_bits())
                .eq(loaded.chunks().flatten().map(|v| v.to_bits()))
        );
        let mut parts = store
            .db
            .query(
                "SELECT count(*) FROM fallback_vectors WHERE kind='roundtrip'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(
            parts.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            2
        );
        drop(parts);
        store
            .db
            .execute(
                "UPDATE fallback_artifacts SET model='different' WHERE kind='roundtrip'",
                (),
            )
            .await
            .unwrap();
        assert!(
            store
                .fallback_artifact::<Vec<String>>("roundtrip")
                .await
                .unwrap()
                .is_none()
        );
        store
            .db
            .execute(
                "UPDATE fallback_artifacts SET model=?,generation=x'00' WHERE kind='roundtrip'",
                [crate::language::embedding::ARTIFACT_MODEL],
            )
            .await
            .unwrap();
        assert!(
            store
                .fallback_artifact::<Vec<String>>("roundtrip")
                .await
                .unwrap()
                .is_none()
        );
    }

    async fn hydrate_structure(store: &mut Store) {
        store.materialize_structure().await.unwrap();
        if !store.structure_lazy {
            return;
        }
        let mut words = Vec::new();
        let mut result = store
            .db
            .query("SELECT surface FROM nav_lexemes", ())
            .await
            .unwrap();
        while let Some(row) = result.next().await.unwrap() {
            words.push(row.get::<String>(0).unwrap());
        }
        store.load_surface(&words).await.unwrap();
        store.load_section_words(&words).await.unwrap();
        let mut q = query::Query::parse("").unwrap();
        q.words = words.iter().map(|w| query::norm(w)).collect();
        q.need = 1;
        store.load_navigation(&q).await.unwrap();
        for deep in [false, true] {
            let count = if deep {
                store.deep.len()
            } else {
                store.direct.len()
            };
            for id in 0..count {
                if store.tree_rows.contains(&(deep, id)) {
                    continue;
                }
                let mut result = store
                    .db
                    .query(
                        "SELECT block FROM nav_tree_rows WHERE kind=? AND tree=? ORDER BY block",
                        params![i64::from(deep), id as i64],
                    )
                    .await
                    .unwrap();
                let tree = if deep {
                    store.deep.get_mut(&id).unwrap()
                } else {
                    store.direct.get_mut(&id).unwrap()
                };
                while let Some(row) = result.next().await.unwrap() {
                    tree.rows.push(row.get::<i64>(0).unwrap() as usize);
                }
                store.tree_rows.insert((deep, id));
            }
        }
    }

    async fn check_lookups(store: &mut Store) {
        let have: HashSet<_> = store.records.keys().take(2).cloned().collect();
        let lazy = store.structure_lazy;
        let indexed_neighbors = store.linked_neighbors(&have).await.unwrap();
        store.materialize_structure().await.unwrap();
        store.structure_lazy = false;
        let scanned_neighbors = store.linked_neighbors(&have).await.unwrap();
        store.structure_lazy = lazy;
        assert_eq!(indexed_neighbors, scanned_neighbors);
        hydrate_structure(store).await;
        let link = regex::Regex::new(r"\]\(([^)]+)\)").unwrap();
        let mut expected_direct = Vec::new();
        let mut expected_deep = Vec::new();
        for record in store.records.values() {
            let dests: Vec<_> = link
                .captures_iter(&record.body)
                .map(|c| c[1].to_owned())
                .collect();
            let dir = Path::new(record.source_path.as_deref().unwrap_or(""))
                .parent()
                .unwrap_or(Path::new(""));
            let paths: HashSet<_> = dests
                .iter()
                .map(|d| normalize_path(&dir.join(d.split('#').next().unwrap_or(""))))
                .collect();
            let targets: HashSet<_> = store
                .records
                .values()
                .filter(|other| {
                    dests.contains(&other.id)
                        || other
                            .source_path
                            .as_ref()
                            .is_some_and(|p| paths.contains(p))
                })
                .map(|r| r.id.clone())
                .collect();
            // Forget deliberately retains resolved links until rebuild/reopen.
            let current: HashSet<_> = store.links[&record.id]
                .iter()
                .filter(|id| store.records.contains_key(*id))
                .cloned()
                .collect();
            assert_eq!(current, targets);
            if !store.tree_eligible(record) {
                continue;
            }
            let rows: Vec<_> = store
                .rows
                .values()
                .filter(|b| b.record == record.id)
                .collect();
            let mut paths = Vec::new();
            for block in &rows {
                if !paths.contains(&block.path) {
                    paths.push(block.path.clone());
                }
            }
            for path in paths {
                for (deep, expected) in [(false, &mut expected_direct), (true, &mut expected_deep)]
                {
                    let matching: Vec<_> = rows
                        .iter()
                        .filter(|b| {
                            b.path.starts_with(&path) && (deep || b.path.len() <= path.len() + 1)
                        })
                        .collect();
                    expected.push((
                        record.id.clone(),
                        matching.iter().map(|b| b.id).collect::<Vec<_>>(),
                        matching
                            .iter()
                            .flat_map(|b| b.words.iter().cloned())
                            .collect::<HashSet<_>>(),
                    ));
                }
            }
        }
        for (trees, expected) in [
            (&store.direct, expected_direct),
            (&store.deep, expected_deep),
        ] {
            assert_eq!(trees.len(), expected.len());
            for (tree, (record, rows, words)) in trees.values().zip(expected) {
                assert_eq!(
                    (&tree.record, &tree.rows, &tree.words),
                    (&record, &rows, &words)
                );
            }
        }
        assert_eq!(store.lookups.len(), store.records.len());
        for (rid, record) in &store.records {
            // Compare the cached views with the original ordered full scans.
            let scan: Vec<_> = store.rows.values().filter(|b| &b.record == rid).collect();
            let ids: Vec<_> = scan.iter().map(|b| b.id).collect();
            assert_eq!(
                store.record_blocks(rid).map(|b| b.id).collect::<Vec<_>>(),
                ids
            );
            assert_eq!(
                store.lookups[rid].words,
                scan.iter().flat_map(|b| b.words.iter().cloned()).collect()
            );
            for (section, tokens) in store.sections[rid]
                .iter()
                .zip(store.lookups[rid].sections.values())
            {
                assert_eq!(
                    tokens.own,
                    query::surface(&format!(
                        "{} {}",
                        section.heading,
                        &record.body[section.span.clone()]
                    ))
                    .into_iter()
                    .collect()
                );
                assert_eq!(
                    tokens.extra,
                    query::surface(&format!("{} {}", section.path.join(" "), record.label))
                        .into_iter()
                        .collect()
                );
            }
            assert_eq!(store.lookups[rid].sections.len(), store.sections[rid].len());
            let mut rows = store
                .db
                .query(
                    "SELECT id FROM sections WHERE record=? ORDER BY id",
                    [rid.clone()],
                )
                .await
                .unwrap();
            let mut persisted = Vec::new();
            while let Some(row) = rows.next().await.unwrap() {
                persisted.push(row.get::<i64>(0).unwrap() as usize);
            }
            assert_eq!(ids, persisted);
        }
        assert_eq!(store.record_blocks("missing").count(), 0);
        let expected_sections: Vec<_> = store
            .sections
            .iter()
            .flat_map(|(rid, sections)| sections.iter().map(|s| (rid.clone(), s.ordinal)))
            .collect();
        assert_eq!(
            store.indexes.sections.values().cloned().collect::<Vec<_>>(),
            expected_sections
        );
        for words in [
            vec!["résumé", "child"],
            vec!["labelonly", "café"],
            vec!["foo_bar", "foo-bar", "child"],
            vec!["missing", "missing"],
        ] {
            let unique: HashSet<_> = words.iter().map(|w| w.to_string()).collect();
            for need in 1..=3 {
                for related in [false, true] {
                    let postings = if related {
                        &store.indexes.related
                    } else {
                        &store.indexes.own
                    };
                    let actual = posting_candidates(postings, words.iter().copied(), need);
                    let expected: Vec<_> = store
                        .indexes
                        .sections
                        .iter()
                        .filter(|(_, (rid, ordinal))| {
                            let tokens = &store.lookups[rid].sections[ordinal];
                            unique
                                .iter()
                                .filter(|w| {
                                    tokens.own.contains(*w)
                                        || (related && tokens.extra.contains(*w))
                                })
                                .count()
                                >= need
                        })
                        .map(|(&id, _)| id)
                        .collect();
                    assert_eq!(actual, expected);
                }
            }
        }
        for text in ["résumé child", "\"résumé\" résumé", "foo-bar camelCase"] {
            let q = query::Query::parse(text).unwrap();
            assert_eq!(q.need, query::need(q.content().len()));
            for (trees, postings) in [
                (&store.direct, &store.indexes.direct),
                (&store.deep, &store.indexes.deep),
            ] {
                let expected: Vec<_> = trees
                    .iter()
                    .filter(|(_, tree)| q.words.intersection(&tree.words).count() >= q.need)
                    .map(|(&id, _)| id)
                    .collect();
                assert_eq!(
                    posting_candidates(postings, q.words.iter().map(String::as_str), q.need),
                    expected
                );
            }
        }
    }

    #[tokio::test]
    async fn derived_views_follow_lifecycle_and_persisted_row_gaps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let record = |id: &str, body: &str, state: &str| -> Record {
            serde_json::from_value(serde_json::json!({"id":id,"body":body,"label":"LabelOnly CAFÉ","state":state,"source_path":"shared.nix"})).unwrap()
        };
        let body = format!(
            "Preamble résumé.\n# Root\n{}\n\n## Child\nRésumé child foo_bar foo-bar camelCase.\n[all](./shared.nix#Root) [id](candidate) [missing](missing.md)\n",
            "long content line\n".repeat(190)
        );
        let mut store = Store::build(
            &path,
            vec![
                record(
                    "a",
                    "Removed early rows.\n# Other\nOther passage.",
                    "accepted",
                ),
                record("z", &body, "accepted"),
                record("candidate", "Candidate résumé content.", "candidate"),
                record("empty", "", "accepted"),
                record("spaces", "   ", "accepted"),
            ],
        )
        .await
        .unwrap();
        check_lookups(&mut store).await;
        let child = store.sections["z"]
            .iter()
            .position(|s| s.heading == "Child")
            .unwrap();
        let tokens = &store.lookups["z"].sections[&child];
        assert!(tokens.own.contains("résumé"));
        assert!(!tokens.own.contains("resume"));
        assert!(!tokens.own.contains("labelonly"));
        assert!(tokens.extra.contains("labelonly"));
        assert!(store.lookups["z"].words.contains("resume"));
        let links_before_forget = store.links.clone();
        store.forget("a").await.unwrap();
        assert_eq!(store.links, links_before_forget);
        check_lookups(&mut store).await;
        assert!(!store.lookups.contains_key("a"));
        assert!(store.rows.keys().next().unwrap() > &1);
        let queries = [
            "résumé child",
            "labelonly café",
            "\"shared.nix\"",
            "zzabsentword xxmissingword",
        ];
        let mut before = Vec::new();
        for query in queries {
            before.push(
                serde_json::to_value(store.agent(query, &Context::default()).await.unwrap())
                    .unwrap(),
            );
        }
        // A legacy schema may predate the auxiliary index.
        store
            .db
            .execute_batch("DROP INDEX records_writer_created; DROP INDEX records_writer_expiry; DROP INDEX sections_record; DROP TABLE sync_remotes; DROP TABLE sync_state; DROP INDEX sync_scope_records; PRAGMA user_version=2;")
            .await
            .unwrap();
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        check_lookups(&mut store).await;
        for (query, expected) in queries.into_iter().zip(before) {
            assert_eq!(
                serde_json::to_value(store.agent(query, &Context::default()).await.unwrap())
                    .unwrap(),
                expected
            );
        }
        store
            .add(record(
                "new",
                "# Fresh\nFresh original content.",
                "accepted",
            ))
            .await
            .unwrap();
        check_lookups(&mut store).await;
        assert!(store.lookups["new"].words.contains("fresh"));
        store.rebuild().await.unwrap();
        check_lookups(&mut store).await;
        let mut plan = store
            .db
            .query(
                "EXPLAIN QUERY PLAN SELECT id FROM sections WHERE record='z' ORDER BY id",
                (),
            )
            .await
            .unwrap();
        assert!(
            plan.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(3)
                .unwrap()
                .contains("sections_record")
        );
        store.integrity().await.unwrap();
    }
}

#[cfg(test)]
mod selective_read_tests {
    use super::*;
    #[tokio::test]
    async fn match_document_counts_preserve_literal_vocabulary_and_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("counts.db");
        let body = "# Alpha café\nalpha alpha foo-bar foo_bar Café 東京 ẞ ﬀ ²";
        let mut records: Vec<Record> = ["live", "expired", "candidate", "closed"]
            .into_iter()
            .map(|id| serde_json::from_value(serde_json::json!({"id":id,"body":body})).unwrap())
            .collect();
        records[1].stale_after = Some("2026-11-05T00:00:00Z".into());
        records[2].state = "candidate".into();
        records.push(
            serde_json::from_value(serde_json::json!({
                "id":"close", "body":"", "close":{"target":"closed","reason":"closed"}
            }))
            .unwrap(),
        );
        Store::build(&path, records).await.unwrap();
        let mut store = Store::open(&path).await.unwrap();
        let q = query::Query::parse("alpha café foo-bar foo_bar 東京 ẞ ﬀ ² missing").unwrap();
        for now in ["2026-11-04T00:00:00Z", NOW] {
            let stats = store.scope_stats(now, &q).await.unwrap();
            for (word, actual) in &stats.df {
                let mut rows = store.db.query(
                    "SELECT count(*) FROM (SELECT DISTINCT doc FROM vocab WHERE term=?) v JOIN nav_blocks b ON b.id=v.doc JOIN record_read_index c ON c.record=b.record WHERE (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>?)",
                    params![word.as_str(), now],
                ).await.unwrap();
                assert_eq!(
                    *actual,
                    rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap() as f64,
                    "{word} at {now}"
                );
            }
        }
        assert!(store.rows.is_empty() && store.records.is_empty());
    }
    #[tokio::test]
    async fn sql_tree_selection_preserves_winners_without_loading_members() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("selection.db");
        let records = [
            (
                "a",
                "# Root\nalpha beta\n## Child\nalpha gamma\n## Last\nbeta gamma",
                "accepted",
            ),
            (
                "z",
                "# Root\nalpha beta gamma foo-bar\n## Child\nalpha beta gamma",
                "accepted",
            ),
            ("candidate", "# Root\nalpha beta gamma", "candidate"),
        ]
        .into_iter()
        .map(|(id, body, state)| {
            serde_json::from_value(serde_json::json!({"id":id,"body":body,"state":state})).unwrap()
        })
        .collect();
        let full = Store::build(&path, records).await.unwrap();
        let mut lazy = Store::open(&path).await.unwrap();
        let raw = lazy.fts_ids(false, "alpha").await.unwrap();
        assert!(lazy.rows.is_empty() && lazy.records.is_empty());
        let eligible = lazy.eligible_hit_ids(&raw, NOW, None).await.unwrap();
        assert!(eligible.len() < raw.len());
        assert_eq!(
            lazy.eligible_fts_hits(false, "alpha", NOW).await.unwrap(),
            eligible
        );
        assert_eq!(
            lazy.eligible_fts_hits(false, "alpha", NOW).await.unwrap(),
            eligible
        );
        assert!(lazy.rows.is_empty() && lazy.records.is_empty());
        for text in [
            "alpha beta gamma",
            "alpha",
            "gamma beta",
            "missing",
            "\"alpha beta\"",
            "foo-bar FOO-BAR alpha beta",
            "alpha beta gamma",
        ] {
            let q = query::Query::parse(text).unwrap();
            lazy.load_search_navigation(&q).await.unwrap();
            for (expected, actual) in [(&full.direct, &lazy.direct), (&full.deep, &lazy.deep)] {
                let expected: BTreeMap<_, _> = expected
                    .iter()
                    .filter(|(_, tree)| {
                        full.hint_eligible(&tree.record)
                            && q.words.intersection(&tree.words).count() >= q.need.max(1)
                            && q.hard_words.iter().all(|w| tree.words.contains(w))
                    })
                    .map(|(&id, tree)| {
                        let winner = *tree
                            .rows
                            .iter()
                            .max_by_key(|id| {
                                (
                                    q.words.intersection(&full.rows[id].words).count(),
                                    std::cmp::Reverse(**id),
                                )
                            })
                            .unwrap();
                        (id, winner)
                    })
                    .collect();
                assert_eq!(
                    actual
                        .iter()
                        .map(|(&id, t)| {
                            assert_eq!(t.rows.len(), 1);
                            (id, t.rows[0])
                        })
                        .collect::<BTreeMap<_, _>>(),
                    expected
                );
            }
        }
        for text in ["alpha beta gamma", "alpha", "missing", "", "\"alpha beta\""] {
            let q = query::Query::parse(text).unwrap();
            for excluded in [HashSet::new(), HashSet::from(["z".to_owned()])] {
                let need = q.need.saturating_sub(1);
                let expected = full
                    .lookups
                    .iter()
                    .filter(|(rid, l)| {
                        !excluded.contains(*rid)
                            && full.record_eligible(rid)
                            && l.count > 0
                            && l.words.intersection(&q.words).count() >= need
                            && q.hard_words.iter().all(|w| l.words.contains(w))
                    })
                    .max_by_key(|(rid, l)| (l.words.intersection(&q.words).count(), *rid))
                    .and_then(|(rid, _)| {
                        full.record_blocks(rid).max_by_key(|b| {
                            (
                                b.words.intersection(&q.words).count(),
                                std::cmp::Reverse(b.id),
                            )
                        })
                    })
                    .map(|b| b.id);
                assert_eq!(
                    lazy.selected_record_fill(&q, &excluded).await.unwrap(),
                    expected,
                    "{text}"
                );
            }
        }
        assert!(lazy.loaded_blocks.is_empty());
        assert!(lazy.loaded_block_headers.is_empty());
        let id = *lazy.rows.keys().next().unwrap();
        lazy.load_candidate_frequencies(["alpha".into()], &[id])
            .await
            .unwrap();
        let frequency = lazy.rows[&id].tf.clone();
        assert!(lazy.rows[&id].body_positions.is_empty());
        lazy.load_blocks([id]).await.unwrap();
        assert_eq!(lazy.rows[&id].heading, full.rows[&id].heading);
        assert_eq!(lazy.rows[&id].path, full.rows[&id].path);
        assert_eq!(lazy.rows[&id].tf, frequency);
        lazy.load_candidate_terms(["alpha".into()], false, &[id])
            .await
            .unwrap();
        assert_eq!(lazy.rows[&id].body_positions["alpha"], vec![1]);
        assert_eq!(lazy.rows[&id].tf, frequency);
        let pointer_id = *full
            .rows
            .keys()
            .find(|id| !lazy.rows.contains_key(id))
            .unwrap();
        let pointer = &full.rows[&pointer_id];
        lazy.load_pointer_blocks(&[(pointer.record.clone(), pointer.start)])
            .await
            .unwrap();
        assert_eq!(lazy.rows[&pointer_id].path, pointer.path);
        assert!(!lazy.loaded_blocks.contains(&pointer_id));
    }

    #[tokio::test]
    async fn subset_features_and_batched_porter_preserve_exact_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reads.db");
        let records = [("a", "alpha running foo-bar"), ("b", "alpha alpha cats")]
            .into_iter()
            .map(|(id, body)| {
                serde_json::from_value(serde_json::json!({"id":id,"body":body})).unwrap()
            })
            .collect();
        drop(Store::build(&path, records).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        store
            .load_candidate_terms(["alpha".into()], false, &[1])
            .await
            .unwrap();
        assert!(!store.rows.contains_key(&2));
        assert_eq!(store.rows[&1].tf["alpha"][1], 1.);
        store
            .load_candidate_terms(["alpha".into()], false, &[1, 2])
            .await
            .unwrap();
        assert_eq!(store.rows[&1].tf["alpha"][1], 1.);
        assert_eq!(store.rows[&2].tf["alpha"][1], 2.);
        store.load_terms(["alpha".into()], false).await.unwrap();
        assert_eq!(store.rows[&1].tf["alpha"][1], 1.);
        let words = [
            "Running",
            "foo-bar",
            "snake_case",
            "résumés",
            "",
            "a.b",
            "dogs cats",
        ];
        let mut expected = Vec::new();
        for word in words {
            expected.push(store.porter(word).await.unwrap());
        }
        store.porter_cache.clear();
        store
            .prepare_porter(words.into_iter().map(str::to_owned))
            .await
            .unwrap();
        for (word, expected) in words.into_iter().zip(expected) {
            assert_eq!(store.porter(word).await.unwrap(), expected);
        }
    }
    #[tokio::test]
    async fn metadata_and_bounded_display_preserve_unicode_and_nuls() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("display.db");
        let body = format!("é\0{}", "🦀".repeat(1601));
        drop(
            Store::build(
                &path,
                vec![serde_json::from_value(serde_json::json!({"id":"a","body":body})).unwrap()],
            )
            .await
            .unwrap(),
        );
        let mut store = Store::open(&path).await.unwrap();
        store.resolve_metadata("a", None).await.unwrap();
        store.load_record_sections("a").await.unwrap();
        assert!(store.records["a"].body.is_empty());
        for span in [[0, body.len()], [1, 12], [0, 11], [3, body.len()]] {
            let expected = String::from_utf8_lossy(&body.as_bytes()[span[0]..span[1]]);
            for all in [false, true] {
                let (text, count) = store.display_span("a", span, all).await.unwrap();
                assert_eq!(count, expected.chars().count(), "{span:?}");
                assert_eq!(
                    text,
                    if all {
                        expected.to_string()
                    } else {
                        expected.chars().take(1500).collect()
                    }
                );
            }
        }
        assert!(store.records["a"].body.is_empty());
    }
}

#[cfg(test)]
mod embedding_publication_tests {
    use super::*;

    #[tokio::test]
    async fn incompatible_block_model_requires_explicit_rebuild() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store.db");
        let record: Record = serde_json::from_value(serde_json::json!({
            "id": "source", "body": "A preserved original body.", "state": "accepted"
        }))
        .unwrap();
        let store = Store::build(&path, vec![record]).await.unwrap();
        store
            .db
            .execute("UPDATE vectors SET emb=zeroblob(2048)", ())
            .await
            .unwrap();
        store
            .db
            .execute(
                "UPDATE projections SET value=x'00' WHERE key='block-model-v1'",
                (),
            )
            .await
            .unwrap();
        drop(store);
        assert!(
            matches!(Store::open(&path).await, Err(Error::Invalid(message)) if message.contains("index:rebuild"))
        );
        let mut store = Store::open_for_rebuild(&path).await.unwrap();
        store.rebuild().await.unwrap();
        drop(store);
        let mut store = Store::open(&path).await.unwrap();
        store.load_records(["source".to_owned()]).await.unwrap();
        assert_eq!(store.records["source"].body, "A preserved original body.");
        store.load_vectors().await.unwrap();
        assert_eq!(
            store.vectors[&1],
            Encoder::load()
                .unwrap()
                .encode("A preserved original body.")
        );
        store
            .db
            .execute("DELETE FROM projections WHERE key='block-model-v1'", ())
            .await
            .unwrap();
        drop(store);
        assert!(
            matches!(Store::open(&path).await, Err(Error::Invalid(message)) if message.contains("index:rebuild"))
        );
    }
}

#[cfg(test)]
mod query_lifecycle_tests {
    use super::*;

    #[tokio::test]
    async fn clocks_filter_every_output_and_reopen_preserves_historical_trees() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lifecycle.db");
        let records: Vec<Record> = [
            serde_json::json!({"id":"old","body":"# Amber

Cobalt quasar resolver integration.","source_path":"old.md","stale_after":"2020-01-02T00:00:00Z"}),
            serde_json::json!({"id":"candidate","body":"# Amber

Cobalt quasar candidate secret.","state":"candidate"}),
            serde_json::json!({"id":"parent","body":"# Amber

Cobalt quasar obsolete secret."}),
            serde_json::json!({"id":"child","body":"Replacement knowledge.","parent":"parent"}),
            serde_json::json!({"id":"closed","body":"# Amber

Cobalt quasar closed secret."}),
            serde_json::json!({"id":"close","body":"","close":{"target":"closed","reason":"wrong"}}),
        ].into_iter().map(|v| serde_json::from_value(v).unwrap()).collect();
        let mut store = Store::build(&path, records).await.unwrap();
        for reopened in [false, true] {
            if reopened {
                store
                    .db
                    .execute(
                        "UPDATE projections SET value=? WHERE key='structure'",
                        [b"navigation-v1-shared".to_vec()],
                    )
                    .await
                    .unwrap();
                drop(store);
                store = Store::open(&path).await.unwrap();
            }
            for now in [
                "2020-01-01T00:00:00Z",
                "2020-01-02T00:00:00Z",
                "2020-01-03T00:00:00Z",
                "2020-01-01T00:00:00Z",
            ] {
                let ctx = Context {
                    now: now.into(),
                    ..Context::default()
                };
                for query in [
                    "amber cobalt quasar",
                    "\"old.md\"",
                    "cobalt integration unknown",
                ] {
                    let calls = store.agent(query, &ctx).await.unwrap();
                    for call in &calls {
                        for id in call
                            .cards
                            .iter()
                            .map(|c| &c.record)
                            .chain(call.hints.iter().map(|h| &h.record))
                            .chain(call.related.iter().map(|h| &h.record))
                        {
                            assert!(store.hint_eligible(id), "{now}: {query}: {id}");
                            assert!(
                                !["candidate", "parent", "closed", "close"].contains(&id.as_str())
                            );
                        }
                        assert!(!call.text.contains("candidate secret"));
                        assert!(!call.text.contains("obsolete secret"));
                        assert!(!call.text.contains("closed secret"));
                    }
                    if query == "\"old.md\"" {
                        let shown = calls
                            .iter()
                            .flat_map(|c| &c.cards)
                            .any(|c| c.record == "old");
                        assert_eq!(shown, now < "2020-01-02T00:00:00Z");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod sync_snapshot_tests {
    use super::*;

    #[tokio::test]
    async fn query_memos_follow_query_scope_and_clock_without_changing_pages() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memos.db");
        let record: Record = serde_json::from_value(serde_json::json!({
            "id":"r", "state":"accepted", "body":"Original publication routing evidence."
        }))
        .unwrap();
        drop(Store::build(&path, vec![record]).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        let mut ctx = Context {
            now: "2026-11-13T00:00:00Z".into(),
            ..Default::default()
        };
        let expected =
            serde_json::to_value(store.agent("publication routing", &ctx).await.unwrap()).unwrap();
        for i in 0..40 {
            ctx.now = format!("2026-11-13T00:00:{i:02}Z");
            assert_eq!(
                serde_json::to_value(store.agent("publication routing", &ctx).await.unwrap())
                    .unwrap(),
                expected
            );
            assert_eq!(store.stats.len(), 1);
            assert!(store.eligible_hits.keys().all(|key| key.2 == ctx.now));
        }
        for i in 0..40 {
            let query = format!("unmatchedword{i}");
            store.agent(&query, &ctx).await.unwrap();
            if i != 0 {
                let previous = format!("unmatchedword{}", i - 1);
                assert!(!store.porter_cache.contains_key(&previous));
                assert!(!store.surface_words.contains(&previous));
                assert!(!store.indexes.block_surface.contains_key(&previous));
                assert!(!store.vocabulary_probes.contains(&previous));
            }
        }
        ctx.now = "2026-11-13T00:00:00Z".into();
        assert_eq!(
            serde_json::to_value(store.agent("publication routing", &ctx).await.unwrap()).unwrap(),
            expected
        );
        ctx.scope = Some("other/repo".into());
        let scoped = store.agent("publication routing", &ctx).await.unwrap();
        let mut fresh = Store::open(&path).await.unwrap();
        assert_eq!(
            serde_json::to_value(scoped).unwrap(),
            serde_json::to_value(fresh.agent("publication routing", &ctx).await.unwrap()).unwrap()
        );
        let mut rows = store
            .db
            .query("SELECT id FROM nav_blocks LIMIT 1", ())
            .await
            .unwrap();
        let id = rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap() as usize;
        drop(rows);
        for i in 0..300 {
            let query = format!("distinctmiss{i}");
            store.begin_query(&query, &ctx);
            store.fts(false, &query).await.unwrap();
            store
                .eligible_fts_hits(false, &query, &ctx.now)
                .await
                .unwrap();
            store
                .load_candidate_terms([query], false, &[id])
                .await
                .unwrap();
            assert!(store.hits.len() <= 257);
            assert!(store.eligible_hits.len() <= 257);
            assert!(store.candidate_terms.len() <= 257);
        }
    }

    #[tokio::test]
    async fn numerical_timings_use_request_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("timings.db");
        let record = serde_json::from_value::<Record>(serde_json::json!({
            "id":"r", "state":"accepted", "body":"A complete original routing decision."
        }))
        .unwrap();
        drop(Store::build(&path, vec![record]).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        let flags = crate::diagnostics::Diagnostics {
            startup: true,
            ..Default::default()
        };
        let (result, lines) = crate::diagnostics::capture(flags, async {
            store.fallback_artifact::<Vec<String>>("beam").await?;
            let mut rows = store.db.query("SELECT id FROM vectors LIMIT 1", ()).await?;
            let id = rows.next().await?.unwrap().get::<i64>(0)? as usize;
            drop(rows);
            store.candidate_vector_scores(&[id], &[0.; 512]).await?;
            store.block_matrix().await?;
            Ok::<_, Error>(())
        })
        .await;
        result.unwrap();
        for phase in [
            "numerical-array-read",
            "candidate-vector-score",
            "block-matrix-read",
        ] {
            assert!(
                lines.iter().any(|line| line.contains(phase)),
                "missing {phase}: {lines:?}"
            );
        }
    }

    #[tokio::test]
    async fn resident_search_and_list_reject_schema_pending_and_external_changes() {
        for (change, message) in [
            (
                format!("PRAGMA user_version={}", SCHEMA_VERSION + 1),
                "store schema",
            ),
            (
                "INSERT INTO sync_state VALUES('pending',1)".into(),
                "publication incomplete",
            ),
            (
                "INSERT INTO sync_remotes VALUES('peer','ssh://peer',NULL)".into(),
                "reopen",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("resident.db");
            let mut store = Store::open(&path).await.unwrap();
            let database = Builder::new_local(&path).build().await.unwrap();
            let writer = database.connect().unwrap();
            writer.execute_batch(&change).await.unwrap();
            let error = store
                .agent("missing", &Context::default())
                .await
                .unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
            let query = listing::ListQuery {
                now: "2026-11-13T00:00:00Z".into(),
                kind: None,
                writer: None,
                scope: None,
                since: None,
                expires_before: None,
            };
            let error = store
                .list_records(&query, None, 10, None)
                .await
                .err()
                .unwrap();
            assert!(error.to_string().contains(message), "{error}");
        }
    }

    #[tokio::test]
    async fn pending_sync_and_source_reads_share_a_pinned_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.db");
        let database = Builder::new_local(&path).build().await.unwrap();
        let db = database.connect().unwrap();
        migrations::apply(&db).await.unwrap();
        // WAL lets the second connection commit while the reader retains its snapshot.
        db.execute_batch("PRAGMA journal_mode=WAL;").await.unwrap();
        let writer = database.connect().unwrap();
        let (snapshot, _) = Store::source_snapshot(&db, false).await.unwrap();
        writer
            .execute("INSERT INTO sync_state VALUES('pending',1)", ())
            .await
            .unwrap();
        let mut rows = snapshot
            .query("SELECT count(*) FROM sync_state", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
        drop(rows);
        snapshot.rollback().await.unwrap();
        // A commit before the next snapshot must not be accepted as a clean source version.
        let error = Store::source_snapshot(&db, false).await.err().unwrap();
        assert!(error.to_string().contains("index publication incomplete"));
        // Recovery can deliberately open that same pending snapshot.
        let (snapshot, _) = Store::source_snapshot(&db, true).await.unwrap();
        let mut rows = snapshot
            .query("SELECT count(*) FROM sync_state", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            1
        );
        drop(rows);
        snapshot.rollback().await.unwrap();
    }
}

fn writer_fts_statement(
    stem: bool,
    expression: &str,
    writer: &str,
) -> (String, Vec<libsql::Value>) {
    let table = if stem { "stem" } else { "fts" };
    (
        format!(
            "SELECT rowid FROM {table} WHERE {table} MATCH ? AND rowid IN (SELECT b.id FROM records w INDEXED BY records_writer_created JOIN nav_blocks b ON b.record=w.id WHERE w.writer=?)"
        ),
        vec![expression.into(), writer.into()],
    )
}
