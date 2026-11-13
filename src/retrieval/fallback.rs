//! @ref LLP 0002#ordered-fallbacks
use crate::{
    error::Result,
    language::embedding::{dot, unit},
    retrieval::lexical::{Rank, card_lines},
    retrieval::query::{self},
    storage::{Call, Card, Context, Store},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

pub(crate) const SENTENCE_DICTIONARY: &str = "sentences-dictionary-v1";
#[derive(Serialize, Deserialize)]
struct SentenceDictionary {
    blocks: Vec<usize>,
    order: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Form {
    text: String,
    pieces: Vec<String>,
    rows: Vec<usize>,
    df: usize,
    order: usize,
}
pub(crate) struct Forms {
    metadata: Vec<Form>,
    matrix: std::sync::OnceLock<crate::retrieval::similarity::Matrix>,
}
impl std::ops::Deref for Forms {
    type Target = [Form];
    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}
impl Store {
    async fn checklist(&mut self, card: &Card, q: &str, hyphens: bool) -> Result<bool> {
        let Some(b) = self.rows.get(&card.section) else {
            return Ok(false);
        };
        let words = query::surface(&format!("{} {}", b.heading, b.text));
        let mut probes = words.clone();
        if hyphens {
            probes.extend(
                words
                    .iter()
                    .flat_map(|w| w.split('-'))
                    .filter(|p| p.chars().count() > 2 && p.chars().all(char::is_alphabetic))
                    .map(str::to_owned),
            );
        }
        probes.extend(query::content(q));
        self.prepare_porter(probes).await?;
        let mut stems = HashSet::new();
        for w in words {
            stems.insert(self.porter(&w).await?);
            if hyphens && w.contains('-') {
                for p in w
                    .split('-')
                    .filter(|p| p.chars().count() > 2 && p.chars().all(char::is_alphabetic))
                {
                    stems.insert(self.porter(p).await?);
                }
            }
        }
        let need = query::content(q);
        if need.is_empty() {
            return Ok(false);
        }
        for w in need {
            if !stems.contains(&self.porter(&w).await?) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    async fn gated(
        &mut self,
        calls: Vec<Call>,
        q: &str,
        a: &[Rank],
        b: &[Rank],
    ) -> Result<Vec<Call>> {
        let mut calls = calls;
        let mut kept = Vec::new();
        if let Some(call) = calls.first() {
            for c in &call.cards {
                if self.checklist(c, q, true).await?
                    || a.iter().chain(b).any(|r| r.sid == c.section && r.tier <= 1)
                    || self.rankings.iter().any(|((text, _, _, _), rows)| {
                        text == q && rows.iter().any(|r| r.sid == c.section && r.tier <= 1)
                    })
                {
                    kept.push(c.clone());
                }
            }
        }
        if kept.is_empty() {
            return Ok(vec![]);
        }
        calls[0].text = card_lines(&kept, false);
        calls[0].cards = kept;
        Ok(calls)
    }
    async fn rewrites(&mut self, raw: &str, ctx: &Context) -> Result<Vec<Call>> {
        let camel = Regex::new(r"^[A-Za-z][A-Za-z0-9]*[a-z][A-Z][A-Za-z0-9]*$").unwrap();
        let boundary = Regex::new(r"([a-z0-9])([A-Z])").unwrap();
        let q = query::rewrite(raw, |t| {
            if !t.contains(':') && camel.is_match(t) && !self.df.contains_key(&t.to_lowercase()) {
                boundary.replace_all(t, "$1 $2").to_string()
            } else {
                t.to_owned()
            }
        });
        let (calls, _, _) = self.base_search(&q, ctx).await?;
        if has_cards(&calls) {
            return Ok(calls);
        }
        if let Some((literal_query, atoms)) = colon_literal_query(&q) {
            let literal_query = self.qualified_suffix_query(&literal_query, ctx).await?;
            let (res, a, b) = self.base_search(&literal_query, ctx).await?;
            let mut kept = self.gated(res, &literal_query, &a, &b).await?;
            for call in &mut kept {
                call.cards.retain(|card| {
                    self.rows.get(&card.section).is_some_and(|row| {
                        atoms
                            .iter()
                            .all(|atom| colon_literal_present(&row.text, atom))
                    })
                });
                call.text = card_lines(&call.cards, false);
            }
            if has_cards(&kept) {
                return Ok(kept);
            }
        }
        let compound = Regex::new(r"^[A-Za-z]{2,}-[A-Za-z]{2,}$").unwrap();
        let rewrite = query::rewrite(&q, |t| {
            if compound.is_match(t) {
                t.replace('-', " ")
            } else {
                t.into()
            }
        });
        if rewrite != q {
            let (res, a, b) = self.base_search(&rewrite, ctx).await?;
            let kept = self.gated(res, &rewrite, &a, &b).await?;
            if has_cards(&kept) {
                return Ok(kept);
            }
        }
        let encoder = self.encoder()?;
        let rewrite = query::rewrite(&q, |t| {
            if t.contains(':') {
                return t.into();
            }
            let pieces = encoder.pieces(t);
            if pieces.is_empty() {
                t.into()
            } else {
                pieces.join(" ")
            }
        });
        if rewrite != q {
            let (res, a, b) = self.base_search(&rewrite, ctx).await?;
            let kept = self.gated(res, &rewrite, &a, &b).await?;
            if has_cards(&kept) {
                return Ok(kept);
            }
        }
        Ok(calls)
    }
    async fn forms(&mut self) -> Result<std::sync::Arc<Forms>> {
        if let Some(forms) = &self.form_cache {
            return Ok(forms.clone());
        }
        if let Some((forms, rows)) = self.fallback_metadata::<Vec<Form>>("forms").await? {
            if forms.len() != rows {
                return Err(crate::error::Error::Invalid(
                    "invalid form projection".into(),
                ));
            }
            let forms = std::sync::Arc::new(Forms {
                metadata: forms,
                matrix: std::sync::OnceLock::new(),
            });
            self.form_cache = Some(forms.clone());
            return Ok(forms);
        }
        let tricky=Regex::new(r"[A-Za-z]+(?:[_-][A-Za-z0-9]+)+|[A-Za-z][A-Za-z0-9]*[a-z][A-Z][A-Za-z0-9]*|[A-Za-z]*\d[A-Za-z0-9]*").unwrap();
        let mut forms = BTreeMap::<String, (Vec<String>, Vec<usize>, usize)>::new();
        for b in self.rows.values() {
            for m in tricky.find_iter(&format!("{} {}", b.heading, b.text)) {
                let term = m.as_str().to_lowercase();
                if term.len() < 5 {
                    continue;
                }
                let order = forms.len();
                let entry = forms.entry(term).or_insert_with(|| {
                    let mut seen = HashSet::new();
                    (
                        query::pieces(m.as_str())
                            .into_iter()
                            .filter(|p| seen.insert(p.clone()))
                            .collect(),
                        vec![],
                        order,
                    )
                });
                entry.1.push(b.id);
            }
        }
        let texts: Vec<_> = forms.keys().cloned().collect();
        let vectors = if texts.is_empty() {
            Vec::new()
        } else {
            self.encoder()?.batch(&texts)
        };
        let forms = std::sync::Arc::new(Forms {
            matrix: crate::retrieval::similarity::Matrix::owned(vectors).into(),
            metadata: forms
                .into_iter()
                .map(|(text, (pieces, rows, order))| {
                    let df = rows
                        .iter()
                        .map(|id| &self.rows[id].record)
                        .collect::<HashSet<_>>()
                        .len();
                    Form {
                        text,
                        pieces,
                        rows,
                        df,
                        order,
                    }
                })
                .collect(),
        });
        self.form_cache = Some(std::sync::Arc::clone(&forms));
        Ok(forms)
    }
    async fn form_vectors<'a>(
        &self,
        forms: &'a Forms,
    ) -> Result<&'a crate::retrieval::similarity::Matrix> {
        if forms.matrix.get().is_none() {
            let started = std::time::Instant::now();
            let matrix = self.fallback_matrix("forms", forms.len()).await?;
            let _ = forms.matrix.set(matrix);
            if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
                crate::diagnostics::emit(format!(
                    "linn query: {}",
                    serde_json::json!({"phase":"form-vector-read","rows":forms.len(),"ms":started.elapsed().as_secs_f64()*1000.})
                ));
            }
        }
        Ok(forms.matrix.get().unwrap())
    }
    async fn beam_data(&mut self) -> Result<std::sync::Arc<crate::storage::BeamCache>> {
        if self.beam_cache.is_none() {
            let (proposals, matrix) =
                if let Some(cached) = self.fallback_artifact::<Vec<String>>("beam").await? {
                    cached
                } else {
                    self.load_surface_dictionary().await?;
                    let proposals: Vec<_> = self
                        .word_order
                        .iter()
                        .filter(|w| {
                            w.chars().all(char::is_alphabetic)
                                && w.chars().count() > 2
                                && self.idf(w) >= 2.5
                        })
                        .cloned()
                        .collect();
                    let vectors = if proposals.is_empty() {
                        Vec::new()
                    } else {
                        self.encoder()?.batch(&proposals)
                    };
                    let matrix = crate::retrieval::similarity::Matrix::owned(vectors);
                    (proposals, matrix)
                };
            if proposals.len() != matrix.row_count() {
                return Err(crate::error::Error::Invalid(
                    "invalid beam projection".into(),
                ));
            }
            self.beam_cache = Some(std::sync::Arc::new(crate::storage::BeamCache {
                proposals,
                matrix,
            }));
        }
        let cached = self.beam_cache.as_ref().unwrap().clone();
        Ok(cached)
    }
    async fn beam(&mut self, q: &str, ctx: &Context) -> Result<Vec<Card>> {
        let words = query::content(q);
        let text = query::split(q).1;
        if text.trim().is_empty() || words.is_empty() {
            return Ok(vec![]);
        }
        let cached = self.beam_data().await?;
        let proposals = &cached.proposals;
        if proposals.is_empty() {
            return Ok(vec![]);
        }
        let encoder = self.encoder()?;
        let target = encoder.encode(text.trim());
        let index = self.candidate_beam_terms(&words).await?;
        let banned: HashSet<_> = words.iter().filter_map(|w| index.get(w).copied()).collect();
        let mut sum = vec![0.; 512];
        let mut n = 0;
        for w in &words {
            if let Some(&i) = index.get(w) {
                for (a, b) in sum.iter_mut().zip(cached.matrix.row(i)) {
                    *a += *b;
                }
                n += 1;
            }
        }
        let mut beams = vec![(0f32, sum, n, Vec::<usize>::new())];
        for _ in 0..2 {
            let mut candidates = Vec::new();
            for (_, sum, n, extras) in beams {
                let residual = if n > 0 {
                    let mean = unit(sum.iter().map(|v| v / n as f32).collect());
                    target
                        .iter()
                        .zip(mean)
                        .map(|(t, m)| t - m)
                        .collect::<Vec<_>>()
                } else {
                    target.clone()
                };
                let options: Vec<_> = cached
                    .matrix
                    .multiply(&residual)
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        (
                            i,
                            if banned.contains(&i) || extras.contains(&i) {
                                -1e9
                            } else {
                                value
                            },
                        )
                    })
                    .collect();
                let order = crate::retrieval::selection::top(
                    &options.iter().map(|r| r.1).collect::<Vec<_>>(),
                    20,
                );
                for index in order {
                    let (i, score) = options[index];
                    if score < -1e8 {
                        continue;
                    }
                    let next: Vec<_> = sum
                        .iter()
                        .zip(cached.matrix.row(i))
                        .map(|(a, b)| a + b)
                        .collect();
                    let mean = unit(next.iter().map(|v| v / (n + 1) as f32).collect());
                    let mut e = extras.clone();
                    e.push(i);
                    candidates.push((dot(&mean, &target), next, n + 1, e));
                }
            }
            candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
            candidates.truncate(4);
            beams = candidates;
        }
        let Some(best) = beams.first() else {
            return Ok(vec![]);
        };
        let q = best
            .3
            .iter()
            .map(|i| proposals[*i].as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let res = self.rewrites(&q, ctx).await?;
        Ok(res
            .first()
            .into_iter()
            .flat_map(|c| &c.cards)
            .filter(|c| {
                self.rows.get(&c.section).is_some_and(|b| {
                    words.iter().filter(|w| b.surface.contains(*w)).count()
                        >= query::need(words.len())
                })
            })
            .take(5)
            .cloned()
            .collect())
    }
    async fn form_block(&mut self, q: &str, forms: &Forms) -> Result<Option<Card>> {
        let text = query::split(q).1;
        let words: Vec<_> = query::content(q)
            .into_iter()
            .filter(|w| w.chars().count() > 3)
            .collect();
        if text.trim().is_empty() || words.is_empty() {
            return Ok(None);
        }
        self.load_surface(&words).await?;
        if self.rare_blocks(&words, 5.).is_empty() {
            return Ok(None);
        }
        if !forms.iter().any(|f| {
            f.df <= 8
                && words.iter().any(|word| {
                    f.text != *word && f.pieces.contains(word) && f.pieces.iter().any(|p| p != word)
                })
        }) {
            return Ok(None);
        }
        let encoder = self.encoder()?;
        let query_vector = encoder.encode(text.trim());
        let matrix = self.form_vectors(forms).await?;
        let mut claimed = HashSet::new();
        let mut candidates = Vec::new();
        for word in &words {
            let v = encoder.encode(word);
            let options: Vec<_> = matrix.multiply(&v).into_iter().enumerate().collect();
            let order = crate::retrieval::selection::top(
                &options.iter().map(|r| r.1).collect::<Vec<_>>(),
                15,
            );
            for index in order {
                let (i, score) = options[index];
                let f = &forms[i];
                if f64::from(score) < 0.70
                    || !f.pieces.contains(word)
                    || f.text == *word
                    || f.df > 8
                    || f.pieces.iter().all(|p| p == word)
                    || !claimed.insert(i)
                {
                    continue;
                }
                self.load_block_headers(f.rows.iter().copied()).await?;
                let eligible: Vec<_> = f
                    .rows
                    .iter()
                    .copied()
                    .filter(|sid| self.hint_eligible(&self.rows[sid].record))
                    .collect();
                let vectors = self.selected_vectors(&eligible).await?;
                let mut best = None;
                let mut best_cos = -1.;
                for sid in &f.rows {
                    let b = &self.rows[sid];
                    if !self.hint_eligible(&b.record) {
                        continue;
                    }
                    let cos = dot(&vectors[sid], &query_vector);
                    if cos > best_cos {
                        best_cos = cos;
                        best = Some(*sid);
                    }
                }
                if let Some(sid) = best
                    && f64::from(best_cos) >= 0.35
                    && words
                        .iter()
                        .any(|w| self.rows[&sid].surface.contains(w) && self.idf(w) >= 5.)
                {
                    candidates.push((f.df, -score, f.text.clone(), sid));
                }
            }
        }
        candidates.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.total_cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
        });
        self.load_blocks(candidates.first().map(|c| c.3)).await?;
        Ok(candidates.first().map(|c| self.card(c.3, None)))
    }
    async fn form_section(&mut self, q: &str) -> Result<Option<Card>> {
        let text = query::split(q).1;
        let words: Vec<_> = query::content(q)
            .into_iter()
            .filter(|w| w.chars().count() > 3)
            .collect();
        if text.trim().is_empty() || words.is_empty() {
            return Ok(None);
        }
        let candidates = self.rare_blocks(&words, 5.);
        if candidates.is_empty() {
            return Ok(None);
        }
        self.load_block_headers(candidates.iter().copied()).await?;
        let candidates: HashSet<_> = candidates
            .into_iter()
            .filter(|sid| self.hint_eligible(&self.rows[sid].record))
            .collect();
        let means = self.candidate_section_means(&candidates).await?;
        let vector = self.encoder()?.encode(text.trim());
        let mut best = None;
        let mut best_cos = -1.;
        for (sid, mean) in &means {
            let b = &self.rows[sid];
            if !self.hint_eligible(&b.record)
                || !words
                    .iter()
                    .any(|w| b.surface.contains(w) && self.idf(w) >= 5.)
            {
                continue;
            }
            let cos = dot(mean, &vector);
            if f64::from(cos) >= 0.35 && cos > best_cos {
                best_cos = cos;
                best = Some(*sid);
            }
        }
        self.load_blocks(best).await?;
        Ok(best.map(|id| self.card(id, None)))
    }
    async fn sentence_data(&mut self) -> Result<crate::storage::SentenceCache> {
        if self.sentence_cache.is_none() {
            if let Some((metadata, dictionary)) = self
                .fallback_artifact::<SentenceDictionary>(SENTENCE_DICTIONARY)
                .await?
            {
                let matrix = dictionary
                    .expanded(&metadata.order)
                    .filter(|matrix| metadata.blocks.len() == matrix.row_count())
                    .ok_or_else(|| {
                        crate::error::Error::Invalid("invalid sentence dictionary".into())
                    })?;
                self.sentence_cache = Some(std::sync::Arc::new((metadata.blocks, matrix)));
                return Ok(self.sentence_cache.as_ref().unwrap().clone());
            }
            if let Some((sids, matrix)) = self.fallback_artifact::<Vec<usize>>("sentences").await? {
                if sids.len() != matrix.row_count() {
                    return Err(crate::error::Error::Invalid(
                        "invalid sentence projection".into(),
                    ));
                }
                self.sentence_cache = Some(std::sync::Arc::new((sids, matrix)));
                return Ok(self.sentence_cache.as_ref().unwrap().clone());
            }
            let split = Regex::new(r"\n+|[.!?]\s+").unwrap(); // Keep punctuation on the preceding sentence below.
            let mut texts = Vec::new();
            let mut sids = Vec::new();
            for block in self.rows.values() {
                let mut start = 0;
                let mut parts = Vec::new();
                for m in split.find_iter(&block.text) {
                    let end = if block.text.as_bytes()[m.start()] == b'\n' {
                        m.start()
                    } else {
                        m.start() + 1
                    };
                    parts.push(&block.text[start..end]);
                    start = m.end();
                }
                parts.push(&block.text[start..]);
                let kept: Vec<String> = parts
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| s.split_whitespace().count() >= 4)
                    .map(|s| s.chars().take(500).collect())
                    .collect();
                if kept.len() < 2 {
                    continue;
                }
                for part in kept {
                    texts.push(part);
                    sids.push(block.id);
                }
            }
            let vecs = if texts.is_empty() {
                Vec::new()
            } else {
                self.encoder()?.batch(&texts)
            };
            self.sentence_cache = Some(std::sync::Arc::new((
                sids,
                crate::retrieval::similarity::Matrix::owned(vecs),
            )));
        }
        let cache = self.sentence_cache.as_ref().unwrap().clone();
        Ok(cache)
    }
    async fn sentence_pair(&mut self, q: &str) -> Result<Option<Card>> {
        let words = query::content(q);
        if words.len() < 3 {
            return Ok(None);
        }
        let (a, b) = if words.len() == 3 {
            (words[..2].join(" "), words[1..].join(" "))
        } else {
            let mid = words.len() / 2;
            (words[..mid].join(" "), words[mid..].join(" "))
        };
        let cache = self.sentence_data().await?;
        let (sids, vecs) = &*cache;
        if vecs.is_empty() {
            return Ok(None);
        }
        let encoder = self.encoder()?;
        let av = encoder.encode(&a);
        let bv = encoder.encode(&b);
        let scores = vecs.pair(&av, &bv);
        let nearest = |column: usize| {
            crate::retrieval::selection::top(
                &scores.iter().map(|r| r[column]).collect::<Vec<_>>(),
                15,
            )
            .into_iter()
            .map(|i| (i, scores[i][column]))
            .collect::<Vec<_>>()
        };
        let (aa, bb) = (nearest(0), nearest(1));
        let possible: Vec<_> = aa
            .iter()
            .filter(|(_, ac)| f64::from(*ac) >= 0.35)
            .filter(|(i, _)| {
                bb.iter()
                    .any(|(j, bc)| f64::from(*bc) >= 0.35 && i != j && sids[*i] == sids[*j])
            })
            .map(|(i, _)| sids[*i])
            .collect();
        self.load_block_headers(possible).await?;
        let mut best = None;
        let mut score = -1.;
        for (i, ac) in aa {
            if f64::from(ac) < 0.35 {
                continue;
            }
            for (j, bc) in &bb {
                if f64::from(*bc) < 0.35
                    || i == *j
                    || sids[i] != sids[*j]
                    || !self.hint_eligible(&self.rows[&sids[i]].record)
                {
                    continue;
                }
                let cos = ac.min(*bc);
                if cos > score {
                    score = cos;
                    best = Some(sids[i]);
                }
            }
        }
        self.load_blocks(best).await?;
        Ok(best.map(|id| self.card(id, None)))
    }
    pub(crate) async fn prepare_fallback_artifacts(&mut self) -> Result<()> {
        if self.fallback_artifacts_ready().await? {
            if !self.fallback_reads_ready().await? {
                let forms = self.forms().await?;
                let beam = self.beam_data().await?;
                let matrix = self.form_vectors(&forms).await?;
                self.publish_fallback_reads(&section_means(&forms, matrix), &beam.proposals)
                    .await?;
                self.form_cache = None;
                self.beam_cache = None;
            }
            return Ok(());
        }
        if !self.legacy_fallback_artifacts_ready().await? {
            self.materialize_structure().await?;
        }
        let forms = self.forms().await?;
        let beam = self.beam_data().await?;
        let sentences = self.sentence_data().await?;
        let form_matrix = self.form_vectors(&forms).await?;
        let (dictionary, order) = sentences.1.dictionary();
        self.publish_fallback_artifacts(&[
            ("forms", serde_json::to_vec(&forms.metadata)?, form_matrix),
            ("beam", serde_json::to_vec(&beam.proposals)?, &beam.matrix),
            (
                SENTENCE_DICTIONARY,
                serde_json::to_vec(&SentenceDictionary {
                    blocks: sentences.0.clone(),
                    order,
                })?,
                &dictionary,
            ),
        ])
        .await?;
        self.publish_fallback_reads(&section_means(&forms, form_matrix), &beam.proposals)
            .await?;
        self.form_cache = None;
        self.beam_cache = None;
        self.sentence_cache = None;
        Ok(())
    }
    async fn lower_fallbacks(&mut self, q: &str, ctx: &Context) -> Result<Vec<Call>> {
        let mut calls = self.rewrites(q, ctx).await?;
        if has_cards(&calls) {
            return Ok(calls);
        }
        if !calls.is_empty() {
            let cards = self.beam(q, ctx).await?;
            if !cards.is_empty() {
                calls[0].cards = cards;
                return Ok(calls);
            }
        }
        let forms = self.forms().await?;
        if let Some(card) = self.form_block(q, &forms).await? {
            put(&mut calls, vec![card]);
            return Ok(calls);
        }
        if let Some(card) = self.form_section(q).await? {
            put(&mut calls, vec![card]);
            return Ok(calls);
        }
        if let Some(card) = self.sentence_pair(q).await? {
            put(&mut calls, vec![card]);
        }
        Ok(calls)
    }
    pub async fn agent(&mut self, q: &str, ctx: &Context) -> Result<Vec<Call>> {
        let snapshot = self.application_snapshot().await?;
        let result = self.agent_in_snapshot(q, ctx).await;
        let rollback = snapshot.rollback().await;
        match result {
            Ok(calls) => {
                rollback?;
                Ok(calls)
            }
            Err(error) => Err(error),
        }
    }
    /// The caller holds a checked snapshot for the complete application read.
    pub(crate) async fn agent_in_snapshot(&mut self, q: &str, ctx: &Context) -> Result<Vec<Call>> {
        self.begin_query(q, ctx);
        let mut ready = self
            .db
            .query("SELECT value FROM projections WHERE key='structure'", ())
            .await?;
        let stamp = ready
            .next()
            .await?
            .map(|row| row.get::<Vec<u8>>(0))
            .transpose()?;
        if stamp.as_ref() != Some(&self.structure_stamp) {
            return Err(crate::error::Error::Invalid(
                "store changed; reopen it before searching".into(),
            ));
        }
        drop(ready);
        let mut calls = self.agent_inner(q, ctx).await?;
        let ids = calls
            .iter()
            .flat_map(|call| {
                call.cards.iter().map(|c| c.record.clone()).chain(
                    call.hints
                        .iter()
                        .chain(&call.related)
                        .map(|p| p.record.clone()),
                )
            })
            .collect::<Vec<_>>();
        self.load_record_headers(ids).await?;
        for call in &mut calls {
            let count = (call.cards.len(), call.hints.len(), call.related.len());
            call.cards.retain(|card| self.record_eligible(&card.record));
            call.hints.retain(|p| self.hint_eligible(&p.record));
            call.related.retain(|p| self.hint_eligible(&p.record));
            if (call.cards.len(), call.hints.len(), call.related.len()) != count {
                call.text = if call.cards.is_empty() {
                    "Nothing qualified.".into()
                } else {
                    card_lines(&call.cards, false)
                };
            }
        }
        Ok(calls)
    }
    async fn qualified_suffix_query(&mut self, q: &str, ctx: &Context) -> Result<String> {
        let Ok(parsed) = query::Query::parse(q) else {
            return Ok(q.into());
        };
        let symbol = |s: &str| {
            let parts: Vec<_> = s.split('_').collect();
            parts.len() >= 2
                && parts
                    .iter()
                    .all(|p| p.len() >= 3 && p.bytes().all(|b| b.is_ascii_uppercase()))
        };
        let mut probes = HashSet::new();
        query::rewrite(q, |atom| {
            if symbol(atom) {
                probes.insert(atom.to_owned());
            }
            atom.into()
        });
        if probes.is_empty() {
            return Ok(q.into());
        }
        self.load_vocabulary_terms(probes.iter().map(|s| s.to_lowercase()).collect())
            .await?;
        let mut replacements = HashMap::new();
        for atom in probes {
            if self.vocab.contains(&atom.to_lowercase()) {
                continue;
            }
            let expression = atom
                .split('_')
                .map(|p| format!("{{parts}}:{}", query::quote(p)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let ids = self.eligible_fts_hits(false, &expression, &ctx.now).await?;
            self.load_blocks(ids.iter().copied()).await?;
            let suffix = format!("_{atom}");
            let mut names = HashSet::new();
            for id in ids {
                let row = &self.rows[&id];
                if !self.allowed(row, &parsed.filters) {
                    continue;
                }
                for m in query::WORD.find_iter(&row.text) {
                    let name = m.as_str();
                    if symbol(name) && name.ends_with(&suffix) {
                        names.insert(name.to_owned());
                    }
                }
            }
            if names.len() == 1 {
                replacements.insert(atom, names.into_iter().next().unwrap());
            }
        }
        Ok(query::rewrite(q, |atom| {
            replacements
                .get(atom)
                .cloned()
                .unwrap_or_else(|| atom.into())
        }))
    }
    async fn agent_inner(&mut self, q: &str, ctx: &Context) -> Result<Vec<Call>> {
        let resolved = self.qualified_suffix_query(q, ctx).await?;
        let q = resolved.as_str();
        self.load_surface(&query::content(q)).await?;
        let mut calls = self.lower_fallbacks(q, ctx).await?;
        if !has_cards(&calls) {
            let mut probes = query::content(q);
            query::rewrite(q, |term| {
                probes.push(term.to_lowercase());
                term.to_owned()
            });
            self.load_vocabulary_terms(probes).await?;
            let ident = Regex::new(r"[_-]|[a-z][A-Z]|\d").unwrap();
            let mut groups = Vec::new();
            let rewritten = query::rewrite(q, |t| {
                let pcs = query::pieces(t);
                if !t.contains(':')
                    && ident.is_match(t)
                    && !self.vocab.contains(&t.to_lowercase())
                    && pcs.len() >= 2
                {
                    groups.push(pcs.clone());
                    pcs.join(" ")
                } else {
                    t.into()
                }
            });
            if !groups.is_empty() {
                let res = self.lower_fallbacks(&rewritten, ctx).await?;
                let mut kept = Vec::new();
                for card in res.first().into_iter().flat_map(|r| &r.cards) {
                    let text = self
                        .rows
                        .get(&card.section)
                        .map(|b| format!("{} {}", b.heading, b.text))
                        .unwrap_or_else(|| card.text.clone());
                    let mut stems = HashSet::new();
                    for w in query::surface(&text) {
                        stems.insert(self.porter(&w).await?);
                    }
                    let mut yes = true;
                    for word in groups.iter().flatten() {
                        if !stems.contains(&self.porter(word).await?) {
                            yes = false;
                            break;
                        }
                    }
                    if yes {
                        kept.push(card.clone());
                    }
                }
                if !kept.is_empty() {
                    put(&mut calls, kept);
                }
            }
        }
        if !has_cards(&calls) {
            let forms = self.forms().await?;
            let ident = Regex::new(r"[_-]|\d").unwrap();
            let mut probes = Vec::new();
            let mut seen = HashSet::new();
            for w in query::content(q) {
                let pcs = query::pieces(&w);
                if ident.is_match(&w) && !self.vocab.contains(&w) && pcs.len() >= 2 {
                    for p in pcs {
                        if seen.insert(p.clone()) {
                            probes.push(p);
                        }
                    }
                } else if seen.insert(w.clone()) {
                    probes.push(w);
                }
            }
            if probes.len() >= 2 {
                let mut stems = Vec::new();
                for p in &probes {
                    stems.push(self.porter(p).await?);
                }
                self.prepare_porter(
                    forms
                        .iter()
                        .filter(|f| f.df <= 8 && f.pieces.len() >= 2)
                        .flat_map(|f| f.pieces.iter().cloned()),
                )
                .await?;
                let mut best = None;
                for (i, f) in forms
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.df <= 8 && f.pieces.len() >= 2)
                {
                    let mut remaining = stems.clone();
                    let mut ok = true;
                    for p in &f.pieces {
                        let s = self.porter(p).await?;
                        if let Some(i) = remaining.iter().position(|p| p == &s) {
                            remaining.remove(i);
                        } else {
                            ok = false;
                            break;
                        }
                    }
                    let fresh = f.pieces.iter().filter(|p| !probes.contains(p)).count();
                    if !ok || fresh == 0 {
                        continue;
                    }
                    let key = (
                        fresh,
                        f.pieces.len(),
                        std::cmp::Reverse(f.df),
                        f.text.clone(),
                    );
                    if best.as_ref().is_none_or(|(prev, _)| key > *prev) {
                        best = Some((key, i));
                    }
                }
                if let Some((_, i)) = best {
                    let mut ids = forms[i].rows.clone();
                    self.load_blocks(ids.iter().copied()).await?;
                    ids.retain(|id| self.hint_eligible(&self.rows[id].record));
                    ids.sort_by_key(|id| self.rows[id].text.chars().count());
                    if let Some(id) = ids.first() {
                        put(&mut calls, vec![self.card(*id, None)]);
                    }
                }
            }
        }
        self.technical_file_stem(q, ctx, &mut calls).await?;
        self.quoted_filename(q, &mut calls).await?;
        Ok(calls)
    }
    // @ref LLP 0002#ordered-fallbacks
    async fn technical_file_stem(
        &mut self,
        raw: &str,
        ctx: &Context,
        calls: &mut Vec<Call>,
    ) -> Result<()> {
        let Ok(q) = query::Query::parse(raw) else {
            return Ok(());
        };
        let mut matched = Vec::new();
        for term in q.terms.iter().filter(|t| !t.phrase && t.identifier()) {
            let name = term.text.to_lowercase();
            let mut needed = HashSet::new();
            for t in q
                .terms
                .iter()
                .filter(|t| t.content() && t.text.to_lowercase() != name)
            {
                for word in query::surface(&t.text) {
                    needed.insert(self.porter(&word).await?);
                }
            }
            if needed.is_empty() {
                continue;
            }
            let mut rows=self.db.query("SELECT name,record FROM record_basenames WHERE name>=? AND name<? ORDER BY name,record",libsql::params![format!("{name}."),format!("{name}/")]).await?;
            let mut ids = std::collections::BTreeSet::new();
            while let Some(row) = rows.next().await? {
                let full: String = row.get(0)?;
                if full
                    .rsplit_once('.')
                    .is_some_and(|(stem, ext)| stem == name && !ext.is_empty())
                {
                    ids.insert(row.get::<String>(1)?);
                }
            }
            self.load_record_headers(ids.iter().cloned()).await?;
            for rid in ids {
                if !self.validity.eligible(&self.records[&rid], &ctx.now) {
                    continue;
                }
                self.load_record_blocks(&rid).await?;
                let candidates: Vec<_> = self
                    .record_blocks(&rid)
                    .filter(|b| self.allowed(b, &q.filters))
                    .map(|b| (b.id, b.text.clone()))
                    .collect();
                for (id, text) in candidates {
                    let mut have = HashSet::new();
                    for word in query::surface(&text) {
                        have.insert(self.porter(&word).await?);
                    }
                    if needed.is_subset(&have) {
                        matched.push(id);
                    }
                }
            }
        }
        let records: HashSet<_> = matched.iter().map(|id| &self.rows[id].record).collect();
        if records.len() == 1 {
            matched.sort_by_key(|id| (self.rows[id].start, *id));
            promote(calls, self.card(matched[0], None));
        }
        Ok(())
    }
    async fn quoted_filename(&mut self, q: &str, calls: &mut Vec<Call>) -> Result<()> {
        let quotes: Vec<_> = query::QUOTE
            .captures_iter(q)
            .map(|c| c[1].to_lowercase())
            .collect();
        self.load_basenames(&quotes).await?;
        let mut unique = HashSet::new();
        for name in &quotes {
            if let Some(ids) = self.basenames.get(name) {
                let mut eligible = ids.iter().filter(|rid| self.hint_eligible(rid));
                if let Some(rid) = eligible.next()
                    && eligible.next().is_none()
                {
                    unique.insert(rid.clone());
                }
            }
        }
        if unique.len() == 1 {
            let rid = unique.iter().next().unwrap();
            if let Some(id) = self.first_record_block(rid).await? {
                self.load_blocks([id]).await?;
                promote(calls, self.card(id, None));
            }
        }
        let mut words = HashSet::new();
        for w in query::content(&query::QUOTE.replace_all(q, "")) {
            words.insert(self.porter(&w).await?);
        }
        if words.len() < 2 {
            return Ok(());
        }
        let mut matches = Vec::new();
        for name in quotes {
            let Some(mut ids) = self.basenames.get(&name).cloned() else {
                continue;
            };
            ids.retain(|rid| self.hint_eligible(rid));
            if ids.len() < 2 {
                continue;
            }
            for rid in &ids {
                if self.hint_eligible(rid) {
                    self.load_record_block_headers(rid).await?;
                }
            }
            let block_ids: Vec<_> = ids
                .iter()
                .filter(|rid| self.hint_eligible(rid))
                .flat_map(|rid| self.record_blocks(rid))
                .map(|b| b.id)
                .collect();
            self.load_blocks(block_ids.iter().copied()).await?;
            let blocks: Vec<_> = block_ids
                .into_iter()
                .map(|id| (id, self.rows[&id].text.clone()))
                .collect();
            for (id, text) in blocks {
                let content = query::content(&text);
                self.prepare_porter(content.iter().cloned()).await?;
                let mut have = HashSet::new();
                for w in content {
                    have.insert(self.porter(&w).await?);
                }
                if words.is_subset(&have) {
                    matches.push(id);
                }
            }
        }
        let records: HashSet<_> = matches
            .iter()
            .map(|id| self.rows[id].record.clone())
            .collect();
        if records.len() == 1 {
            matches.sort_by_key(|id| self.rows[id].start);
            promote(calls, self.card(matches[0], None));
        }
        Ok(())
    }
}
fn section_means(
    forms: &Forms,
    matrix: &crate::retrieval::similarity::Matrix,
) -> Vec<(usize, Vec<f32>)> {
    let mut acc = Vec::<(usize, Vec<f32>, usize)>::new();
    let mut positions = HashMap::<usize, usize>::new();
    let mut ordered: Vec<_> = forms
        .iter()
        .enumerate()
        .filter(|(_, f)| f.df <= 8)
        .collect();
    ordered.sort_by_key(|(_, f)| f.order);
    for (index, form) in ordered {
        let vector = matrix.row(index);
        for sid in &form.rows {
            if let Some(&position) = positions.get(sid) {
                let (_, sum, n) = &mut acc[position];
                for (a, b) in sum.iter_mut().zip(vector) {
                    *a += *b;
                }
                *n += 1;
            } else {
                positions.insert(*sid, acc.len());
                acc.push((*sid, vector.to_vec(), 1));
            }
        }
    }
    acc.into_iter()
        .map(|(sid, sum, n)| (sid, unit(sum.iter().map(|v| v / n as f32).collect())))
        .collect()
}
fn has_cards(calls: &[Call]) -> bool {
    calls.first().is_some_and(|c| !c.cards.is_empty())
}

