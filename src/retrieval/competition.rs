//! Fixed joint lexical/vector competition with native-page protection.
//! @ref LLP 0002#joint-record-competition
use crate::storage::{Call, Card};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub card: Card,
    pub lexical: f64,
    pub pos: f64,
    pub cosine: f32,
    pub normalizer: f64,
    pub far: bool,
    pub exact_coverage: usize,
    pub stem_coverage: usize,
    pub content_terms: usize,
    pub required: usize,
    pub missing_hard: usize,
    pub missing_quoted: usize,
    pub exact_tier: Option<usize>,
    pub stem_tier: Option<usize>,
    pub lexical_pool: bool,
    pub vector_pool: bool,
    pub pos_pool: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub query: String,
    pub model: String,
    pub parse_error: Option<String>,
    pub query_pairs: usize,
    pub candidates: Vec<Candidate>,
    pub baseline: Vec<Call>,
}

use serde_json::{Value, json};
use std::collections::HashMap;
fn num(c: &Value, k: &str) -> f64 {
    c[k].as_f64().unwrap()
}
fn rid(c: &Value) -> &str {
    c["card"]["record"].as_str().unwrap()
}
fn select(f: &Value) -> Value {
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

pub(crate) fn apply(evidence: &Evidence) -> crate::error::Result<Vec<Call>> {
    Ok(serde_json::from_value(select(&serde_json::to_value(
        evidence,
    )?))?)
}
#[cfg(test)]
mod tests {
    use super::*;
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
        json!({"baseline":[{"cards":ids.iter().map(|id|card(id)).collect::<Vec<_>>(),"text":"native","hints":[{"keep":true}],"related":[]},{"cards":[card("z")],"text":"later"}],"candidates":rows})
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
