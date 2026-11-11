//! POS-selected original-occurrence evidence.
//! @ref LLP 0002#ordered-occurrence-support
use crate::{error::Result, retrieval::query::Query, storage::Store};
use std::collections::HashMap;

pub(crate) type Pair = (Vec<String>, Vec<String>, f64);

// Cut original structural units without packing them into index blocks. Heading
// lines are isolated, and fenced bytes supply no natural-language relations.
pub(crate) fn units(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for span in crate::storage::sections::block_spans(text, 0..text.len(), 0) {
        let part = &text[span];
        if part.trim_start().starts_with("```") || part.trim_start().starts_with("~~~") {
            continue;
        }
        let mut start = 0;
        let mut offset = 0;
        for line in part.split_inclusive('\n') {
            if line.contains('|') {
                if start < offset {
                    out.push(&part[start..offset]);
                }
                out.extend(line.split('|').filter(|cell| !cell.trim().is_empty()));
                start = offset + line.len();
            } else if line.trim_start().starts_with('#') {
                if start < offset {
                    out.push(&part[start..offset]);
                }
                out.push(line);
                start = offset + line.len();
            }
            offset += line.len();
        }
        if start < part.len() {
            out.push(&part[start..]);
        }
    }
    out
}

#[cfg(test)]
fn support(tokens: &[String], a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.;
    }
    let mut best = 0.0_f64;
    let mut end = None;
    for i in 0..tokens.len() {
        if i >= a.len() && tokens[i - a.len()..i] == *a {
            end = Some(i);
        }
        if tokens[i..].starts_with(b)
            && let Some(previous) = end
        {
            best = best.max(1. / (1 + i - previous) as f64);
        }
    }
    best
}