fn put(calls: &mut Vec<Call>, cards: Vec<Card>) {
    if calls.is_empty() {
        calls.push(Call {
            text: cards[0].text.clone(),
            cards,
            ..Call::default()
        });
    } else {
        calls[0].cards = cards;
    }
}
fn promote(calls: &mut Vec<Call>, card: Card) {
    if calls.is_empty() {
        put(calls, vec![card]);
    } else {
        calls[0].cards.retain(|c| c.record != card.record);
        calls[0].cards.insert(0, card);
        calls[0].cards.truncate(5);
    }
}

#[cfg(test)]
mod mean_tests {
    use super::*;
    #[tokio::test]
    async fn legacy_sentence_upgrade_uses_stored_bits_without_loading_bodies() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy-sentences.db");
        let record = serde_json::from_value::<crate::storage::record::Record>(serde_json::json!({"id":"r","body":"One sentence has four words. Another sentence has five words. One sentence has four words."})).unwrap();
        let mut built = Store::build(&path, vec![record]).await.unwrap();
        let expected = built.sentence_data().await.unwrap();
        assert!(!expected.0.is_empty());
        built
            .publish_fallback_artifacts(&[(
                "sentences",
                serde_json::to_vec(&expected.0).unwrap(),
                &expected.1,
            )])
            .await
            .unwrap();
        built.db.execute_batch("DELETE FROM fallback_artifacts WHERE kind='sentences-dictionary-v1'; DELETE FROM fallback_vectors WHERE kind='sentences-dictionary-v1';").await.unwrap();
        drop(built);
        let mut upgraded = Store::open(&path).await.unwrap();
        assert!(upgraded.rows.is_empty());
        assert!(upgraded.sentence_cache.is_none());
        assert!(upgraded.fallback_artifacts_ready().await.unwrap());
        assert!(
            upgraded
                .fallback_metadata::<Vec<usize>>("sentences")
                .await
                .unwrap()
                .is_none()
        );
        let actual = upgraded.sentence_data().await.unwrap();
        assert_eq!(expected.0, actual.0);
        assert!(
            expected.1.chunks().flatten().map(|v| v.to_bits()).eq(actual
                .1
                .chunks()
                .flatten()
                .map(|v| v.to_bits()))
        );
    }
    #[tokio::test]
    async fn stored_fallback_arrays_equal_computed_arrays() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifacts.db");
        let record=serde_json::from_value::<crate::storage::record::Record>(serde_json::json!({"id":"r","state":"accepted","body":"# Heading\nalpha-beta foo_bar camelCase and alpha-beta again.\nThis sentence has several words. Another sentence has enough words.\n"})).unwrap();
        let mut computed = Store::build(&path, vec![record]).await.unwrap();
        let a = computed.forms().await.unwrap();
        computed.form_vectors(&a).await.unwrap();
        let b = computed.beam_data().await.unwrap();
        let c = computed.sentence_data().await.unwrap();
        drop(computed);
        let mut loaded = Store::open(&path).await.unwrap();
        assert!(
            loaded.form_cache.is_none()
                && loaded.beam_cache.is_none()
                && loaded.sentence_cache.is_none()
        );
        let means = section_means(&a, a.matrix.get().unwrap());
        assert!(!means.is_empty());
        let candidates: HashSet<_> = means.iter().step_by(2).map(|(id, _)| *id).collect();
        let stored = loaded.candidate_section_means(&candidates).await.unwrap();
        let expected: Vec<_> = means
            .into_iter()
            .filter(|(id, _)| candidates.contains(id))
            .collect();
        assert_eq!(
            stored.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            expected.iter().map(|(id, _)| *id).collect::<Vec<_>>()
        );
        assert!(
            stored
                .iter()
                .flat_map(|(_, v)| v)
                .map(|v| v.to_bits())
                .eq(expected.iter().flat_map(|(_, v)| v).map(|v| v.to_bits()))
        );
        let aa = loaded.forms().await.unwrap();
        assert!(aa.matrix.get().is_none());
        assert!(
            loaded
                .form_block("unattested garnet", &aa)
                .await
                .unwrap()
                .is_none()
        );
        assert!(aa.matrix.get().is_none());
        loaded.form_vectors(&aa).await.unwrap();
        let bb = loaded.beam_data().await.unwrap();
        let cc = loaded.sentence_data().await.unwrap();
        assert_eq!(
            serde_json::to_vec(&a.metadata).unwrap(),
            serde_json::to_vec(&aa.metadata).unwrap()
        );
        assert!(
            a.matrix
                .get()
                .unwrap()
                .chunks()
                .flatten()
                .map(|v| v.to_bits())
                .eq(aa
                    .matrix
                    .get()
                    .unwrap()
                    .chunks()
                    .flatten()
                    .map(|v| v.to_bits()))
        );
        assert_eq!(b.proposals, bb.proposals);
        let terms = loaded.candidate_beam_terms(&b.proposals).await.unwrap();
        assert_eq!(
            terms,
            b.proposals
                .iter()
                .enumerate()
                .map(|(i, word)| (word.clone(), i))
                .collect()
        );
        assert!(
            b.matrix.chunks().flatten().map(|v| v.to_bits()).eq(bb
                .matrix
                .chunks()
                .flatten()
                .map(|v| v.to_bits()))
        );
        assert_eq!(c.0, cc.0);
        assert!(
            c.1.chunks().flatten().map(|v| v.to_bits()).eq(cc
                .1
                .chunks()
                .flatten()
                .map(|v| v.to_bits()))
        );
        loaded.forget("r").await.unwrap();
        let mut rows = loaded
            .db
            .query("SELECT count(*) FROM section_means", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
        let mut rows = loaded
            .db
            .query("SELECT count(*) FROM beam_terms", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            0
        );
    }
    #[test]
    fn indexed_accumulation_keeps_occurrence_weights_order_and_bits() {
        let metadata: Vec<_> = [
            (2, vec![9, 3, 9], 1),
            (0, vec![31, 9, 31], 2),
            (1, vec![3, 31, 3], 8),
            (3, vec![1, 3], 9),
        ]
        .into_iter()
        .map(|(order, rows, df)| Form {
            text: format!("form{order}"),
            pieces: vec![],
            rows,
            df,
            order,
        })
        .collect();
        let vectors: Vec<Vec<f32>> = metadata
            .iter()
            .map(|f| {
                (0..512)
                    .map(|i| ((i * 13 + f.order * 31) as f32).sin())
                    .collect()
            })
            .collect();
        let forms = Forms {
            metadata,
            matrix: crate::retrieval::similarity::Matrix::new(vectors.iter().map(Vec::as_slice))
                .into(),
        };
        let mut slow = Vec::<(usize, Vec<f32>, usize)>::new();
        let mut ordered: Vec<_> = forms.iter().filter(|f| f.df <= 8).collect();
        ordered.sort_by_key(|f| f.order);
        for form in ordered {
            let vector = &vectors[forms.iter().position(|f| std::ptr::eq(f, form)).unwrap()];
            for sid in &form.rows {
                if let Some((_, sum, n)) = slow.iter_mut().find(|(id, _, _)| id == sid) {
                    for (a, b) in sum.iter_mut().zip(vector) {
                        *a += *b;
                    }
                    *n += 1;
                } else {
                    slow.push((*sid, vector.clone(), 1));
                }
            }
        }
        let slow: Vec<_> = slow
            .into_iter()
            .map(|(sid, sum, n)| (sid, unit(sum.iter().map(|v| v / n as f32).collect())))
            .collect();
        let fast = section_means(&forms, forms.matrix.get().unwrap());
        assert_eq!(
            fast.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![31, 9, 3]
        );
        for ((sid, a), (other, b)) in fast.iter().zip(&slow) {
            assert_eq!(sid, other);
            assert_eq!(
                a.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                b.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
        }
    }
}

