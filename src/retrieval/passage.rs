//! @ref LLP 0002#record-local-passage-selection
use crate::retrieval::query::Query;
use regex::Regex;
use std::{cmp::Ordering, collections::BTreeSet};

pub(super) struct Evidence {
    pub id: usize,
    pub exact: BTreeSet<usize>,
    pub union: BTreeSet<usize>,
    pub hard: BTreeSet<usize>,
    pub missing_hard: usize,
    pub missing_quoted: usize,
}

pub(super) struct BoundPair {
    left: usize,
    source: Regex,
}

pub(super) fn bound_pairs(q: &Query) -> Vec<BoundPair> {
    q.terms
        .windows(2)
        .enumerate()
        .filter(|(_, terms)| terms.iter().all(|t| t.content() || t.hard()))
        .map(|(left, terms)| BoundPair {
            left,
            source: Regex::new(&format!(
                r"(?i)(?:^|\W){}[ \t_-]+{}(?:$|\W)",
                regex::escape(&terms[0].text),
                regex::escape(&terms[1].text),
            ))
            .expect("escaped literal query pair"),
        })
        .collect()
}

impl Evidence {
    pub fn qualifies(&self, old: &Self, pairs: &[BoundPair], source: &str) -> bool {
        !self.union.is_empty()
            && self.missing_quoted == 0
            && self.missing_hard <= old.missing_hard
            && old.hard.is_subset(&self.hard)
            && (self.id == old.id
                || pairs.iter().any(|p| {
                    self.exact.contains(&p.left)
                        && self.exact.contains(&(p.left + 1))
                        && !old.exact.contains(&p.left)
                        && !old.exact.contains(&(p.left + 1))
                        && p.source.is_match(source)
                }))
    }