impl Store {
    pub(crate) async fn occurrence_tokens(&self, texts: &[&str]) -> Result<Vec<Vec<String>>> {
        texts
            .iter()
            .map(|text| crate::nlp::porter::exact_tokens(text).map_err(Into::into))
            .collect()
    }
    pub(crate) async fn occurrence_pairs(
        &self,
        q: &Query,
        idfs: &HashMap<String, f64>,
    ) -> Result<Vec<Pair>> {
        let terms: Vec<_> = q
            .terms
            .iter()
            .filter(|t| t.content())
            .map(|t| t.text.as_str())
            .collect();
        let tokens = self.occurrence_tokens(&terms).await?;
        if !self.pos_pairs_possible(&tokens).await? {
            return Ok(Vec::new());
        }
        let selected = self.nominal_relations(&q.original).await?;
        Ok(tokens
            .windows(2)
            .filter(|w| selected.iter().any(|(a, b, _)| a == &w[0] && b == &w[1]))
            .filter(|w| !w[0].is_empty() && !w[1].is_empty())
            .map(|w| {
                let idf = w
                    .iter()
                    .flatten()
                    .map(|t| idfs.get(t).copied().unwrap_or(0.))
                    .fold(f64::INFINITY, f64::min);
                (w[0].clone(), w[1].clone(), idf)
            })
            .collect())
    }
    pub(crate) async fn occurrence_bonuses(
        &self,
        ids: &[usize],
        pairs: &[Pair],
    ) -> Result<HashMap<usize, f64>> {
        let eligible: Vec<_> = ids
            .iter()
            .copied()
            .filter(|id| {
                pairs.iter().any(|(a, b, _)| {
                    a.iter().chain(b).all(|word| {
                        self.rows[id]
                            .tf
                            .get(word)
                            .is_some_and(|counts| counts[1] > 0.)
                    })
                })
            })
            .collect();
        self.projected_pos_bonuses(&eligible, pairs).await
    }
    #[cfg(test)]
    pub(crate) async fn occurrence_bonus(&self, id: usize, pairs: &[Pair]) -> Result<f64> {
        let row = &self.rows[&id];
        // Reuse the on-demand exact postings before parsing candidate bytes.
        // A parts-only or heading-only occurrence cannot support this relation.
        if !pairs.iter().any(|(a, b, _)| {
            a.iter()
                .chain(b)
                .all(|word| row.tf.get(word).is_some_and(|counts| counts[1] > 0.))
        }) {
            return Ok(0.);
        }
        self.projected_pos_bonus(id, pairs).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }
    #[test]
    fn ordered_gaps_and_indivisible_terms() {
        let a = words("alpha");
        let b = words("beta");
        assert_eq!(support(&words("alpha beta"), &a, &b), 1.);
        assert_eq!(support(&words("alpha x beta"), &a, &b), 0.5);
        assert_eq!(support(&words("beta alpha"), &a, &b), 0.);
        assert_eq!(support(&words("alpha"), &a, &a), 0.);
        assert_eq!(support(&words("alpha alpha"), &a, &a), 1.);
        assert_eq!(
            support(
                &words("alpha beta gamma"),
                &words("alpha beta"),
                &words("beta gamma")
            ),
            0.
        );
    }
    #[test]
    fn original_units_do_not_invent_adjacency() {
        assert_eq!(
            units("# alpha\nbeta\n\n- gamma\n- delta\n```\nalpha beta\n```\n"),
            vec!["# alpha\n", "beta\n\n", "- gamma\n", "- delta\n"]
        );
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use super::*;
    #[tokio::test]
    async fn original_sql_token_offsets_and_field_isolation() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::build(&dir.path().join("units.db"), vec![])
            .await
            .unwrap();
        let text = "résumé foo_bar alpha-beta camelCase 你好 alpha, beta";
        let tokens = store.occurrence_tokens(&[text]).await.unwrap();
        assert_eq!(
            tokens[0],
            vec![
                "resume",
                "foo_bar",
                "alpha-beta",
                "camelcase",
                "你好",
                "alpha",
                "beta"
            ]
        );
        let separate = store
            .occurrence_tokens(&units(
                "# alpha\nbeta\n\nalpha\n\nbeta\n```\nalpha beta\n```\n",
            ))
            .await
            .unwrap();
        assert!(
            separate
                .iter()
                .all(|t| support(t, &["alpha".into()], &["beta".into()]) == 0.)
        );
        assert_eq!(support(&tokens[0], &["alpha".into()], &["beta".into()]), 1.);
        assert_eq!(support(&tokens[0], &["foo".into()], &["bar".into()]), 0.);
    }
}

#[cfg(test)]
mod posting_tests {
    use super::*;
    #[tokio::test]
    async fn exact_postings_and_original_units_guard_relations() {
        let dir = tempfile::tempdir().unwrap();
        let records = [
            ("ordered", "alpha beta"),
            ("reverse", "beta alpha"),
            ("fields", "# alpha\nbeta"),
            ("parts", "alpha_beta"),
            ("fence", "```\nalpha beta\n```\n"),
        ]
        .into_iter()
        .map(|(id, body)| {
            serde_json::from_value(
                serde_json::json!({"id":id,"body":body,"kind":"note","state":"accepted"}),
            )
            .unwrap()
        })
        .collect();
        let mut store = Store::build(&dir.path().join("postings.db"), records)
            .await
            .unwrap();
        let q = Query::parse("alpha beta").unwrap();
        let stats = store.scope_stats("2026-11-05T00:00:00Z", &q).await.unwrap();
        store
            .load_terms(["alpha".into(), "beta".into()], false)
            .await
            .unwrap();
        let idfs = ["alpha", "beta"]
            .into_iter()
            .map(|w| {
                (
                    w.to_owned(),
                    (1. + (stats.n - stats.df[w] + 0.5) / (stats.df[w] + 0.5)).ln(),
                )
            })
            .collect();
        let pairs = store.occurrence_pairs(&q, &idfs).await.unwrap();
        for row in store.rows.values() {
            let bonus = store.occurrence_bonus(row.id, &pairs).await.unwrap();
            if row.record == "ordered" {
                assert!(bonus > 0.);
            } else {
                assert_eq!(bonus, 0., "{}", row.record);
            }
        }
    }
}