#[cfg(test)]
mod technical_file_stem_tests {
    use super::*;
    #[tokio::test]
    async fn original_block_disambiguates_filename_without_indexing_it() {
        let dir = tempfile::tempdir().unwrap();
        let rec = |id: &str, body: &str, path: &str| {
            serde_json::from_value::<crate::storage::record::Record>(
                serde_json::json!({"id":id,"body":body,"state":"accepted","source_path":path}),
            )
            .unwrap()
        };
        let st = Store::build(
            &dir.path().join("s.db"),
            vec![
                rec("a", "allowPush is enabled.", "home/git-access.nix"),
                rec("b", "SSH port is fixed.", "other/git-access.nix"),
                rec("c", "allowPush is enabled.", "other/git-access.extra.nix"),
            ],
        )
        .await
        .unwrap();
        drop(st);
        let mut st = Store::open(&dir.path().join("s.db")).await.unwrap();
        let mut plan = st.db.query("EXPLAIN QUERY PLAN SELECT name,record FROM record_basenames WHERE name>=? AND name<? ORDER BY name,record", ["git-access.", "git-access/"]).await.unwrap();
        let mut details = String::new();
        while let Some(row) = plan.next().await.unwrap() {
            details.push_str(&row.get::<String>(3).unwrap());
        }
        assert!(
            details.contains("SEARCH record_basenames USING PRIMARY KEY"),
            "{details}"
        );
        let mut calls = vec![];
        st.technical_file_stem("allowPush git-access", &Context::default(), &mut calls)
            .await
            .unwrap();
        assert_eq!(calls[0].cards[0].record, "a");
        assert!(!calls[0].cards[0].text.contains("git-access"));
        let mut calls = vec![];
        st.technical_file_stem("git-access", &Context::default(), &mut calls)
            .await
            .unwrap();
        assert!(calls.is_empty());
        let mut calls = vec![];
        st.technical_file_stem(
            "kind:doc allowPush git-access",
            &Context::default(),
            &mut calls,
        )
        .await
        .unwrap();
        assert!(calls.is_empty());
        st.forget("a").await.unwrap();
        let mut calls = vec![];
        st.technical_file_stem("allowPush git-access", &Context::default(), &mut calls)
            .await
            .unwrap();
        assert!(calls.is_empty());
    }
}

