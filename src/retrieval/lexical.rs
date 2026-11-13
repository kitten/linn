//! @ref LLP 0002#lexical-evidence
//! @ref LLP 0002#ordered-fallbacks
use crate::{
    error::Result,
    retrieval::query::{self, Query, Term},
    storage::{Block, Call, Card, Context, Pointer, Store, Tree},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

struct Scoring {
    avg: f64,
    pairs: Vec<crate::retrieval::ordered::Pair>,
    terms: Vec<(bool, Vec<(String, f64)>)>,
}
impl Scoring {
    fn new(q: &Query, stats: &crate::storage::Statistics) -> Self {
        Self {
            avg: stats.avg,
            pairs: Vec::new(),
            terms: q
                .terms
                .iter()
                .map(|t| {
                    (
                        t.hard(),
                        query::surface(&t.text)
                            .iter()
                            .map(|w| {
                                let w = query::norm(w);
                                let d = *stats.df.get(&w).unwrap_or(&0.);
                                let idf = (1. + (stats.n - d + 0.5) / (d + 0.5)).ln();
                                (w, idf)
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }
    fn score(&self, row: &Block) -> f64 {
        let mut score = 0.;
        for (hard, words) in &self.terms {
            for (w, idf) in words {
                let tf = row.tf.get(w).copied().unwrap_or_default();
                let tf = tf[1] + 3. * tf[0] + if !hard { 0.3 * tf[2] } else { 0. };
                if tf > 0. {
                    let normalized =
                        tf / (0.25 + 0.75 * row.length as f64 / self.avg.max(1.)) + 0.125;
                    score += idf * normalized * 2.2 / (normalized + 1.2);
                }
            }
        }
        score
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Rank {
    pub tier: usize,
    pub score: f64,
    pub sid: usize,
}
#[derive(Clone)]
pub(crate) struct Matched {
    pub ranking: Vec<Rank>,
    pub counts: HashMap<usize, usize>,
    pub hits: Vec<HashSet<usize>>,
    pub scores: HashMap<usize, f64>,
    pub count_order: Vec<usize>,
    pub confidence: Option<f64>,
    scoring: Arc<Scoring>,
}
#[derive(Clone)]
pub(crate) struct Coverage {
    counts: HashMap<usize, usize>,
    hits: Vec<HashSet<usize>>,
    scores: HashMap<usize, f64>,
    arrival: HashMap<usize, usize>,
}
impl From<&Matched> for Coverage {
    fn from(m: &Matched) -> Self {
        Self {
            counts: m.counts.clone(),
            hits: m.hits.clone(),
            scores: m
                .scores
                .iter()
                .filter(|(id, _)| m.counts.contains_key(id))
                .map(|(k, v)| (*k, *v))
                .collect(),
            arrival: m
                .count_order
                .iter()
                .enumerate()
                .map(|(position, id)| (*id, position))
                .collect(),
        }
    }
}
fn rank_cmp(a: &Rank, b: &Rank, st: &Store) -> std::cmp::Ordering {
    let x = &st.rows[&a.sid];
    let y = &st.rows[&b.sid];
    a.tier
        .cmp(&b.tier)
        .then_with(|| b.score.total_cmp(&a.score))
        .then_with(|| x.hash.cmp(&y.hash))
        .then_with(|| x.ordinal.cmp(&y.ordinal))
        .then_with(|| x.record.cmp(&y.record))
        .then_with(|| a.sid.cmp(&b.sid))
}
impl Store {
    pub fn allowed(&self, b: &Block, filters: &[(String, String)]) -> bool {
        let r = &self.records[&b.record];
        filters.iter().all(|(k, v)| match k.as_str() {
            "kind" => r.kind == *v,
            "writer" => r.writer.as_ref().and_then(|v| v.as_deref()) == Some(v.as_str()),
            "state" => v == "open" || v == "accepted",
            "id" => r.id == *v || b.hash.starts_with(v),
            "under" => b
                .path
                .join(" > ")
                .to_lowercase()
                .contains(&v.to_lowercase()),
            "path" | "cites" => query::pathmatch(v, r.source_path.as_deref().unwrap_or("")),
            "sibling-of" => r.id == *v,
            "linked-from" => self
                .links
                .get(v)
                .is_some_and(|ids| self.records.contains_key(v) && ids.contains(&r.id)),
            "version-of" => {
                r.parent.as_ref() == Some(v)
                    || self
                        .records
                        .get(v)
                        .is_some_and(|other| other.parent.as_ref() == Some(&r.id))
            }
            "replaces" => {
                self.indexes
                    .lifecycle
                    .get(v)
                    .into_iter()
                    .flatten()
                    .any(|id| {
                        self.records[id].close.as_ref().is_some_and(|c| {
                            c.target == *v && c.replacement.as_ref() == Some(&r.id)
                        })
                    })
            }
            _ => true,
        })
    }
    async fn term_hits(&mut self, t: &Term, b: bool) -> Result<HashSet<usize>> {
        let mut hits = self.fts_ids(false, &t.expression(b)).await?;
        if b && !t.identifier() {
            hits.extend(self.fts_ids(true, &query::quote(&t.text)).await?);
        }
        Ok(hits)
    }
    fn allowed_query(&self, b: &Block, q: &Query) -> bool {
        q.filters.iter().all(|filter| {
            if filter.0 == "path" || filter.0 == "cites" {
                q.path_matches(
                    &filter.1,
                    self.records[&b.record].source_path.as_deref().unwrap_or(""),
                )
            } else {
                self.allowed(b, std::slice::from_ref(filter))
            }
        })
    }
    async fn matcher_hits(
        &mut self,
        stem: bool,
        expression: &str,
        q: &Query,
        ctx: &Context,
    ) -> Result<HashSet<usize>> {
        if self.indexed_navigation(q) {
            return self.eligible_fts_hits(stem, expression, &ctx.now).await;
        }
        let mut ids = self.writer_fts_ids(stem, expression, q).await?;
        self.load_block_headers(ids.iter().copied()).await?;
        ids.retain(|id| {
            self.validity
                .eligible(&self.records[&self.rows[id].record], &ctx.now)
                && self.allowed_query(&self.rows[id], q)
        });
        Ok(ids)
    }
    async fn matcher(
        &mut self,
        q: &Query,
        ctx: &Context,
        b: bool,
        scoring: Arc<Scoring>,
        memo: &mut HashMap<usize, f64>,
    ) -> Result<Matched> {
        let stats = self.scope_stats(&ctx.now, q).await?;
        let (n, df) = (stats.n, &stats.df);
        let mut hits = Vec::new();
        let mut orders = Vec::new();
        for t in &q.terms {
            let mut set = self.matcher_hits(false, &t.expression(b), q, ctx).await?;
            let mut posting = crate::retrieval::hit_order::Postings::new(set.iter().copied());
            if b && !t.identifier() {
                let stem = self
                    .matcher_hits(true, &query::quote(&t.text), q, ctx)
                    .await?;
                posting.merge(&crate::retrieval::hit_order::Postings::new(
                    stem.iter().copied(),
                ));
                set.extend(stem);
            }
            let nominal = self.nominal_compound_hits(t, q, ctx).await?;
            posting.merge(&crate::retrieval::hit_order::Postings::new(
                nominal.iter().copied(),
            ));
            set.extend(nominal);
            orders.push(posting.order());
            hits.push(set);
        }
        let content: Vec<_> = q
            .terms
            .iter()
            .enumerate()
            .filter(|(_, t)| t.content())
            .map(|(i, _)| i)
            .collect();
        let mut counts = HashMap::<usize, usize>::new();
        let mut count_order = Vec::new();
        for i in &content {
            for &id in &orders[*i] {
                if !counts.contains_key(&id) {
                    count_order.push(id);
                }
                *counts.entry(id).or_default() += 1;
            }
        }
        let text = content
            .iter()
            .map(|i| q.terms[*i].text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let mut phrase = if text.is_empty() {
            HashSet::new()
        } else {
            self.writer_fts_ids(
                false,
                &format!("{{heading body}}:{}", query::quote(&text)),
                q,
            )
            .await?
        };
        if b && !text.is_empty() && !content.iter().any(|i| q.terms[*i].identifier()) {
            phrase.extend(self.writer_fts_ids(true, &query::quote(&text), q).await?);
        }
        let mut feature_ids: Vec<_> = counts
            .iter()
            .filter_map(|(&id, &count)| {
                let admitted = count >= query::need(content.len())
                    || (count >= q.need.saturating_sub(1) && count < q.need);
                (admitted
                    && q.terms
                        .iter()
                        .enumerate()
                        .all(|(i, t)| !t.hard() || hits[i].contains(&id)))
                .then_some(id)
            })
            .collect();
        if b {
            if self.indexed_navigation(q) {
                self.load_search_navigation(q).await?;
            }
            for deep in [false, true] {
                for id in self.tree_candidates(q, deep) {
                    let tree = if deep {
                        &self.deep[&id]
                    } else {
                        &self.direct[&id]
                    };
                    if let Some((_, id)) = self.tree_block(tree, q) {
                        feature_ids.push(id);
                    }
                }
            }
        }
        feature_ids.sort_unstable();
        feature_ids.dedup();
        if crate::diagnostics::enabled("LINN_POS_TRACE") {
            crate::diagnostics::emit(format!(
                "{}",
                serde_json::json!({"stage":"feature_selection","query":q.original,"matcher_b":b,"positive_rows":counts.len(),"feature_rows":feature_ids.len()})
            ));
        }
        self.load_candidate_frequencies(
            q.terms
                .iter()
                .flat_map(|t| query::surface(&t.text))
                .map(|w| query::norm(&w)),
            &feature_ids,
        )
        .await?;
        let unscored: Vec<_> = feature_ids
            .iter()
            .copied()
            .filter(|id| counts.contains_key(id) && !memo.contains_key(id))
            .collect();
        let bonuses = self.occurrence_bonuses(&unscored, &scoring.pairs).await?;
        let mut scores = HashMap::new();
        // Only positive coverage rows need eager scores. Trees score their chosen row on demand.
        for &id in feature_ids.iter().filter(|id| counts.contains_key(id)) {
            let score = if let Some(score) = memo.get(&id) {
                *score
            } else {
                let lexical = scoring.score(&self.rows[&id]);
                let bonus = 0.75 * bonuses.get(&id).copied().unwrap_or(0.);
                let score = if bonus == 0. {
                    lexical
                } else {
                    lexical + bonus
                };
                if crate::diagnostics::enabled("LINN_POS_TRACE") {
                    let row = &self.rows[&id];
                    crate::diagnostics::emit(format!(
                        "{}",
                        serde_json::json!({"stage":"primary_score", "query":q.original,"row":id,"record":row.record,"lexical":lexical,"bonus":bonus,"score":score})
                    ));
                }
                memo.insert(id, score);
                score
            };
            scores.insert(id, score);
        }
        let mut ranking = Vec::new();
        for (&id, &count) in &counts {
            if count < query::need(content.len())
                || q.terms
                    .iter()
                    .enumerate()
                    .any(|(i, t)| t.hard() && !hits[i].contains(&id))
            {
                continue;
            }
            let tier = if phrase.contains(&id) {
                0
            } else if content.iter().all(|i| hits[*i].contains(&id)) {
                1
            } else {
                2
            };
            ranking.push(Rank {
                tier,
                score: scores[&id],
                sid: id,
            });
        }
        ranking.sort_by(|a, b| rank_cmp(a, b, self));
        if crate::diagnostics::enabled("LINN_POS_RANK_TRACE") {
            let rows: Vec<_> = ranking.iter().map(|r| {
                let row=&self.rows[&r.sid];
                serde_json::json!({"row":r.sid,"tier":r.tier,"score":r.score,"record":row.record,"body_hash":row.hash,"span":[row.start,row.end]})
            }).collect();
            crate::diagnostics::emit(format!(
                "{}",
                serde_json::json!({"stage":"raw_rank","query":q.original,"matcher_b":b,"rows":rows})
            ));
        }
        let mut confidence = None;
        if b {
            let mut stems = Vec::new();
            for term in &q.terms {
                stems.push(self.index_stems(&term.text).await?);
            }
            let mut seen = HashSet::new();
            let ids: Vec<_> = ranking
                .iter()
                .filter(|r| seen.insert(self.rows[&r.sid].record.clone()))
                .take(5)
                .map(|r| r.sid)
                .collect();
            self.load_candidate_terms(
                q.terms
                    .iter()
                    .flat_map(|t| query::surface(&t.text))
                    .map(|w| query::norm(&w)),
                false,
                &ids,
            )
            .await?;
            self.load_candidate_terms(stems.iter().flatten().cloned(), true, &ids)
                .await?;
            confidence =
                crate::retrieval::confidence::confidence(self, q, &hits, &ranking, &stems, n, df);
            debug_assert!(confidence.is_none_or(|v| (0.0..=1.0).contains(&v)));
        }
        Ok(Matched {
            ranking,
            counts,
            hits,
            scores,
            count_order,
            confidence,
            scoring,
        })
    }

    async fn nominal_compound_hits(
        &mut self,
        term: &Term,
        q: &Query,
        ctx: &Context,
    ) -> Result<HashSet<usize>> {
        let Some((a, b)) =
            crate::language::pos::nominal_compound_parts(&term.text, term.phrase, &q.original)
        else {
            return Ok(HashSet::new());
        };
        let phrase = format!("{a} {b}");
        let relations = self.nominal_relations(&phrase).await?;
        if !relations
            .iter()
            .any(|(x, y, _)| x == &[a.to_owned()] && y == &[b.to_owned()])
        {
            return Ok(HashSet::new());
        }
        let candidates = self
            .matcher_hits(true, &query::quote(&phrase), q, ctx)
            .await?;
        let ids: Vec<_> = candidates.into_iter().collect();
        self.load_blocks(ids.iter().copied()).await?;
        self.nominal_compound_sources(&ids, a, b).await
    }
    fn penalized(&self, ranking: &[Rank], ctx: &Context) -> Vec<usize> {
        let mut seen = HashSet::new();
        let mut rows: Vec<_> = ranking
            .iter()
            .filter(|r| seen.insert(self.rows[&r.sid].record.clone()))
            .enumerate()
            .collect();
        rows.sort_by(|(ia, a), (ib, b)| {
            let penalty = |r: &Rank| {
                if self.records[&self.rows[&r.sid].record]
                    .scope
                    .as_ref()
                    .is_some_and(|s| {
                        s.to_lowercase() != ctx.scope.as_deref().unwrap_or("").to_lowercase()
                    })
                {
                    7.
                } else {
                    0.
                }
            };
            a.tier
                .cmp(&b.tier)
                .then_with(|| (-a.score + penalty(a)).total_cmp(&(-b.score + penalty(b))))
                .then_with(|| ia.cmp(ib))
        });
        let cut: Vec<_> = rows.into_iter().take(5).map(|(_, r)| r.sid).collect();
        if crate::diagnostics::enabled("LINN_POS_RANK_TRACE") {
            crate::diagnostics::emit(format!(
                "{}",
                serde_json::json!({"stage":"record_cut","rows":cut})
            ));
        }
        cut
    }
    fn admit(&self, q: &Query, m: &Matched, b: bool, coverage: &Coverage) -> Vec<Rank> {
        let mut out = m.ranking.clone();
        if out.is_empty() {
            return out;
        }
        let have: HashSet<_> = out
            .iter()
            .map(|r| self.rows[&r.sid].record.clone())
            .collect();
        let need = q.need;
        let candidate = coverage
            .counts
            .iter()
            .filter(|(id, count)| {
                **count >= need.saturating_sub(1)
                    && **count < need
                    && q.terms
                        .iter()
                        .enumerate()
                        .all(|(i, t)| !t.hard() || coverage.hits[i].contains(id))
                    && !have.contains(&self.rows[id].record)
            })
            .map(|(&id, _)| Rank {
                tier: 2,
                score: coverage.scores[&id],
                sid: id,
            })
            .min_by(|a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| self.rows[&a.sid].ordinal.cmp(&self.rows[&b.sid].ordinal))
                    .then_with(|| {
                        coverage
                            .arrival
                            .get(&a.sid)
                            .cmp(&coverage.arrival.get(&b.sid))
                    })
            });
        if let Some(row) = candidate {
            out.push(row);
        }
        if b {
            let have: HashSet<_> = out
                .iter()
                .map(|r| self.rows[&r.sid].record.clone())
                .collect();
            let mut best = Vec::<(String, usize, usize)>::new();
            let tree_ids = self
                .tree_candidates(q, false)
                .into_iter()
                .map(|id| &self.direct[&id])
                .chain(
                    self.tree_candidates(q, true)
                        .into_iter()
                        .map(|id| &self.deep[&id]),
                );
            let mut positions = HashMap::<String, usize>::new();
            for tree in tree_ids {
                if have.contains(&tree.record) || !tree_covers(tree, q, need) {
                    continue;
                }
                if let Some((hits, id)) = self.tree_block(tree, q) {
                    if let Some(&position) = positions.get(&tree.record) {
                        let prev = &mut best[position];
                        if hits > prev.1 {
                            *prev = (tree.record.clone(), hits, id);
                        }
                    } else {
                        positions.insert(tree.record.clone(), best.len());
                        best.push((tree.record.clone(), hits, id));
                    }
                }
            }
            out.extend(best.into_iter().map(|(_, _, id)| {
                Rank {
                    tier: 2,
                    score: m
                        .scores
                        .get(&id)
                        .copied()
                        .unwrap_or_else(|| m.scoring.score(&self.rows[&id])),
                    sid: id,
                }
            }));
        }
        out
    }
    fn tree_block(&self, t: &Tree, q: &Query) -> Option<(usize, usize)> {
        if !self.hint_eligible(&t.record) {
            return None;
        }
        let want = &q.words;
        t.rows
            .iter()
            .filter(|id| self.allowed_query(&self.rows[id], q))
            .map(|id| (want.intersection(&self.rows[id].words).count(), *id))
            .max_by(|(a, ia), (b, ib)| a.cmp(b).then_with(|| ib.cmp(ia)))
    }
    fn tree_candidates(&self, q: &Query, deep: bool) -> Vec<usize> {
        crate::storage::posting_candidates(
            if deep {
                &self.indexes.deep
            } else {
                &self.indexes.direct
            },
            q.words.iter().map(String::as_str),
            q.need,
        )
    }
    async fn append_trees(
        &mut self,
        page: &mut Vec<Card>,
        q: &Query,
        deep: bool,
        max: usize,
    ) -> Result<()> {
        let have: HashSet<_> = page.iter().map(|c| c.record.clone()).collect();
        let need = q.need;
        let want = &q.words;
        let mut best = Vec::<(usize, String, usize)>::new();
        let mut positions = HashMap::<String, usize>::new();
        for id in self.tree_candidates(q, deep) {
            let tree = if deep {
                &self.deep[&id]
            } else {
                &self.direct[&id]
            };
            if have.contains(&tree.record) || !tree_covers(tree, q, need) {
                continue;
            }
            let Some((_, sid)) = self.tree_block(tree, q) else {
                continue;
            };
            let count = want.intersection(&tree.words).count();
            if let Some(&position) = positions.get(&tree.record) {
                let prev = &mut best[position];
                if count > prev.0 {
                    *prev = (count, tree.record.clone(), sid);
                }
            } else {
                positions.insert(tree.record.clone(), best.len());
                best.push((count, tree.record.clone(), sid));
            }
        }
        best.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
        for (_, _, sid) in best.into_iter().take(max) {
            if page.len() == 5 {
                break;
            }
            self.load_blocks([sid]).await?;
            page.push(self.card(sid, Some(q)));
        }
        Ok(())
    }
    async fn append_record(&mut self, page: &mut Vec<Card>, q: &Query) -> Result<()> {
        let have: HashSet<_> = page.iter().map(|c| c.record.clone()).collect();
        if self.indexed_navigation(q) {
            if let Some(id) = self.selected_record_fill(q, &have).await? {
                self.load_blocks([id]).await?;
                page.push(self.card(id, Some(q)));
            }
            return Ok(());
        }
        let want = &q.words;
        let hard = &q.hard_words;
        let need = q.need.saturating_sub(1);
        let mut best = None;
        let mut candidates = crate::storage::posting_candidates(
            &self.indexes.record_words,
            want.iter().map(String::as_str),
            need.max(1),
        );
        if need == 0
            && hard.is_empty()
            && let Some(id) = self.last_nonempty_record(&have).await?
        {
            candidates.push(id);
        }
        for id in candidates {
            let r = &self.indexes.records[&id];
            if have.contains(r) || !self.record_eligible(r) {
                continue;
            }
            let lookup = &self.lookups[r];
            let union = &lookup.words;
            let count = union.intersection(want).count();
            if lookup.count == 0 || count < need || hard.iter().any(|w| !union.contains(w)) {
                continue;
            }
            if best
                .as_ref()
                .is_none_or(|(previous_count, previous): &(usize, &String)| {
                    count > *previous_count || (count == *previous_count && r > *previous)
                })
            {
                best = Some((count, r));
            }
        }
        let best = best.map(|(count, r)| (count, r.clone()));
        if let Some((_, r)) = &best {
            self.load_record_block_headers(r).await?;
        }
        if let Some((_, r)) = &best
            && let Some(b) = self.record_blocks(r).max_by(|a, b| {
                a.words
                    .intersection(want)
                    .count()
                    .cmp(&b.words.intersection(want).count())
                    .then_with(|| b.id.cmp(&a.id))
            })
        {
            let id = b.id;
            self.load_blocks([id]).await?;
            page.push(self.card(id, Some(q)));
        }
        Ok(())
    }
    async fn vector_scores(
        &mut self,
        vector: &[f32],
    ) -> Result<(Arc<crate::storage::BlockMatrix>, Vec<f32>)> {
        let cached = self.block_matrix().await?;
        let values = cached.matrix.multiply(vector);
        Ok((cached, values))
    }
    async fn page(
        &mut self,
        text: &str,
        q: &Query,
        ctx: &Context,
    ) -> Result<(Vec<Card>, Vec<Call>, Matched, Matched)> {
        self.set_query_clock(&ctx.now);
        if !self.indexed_navigation(q) {
            self.load_search_navigation(q).await?;
        }
        self.load_surface(&query::content(text)).await?;
        let scoring = {
            let stats = self.scope_stats(&ctx.now, q).await?;
            let mut scoring = Scoring::new(q, &stats);
            let idfs = scoring
                .terms
                .iter()
                .flat_map(|(_, words)| words.iter().cloned())
                .collect();
            scoring.pairs = self.occurrence_pairs(q, &idfs).await?;
            Arc::new(scoring)
        };
        let mut memo = HashMap::new();
        let a = self
            .matcher(q, ctx, false, scoring.clone(), &mut memo)
            .await?;
        let b = self.matcher(q, ctx, true, scoring, &mut memo).await?;
        self.rankings.insert(
            (text.into(), ctx.scope.clone(), ctx.now.clone(), false),
            a.ranking.clone(),
        );
        self.rankings.insert(
            (text.into(), ctx.scope.clone(), ctx.now.clone(), true),
            b.ranking.clone(),
        );
        self.confidences.insert(
            (text.into(), ctx.scope.clone(), ctx.now.clone()),
            b.confidence,
        );
        let atop = self.penalized(&a.ranking, ctx);
        let btop = self.penalized(&b.ranking, ctx);
        let (base, top, is_b) = if atop.is_empty() {
            (&b, &btop, true)
        } else {
            (&a, &atop, false)
        };
        self.load_blocks(atop.iter().chain(&btop).copied()).await?;
        let mut calls = vec![Call {
            cards: interleave(
                &atop
                    .iter()
                    .map(|id| self.card(*id, None))
                    .collect::<Vec<_>>(),
                &btop
                    .iter()
                    .map(|id| self.card(*id, None))
                    .collect::<Vec<_>>(),
            ),
            text: "Nothing qualified.".into(),
            ..Call::default()
        }];
        if !top.is_empty() {
            let mut seen: HashSet<_> = top.iter().copied().collect();
            let mut additional = Vec::new();
            for id in top {
                let rid = &self.rows[id].record;
                if let Some(r) = base
                    .ranking
                    .iter()
                    .find(|r| self.rows[&r.sid].record == *rid && !seen.contains(&r.sid))
                {
                    let sid = r.sid;
                    seen.insert(sid);
                    additional.push(sid);
                }
            }
            if !additional.is_empty() {
                self.load_blocks(additional.iter().copied()).await?;
                calls.push(call_from(
                    additional
                        .into_iter()
                        .map(|sid| self.later_card(sid, q))
                        .collect(),
                ));
            }
            let have: HashSet<_> = top.iter().map(|id| self.rows[id].record.clone()).collect();
            let neighbors = self.linked_neighbors(&have).await?;
            let mut hits = Vec::new();
            if !neighbors.is_empty() {
                for term in &q.terms {
                    hits.push(self.term_hits(term, is_b).await?);
                }
            }
            let hit_ids: HashSet<_> = hits.iter().flat_map(|hit| hit.iter().copied()).collect();
            let hit_ids = self
                .eligible_hit_ids(
                    &hit_ids,
                    &ctx.now,
                    Some(&neighbors.keys().cloned().collect()),
                )
                .await?;
            self.load_block_headers(hit_ids.iter().copied()).await?;
            let mut options: Vec<_> = hit_ids
                .into_iter()
                .filter_map(|id| {
                    let row = &self.rows[&id];
                    if !neighbors.contains_key(&row.record)
                        || !self.validity.eligible(&self.records[&row.record], &ctx.now)
                        || seen.contains(&row.id)
                    {
                        return None;
                    }
                    let count = hits.iter().filter(|h| h.contains(&row.id)).count();
                    (count > 0).then_some((count, row.id))
                })
                .collect();
            options.sort_by(|(a, ia), (b, ib)| {
                let x = &self.rows[ia];
                let y = &self.rows[ib];
                b.cmp(a)
                    .then_with(|| x.hash.cmp(&y.hash))
                    .then_with(|| x.ordinal.cmp(&y.ordinal))
                    .then_with(|| x.record.cmp(&y.record))
                    .then_with(|| ia.cmp(ib))
            });
            let mut used = HashSet::new();
            let mut selected = Vec::new();
            for (_, sid) in options {
                let rid = self.rows[&sid].record.clone();
                if !used.insert(rid.clone()) {
                    continue;
                }
                selected.push((sid, rid));
                if selected.len() == 5 {
                    break;
                }
            }
            self.load_blocks(selected.iter().map(|(sid, _)| *sid))
                .await?;
            let cards = selected
                .into_iter()
                .map(|(sid, rid)| {
                    let mut c = self.later_card(sid, q);
                    c.labels = neighbors[&rid].iter().cloned().collect();
                    c.labels.sort();
                    c
                })
                .collect::<Vec<_>>();
            if !cards.is_empty() {
                calls.push(call_from(cards));
            }
        }
        if !calls[0].cards.is_empty() {
            calls[0].related = self.related(q, &calls[0].cards).await?;
        }
        for call in &mut calls {
            for c in &mut call.cards {
                c.text = centered(
                    &self.rows[&c.section].text,
                    &query::content(
                        &q.terms
                            .iter()
                            .map(|t| t.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                );
            }
            if !call.cards.is_empty() {
                call.text = card_lines(&call.cards, true);
            }
        }
        let ac = self
            .coverage
            .entry((text.into(), ctx.scope.clone(), false))
            .or_insert_with(|| std::sync::Arc::new(Coverage::from(&a)))
            .clone();
        let bc = self
            .coverage
            .entry((text.into(), ctx.scope.clone(), true))
            .or_insert_with(|| std::sync::Arc::new(Coverage::from(&b)))
            .clone();
        let ar = self.admit(q, &a, false, &ac);
        let br = self.admit(q, &b, true, &bc);
        let ar_top = self.penalized(&ar, ctx);
        let br_top = self.penalized(&br, ctx);
        self.load_blocks(ar_top.iter().chain(&br_top).copied())
            .await?;
        let mut page = interleave(
            &ar_top
                .iter()
                .map(|id| self.card(*id, None))
                .collect::<Vec<_>>(),
            &br_top
                .iter()
                .map(|id| self.card(*id, None))
                .collect::<Vec<_>>(),
        );
        if !page.is_empty() && page.len() < 5 {
            self.append_trees(&mut page, q, false, 2).await?;
            if page.len() < 5 {
                self.append_trees(&mut page, q, true, 1).await?;
            }
            if page.len() < 5 {
                self.append_record(&mut page, q).await?;
            }
        }
        Ok((page, calls, a, b))
    }
    fn later_card(&self, sid: usize, q: &Query) -> Card {
        let mut c = self.card(sid, Some(q));
        c.text = centered(
            &self.rows[&sid].text,
            &query::content(
                &q.terms
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );
        c
    }
    pub(crate) async fn base_search(
        &mut self,
        query_text: &str,
        ctx: &Context,
    ) -> Result<(Vec<Call>, Vec<Rank>, Vec<Rank>)> {
        self.set_query_clock(&ctx.now);
        let q = match Query::parse(query_text) {
            Ok(q) => q,
            Err(e) => {
                let mut call = Call {
                    text: format!("Query error: {e}"),
                    ..Call::default()
                };
                self.hints(query_text, &mut call).await?;
                return Ok((
                    vec![call].into_iter().take(ctx.calls_left).collect(),
                    vec![],
                    vec![],
                ));
            }
        };
        if q.content().is_empty() {
            let mut call = Call {
                text: if q.terms.is_empty() {
                    "Nothing qualified. Supply text terms, or use show/outline navigation.".into()
                } else {
                    "Nothing qualified.".into()
                },
                ..Call::default()
            };
            self.hints(query_text, &mut call).await?;
            return Ok((
                vec![call].into_iter().take(ctx.calls_left).collect(),
                vec![],
                vec![],
            ));
        }
        let (mut page, mut calls, a, b) = self.page(query_text, &q, ctx).await?;
        if !page.is_empty() && page.len() < 5 {
            let text = query::split(query_text).1;
            let words = query::content(query_text);
            if !text.trim().is_empty() && !words.is_empty() {
                let have: HashSet<_> = page.iter().map(|c| c.record.clone()).collect();
                let mut candidates = self.rare_blocks(&words, 2.5);
                self.load_block_headers(candidates.iter().copied()).await?;
                candidates.retain(|sid| {
                    !have.contains(&self.rows[sid].record)
                        && self.hint_eligible(&self.rows[sid].record)
                });
                if !candidates.is_empty() {
                    let vector = self.encoder()?.encode(text.trim());
                    let best = self
                        .candidate_vector_scores(
                            &candidates.into_iter().collect::<Vec<_>>(),
                            &vector,
                        )
                        .await?
                        .into_iter()
                        .min_by(similarity_order);
                    if let Some((sid, _)) = best {
                        self.load_blocks([sid]).await?;
                        page.push(self.card(sid, None));
                    }
                }
            }
        }
        if !page.is_empty() {
            calls[0].text = card_lines(&page, false);
            calls[0].cards = page;
        } else {
            self.hints(query_text, &mut calls[0]).await?;
        }
        calls.truncate(ctx.calls_left);
        Ok((calls, a.ranking, b.ranking))
    }
    async fn related(&mut self, q: &Query, cards: &[Card]) -> Result<Vec<Pointer>> {
        let text = q
            .terms
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let words = query::content(&text);
        if words.len() < 2 {
            return Ok(vec![]);
        }
        let have: HashSet<_> = cards.iter().map(|c| c.record.as_str()).collect();
        let selected = if let Some(selected) = self
            .indexed_pointer_sections(
                &words,
                true,
                &have.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>(),
            )
            .await?
        {
            selected
        } else {
            self.load_section_words(&words).await?;
            let mut candidates = Vec::new();
            for id in crate::storage::posting_candidates(
                &self.indexes.related,
                words.iter().map(String::as_str),
                2.max(words.len().div_ceil(2)),
            ) {
                let rid = self.indexes.sections[&id].0.clone();
                if have.contains(rid.as_str()) {
                    continue;
                }
                self.load_record_headers([rid.clone()]).await?;
                if !self.hint_eligible(&rid) {
                    continue;
                }
                self.load_record_sections(&rid).await?;
                let (rid, ordinal) = &self.indexes.sections[&id];
                let sec = &self.sections[rid][*ordinal];
                if have.contains(rid.as_str()) || !self.hint_eligible(rid) {
                    continue;
                }
                if sec.span.is_empty() {
                    continue;
                }
                let tokens = &self.lookups[rid].sections[&sec.ordinal];
                let own = &tokens.own;
                let extra = &tokens.extra;
                let matched: Vec<_> = words
                    .iter()
                    .filter(|w| own.contains(*w) || extra.contains(*w))
                    .cloned()
                    .collect();
                let heading_only = matched.iter().filter(|w| !own.contains(*w)).count();
                if !tokens.own_nonempty
                    || matched.len() < 2.max(words.len().div_ceil(2))
                    || heading_only == 0
                {
                    continue;
                }
                candidates.push((
                    matched.len(),
                    heading_only,
                    sec.span.len(),
                    rid.clone(),
                    sec.span.start,
                    sec.clone(),
                    matched,
                ));
            }
            candidates.sort_by(|a, b| {
                b.0.cmp(&a.0)
                    .then_with(|| b.1.cmp(&a.1))
                    .then_with(|| a.2.cmp(&b.2))
                    .then_with(|| a.3.cmp(&b.3))
                    .then_with(|| a.4.cmp(&b.4))
            });
            let mut seen = HashSet::new();
            let selected: Vec<_> = candidates
                .into_iter()
                .filter(|c| seen.insert(c.3.clone()))
                .take(2)
                .map(|(_, _, _, rid, _, sec, matched)| (rid, sec, matched))
                .collect();
            selected
        };
        self.load_records(selected.iter().map(|c| c.0.clone()))
            .await?;
        Ok(selected
            .into_iter()
            .map(|(rid, sec, matched)| {
                let span = if sec.span.len() > 1500 {
                    crate::storage::sections::hint_spans(
                        self.records[&rid].body.as_bytes(),
                        sec.span.clone(),
                    )[0]
                    .clone()
                } else {
                    sec.span.clone()
                };
                let text =
                    String::from_utf8_lossy(&self.records[&rid].body.as_bytes()[span.clone()]);
                Pointer {
                    record: rid.clone(),
                    label: self.records[&rid].label.clone(),
                    heading: sec.path.join(" > "),
                    span: [span.start, span.end],
                    missing: words
                        .iter()
                        .filter(|w| !matched.contains(w))
                        .cloned()
                        .collect(),
                    matched,
                    excerpt: centered(&text, &words),
                    via: Some("heading".into()),
                }
            })
            .collect())
    }
    async fn hints(&mut self, text: &str, call: &mut Call) -> Result<()> {
        let words = query::content(text);
        self.load_surface(&words).await?;
        let candidates =
            if let Some(selected) = self.indexed_pointer_sections(&words, false, &[]).await? {
                selected
            } else {
                self.load_section_words(&words).await?;
                let mut candidates = Vec::new();
                if words.len() >= 2 {
                    for id in crate::storage::posting_candidates(
                        &self.indexes.own,
                        words.iter().map(String::as_str),
                        2.max(words.len().div_ceil(2)),
                    ) {
                        let rid = self.indexes.sections[&id].0.clone();
                        self.load_record_headers([rid.clone()]).await?;
                        if !self.hint_eligible(&rid) {
                            continue;
                        }
                        self.load_record_sections(&rid).await?;
                        let (rid, ordinal) = &self.indexes.sections[&id];
                        let sec = &self.sections[rid][*ordinal];
                        if !self.hint_eligible(rid) {
                            continue;
                        }
                        if sec.span.is_empty() {
                            continue;
                        }
                        let tokens = &self.lookups[rid].sections[&sec.ordinal].own;
                        let matched: Vec<_> = words
                            .iter()
                            .filter(|w| tokens.contains(*w))
                            .cloned()
                            .collect();
                        if matched.len() < 2.max(words.len().div_ceil(2)) {
                            continue;
                        }
                        candidates.push((
                            matched.len(),
                            sec.span.len(),
                            rid.clone(),
                            sec.span.start,
                            sec.clone(),
                            matched,
                        ));
                    }
                }
                candidates.sort_by(|a, b| {
                    b.0.cmp(&a.0)
                        .then_with(|| a.1.cmp(&b.1))
                        .then_with(|| a.2.cmp(&b.2))
                        .then_with(|| a.3.cmp(&b.3))
                });
                candidates.truncate(3);
                candidates
                    .into_iter()
                    .map(|(_, _, rid, _, sec, matched)| (rid, sec, matched))
                    .collect()
            };
        self.load_records(candidates.iter().map(|c| c.0.clone()))
            .await?;
        let mut word_hints = Vec::new();
        for (rid, sec, matched) in candidates {
            let r = &self.records[&rid];
            let span = if sec.span.len() <= 1500 {
                sec.span.clone()
            } else {
                let mut spans =
                    crate::storage::sections::hint_spans(r.body.as_bytes(), sec.span.clone());
                spans.sort_by_key(|span| {
                    std::cmp::Reverse(
                        matched
                            .iter()
                            .filter(|w| {
                                query::surface(&String::from_utf8_lossy(
                                    &r.body.as_bytes()[(*span).clone()],
                                ))
                                .contains(w)
                            })
                            .count(),
                    )
                });
                spans[0].clone()
            };
            let text = String::from_utf8_lossy(&r.body.as_bytes()[span.clone()]);
            let excerpt = text
                .lines()
                .find(|line| {
                    let tokens = query::surface(line);
                    matched.iter().any(|w| tokens.contains(w))
                })
                .unwrap_or("")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(160)
                .collect();
            word_hints.push(Pointer {
                record: rid,
                label: r.label.clone(),
                heading: sec.path.join(" > "),
                span: [span.start, span.end],
                missing: words
                    .iter()
                    .filter(|w| !matched.contains(w))
                    .cloned()
                    .collect(),
                matched,
                excerpt,
                via: None,
            });
        }
        let mut vector_hints = Vec::new();
        let text = query::split(text).1;
        if !text.trim().is_empty() && !self.rare_blocks(&words, 2.5).is_empty() {
            let vector = self.encoder()?.encode(text.trim());
            let mut seen: HashSet<_> = word_hints.iter().map(|h| h.record.clone()).collect();
            let mut selected = 0;
            let (cached, scores) = self.vector_scores(&vector).await?;
            let seen_records = seen
                .iter()
                .filter_map(|id| self.indexes.record_positions.get(id))
                .copied()
                .collect();
            let nearest = self.hint_candidates(
                &cached.ids,
                &cached.records,
                &cached
                    .eligible
                    .iter()
                    .zip(&cached.expires)
                    .map(|(eligible, expires)| {
                        *eligible && (expires.is_empty() || expires > &self.query_clock)
                    })
                    .collect::<Vec<_>>(),
                scores,
                &seen_records,
            );
            self.load_blocks(nearest.iter().map(|(id, _)| *id)).await?;
            for (sid, _) in nearest {
                let b = &self.rows[&sid];
                if !self.hint_eligible(&b.record) || !seen.insert(b.record.clone()) {
                    continue;
                }
                selected += 1;
                let matched: Vec<_> = words
                    .iter()
                    .filter(|w| b.surface.contains(*w))
                    .cloned()
                    .collect();
                if matched.iter().any(|w| self.idf(w) >= 2.5) {
                    vector_hints.push(Pointer {
                        record: b.record.clone(),
                        label: self.records[&b.record].label.clone(),
                        heading: b.path.join(" > "),
                        span: [b.start, b.end],
                        missing: words
                            .iter()
                            .filter(|w| !matched.contains(w))
                            .cloned()
                            .collect(),
                        matched,
                        excerpt: centered(&b.text, &words),
                        via: Some("vector".into()),
                    });
                }
                if selected == 3 {
                    break;
                }
            }
        }
        for i in 0..3 {
            if let Some(h) = word_hints.get(i) {
                call.hints.push(h.clone());
            }
            if let Some(h) = vector_hints.get(i) {
                call.hints.push(h.clone());
            }
        }
        call.hints.truncate(3);
        if !word_hints.is_empty() {
            call.text
                .push_str("\nNothing qualified. Near misses (not results):");
            for (i, h) in word_hints.iter().enumerate() {
                call.text.push_str(&format!(
                    "\n[{}] {} — {} — matched: {}; missing: {} — {}",
                    i + 1,
                    if h.label.is_empty() {
                        &h.record
                    } else {
                        &h.label
                    },
                    if h.heading.is_empty() {
                        "(document start)"
                    } else {
                        &h.heading
                    },
                    h.matched.join(", "),
                    if h.missing.is_empty() {
                        "-".into()
                    } else {
                        h.missing.join(", ")
                    },
                    h.excerpt
                ));
            }
        }
        Ok(())
    }
    fn hint_candidates(
        &self,
        ids: &[usize],
        records: &[usize],
        eligible: &[bool],
        scores: Vec<f32>,
        seen: &HashSet<usize>,
    ) -> Vec<(usize, f32)> {
        let mut best = HashMap::<usize, (usize, f32)>::new();
        for (((&sid, &record), &eligible), score) in
            ids.iter().zip(records).zip(eligible).zip(scores)
        {
            if seen.contains(&record) || !eligible {
                continue;
            }
            let candidate = (sid, score);
            let entry = best.entry(record).or_insert(candidate);
            if similarity_order(&candidate, entry).is_lt() {
                *entry = candidate;
            }
        }
        let mut nearest: Vec<_> = best.into_values().collect();
        if nearest.len() > 3 {
            nearest.select_nth_unstable_by(3, similarity_order);
        }
        nearest.truncate(3);
        nearest.sort_by(similarity_order);
        nearest
    }
}
fn similarity_order(a: &(usize, f32), b: &(usize, f32)) -> std::cmp::Ordering {
    b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0))
}

fn tree_covers(t: &Tree, q: &Query, need: usize) -> bool {
    let want = &q.words;
    !want.is_empty()
        && want.intersection(&t.words).count() >= need
        && q.hard_words.iter().all(|word| t.words.contains(word))
}
fn interleave(a: &[Card], b: &[Card]) -> Vec<Card> {
    let mut out = Vec::<Card>::new();
    for i in 0..a.len().max(b.len()) {
        for list in [a, b] {
            if let Some(c) = list.get(i) {
                if !out
                    .iter()
                    .any(|x| x.record == c.record && x.span[0] < c.span[1] && c.span[0] < x.span[1])
                {
                    out.push(c.clone());
                }
                if out.len() == 5 {
                    return out;
                }
            }
        }
    }
    out
}
pub(crate) fn card_lines(cards: &[Card], numbered: bool) -> String {
    cards
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "{}{}: {}",
                if numbered {
                    format!("[{}] ", i + 1)
                } else {
                    String::new()
                },
                if numbered && c.heading.is_empty() {
                    &c.record
                } else {
                    &c.heading
                },
                c.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn call_from(cards: Vec<Card>) -> Call {
    Call {
        text: card_lines(&cards, true),
        cards,
        ..Call::default()
    }
}
pub fn centered(text: &str, words: &[String]) -> String {
    let lines: Vec<_> = text.lines().filter(|s| !s.trim().is_empty()).collect();
    let Some(line) = lines
        .iter()
        .enumerate()
        .max_by_key(|(i, s)| {
            let tokens = query::surface(s);
            (
                words.iter().filter(|w| tokens.contains(w)).count(),
                std::cmp::Reverse(*i),
            )
        })
        .map(|(_, s)| *s)
    else {
        return String::new();
    };
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() <= 160 {
        return line;
    }
    let lower = line.to_lowercase();
    let pos = words
        .iter()
        .filter_map(|w| lower.find(w))
        .min()
        .map(|p| lower[..p].chars().count())
        .unwrap_or(0);
    let start = pos
        .saturating_sub(40)
        .min(line.chars().count().saturating_sub(159));
    if start > 0 {
        format!(
            "…{}",
            line.chars().skip(start).take(159).collect::<String>()
        )
    } else {
        line.chars().take(160).collect()
    }
}

#[cfg(test)]
mod indexed_tests {
    use super::*;
    #[tokio::test]
    async fn indexed_hint_selection_matches_materialized_sections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pointers.db");
        let mut records = Vec::new();
        for (id, body, state, expires) in [
            ("a", "# alpha\n## Leaf\nbeta gamma", "accepted", ""),
            ("b", "# alpha\n## Leaf\nbeta gamma", "accepted", ""),
            (
                "c",
                "# alpha beta\ngamma delta\n## Child\nbeta gamma delta",
                "accepted",
                "",
            ),
            ("candidate", "alpha beta gamma delta", "candidate", ""),
            (
                "expired",
                "alpha beta gamma delta",
                "accepted",
                crate::storage::NOW,
            ),
            ("blank", "   ", "accepted", ""),
        ] {
            records.push(
                serde_json::from_value(serde_json::json!({
                    "id":id,"body":body,"state":state,"stale_after":expires,"label":"delta"
                }))
                .unwrap(),
            );
        }
        let mut full = Store::build(&path, records).await.unwrap();
        let mut lazy = Store::open(&path).await.unwrap();
        assert!(
            full.indexed_pointer_sections(&[], false, &[])
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            lazy.indexed_pointer_sections(&[], false, &[])
                .await
                .unwrap()
                .is_some()
        );
        for text in [
            "alpha beta",
            "alpha alpha beta",
            "alpha alpha",
            "delta beta",
            "beta gamma",
            "gamma delta beta",
        ] {
            let mut a = Call::default();
            let mut b = Call::default();
            full.hints(text, &mut a).await.unwrap();
            lazy.hints(text, &mut b).await.unwrap();
            assert_eq!(
                serde_json::to_value(a).unwrap(),
                serde_json::to_value(b).unwrap(),
                "hints: {text}"
            );
            let query = Query::parse(text).unwrap();
            for cards in [
                vec![],
                vec![Card {
                    record: "a".into(),
                    ..Card::default()
                }],
            ] {
                let a = full.related(&query, &cards).await.unwrap();
                let b = lazy.related(&query, &cards).await.unwrap();
                assert_eq!(
                    serde_json::to_value(a).unwrap(),
                    serde_json::to_value(b).unwrap(),
                    "related: {text}"
                );
            }
        }
    }
    #[tokio::test]
    #[ignore = "requires LINN_VECTOR_PARITY_STORE pointing to a copied store"]
    async fn candidate_vector_corpus_bits() {
        let path = std::env::var_os("LINN_VECTOR_PARITY_STORE").unwrap();
        let mut full = Store::open(std::path::Path::new(&path)).await.unwrap();
        let sparse = Store::open(std::path::Path::new(&path)).await.unwrap();
        let matrix = full.block_matrix().await.unwrap();
        assert!(!matrix.ids.is_empty(), "requires a populated copied store");
        crate::diagnostics::emit(format!(
            "checking selected float32 scores across {} rows",
            matrix.ids.len()
        ));
        for stride in [37, 101] {
            let ids: Vec<_> = matrix.ids.iter().step_by(stride).copied().collect();
            for text in [
                "cache policy",
                "database query",
                "alpha-beta",
                "the memory of a distant city",
            ] {
                let query = full.encoder().unwrap().encode(text);
                let expected = full.candidate_vector_scores(&ids, &query).await.unwrap();
                let actual = sparse.candidate_vector_scores(&ids, &query).await.unwrap();
                assert_eq!(
                    actual
                        .iter()
                        .map(|(id, score)| (*id, score.to_bits()))
                        .collect::<Vec<_>>(),
                    expected
                        .iter()
                        .map(|(id, score)| (*id, score.to_bits()))
                        .collect::<Vec<_>>()
                );
            }
        }
        assert!(sparse.vector_matrix.is_none());
    }

    #[tokio::test]
    async fn candidate_vector_scores_preserve_full_matrix_bits_and_gaps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectors.db");
        let records = (0..37)
            .map(|i| {
                record(
                    &format!("r{i}"),
                    &format!("cache database query policy {i}"),
                    "accepted",
                )
            })
            .collect();
        drop(Store::build(&path, records).await.unwrap());
        for gapped in [false, true] {
            let mut full = Store::open(&path).await.unwrap();
            if gapped {
                full.db
                    .execute("DELETE FROM vectors WHERE id % 5=0", ())
                    .await
                    .unwrap();
            }
            let matrix = full.block_matrix().await.unwrap();
            let selected: Vec<_> = matrix
                .ids
                .iter()
                .step_by(3)
                .copied()
                .chain([usize::MAX / 2])
                .collect();
            let sparse = Store::open(&path).await.unwrap();
            for text in ["cache policy", "database query", "alpha-beta"] {
                let query = full.encoder().unwrap().encode(text);
                let expected = full
                    .candidate_vector_scores(&selected, &query)
                    .await
                    .unwrap();
                let actual = sparse
                    .candidate_vector_scores(&selected, &query)
                    .await
                    .unwrap();
                assert_eq!(
                    actual
                        .iter()
                        .map(|(id, score)| (*id, score.to_bits()))
                        .collect::<Vec<_>>(),
                    expected
                        .iter()
                        .map(|(id, score)| (*id, score.to_bits()))
                        .collect::<Vec<_>>()
                );
                assert!(sparse.vector_matrix.is_none());
            }
        }
    }

    #[tokio::test]
    async fn shown_rare_neighbors_do_not_load_vectors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let mut records = vec![record("target", "quasar-control", "accepted")];
        records.extend((0..15).map(|i| {
            record(
                &format!("filler-{i}"),
                "ordinary filler padding",
                "accepted",
            )
        }));
        drop(Store::build(&path, records).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        let (calls, _, _) = store
            .base_search("quasar-control", &Context::default())
            .await
            .unwrap();
        assert_eq!(calls[0].cards[0].record, "target");
        assert!(store.vector_matrix.is_none());
    }

    fn record(id: &str, body: &str, state: &str) -> crate::storage::record::Record {
        serde_json::from_value(
            serde_json::json!({"id":id,"body":body,"state":state,"label":"label only"}),
        )
        .unwrap()
    }
    #[tokio::test]
    async fn sparse_scores_and_fills_equal_full_scans() {
        let dir = tempfile::tempdir().unwrap();
        let mut expires = record("expires", "# Group\nalpha\n## Sub\nbeta", "accepted");
        expires.stale_after = Some("2026-11-06T00:00:00Z".into());
        let mut store = Store::build(
            &dir.path().join("test.db"),
            vec![
                record("a", "alpha beta foo_bar résumé", "accepted"),
                record(
                    "b",
                    "# Heading\nbeta\n## Child\nalpha foo-bar camelCase",
                    "accepted",
                ),
                record("z", "unrelated closed candidate", "candidate"),
                record("zz", "   ", "accepted"),
                expires,
            ],
        )
        .await
        .unwrap();
        for text in [
            "alpha beta",
            "alpha",
            "missing",
            "command",
            "\"alpha\" alpha",
            "foo_bar résumé",
            "path:elsewhere/** alpha",
        ] {
            let q = Query::parse(text).unwrap();
            for clock in [crate::storage::NOW, "2027-01-01T00:00:00Z"] {
                let stats = store.scope_stats(clock, &q).await.unwrap();
                store
                    .load_terms(
                        q.terms
                            .iter()
                            .flat_map(|t| query::surface(&t.text))
                            .map(|w| query::norm(&w)),
                        false,
                    )
                    .await
                    .unwrap();
                let eligible: Vec<_> = store
                    .rows
                    .values()
                    .filter(|b| store.validity.eligible(&store.records[&b.record], clock))
                    .collect();
                let n = eligible.len() as f64;
                let avg = eligible.iter().map(|b| b.length).sum::<usize>() as f64 / n.max(1.);
                assert_eq!(stats.n.to_bits(), n.to_bits());
                assert_eq!(stats.avg.to_bits(), avg.to_bits());
                let scorer = Scoring::new(&q, &stats);
                for row in store.rows.values() {
                    let mut expected = 0.;
                    for term in &q.terms {
                        for word in query::surface(&term.text) {
                            let word = query::norm(&word);
                            let df = store
                                .rows
                                .values()
                                .filter(|b| {
                                    store.validity.eligible(&store.records[&b.record], clock)
                                        && b.tf.contains_key(&word)
                                })
                                .count() as f64;
                            assert_eq!(stats.df.get(&word).copied().unwrap_or(0.), df);
                            let idf = (1. + (stats.n - df + 0.5) / (df + 0.5)).ln();
                            let raw = row.tf.get(&word).copied().unwrap_or_default();
                            let tf =
                                raw[1] + 3. * raw[0] + if !term.hard() { 0.3 * raw[2] } else { 0. };
                            if tf > 0. {
                                let normalized = tf
                                    / (0.25 + 0.75 * row.length as f64 / stats.avg.max(1.))
                                    + 0.125;
                                expected += idf * normalized * 2.2 / (normalized + 1.2);
                            }
                        }
                    }
                    assert_eq!(scorer.score(row).to_bits(), expected.to_bits());
                }
            }
            let mut original = vec![store.card(*store.rows.keys().next().unwrap(), None)];
            let mut fast = original.clone();
            let have: HashSet<_> = original.iter().map(|c| c.record.clone()).collect();
            let want: HashSet<_> = q.content().iter().map(|t| query::norm(&t.text)).collect();
            let hard: Vec<_> = q
                .terms
                .iter()
                .filter(|t| t.hard())
                .map(|t| query::norm(&t.text))
                .collect();
            let need = query::need(q.content().len()).saturating_sub(1);
            let mut options = Vec::new();
            for rid in store
                .records
                .keys()
                .filter(|id| !have.contains(*id) && store.record_eligible(id))
            {
                let rows: Vec<_> = store.rows.values().filter(|b| &b.record == rid).collect();
                let union: HashSet<_> = rows.iter().flat_map(|b| b.words.iter().cloned()).collect();
                let count = union.intersection(&want).count();
                if count < need || hard.iter().any(|w| !union.contains(w)) {
                    continue;
                }
                if let Some(block) = rows.iter().max_by(|a, b| {
                    a.words
                        .intersection(&want)
                        .count()
                        .cmp(&b.words.intersection(&want).count())
                        .then_with(|| b.id.cmp(&a.id))
                }) {
                    options.push((count, rid, block.id));
                }
            }
            options.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(a.1)));
            if let Some((_, _, sid)) = options.first() {
                original.push(store.card(*sid, Some(&q)));
            }
            store.append_record(&mut fast, &q).await.unwrap();
            assert_eq!(
                serde_json::to_value(fast).unwrap(),
                serde_json::to_value(original).unwrap()
            );
        }
        let q = Query::parse("alpha beta").unwrap();
        let ctx = Context {
            now: "2027-01-01T00:00:00Z".into(),
            ..Context::default()
        };
        let (_, _, _, matched) = store.page("alpha beta", &q, &ctx).await.unwrap();
        let expired_id = store.record_blocks("expires").next().unwrap().id;
        assert!(!matched.scores.contains_key(&expired_id));
        let coverage = Coverage::from(&matched);
        let admitted = store.admit(&q, &matched, true, &coverage);
        assert!(admitted.iter().all(|r| r.sid != expired_id));
    }
    #[tokio::test]
    async fn partial_nearest_keeps_global_record_order_before_word_gates() {
        let dir = tempfile::tempdir().unwrap();
        let body = format!(
            "{}\n\n{}",
            "alpha line\n".repeat(150),
            "beta line\n".repeat(150)
        );
        let store = Store::build(
            &dir.path().join("test.db"),
            ["a", "b", "c", "d", "e"]
                .iter()
                .map(|id| record(id, &body, if *id == "d" { "candidate" } else { "accepted" }))
                .collect(),
        )
        .await
        .unwrap();
        let ids: Vec<_> = store.rows.keys().copied().collect();
        let values = [0., -0., 0.9, 0.8, -0.3, f32::NAN, 0.9];
        for seen in [HashSet::new(), HashSet::from(["a".into(), "c".into()])] {
            let scores: Vec<_> = (0..ids.len()).map(|i| values[i % values.len()]).collect();
            let mut old: Vec<_> = ids.iter().copied().zip(scores.clone()).collect();
            old.sort_by(similarity_order);
            let mut used = seen.clone();
            let expected: Vec<_> = old
                .into_iter()
                .filter(|(sid, _)| {
                    let rid = &store.rows[sid].record;
                    store.hint_eligible(rid) && used.insert(rid.clone())
                })
                .take(3)
                .map(|(id, _)| id)
                .collect();
            let actual: Vec<_> = store
                .hint_candidates(
                    &ids,
                    &ids.iter()
                        .map(|id| store.indexes.record_positions[&store.rows[id].record])
                        .collect::<Vec<_>>(),
                    &ids.iter()
                        .map(|id| store.hint_eligible(&store.rows[id].record))
                        .collect::<Vec<_>>(),
                    scores,
                    &seen
                        .iter()
                        .map(|rid| store.indexes.record_positions[rid])
                        .collect(),
                )
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