    // Numerical scores cannot displace the incumbent on equal coverage.
    pub fn coverage_cmp(&self, other: &Self, incumbent: usize) -> Ordering {
        other
            .union
            .len()
            .cmp(&self.union.len())
            .then(other.exact.len().cmp(&self.exact.len()))
            .then((self.id != incumbent).cmp(&(other.id != incumbent)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn evidence(id: usize, exact: &[usize], union: &[usize]) -> Evidence {
        Evidence {
            id,
            exact: exact.iter().copied().collect(),
            union: union.iter().copied().collect(),
            hard: BTreeSet::new(),
            missing_hard: 0,
            missing_quoted: 0,
        }
    }
    #[test]
    fn certificate_requires_two_novel_exact_terms_bound_in_original_source() {
        let pairs = bound_pairs(&Query::parse("alpha beta gamma delta").unwrap());
        let old = evidence(1, &[2, 3], &[2, 3]);
        let mut new = evidence(2, &[0, 1], &[0, 1, 2]);
        for source in ["alpha beta", "Alpha-beta", "(alpha_beta)", "alpha\tbeta"] {
            assert!(new.qualifies(&old, &pairs, source));
        }
        for source in [
            "alpha\nbeta",
            "alpha other beta",
            "xalpha beta",
            "alpha betas",
        ] {
            assert!(!new.qualifies(&old, &pairs, source));
        }
        new.exact.remove(&1);
        assert!(!new.qualifies(&old, &pairs, "alpha beta"));
    }
    #[test]
    fn query_indices_literal_obligations_and_hard_identities_are_preserved() {
        let pairs = bound_pairs(&Query::parse("alpha command beta gamma").unwrap());
        let mut old = evidence(1, &[3], &[3]);
        let mut new = evidence(2, &[0, 2], &[0, 2, 3]);
        assert!(!new.qualifies(&old, &pairs, "alpha beta"));
        let pairs = bound_pairs(&Query::parse("alpha beta gamma delta").unwrap());
        new.exact = BTreeSet::from([0, 1]);
        assert!(new.qualifies(&old, &pairs, "alpha beta"));
        new.missing_quoted = 1;
        assert!(!new.qualifies(&old, &pairs, "alpha beta"));
        new.missing_quoted = 0;
        old.hard.insert(3);
        new.hard.insert(2);
        assert!(!new.qualifies(&old, &pairs, "alpha beta"));
        new.hard.insert(3);
        assert!(new.qualifies(&old, &pairs, "alpha beta"));
        new.missing_hard = 1;
        assert!(!new.qualifies(&old, &pairs, "alpha beta"));
    }
    #[test]
    fn coverage_ties_keep_the_incumbent_before_numerical_ranking() {
        let old = evidence(1, &[2, 3], &[2, 3]);
        let mut new = evidence(2, &[0, 1], &[0, 1]);
        assert_eq!(old.coverage_cmp(&new, old.id), Ordering::Less);
        new.union.insert(2);
        assert_eq!(old.coverage_cmp(&new, old.id), Ordering::Greater);
    }
    #[tokio::test]
    async fn native_stage_changes_only_a_qualified_source_and_respects_cold_eligibility() {
        use crate::storage::{Call, Context, NOW, Store, record::Record};
        use serde_json::json;
        use std::collections::HashSet;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("passage.db");
        let records: Vec<Record> = [
            json!({"id":"r","kind":"doc","body":"# Old\ngamma delta\n\n# New\nalpha beta gamma\n"}),
            json!({"id":"closed","kind":"doc","body":"# Old\ngamma delta\n\n# New\nalpha beta gamma\n"}),
            json!({"id":"close","body":"","close":{"target":"closed","reason":"correct"}}),
        ].into_iter().map(|r| serde_json::from_value(r).unwrap()).collect();
        let built = Store::build(&path, records).await.unwrap();
        let old_id = built
            .record_blocks("r")
            .find(|b| b.text.contains("gamma delta"))
            .unwrap()
            .id;
        let closed_id = built.record_blocks("closed").next().unwrap().id;
        let original = vec![
            Call {
                cards: vec![built.card(old_id, None), built.card(closed_id, None)],
                text: "keep exact original rendering".into(),
                ..Default::default()
            },
            Call {
                text: "later call untouched".into(),
                ..Default::default()
            },
        ];
        drop(built);
        let mut store = Store::open(&path).await.unwrap();
        let ctx = Context {
            now: NOW.into(),
            ..Default::default()
        };
        let query = "alpha beta gamma delta";
        let mut protected = original.clone();
        store
            .refine_record_passages(query, &ctx, &HashSet::from(["r".into()]), &mut protected)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&protected).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        let mut filtered = original.clone();
        store
            .refine_record_passages(
                "kind:note alpha beta gamma delta",
                &ctx,
                &HashSet::new(),
                &mut filtered,
            )
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&filtered).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        let mut duplicate = vec![Call {
            cards: vec![original[0].cards[0].clone(); 2],
            text: "duplicates untouched".into(),
            ..Default::default()
        }];
        let before = serde_json::to_value(&duplicate).unwrap();
        store
            .refine_record_passages(query, &ctx, &HashSet::new(), &mut duplicate)
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&duplicate).unwrap(), before);
        let mut calls = original.clone();
        store
            .refine_record_passages(query, &ctx, &HashSet::new(), &mut calls)
            .await
            .unwrap();
        assert_eq!(calls[0].cards.len(), original[0].cards.len());
        assert_eq!(calls[0].cards[0].record, "r");
        assert_ne!(calls[0].cards[0].span, original[0].cards[0].span);
        assert!(calls[0].cards[0].text.contains("alpha beta gamma"));
        assert_eq!(
            serde_json::to_value(&calls[0].cards[1]).unwrap(),
            serde_json::to_value(&original[0].cards[1]).unwrap()
        );
        assert_eq!(calls[1].text, original[1].text);
        assert!(store.record_blocks("closed").all(|b| b.text.is_empty()));
    }
}
