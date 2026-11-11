//! @ref LLP 0002#lexical-evidence
//! Diagnostic confidence is computed, but the specified open gate never filters.
use crate::{
    retrieval::lexical::Rank,
    retrieval::query::{self, Query},
    storage::Store,
};
use std::collections::{HashMap, HashSet};
const WEIGHTS: [f64; 36] = [
    -1.3855513434691795,
    0.9593878241251049,
    0.7532728866585706,
    0.17664441460611954,
    0.24737742197579676,
    0.5919029905332762,
    0.021904050941645846,
    -1.5402801189515525,
    1.0004888367231775,
    0.7476384047906712,
    0.3635049016679036,
    0.2128668102983276,
    -0.10219770424770558,
    -0.569738296022477,
    0.,
    0.,
    -0.5926065130891044,
    -0.1464830955386999,
    -1.3855513434691795,
    0.93621863635448,
    0.8070180966375439,
    0.3051328046511108,
    -0.389265204940308,
    -0.04685987846064951,
    -0.5989768934538516,
    -0.5154316163892562,
    1.0868875274033045,
    0.7797442807149891,
    0.3635049016679036,
    0.09247920198147007,
    -0.10219770424770558,
    -0.569738296022477,
    0.,
    0.,
    -0.4994360850540691,
    -0.1464830955386999,
];
pub(crate) fn confidence(
    store: &Store,
    q: &Query,
    hits: &[HashSet<usize>],
    ranking: &[Rank],
    stems: &[Vec<String>],
    n: f64,
    df: &HashMap<String, f64>,
) -> Option<f64> {
    let first = ranking.first()?;
    let content: Vec<_> = q
        .terms
        .iter()
        .enumerate()
        .filter(|(_, t)| t.content())
        .map(|(i, _)| i)
        .collect();
    let words: Vec<Vec<_>> = q
        .terms
        .iter()
        .map(|t| {
            query::surface(&t.text)
                .iter()
                .map(|w| query::norm(w))
                .collect()
        })
        .collect();
    let mass: Vec<f64> = words
        .iter()
        .map(|ws| {
            ws.iter()
                .map(|w| {
                    let d = *df.get(w).unwrap_or(&0.);
                    (1. + (n - d + 0.5) / (d + 0.5)).ln()
                })
                .sum()
        })
        .collect();
    let features = |rank: &Rank| {
        let row = &store.rows[&rank.sid];
        let covered: Vec<_> = hits.iter().map(|s| s.contains(&rank.sid)).collect();
        let matched: Vec<_> = content.iter().filter(|i| covered[**i]).copied().collect();
        let stem_only = matched
            .iter()
            .any(|i| !words[*i].iter().any(|w| row.body_positions.contains_key(w)));
        let mut events = Vec::new();
        for (term, i) in matched.iter().enumerate() {
            let ws = if stem_only { &stems[*i] } else { &words[*i] };
            let positions = if stem_only {
                &row.stem_positions
            } else {
                &row.body_positions
            };
            for w in ws {
                for pos in positions.get(w).into_iter().flatten() {
                    events.push((*pos, term));
                }
            }
        }
        events.sort_unstable();
        let mut span = 100000usize;
        let mut counts = HashMap::<usize, usize>::new();
        let mut left = 0;
        for (right, (pos, term)) in events.iter().enumerate() {
            *counts.entry(*term).or_default() += 1;
            while counts.len() == matched.len() && !counts.is_empty() {
                span = span.min(pos - events[left].0 + 1);
                let entry = counts.get_mut(&events[left].1).unwrap();
                *entry -= 1;
                if *entry == 0 {
                    counts.remove(&events[left].1);
                }
                left += 1;
                if left > right {
                    break;
                }
            }
        }
        let all = covered.iter().all(|v| *v);
        let heading = words
            .iter()
            .filter(|ws| {
                ws.iter()
                    .any(|w| row.tf.get(w).is_some_and(|tf| tf[0] > 0.))
            })
            .count();
        let other = ranking
            .iter()
            .find(|r| store.rows[&r.sid].record != row.record)
            .map(|r| r.score)
            .unwrap_or(0.);
        [
            1.,
            mass.iter()
                .zip(&covered)
                .filter(|(_, yes)| **yes)
                .map(|(m, _)| *m)
                .sum::<f64>()
                / mass.iter().sum::<f64>().max(0.001),
            covered.iter().filter(|v| **v).count() as f64 / q.terms.len() as f64,
            matched.len() as f64 / content.len().max(1) as f64,
            1. / (1. + span.saturating_sub(matched.len()) as f64 / 8.),
            f64::from(span <= 16),
            heading as f64 / q.terms.len().max(1) as f64,
            (row.length as f64).ln_1p() / 8.,
            mass.iter()
                .zip(&covered)
                .filter(|(m, v)| **m >= 3. && **v)
                .count()
                .min(5) as f64
                / 5.,
            rank.score.ln_1p() / 5.,
            q.terms.len().min(10) as f64 / 10.,
            f64::from(all),
            f64::from(q.terms.iter().any(|t| t.phrase)),
            hits.iter().filter(|s| s.is_empty()).count() as f64 / q.terms.len() as f64,
            0.,
            0.,
            (rank.score - other) / (rank.score + other).max(1.),
            f64::from(rank.tier == 0),
        ]
    };
    let mut seen = HashSet::new();
    let rows: Vec<_> = ranking
        .iter()
        .filter(|r| seen.insert(&store.rows[&r.sid].record))
        .take(5)
        .map(features)
        .collect();
    let mut query = features(first).to_vec();
    for i in 0..18 {
        query.push(rows.iter().map(|r| r[i]).fold(f64::NEG_INFINITY, f64::max));
    }
    let z = WEIGHTS
        .iter()
        .zip(query)
        .map(|(w, x)| w * x)
        .sum::<f64>()
        .clamp(-40., 40.);
    Some(1. / (1. + (-z).exp()))
}