#[cfg(test)]
mod suffix_tests {
    use super::*;
    fn record(id: &str, body: &str) -> crate::record::Record {
        serde_json::from_value(serde_json::json!({"id":id,"body":body,"state":"accepted"})).unwrap()
    }
    #[tokio::test]
    async fn suffix_resolution_preserves_literals_and_ambiguity() {
        let d = tempfile::tempdir().unwrap();
        let mut st = Store::build(
            &d.path().join("test.db"),
            vec![record(
                "a",
                "ERR_REQUIRE_ESM_RACE_CONDITION means an import is still loading.",
            )],
        )
        .await
        .unwrap();
        let ctx = Context::default();
        assert_eq!(
            st.qualified_suffix_query("RACE_CONDITION error code", &ctx)
                .await
                .unwrap(),
            "ERR_REQUIRE_ESM_RACE_CONDITION error code"
        );
        for q in [
            "\"RACE_CONDITION\" error",
            "path:RACE_CONDITION",
            "race_condition",
            "RACE_CONDITION, error",
            "NO_SUCH_CODE",
        ] {
            assert_eq!(st.qualified_suffix_query(q, &ctx).await.unwrap(), q);
        }
        let mut exact = Store::build(
            &d.path().join("exact.db"),
            vec![
                record("a", "ERR_REQUIRE_ESM_RACE_CONDITION"),
                record("b", "RACE_CONDITION"),
            ],
        )
        .await
        .unwrap();
        assert_eq!(
            exact
                .qualified_suffix_query("RACE_CONDITION", &ctx)
                .await
                .unwrap(),
            "RACE_CONDITION"
        );
        let mut ambiguous = Store::build(
            &d.path().join("ambiguous.db"),
            vec![
                record("a", "ERR_REQUIRE_ESM_RACE_CONDITION"),
                record("b", "ERR_OTHER_RACE_CONDITION"),
            ],
        )
        .await
        .unwrap();
        assert_eq!(
            ambiguous
                .qualified_suffix_query("RACE_CONDITION", &ctx)
                .await
                .unwrap(),
            "RACE_CONDITION"
        );
    }
}

