//! Fixed joint lexical/vector competition with native-page protection.
//! @ref LLP 0002#joint-record-competition
use crate::storage::{Call, Card};
use std::collections::HashMap;

#[cfg_attr(test, derive(serde::Serialize, serde::Deserialize))]
pub(super) struct Candidate {
    pub card: Card,
    pub lexical: f64,
    pub pos: f64,
    pub cosine: f32,
    pub normalizer: f64,
    pub far: bool,
    pub missing_quoted: usize,
}
#[cfg_attr(test, derive(serde::Serialize, serde::Deserialize))]
pub(super) struct Evidence {
    pub candidates: Vec<Candidate>,
    pub baseline: Vec<Call>,
}
#[derive(Clone, Copy)]
struct Scores(f64, f64);
impl Scores {
    fn better(self, other: Self) -> bool {
        self.0 > other.0 && self.1 > other.1
    }
    fn exceeds(self, other: Self, margin: f64) -> bool {
        self.0 - other.0 > margin * other.0.abs().max(1e-12)
            && self.1 - other.1 > margin * other.1.abs().max(1e-12)
    }
}
#[derive(Clone, Copy)]
struct Scored<'a> {
    candidate: &'a Candidate,
    scores: Scores,
}
pub(super) fn apply(evidence: Evidence) -> Vec<Call> {
    let Evidence {
        candidates,
        mut baseline,
    } = evidence;
    let Some(first) = baseline.first_mut() else {
        return baseline;
    };
    let orig = &first.cards;
    let native: HashMap<&str, &Card> = orig.iter().map(|c| (c.record.as_str(), c)).collect();
    if native.len() != orig.len() {
        return baseline;
    }
    let mut rows: Vec<_> = candidates
        .iter()
        .filter(|c| c.missing_quoted == 0)
        .collect();
    let lp = |c: &Candidate| (c.lexical + c.pos) / c.normalizer;
    let mut ml = rows.iter().map(|c| lp(c)).reduce(f64::max).unwrap_or(0.);
    if ml == 0. {
        ml = 1.
    }
    let mut mv = rows
        .iter()
        .map(|c| f64::from(c.cosine).max(0.))
        .reduce(f64::max)
        .unwrap_or(0.);
    if mv == 0. {
        mv = 1.
    }
    let mut rows: Vec<_> = rows
        .drain(..)
        .map(|c| {
            let l = lp(c) - if c.far { 7. / c.normalizer } else { 0. };
            let v = f64::from(c.cosine).max(0.);
            Scored {
                candidate: c,
                scores: Scores(l + v, l + 0.5 * ml / mv * v),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.scores
            .0
            .total_cmp(&a.scores.0)
            .then_with(|| a.candidate.card.record.cmp(&b.candidate.card.record))
            .then_with(|| a.candidate.card.span.cmp(&b.candidate.card.span))
            .then_with(|| a.candidate.card.section.cmp(&b.candidate.card.section))
    });
    let mut reps = HashMap::new();
    let mut order = Vec::new();
    for row in rows {
        let r = row.candidate.card.record.as_str();
        if let std::collections::hash_map::Entry::Vacant(entry) = reps.entry(r) {
            entry.insert(row);
            order.push(r);
        }
    }
    let better = |a: &str, b: &str| match (reps.get(a), reps.get(b)) {
        (Some(a), Some(b)) => a.scores.better(b.scores),
        _ => false,
    };
    let mut page: Vec<_> = orig.iter().map(|c| c.record.as_str()).collect();
    for i in 1..page.len() {
        let mut j = i;
        while j > 0 && better(page[j], page[j - 1]) {
            page.swap(j, j - 1);
            j -= 1;
        }
    }
    for r in order {
        if native.contains_key(r) {
            continue;
        }
        if page.len() == 5 && !better(r, page[4]) {
            continue;
        }
        page.push(r);
        let mut j = page.len() - 1;
        while j > 0 && better(page[j], page[j - 1]) {
            page.swap(j, j - 1);
            j -= 1;
        }
        page.truncate(5);
    }
    for incumbent in orig {
        let old = incumbent.record.as_str();
        if page.contains(&old) {
            continue;
        }
        let incoming = page
            .iter()
            .copied()
            .filter(|id| !native.contains_key(id))
            .min_by(|a, b| reps[a].scores.0.total_cmp(&reps[b].scores.0).then(a.cmp(b)));
        let Some(new) = incoming else { continue };
        let replace = reps
            .get(old)
            .is_some_and(|old| reps[new].scores.exceeds(old.scores, 0.1));
        if !replace {
            page.retain(|id| *id != new);
            page.push(old);
        }
    }
    if let (Some(old), Some(new)) = (orig.first(), page.first().copied()) {
        let old = old.record.as_str();
        if new != old {
            let exceeds = match (reps.get(new), reps.get(old)) {
                (Some(a), Some(b)) => a.scores.exceeds(b.scores, 0.2),
                _ => false,
            };
            if !exceeds {
                page.retain(|id| *id != old);
                page.insert(0, old);
                page.truncate(5);
            }
        }
    }
    if !page
        .iter()
        .copied()
        .eq(orig.iter().map(|c| c.record.as_str()))
    {
        let cards: Vec<_> = page
            .iter()
            .map(|r| {
                native
                    .get(r)
                    .copied()
                    .unwrap_or_else(|| &reps[r].candidate.card)
                    .clone()
            })
            .collect();
        first.text = if cards.is_empty() {
            "Nothing qualified.".into()
        } else {
            cards
                .iter()
                .map(|c| format!("{}: {}", c.heading, c.text))
                .collect::<Vec<_>>()
                .join("\n")
        };
        first.cards = cards;
    }
    baseline
}

#[cfg(test)]
mod legacy {
    use serde_json::{Value, json};
    use std::collections::HashMap;
    fn num(c: &Value, k: &str) -> f64 {
        c[k].as_f64().unwrap()
    }
    fn rid(c: &Value) -> &str {
        c["card"]["record"].as_str().unwrap()
    }
    pub(super) fn select(f: &Value) -> Value {
        let mut calls = f["baseline"].clone();
        let Some(first) = calls.as_array_mut().and_then(|a| a.first_mut()) else {
            return calls;
        };
        let orig = first["cards"].as_array().unwrap().clone();
        let native: HashMap<&str, &Value> = orig
            .iter()
            .map(|c| (c["record"].as_str().unwrap(), c))
            .collect();
        if native.len() != orig.len() {
            return calls;
        }
        let mut rows: Vec<&Value> = f["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["missing_quoted"].as_u64() == Some(0))
            .collect();
        let lp = |c: &Value| (num(c, "lexical") + num(c, "pos")) / num(c, "normalizer");
        let mut ml = rows.iter().map(|c| lp(c)).reduce(f64::max).unwrap_or(0.);
        if ml == 0. {
            ml = 1.
        }
        let mut mv = rows
            .iter()
            .map(|c| num(c, "cosine").max(0.))
            .reduce(f64::max)
            .unwrap_or(0.);
        if mv == 0. {
            mv = 1.
        }
        let scores = |c: &Value| {
            let l = lp(c)
                - if c["far"].as_bool().unwrap() {
                    7. / num(c, "normalizer")
                } else {
                    0.
                };
            let v = num(c, "cosine").max(0.);
            (l + v, l + 0.5 * ml / mv * v)
        };
        rows.sort_by(|a, b| {
            scores(b)
                .0
                .total_cmp(&scores(a).0)
                .then_with(|| rid(a).cmp(rid(b)))
                .then_with(|| {
                    let s = |c: &Value| {
                        let a = &c["card"]["span"];
                        (
                            a[0].as_u64().unwrap(),
                            a[1].as_u64().unwrap(),
                            c["card"]["section"].as_u64().unwrap(),
                        )
                    };
                    s(a).cmp(&s(b))
                })
        });
        let mut reps = HashMap::new();
        let mut order = Vec::new();
        for c in rows {
            let r = rid(c);
            if !reps.contains_key(r) {
                reps.insert(r, c);
                order.push(r);
            }
        }
        let better = |a: &str, b: &str| match (reps.get(a), reps.get(b)) {
            (Some(a), Some(b)) => {
                let a = scores(a);
                let b = scores(b);
                a.0 > b.0 && a.1 > b.1
            }
            _ => false,
        };
        let mut page: Vec<&str> = orig.iter().map(|c| c["record"].as_str().unwrap()).collect();
        for i in 1..page.len() {
            let mut j = i;
            while j > 0 && better(page[j], page[j - 1]) {
                page.swap(j, j - 1);
                j -= 1;
            }
        }
        for r in order {
            if native.contains_key(r) {
                continue;
            }
            if page.len() == 5 && !better(r, page[4]) {
                continue;
            }
            page.push(r);
            let mut j = page.len() - 1;
            while j > 0 && better(page[j], page[j - 1]) {
                page.swap(j, j - 1);
                j -= 1;
            }
            page.truncate(5);
        }
        {
            // Protect native membership before protecting its first result.
            for incumbent in &orig {
                let old = incumbent["record"].as_str().unwrap();
                if page.contains(&old) {
                    continue;
                }
                let incoming = page
                    .iter()
                    .copied()
                    .filter(|id| !native.contains_key(id))
                    .min_by(|a, b| {
                        scores(reps[a])
                            .0
                            .total_cmp(&scores(reps[b]).0)
                            .then(a.cmp(b))
                    });
                let Some(new) = incoming else {
                    continue;
                };
                let replace = if let Some(old_row) = reps.get(old) {
                    let a = scores(reps[new]);
                    let b = scores(old_row);
                    a.0 - b.0 > 0.1 * b.0.abs().max(1e-12) && a.1 - b.1 > 0.1 * b.1.abs().max(1e-12)
                } else {
                    false
                };
                if !replace {
                    page.retain(|id| *id != new);
                    page.push(old);
                }
            }
        }
        if !orig.is_empty() && !page.is_empty() {
            let old = orig[0]["record"].as_str().unwrap();
            if page[0] != old {
                let margin = 0.2;
                let exceeds = match (reps.get(page[0]), reps.get(old)) {
                    (Some(a), Some(b)) => {
                        let a = scores(a);
                        let b = scores(b);
                        a.0 - b.0 > margin * b.0.abs().max(1e-12)
                            && a.1 - b.1 > margin * b.1.abs().max(1e-12)
                    }
                    _ => false,
                };
                if !exceeds {
                    page.retain(|id| *id != old);
                    page.insert(0, old);
                    page.truncate(5);
                }
            }
        }
        let cards: Vec<Value> = page
            .iter()
            .map(|r| {
                native
                    .get(r)
                    .map(|c| (*c).clone())
                    .unwrap_or_else(|| reps[r]["card"].clone())
            })
            .collect();
        if cards != orig {
            first["text"] = Value::String(if cards.is_empty() {
                "Nothing qualified.".into()
            } else {
                cards
                    .iter()
                    .map(|c| {
                        format!(
                            "{}: {}",
                            c["heading"].as_str().unwrap(),
                            c["text"].as_str().unwrap()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            });
            first["cards"] = json!(cards);
        }
        calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn select(f: &Value) -> Value {
        serde_json::to_value(apply(serde_json::from_value(f.clone()).unwrap())).unwrap()
    }
    fn card(id: &str) -> Value {
        json!({"record":id,"span":[0,10],"section":1,"heading":"","text":id})
    }
    fn row(id: &str, l: f64) -> Value {
        json!({"card":card(id),"lexical":l,"pos":0.,"normalizer":1.,"cosine":0.,"far":false,"missing_quoted":0})
    }
    fn fixture(extra: f64) -> Value {
        let ids = ["a", "b", "c", "d", "e"];
        let mut rows: Vec<_> = ids.iter().map(|id| row(id, 1.)).collect();
        rows.push(row("x", extra));
        let fixture = json!({"baseline":[{"cards":ids.iter().map(|id|card(id)).collect::<Vec<_>>(),"text":"native","hints":[{"record":"hint","label":"","heading":"","span":[0,1],"matched":[],"missing":[],"excerpt":""}],"related":[]},{"cards":[card("z")],"text":"later"}],"candidates":rows});
        serde_json::to_value(serde_json::from_value::<Evidence>(fixture).unwrap()).unwrap()
    }
    #[test]
    fn weak_agreement_preserves_membership_but_strong_can_replace() {
        let weak = select(&fixture(1.05));
        assert!(
            weak[0]["cards"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["record"] == "e")
        );
        assert!(
            !weak[0]["cards"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["record"] == "x")
        );
        let f = fixture(1.21);
        let strong = select(&f);
        assert_eq!(strong[0]["cards"][0]["record"], "x");
        assert_eq!(strong[0]["cards"].as_array().unwrap().len(), 5);
        assert_eq!(strong[1], f["baseline"][1]);
        assert_eq!(strong[0]["hints"], f["baseline"][0]["hints"]);
    }
    #[test]
    fn empty_admission_is_not_disabled() {
        let mut f = fixture(1.2);
        f["baseline"][0]["cards"] = json!([]);
        assert_eq!(select(&f)[0]["cards"].as_array().unwrap().len(), 5);
    }
    #[test]
    fn duplicate_native_page_is_preserved() {
        let mut f = fixture(9.);
        f["baseline"][0]["cards"] = json!([card("a"), card("a")]);
        assert_eq!(select(&f), f["baseline"]);
    }
    #[test]
    fn quote_and_unknown_native_barriers_are_preserved() {
        let mut f = fixture(9.);
        f["candidates"][5]["missing_quoted"] = json!(1);
        assert_eq!(select(&f), f["baseline"]);
        f["candidates"] = json!([row("x", 9.)]);
        assert_eq!(select(&f), f["baseline"]);
    }
}

#[cfg(test)]
mod margin_tests {
    use super::*;
    use serde_json::{Value, json};
    fn select(f: &Value) -> Value {
        serde_json::to_value(apply(serde_json::from_value(f.clone()).unwrap())).unwrap()
    }
    #[test]
    fn full_score_advantage_alone_cannot_replace_native_head() {
        let card = |id: &str| json!({"record":id,"span":[0,10],"section":1,"heading":"","text":id});
        let row = |id: &str, l: f64, v: f64| json!({"card":card(id),"lexical":l,"pos":0.,"normalizer":1.,"cosine":v,"far":false,"missing_quoted":0});
        let f = json!({"baseline":[{"cards":[card("native")],"text":"unchanged"}],"candidates":[row("native",1.,0.),row("incoming",0.5,0.8)]});
        let calls = select(&f);
        assert_eq!(calls[0]["cards"][0]["record"], "native");
        assert_eq!(calls[0]["cards"][1]["record"], "incoming");
    }
}

#[cfg(test)]
mod parity_tests {
    use super::*;
    use crate::storage::Pointer;
    #[test]
    fn typed_selector_matches_frozen_json_reference() {
        for seed in 0..256 {
            let candidates = (0..32)
                .map(|i| Candidate {
                    card: Card {
                        record: format!("r{}", i % 11),
                        span: [i * 10, i * 10 + 9],
                        section: i + 1,
                        heading: format!("heading {i}"),
                        text: format!("original source {i}"),
                        ..Card::default()
                    },
                    lexical: ((seed + i * 17) % 31) as f64 * 0.1 - 0.2,
                    pos: ((seed + i) % 5) as f64 * 0.075,
                    cosine: [-0.5, -0., 0., 0.125, 0.3, 0.8][(seed + i) % 6],
                    normalizer: [1., 3., 67.95][(seed + i) % 3],
                    far: (seed + i) % 3 == 0,
                    missing_quoted: usize::from((seed + i) % 9 == 0),
                })
                .collect();
            let mut cards: Vec<_> = (0..seed % 6)
                .map(|i| Card {
                    record: if seed % 7 == 0 {
                        format!("unknown{i}")
                    } else {
                        format!("r{i}")
                    },
                    text: format!("native bytes {i}"),
                    span: [i, i + 1],
                    section: 100 + i,
                    labels: vec!["native".into()],
                    fetch: Some("original fetch".into()),
                    ..Card::default()
                })
                .collect();
            if seed % 13 == 0 && cards.len() > 1 {
                cards[1] = cards[0].clone();
            }
            let evidence = Evidence {
                candidates,
                baseline: vec![
                    Call {
                        cards,
                        text: "native formatting".into(),
                        hints: vec![Pointer {
                            record: "hint".into(),
                            ..Pointer::default()
                        }],
                        related: vec![Pointer {
                            record: "related".into(),
                            ..Pointer::default()
                        }],
                    },
                    Call {
                        text: "later call".into(),
                        cards: vec![Card {
                            record: "later".into(),
                            ..Card::default()
                        }],
                        ..Call::default()
                    },
                ],
            };
            let serialized = serde_json::to_value(&evidence).unwrap();
            let expected = legacy::select(&serialized);
            let actual = serde_json::to_value(apply(evidence)).unwrap();
            assert_eq!(actual, expected, "seed {seed}");
        }
    }
}
