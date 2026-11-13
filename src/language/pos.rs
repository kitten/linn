//! Original-byte linguistic relation selection and guarded publication.
//! @ref LLP 0002#pos-relation-selection
//! @ref LLP 0001#pos-publication
use crate::{
    error::Result,
    retrieval::ordered::{Pair, units},
    storage::Store,
};
fn tokenizer() -> Result<&'static crate::nlp::Tokenizer> {
    use std::sync::OnceLock;
    static TOKENIZER: OnceLock<std::result::Result<crate::nlp::Tokenizer, String>> =
        OnceLock::new();
    TOKENIZER
        .get_or_init(|| {
            let started = std::time::Instant::now();
            // The build verifies the pinned source digest before compilation.
            let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/english.nlp"));
            let result =
                crate::nlp::Tokenizer::from_compiled(bytes.as_slice()).map_err(|e| e.to_string());
            pos_timing("tagger-deserialization", started.elapsed());
            result
        })
        .as_ref()
        .map_err(|e| crate::error::Error::Invalid(e.clone()))
}

fn nominal(tags: &[&str]) -> bool {
    tags.iter()
        .any(|p| matches!(*p, "NN" | "NN:U" | "NN:UN" | "NNS" | "NNP" | "NNPS"))
}
fn modifier(tags: &[&str]) -> bool {
    nominal(tags) || tags.iter().any(|p| matches!(*p, "JJ" | "JJR" | "JJS"))
}
impl Store {
    fn pair_lookup_identity(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&(POS_IDENTITY, &self.structure_stamp))?)
    }

    async fn pair_lookup_ready(&self) -> Result<bool> {
        let mut rows = self
            .db
            .query(
                "SELECT value FROM projections WHERE key='pos-pair-lookup-v1'",
                (),
            )
            .await?;
        Ok(match rows.next().await? {
            Some(row) => row.get::<Vec<u8>>(0)? == self.pair_lookup_identity()?,
            None => false,
        })
    }

    // @ref LLP 0001#indexed-pos-preflight
    pub(crate) async fn prepare_pos_pair_lookup(&self) -> Result<()> {
        if self.pair_lookup_ready().await? {
            return Ok(());
        }
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        let mut rows = tx.query(
            "SELECT count(*) FROM pos_relations p JOIN sections s ON s.id=p.block AND s.record=p.record JOIN records r ON r.id=p.record AND r.body_hash=p.body_hash WHERE p.identity=? AND p.generation=?",
            libsql::params![POS_IDENTITY, self.structure_stamp.clone()],
        ).await?;
        let valid = rows.next().await?.unwrap().get::<i64>(0)? as usize;
        drop(rows);
        let mut rows = tx.query("SELECT count(*) FROM pos_relations", ()).await?;
        let count = rows.next().await?.unwrap().get::<i64>(0)? as usize;
        drop(rows);
        if valid != self.block_count() || count != valid {
            tx.rollback().await?;
            return Ok(());
        }
        tx.execute("DELETE FROM pos_pair_lookup", ()).await?;
        tx.execute("INSERT OR IGNORE INTO pos_pair_lookup SELECT json_extract(r.value,'$.left'),json_extract(r.value,'$.right'),p.block FROM pos_relations p,json_each(CAST(p.data AS TEXT)) u,json_each(u.value,'$[2]') r", ()).await?;
        tx.execute(
            "INSERT OR REPLACE INTO projections(key,value) VALUES('pos-pair-lookup-v1',?)",
            [self.pair_lookup_identity()?],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    // Absence is conclusive only for a complete, current source publication.
    pub(crate) async fn pos_pairs_possible(&self, tokens: &[Vec<String>]) -> Result<bool> {
        let started = std::time::Instant::now();
        if !tokens
            .windows(2)
            .any(|p| !p[0].is_empty() && !p[1].is_empty())
        {
            pos_timing("tagger-skipped-no-query-pair", started.elapsed());
            return Ok(false);
        }
        if !self.pair_lookup_ready().await? {
            return Ok(true);
        }
        let keys: Vec<_> = tokens
            .windows(2)
            .filter(|p| !p[0].is_empty() && !p[1].is_empty())
            .map(|p| Ok((serde_json::to_string(&p[0])?, serde_json::to_string(&p[1])?)))
            .collect::<Result<_>>()?;
        let mut rows = self.db.query(
            "SELECT 1 FROM json_each(?) q WHERE EXISTS(SELECT 1 FROM pos_pair_lookup WHERE left_key=json_extract(q.value,'$[0]') AND right_key=json_extract(q.value,'$[1]')) LIMIT 1",
            [serde_json::to_string(&keys)?],
        ).await?;
        if rows.next().await?.is_some() {
            return Ok(true);
        }
        pos_timing("tagger-skipped-no-source-pair", started.elapsed());
        Ok(false)
    }

    // FTS coordinates and NLP byte coordinates are independently tokenized and
    // reconciled in full. Any split disagreement conservatively rejects the unit.
    pub(crate) async fn nominal_relations(
        &self,
        text: &str,
    ) -> Result<Vec<(Vec<String>, Vec<String>, usize)>> {
        let started = std::time::Instant::now();
        let relations = self.nominal_annotations(text).await?;
        pos_timing("query-annotation", started.elapsed());
        Ok(relations
            .into_iter()
            .map(|r| (r.left, r.right, 0))
            .collect())
    }
    async fn nominal_annotations(&self, text: &str) -> Result<Vec<NominalRelation>> {
        let mut out = Vec::new();
        for (sentence_id, sentence) in tokenizer()?.pipe(text).enumerate() {
            let ts = sentence.tokens();
            let surfaces: Vec<_> = ts.iter().map(|t| &text[t.span().byte().clone()]).collect();
            let sentence_start = ts.first().map(|t| t.span().byte().start).unwrap_or(0);
            let sentence_end = ts.last().map(|t| t.span().byte().end).unwrap_or(0);
            let mut inputs = surfaces.clone();
            inputs.push(&text[sentence_start..sentence_end]);
            let mut streams = self.occurrence_tokens(&inputs).await?;
            let whole = streams.pop().unwrap();
            if streams.iter().flatten().ne(whole.iter()) {
                continue;
            }
            for i in 0..ts.len().saturating_sub(1) {
                let a = &ts[i];
                let b = &ts[i + 1];
                let at: Vec<_> = a.word().tags().iter().map(|t| t.pos().as_str()).collect();
                let bt: Vec<_> = b.word().tags().iter().map(|t| t.pos().as_str()).collect();
                let same_np = a.chunks().iter().any(|ac| {
                    let ac = ac.to_string();
                    let Some(kind) = ac.strip_prefix("B-").or_else(|| ac.strip_prefix("I-")) else {
                        return false;
                    };
                    let label = |index: usize, prefix: &str| {
                        ts[index].chunks().iter().any(|c| {
                            let value = c.to_string();
                            value == format!("{prefix}-{kind}")
                        })
                    };
                    let mut begin = i;
                    while begin > 0 && !label(begin, "B") && label(begin, "I") {
                        begin -= 1;
                    }
                    let mut end = i + 1;
                    while end + 1 < ts.len() && !label(end, "E") && label(end, "I") {
                        end += 1;
                    }
                    label(begin, "B")
                        && label(end, "E")
                        && kind.starts_with("NP-")
                        && b.chunks().iter().any(|bc| {
                            let bc = bc.to_string();
                            bc == format!("I-{kind}") || bc == format!("E-{kind}")
                        })
                });
                let protected_atom = |span: &std::ops::Range<usize>| {
                    let start = text[..span.start]
                        .char_indices()
                        .rfind(|(_, c)| c.is_whitespace())
                        .map_or(0, |(i, c)| i + c.len_utf8());
                    let end = text[span.end..]
                        .find(char::is_whitespace)
                        .map_or(text.len(), |i| span.end + i);
                    let atom =
                        text[start..end].trim_matches(|c: char| "\"'`()[]{}.,;:!?".contains(c));
                    atom.contains(['_', '-', '.', '/', '\\', '@', '=', ':', '#'])
                        || crate::retrieval::query::CAMEL.is_match(atom)
                };
                if protected_atom(a.span().byte())
                    || protected_atom(b.span().byte())
                    || !modifier(&at)
                    || !nominal(&bt)
                    || !same_np
                    || !text[a.span().byte().end..b.span().byte().start]
                        .chars()
                        .all(char::is_whitespace)
                    || surfaces[i..=i + 1].iter().any(|s| {
                        s.contains(['_', '-', '`']) || crate::retrieval::query::CAMEL.is_match(s)
                    })
                    || text[..a.span().byte().start].matches('`').count() % 2 == 1
                    || streams[i].len() != 1
                    || streams[i + 1].len() != 1
                {
                    continue;
                }
                if crate::diagnostics::enabled("LINN_POS_TRACE") {
                    crate::diagnostics::emit(format!(
                        "{}",
                        serde_json::json!({"pos_relation": [surfaces[i],surfaces[i+1]], "bytes":[a.span().byte().start,b.span().byte().end],"tags":[at,bt],"reason":"adjacent possible modifier/nominal NP"})
                    ));
                }
                out.push(NominalRelation {
                    left: streams[i].clone(),
                    right: streams[i + 1].clone(),
                    span: a.span().byte().start..b.span().byte().end,
                    sentence: sentence_id,
                    analyses: [
                        at.iter().map(|s| s.to_string()).collect(),
                        bt.iter().map(|s| s.to_string()).collect(),
                    ],
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod pos_mapping_tests {
    use super::*;
    #[test]
    fn all_public_analyses_remain_alternatives() {
        assert!(nominal(&["UNKNOWN", "VB", "NN:UN"]));
        assert!(nominal(&["NN:U"]));
        assert!(nominal(&["NNS"]));
        assert!(modifier(&["JJ", "VB"]));
        assert!(!nominal(&["UNKNOWN", "SENT_END", "JJ"]));
        assert!(!modifier(&["UNKNOWN", "VB"]));
    }
}

#[cfg(test)]
mod actual_pos_tests {
    use super::*;
    #[tokio::test]
    async fn sql_coordinates_and_public_pipeline() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::build(&dir.path().join("pos.db"), vec![])
            .await
            .unwrap();
        let relations = store
            .nominal_relations("cache invalidation policy")
            .await
            .unwrap();
        assert!(!relations.is_empty());
        for space in ["\u{00a0}", "\u{2003}", "\u{3000}"] {
            assert_eq!(
                store
                    .nominal_relations(&format!("{space}cache invalidation policy"))
                    .await
                    .unwrap(),
                relations
            );
        }
        assert!(
            relations
                .iter()
                .all(|(a, b, _)| a.len() == 1 && b.len() == 1)
        );
        assert!(
            store
                .nominal_relations("cache. Policy.")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .nominal_relations("foo_bar alpha-beta")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .nominal_relations("cache.json policy")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .nominal_relations("read/local cache")
                .await
                .unwrap()
                .is_empty()
        );
        for text in [
            "résumé policy",
            "cafe\u{301} policy",
            "worker’s café\r\nsettings",
        ] {
            let _ = store.nominal_relations(text).await.unwrap();
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NominalRelation {
    left: Vec<String>,
    right: Vec<String>,
    span: std::ops::Range<usize>,
    sentence: usize,
    analyses: [Vec<String>; 2],
}
const POS_IDENTITY: &str = "nlprule-0.6.4:e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773:english-nominal-v2-protected-atoms:markdown-units-v1:sqlite-unicode61-v1";
impl Store {
    pub(crate) async fn publish_pos(&mut self) -> Result<()> {
        self.prepare_pos().await.map(|_| ())
    }
    async fn prepare_pos(&mut self) -> Result<usize> {
        self.materialize_structure().await?;
        let mut prepared = Vec::new();
        let mut reused = 0;
        let mut unit_cache =
            std::collections::HashMap::<&str, std::sync::Arc<Vec<NominalRelation>>>::new();
        let existing = self.db.prepare(
            "SELECT data FROM pos_relations WHERE block=? AND record=? AND body_hash=? AND identity=?",
        ).await?;
        for row in self.rows.values() {
            let source_units = units(&row.text);
            existing.reset();
            let mut previous = existing
                .query(libsql::params![
                    row.id as i64,
                    row.record.clone(),
                    row.hash.clone(),
                    POS_IDENTITY
                ])
                .await?;
            if let Some(previous) = previous.next().await? {
                let data = previous.get::<Vec<u8>>(0)?;
                let annotations: Vec<(usize, usize, Vec<NominalRelation>)> =
                    serde_json::from_slice(&data)?;
                if annotations.len() == source_units.len()
                    && annotations
                        .iter()
                        .zip(&source_units)
                        .all(|((start, end, _), unit)| {
                            let base = unit.as_ptr() as usize - row.text.as_ptr() as usize;
                            *start == row.start + base && *end == row.start + base + unit.len()
                        })
                {
                    reused += 1;
                    continue;
                }
            }

            if crate::diagnostics::enabled("LINN_POS_TIMINGS") && prepared.len() % 1000 == 0 {
                crate::diagnostics::emit(format!(
                    "POS annotation: {}/{} blocks",
                    prepared.len(),
                    self.rows.len()
                ));
            }
            let mut annotations = Vec::new();
            for unit in source_units {
                let base = unit.as_ptr() as usize - row.text.as_ptr() as usize;
                if self.records[&row.record]
                    .body
                    .get(row.start + base..row.start + base + unit.len())
                    != Some(unit)
                {
                    return Err(crate::error::Error::Invalid(
                        "POS source base does not address original body bytes".into(),
                    ));
                }
                let relations = if let Some(cached) = unit_cache.get(unit) {
                    cached.clone()
                } else {
                    let relations = std::sync::Arc::new(self.nominal_annotations(unit).await?);
                    unit_cache.insert(unit, relations.clone());
                    relations
                };
                annotations.push((row.start + base, row.start + base + unit.len(), relations));
            }
            prepared.push((
                row.id,
                row.record.clone(),
                row.hash.clone(),
                serde_json::to_vec(&annotations)?,
            ));
        }
        drop(unit_cache);
        drop(existing);
        let tx = self
            .db
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;
        self.verify_source_version(&tx).await?;
        tx.execute(
            "DELETE FROM pos_relations WHERE block NOT IN (SELECT id FROM sections)",
            (),
        )
        .await?;
        for (block, record, hash, data) in prepared {
            tx.execute("DELETE FROM pos_relations WHERE block=?", [block as i64])
                .await?;
            tx.execute(
                "INSERT INTO pos_relations VALUES(?,?,?,?,?,?)",
                libsql::params![
                    block as i64,
                    record,
                    hash,
                    POS_IDENTITY,
                    self.structure_stamp.clone(),
                    data
                ],
            )
            .await?;
        }
        tx.execute(
            "UPDATE pos_relations SET generation=?",
            [self.structure_stamp.clone()],
        )
        .await?;
        tx.execute(
            "INSERT OR REPLACE INTO projections(key,value) VALUES('pos-pair-lookup-v1',?)",
            [self.pair_lookup_identity()?],
        )
        .await?;
        tx.commit().await?;
        Ok(reused)
    }
    pub(crate) async fn projected_pos_bonuses(
        &self,
        ids: &[usize],
        pairs: &[Pair],
    ) -> Result<std::collections::HashMap<usize, f64>> {
        let mut scores = std::collections::HashMap::new();
        if ids.is_empty() || pairs.is_empty() {
            return Ok(scores);
        }
        if !self.pair_lookup_ready().await? {
            for &id in ids {
                scores.insert(id, self.projected_pos_bonus(id, pairs).await?);
            }
            return Ok(scores);
        }
        let keys: Vec<_> = pairs
            .iter()
            .map(|(a, b, _)| Ok((serde_json::to_string(a)?, serde_json::to_string(b)?)))
            .collect::<Result<_>>()?;
        let keys = serde_json::to_string(&keys)?;
        for chunk in ids.chunks(256) {
            let sources: Vec<_> = chunk.iter().map(|id| (*id, &self.rows[id].hash)).collect();
            let mut rows = self.db.query(
                "SELECT p.block,CAST(q.key AS INTEGER),EXISTS(SELECT 1 FROM pos_pair_lookup l WHERE l.left_key=json_extract(q.value,'$[0]') AND l.right_key=json_extract(q.value,'$[1]') AND l.block=p.block) FROM json_each(?1) s JOIN pos_relations p ON p.block=json_extract(s.value,'$[0]') AND p.body_hash=json_extract(s.value,'$[1]'),json_each(?2) q WHERE p.identity=?3 AND p.generation=?4 ORDER BY p.block,CAST(q.key AS INTEGER)",
                libsql::params![serde_json::to_string(&sources)?,keys.clone(),POS_IDENTITY,self.structure_stamp.clone()],
            ).await?;
            let mut count = 0;
            while let Some(row) = rows.next().await? {
                count += 1;
                let score = scores.entry(row.get::<i64>(0)? as usize).or_insert(0.);
                if row.get::<i64>(2)? != 0 {
                    *score += pairs[row.get::<i64>(1)? as usize].2;
                }
            }
            if count != chunk.len() * pairs.len() {
                return Err(crate::error::Error::Invalid(
                    "POS projection unavailable or stale; run index:rebuild".into(),
                ));
            }
        }
        if crate::diagnostics::enabled("LINN_POS_TRACE") {
            for &id in ids {
                let witness = self.projected_pos_bonus(id, pairs).await?;
                if f64::to_bits(scores[&id]) != witness.to_bits() {
                    return Err(crate::error::Error::Invalid(
                        "POS batch differs from stored witnesses".into(),
                    ));
                }
            }
        }
        Ok(scores)
    }
    pub(crate) async fn projected_pos_bonus(&self, id: usize, pairs: &[Pair]) -> Result<f64> {
        let started = std::time::Instant::now();
        let row = &self.rows[&id];
        let mut rows=self.db.query("SELECT data FROM pos_relations WHERE block=? AND body_hash=? AND identity=? AND generation=?",libsql::params![id as i64,row.hash.clone(),POS_IDENTITY,self.structure_stamp.clone()]).await?;
        let Some(stored) = rows.next().await? else {
            return Err(crate::error::Error::Invalid(
                "POS projection unavailable or stale; run index:rebuild".into(),
            ));
        };
        let data = stored.get::<Vec<u8>>(0)?;
        pos_timing("candidate-io", started.elapsed());
        let started = std::time::Instant::now();
        let annotations: Vec<(usize, usize, Vec<NominalRelation>)> = serde_json::from_slice(&data)?;
        let mut score = 0.;
        for (a, b, idf) in pairs {
            let witness = annotations.iter().find_map(|(base, _end, rs)| {
                rs.iter()
                    .find(|r| &r.left == a && &r.right == b)
                    .map(|r| (*base, r))
            });
            if let Some((base, r)) = witness {
                score += idf;
                if crate::diagnostics::enabled("LINN_POS_TRACE") {
                    crate::diagnostics::emit(format!(
                        "{}",
                        serde_json::json!({"stage":"stored_source_relation","record":row.record,"body_hash":row.hash,"block":id,"generation":self.structure_stamp,"identity":POS_IDENTITY,"unit_base":base,"relation":r,"bonus":idf})
                    ));
                }
            }
        }
        pos_timing("group-decode-match", started.elapsed());
        Ok(score)
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;
    #[tokio::test]
    async fn repeated_units_preserve_each_original_base_and_annotation() {
        let dir = tempfile::tempdir().unwrap();
        let records = [
            ("a", "# Alpha\n\ncafé cache invalidation policy.\n\n## More\n\ncafé cache invalidation policy."),
            ("b", "# Different heading\n\ncafé cache invalidation policy."),
        ].into_iter().map(|(id, body)| serde_json::from_value(serde_json::json!({"id":id,"body":body,"state":"accepted"})).unwrap()).collect();
        let store = Store::build(&dir.path().join("units.db"), records)
            .await
            .unwrap();
        let mut seen = std::collections::HashSet::new();
        let mut repeated = false;
        for row in store.rows.values() {
            let mut expected = Vec::new();
            for unit in units(&row.text) {
                repeated |= !seen.insert(unit);
                let base = row.start + unit.as_ptr() as usize - row.text.as_ptr() as usize;
                expected.push((
                    base,
                    base + unit.len(),
                    store.nominal_annotations(unit).await.unwrap(),
                ));
            }
            let mut stored = store
                .db
                .query(
                    "SELECT data FROM pos_relations WHERE block=?",
                    [row.id as i64],
                )
                .await
                .unwrap();
            let actual = stored
                .next()
                .await
                .unwrap()
                .unwrap()
                .get::<Vec<u8>>(0)
                .unwrap();
            assert_eq!(actual, serde_json::to_vec(&expected).unwrap());
        }
        assert!(repeated);
    }

    #[tokio::test]
    async fn unchanged_annotations_survive_forget_but_not_identity_or_span_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse.db");
        let records = ["r", "s"]
            .into_iter()
            .map(|id| {
                serde_json::from_value(
                    serde_json::json!({"id":id,"body":"cache invalidation policy"}),
                )
                .unwrap()
            })
            .collect();
        let mut store = Store::build(&path, records).await.unwrap();
        assert_eq!(store.prepare_pos().await.unwrap(), 2);
        store
            .db
            .execute(
                "UPDATE pos_relations SET identity='old' WHERE record='r'",
                (),
            )
            .await
            .unwrap();
        assert_eq!(store.prepare_pos().await.unwrap(), 1);
        store.db.execute("UPDATE pos_relations SET data=CAST(json_set(CAST(data AS TEXT),'$[0][0]',999) AS BLOB) WHERE record='r'", ()).await.unwrap();
        assert_eq!(store.prepare_pos().await.unwrap(), 1);
        let tx = store.db.transaction().await.unwrap();
        tx.execute("DELETE FROM records WHERE id='r'", ())
            .await
            .unwrap();
        let mut rows = tx
            .query("SELECT record FROM pos_relations", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "s"
        );
        assert!(rows.next().await.unwrap().is_none());
        drop(rows);
        tx.rollback().await.unwrap();
        assert_eq!(store.prepare_pos().await.unwrap(), 2);
        let mut rows = store
            .db
            .query("SELECT data FROM pos_relations WHERE record='s'", ())
            .await
            .unwrap();
        let before = rows
            .next()
            .await
            .unwrap()
            .unwrap()
            .get::<Vec<u8>>(0)
            .unwrap();
        drop(rows);
        store.forget("r").await.unwrap();
        assert_eq!(store.prepare_pos().await.unwrap(), 1);
        let mut rows = store
            .db
            .query("SELECT record,data,generation FROM pos_relations", ())
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<String>(0).unwrap(), "s");
        assert_eq!(row.get::<Vec<u8>>(1).unwrap(), before);
        assert_eq!(row.get::<Vec<u8>>(2).unwrap(), store.structure_stamp);
        assert!(rows.next().await.unwrap().is_none());
    }
    #[tokio::test]
    async fn publication_forget_and_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("projection.db");
        let record = serde_json::from_value(
            serde_json::json!({"id":"r","state":"accepted","body":"cache invalidation policy"}),
        )
        .unwrap();
        let mut store = Store::build(&path, vec![record]).await.unwrap();
        store.prepare_pos().await.unwrap();
        let pairs = vec![(vec!["cache".into()], vec!["invalidation".into()], 2.)];
        let expected = store.projected_pos_bonus(1, &pairs).await.unwrap();
        assert!(expected > 0.);
        assert_eq!(
            store.projected_pos_bonuses(&[1], &pairs).await.unwrap()[&1].to_bits(),
            expected.to_bits()
        );
        let repeated: Vec<_> = [1e16, 1., -1e16]
            .into_iter()
            .map(|weight| (pairs[0].0.clone(), pairs[0].1.clone(), weight))
            .collect();
        assert_eq!(
            store.projected_pos_bonuses(&[1], &repeated).await.unwrap()[&1].to_bits(),
            store
                .projected_pos_bonus(1, &repeated)
                .await
                .unwrap()
                .to_bits()
        );
        let present = vec![vec!["cache".into()], vec!["invalidation".into()]];
        let absent = vec![vec!["nonexistent".into()], vec!["relation".into()]];
        assert!(store.pos_pairs_possible(&present).await.unwrap());
        assert!(!store.pos_pairs_possible(&absent).await.unwrap());
        let tx = store.db.transaction().await.unwrap();
        tx.execute("DELETE FROM pos_relations", ()).await.unwrap();
        tx.rollback().await.unwrap();
        assert!(!store.pos_pairs_possible(&absent).await.unwrap());
        // Older publications gain the lookup without invoking the tagger.
        store.db.execute_batch("DELETE FROM pos_pair_lookup; DELETE FROM projections WHERE key='pos-pair-lookup-v1';").await.unwrap();
        assert!(store.pos_pairs_possible(&absent).await.unwrap());
        store.prepare_pos_pair_lookup().await.unwrap();
        assert!(store.pos_pairs_possible(&present).await.unwrap());
        assert!(!store.pos_pairs_possible(&absent).await.unwrap());
        store
            .db
            .execute("UPDATE pos_relations SET identity='wrong'", ())
            .await
            .unwrap();
        assert!(store.projected_pos_bonus(1, &pairs).await.is_err());
        assert!(store.pos_pairs_possible(&absent).await.unwrap());
        store.prepare_pos_pair_lookup().await.unwrap();
        assert!(store.pos_pairs_possible(&absent).await.unwrap());
        assert!(
            !store
                .pos_pairs_possible(&[vec!["cache".into()]])
                .await
                .unwrap()
        );
        store.prepare_pos().await.unwrap();
        let mut stale = Store::open(&path).await.unwrap();
        store.forget("r").await.unwrap();
        assert!(
            stale
                .prepare_pos()
                .await
                .unwrap_err()
                .to_string()
                .contains("reopen")
        );
        let mut rows = store
            .db
            .query("SELECT count(*) FROM pos_relations", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
        let mut rows = store
            .db
            .query("SELECT count(*) FROM pos_pair_lookup", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
    }
}

fn pos_timing(phase: &str, duration: std::time::Duration) {
    if crate::diagnostics::enabled("LINN_POS_TIMINGS") {
        crate::diagnostics::emit(format!(
            "POS timing: {}",
            serde_json::json!({"phase":phase,"ms":duration.as_secs_f64()*1000.})
        ));
    }
}

pub(crate) fn nominal_compound_parts<'a>(
    text: &'a str,
    quoted: bool,
    raw: &str,
) -> Option<(&'a str, &'a str)> {
    if quoted || raw.contains(&format!("\u{60}{text}\u{60}")) {
        return None;
    }
    let (a, b) = text.split_once('-')?;
    if [a, b]
        .iter()
        .all(|p| p.len() >= 2 && p.bytes().all(|c| c.is_ascii_lowercase()))
    {
        Some((a, b))
    } else {
        None
    }
}
fn compound_stem(s: &str) -> Result<String> {
    crate::nlp::porter::word(s).map_err(|e| crate::error::Error::Invalid(e.to_string()))
}
fn compound_source(rs: &[NominalRelation], a: &str, b: &str) -> Result<bool> {
    for r in rs {
        if r.left.len() == 1
            && r.right.len() == 1
            && compound_stem(&r.left[0])? == a
            && compound_stem(&r.right[0])? == b
        {
            return Ok(true);
        }
    }
    Ok(false)
}
impl Store {
    pub(crate) async fn nominal_compound_sources(
        &self,
        ids: &[usize],
        a: &str,
        b: &str,
    ) -> Result<std::collections::HashSet<usize>> {
        let mut out = std::collections::HashSet::new();
        let a = compound_stem(a)?;
        let b = compound_stem(b)?;
        for chunk in ids.chunks(256) {
            let keys: Vec<_> = chunk.iter().map(|id| (*id, &self.rows[id].hash)).collect();
            let mut rows=self.db.query(
                "SELECT p.block,p.data FROM json_each(?1) s JOIN pos_relations p ON p.block=json_extract(s.value,'$[0]') AND p.body_hash=json_extract(s.value,'$[1]') WHERE p.identity=?2 AND p.generation=?3 ORDER BY p.block",
                libsql::params![serde_json::to_string(&keys)?,POS_IDENTITY,self.structure_stamp.clone()],
            ).await?;
            let mut count = 0;
            while let Some(row) = rows.next().await? {
                count += 1;
                let data = row.get::<Vec<u8>>(1)?;
                let units: Vec<(usize, usize, Vec<NominalRelation>)> =
                    serde_json::from_slice(&data)?;
                for (_, _, rs) in units {
                    if compound_source(&rs, &a, &b)? {
                        out.insert(row.get::<i64>(0)? as usize);
                        break;
                    }
                }
            }
            if count != chunk.len() {
                return Err(crate::error::Error::Invalid(
                    "POS projection unavailable or stale; run index:rebuild".into(),
                ));
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod nominal_compound_contract {
    use super::*;
    #[tokio::test]
    async fn complete_typed_original_pair_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::build(&dir.path().join("nominal.db"), vec![])
            .await
            .unwrap();
        assert_eq!(
            nominal_compound_parts("local-cache", false, "local-cache"),
            Some(("local", "cache"))
        );
        for term in [
            "local_cache",
            "local-cache-key",
            "Local-cache",
            "local-2",
            "--local-cache",
        ] {
            assert!(nominal_compound_parts(term, false, term).is_none());
        }
        assert!(nominal_compound_parts("local-cache", true, "\"local-cache\"").is_none());
        assert!(nominal_compound_parts("local-cache", false, "\u{60}local-cache\u{60}").is_none());
        assert!(
            !store
                .nominal_relations("local cache")
                .await
                .unwrap()
                .is_empty()
        );
        let a = compound_stem("local").unwrap();
        let b = compound_stem("cache").unwrap();
        let source = store
            .nominal_annotations("The local caches are ready.")
            .await
            .unwrap();
        assert!(compound_source(&source, &a, &b).unwrap());
        let source = store
            .nominal_annotations("The cache is local.")
            .await
            .unwrap();
        assert!(!compound_source(&source, &a, &b).unwrap());
        let source = store
            .nominal_annotations("The local system exists. A cache is ready.")
            .await
            .unwrap();
        assert!(!compound_source(&source, &a, &b).unwrap());
    }
}