fn colon_literal_query(raw: &str) -> Option<(String, Vec<String>)> {
    if query::Query::parse(raw).err().as_deref() != Some("Unknown prefix") {
        return None;
    }
    let re = Regex::new(r"(^|\s)([a-z][a-z-]*):([A-Za-z][A-Za-z0-9_-]*)").unwrap();
    let mut spans = Vec::new();
    for cap in re.captures_iter(raw) {
        let key = cap.get(2).unwrap();
        let value = cap.get(3).unwrap();
        if !raw[..key.start()].matches('"').count().is_multiple_of(2)
            || raw[value.end()..]
                .chars()
                .next()
                .is_some_and(|c| !c.is_whitespace())
            || query::Query::parse(&format!("{}:x text", key.as_str())).is_ok()
        {
            continue;
        }
        spans.push(key.start()..value.end());
    }
    if spans.is_empty() {
        return None;
    }
    let atoms = spans.iter().map(|r| raw[r.clone()].to_owned()).collect();
    let mut rewritten = raw.to_owned();
    for span in spans.into_iter().rev() {
        rewritten.insert(span.end, '"');
        rewritten.insert(span.start, '"');
    }
    query::Query::parse(&rewritten).ok()?;
    Some((rewritten, atoms))
}
fn colon_literal_present(body: &str, atom: &str) -> bool {
    let constituent = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | ':');
    body.match_indices(atom).any(|(start, _)| {
        !body[..start].chars().next_back().is_some_and(constituent)
            && !body[start + atom.len()..]
                .chars()
                .next()
                .is_some_and(constituent)
    })
}
#[cfg(test)]
mod colon_literal_tests {
    use super::*;
    #[tokio::test]
    async fn colon_and_suffix_recovery_compose() {
        let dir = tempfile::tempdir().unwrap();
        let record=serde_json::from_value::<crate::storage::record::Record>(serde_json::json!({"id":"literal","state":"accepted","body":"build:refresh reports ERR_RACE_CONDITION during loading."})).unwrap();
        let mut store = Store::build(&dir.path().join("mixed.db"), vec![record])
            .await
            .unwrap();
        let calls = store
            .rewrites("build:refresh RACE_CONDITION", &Context::default())
            .await
            .unwrap();
        assert!(has_cards(&calls));
        assert!(
            calls
                .iter()
                .flat_map(|c| &c.cards)
                .all(|c| c.record == "literal")
        );
    }

    #[tokio::test]
    async fn colon_recovery_requires_literal_body_on_every_call() {
        let dir = tempfile::tempdir().unwrap();
        let rec = |id: &str, body: &str| {
            serde_json::from_value::<crate::storage::record::Record>(
                serde_json::json!({"id":id,"state":"accepted","body":body}),
            )
            .unwrap()
        };
        let mut store = Store::build(
            &dir.path().join("literal.db"),
            vec![
                rec("literal", "Read build:refresh status logs."),
                rec("lookalike", "Read build refresh status logs."),
            ],
        )
        .await
        .unwrap();
        let calls = store
            .rewrites("build:refresh status", &Context::default())
            .await
            .unwrap();
        assert!(has_cards(&calls));
        assert!(
            calls
                .iter()
                .flat_map(|c| &c.cards)
                .all(|c| c.record == "literal")
        );
        let mut absent = Store::build(
            &dir.path().join("absent.db"),
            vec![rec("lookalike", "Read build refresh status logs.")],
        )
        .await
        .unwrap();
        let calls = absent
            .rewrites("build:refresh status", &Context::default())
            .await
            .unwrap();
        assert!(!has_cards(&calls));
    }

    #[test]
    fn recovery_keeps_filters_quotes_and_exact_source_boundaries() {
        assert_eq!(
            colon_literal_query("metro:bundling spans").unwrap().0,
            "\"metro:bundling\" spans"
        );
        assert_eq!(
            colon_literal_query("path:foo env:pull task:run").unwrap().0,
            "path:foo \"env:pull\" \"task:run\""
        );
        for q in [
            "path:foo spans",
            "\"metro:bundling\" spans",
            "\"unclosed",
            "bad:\"quoted\"",
            "bad:",
            "bad:thing.json",
        ] {
            assert!(colon_literal_query(q).is_none(), "{q}");
        }
        assert!(colon_literal_present(
            "--filter metro:bundling --spans",
            "metro:bundling"
        ));
        assert!(colon_literal_present("`env:pull`", "env:pull"));
        for text in [
            "metro:bundling-more",
            "xmetro:bundling",
            "metro bundling",
            "Metro:bundling",
            "émetro:bundling",
        ] {
            assert!(!colon_literal_present(text, "metro:bundling"));
        }
    }
}
