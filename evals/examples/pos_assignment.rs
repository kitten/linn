//! Bounded shadow assignment. No admission or semantic certificate.
use linn_eval::{Analysis, analyze};
use serde_json::json;
use std::{
    collections::HashMap,
    io::{self, BufRead},
};
#[derive(Clone, Default)]
struct State {
    source_sentence: Option<usize>,
    covered: usize,
    similarity: f64,
    pairs: Vec<(usize, usize)>,
}
fn solve(
    q: &Analysis,
    s: &Analysis,
    qt: &str,
    st: &str,
    vectors: &HashMap<String, Vec<f32>>,
    mode: &str,
    vector: bool,
) -> State {
    let mut best = State::default();
    let sentences: std::collections::BTreeSet<_> =
        s.occurrences.iter().map(|t| t.sentence).collect();
    for source_sentence in sentences {
        let mut dp = vec![vec![State::default(); s.occurrences.len() + 1]; q.occurrences.len() + 1];
        let allowed = |a: &linn_eval::Group| match mode {
            "singleton" => a.tokens.len() == 1,
            "structural" => true,
            _ => a.tokens.len() == 1 || a.kind != "contiguous",
        };
        for i in 0..=q.occurrences.len() {
            for j in 0..=s.occurrences.len() {
                let mut choices = vec![];
                if i > 0 {
                    choices.push(dp[i - 1][j].clone());
                }
                if j > 0 {
                    choices.push(dp[i][j - 1].clone());
                }
                for (qi, a) in q
                    .groups
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.tokens.end == i && allowed(g))
                {
                    for (si, b) in s
                        .groups
                        .iter()
                        .enumerate()
                        .filter(|(_, g)| g.tokens.end == j && allowed(g))
                    {
                        let left = &qt[a.span.clone()];
                        let right = &st[b.span.clone()];
                        let exact = left.eq_ignore_ascii_case(right);
                        let classes = |analysis: &Analysis, group: &linn_eval::Group| {
                            let mut classes = std::collections::BTreeSet::new();
                            for t in &analysis.occurrences[group.tokens.clone()] {
                                for (_, tag) in &t.analyses {
                                    let class = match tag.as_str() {
                                        "NN" | "NN:U" | "NN:UN" | "NNS" | "NNP" | "NNPS" => {
                                            "nominal"
                                        }
                                        "JJ" | "JJR" | "JJS" => "adjective",
                                        "VB" | "VBD" | "VBG" | "VBN" | "VBP" | "VBZ" => "verb",
                                        _ => continue,
                                    };
                                    classes.insert(class);
                                }
                            }
                            classes
                        };
                        let compatible = !classes(q, a).is_disjoint(&classes(s, b));
                        let protected = |t: &str| {
                            t.contains(['_', '-']) || t.chars().any(|c| c.is_ascii_digit())
                        };
                        if !exact
                            && (!vector
                                || !compatible
                                || protected(left)
                                || protected(right)
                                || (mode == "pos" && a.kind != b.kind))
                        {
                            continue;
                        }
                        let mut next = dp[a.tokens.start][b.tokens.start].clone();
                        let sentence = s.occurrences[b.tokens.start].sentence;
                        if sentence != source_sentence {
                            continue;
                        }
                        if next.source_sentence.is_some_and(|old| old != sentence) {
                            continue;
                        }
                        next.source_sentence = Some(sentence);
                        next.covered += a.tokens.len();
                        next.similarity += a.tokens.len() as f64
                            * if exact {
                                1.
                            } else {
                                linn::embed::Encoder::similarity(&vectors[left], &vectors[right])
                                    as f64
                            };
                        next.pairs.push((qi, si));
                        choices.push(next);
                    }
                }
                dp[i][j] = choices
                    .into_iter()
                    .max_by(|a, b| {
                        a.covered
                            .cmp(&b.covered)
                            .then(a.similarity.total_cmp(&b.similarity))
                            .then(b.pairs.len().cmp(&a.pairs.len()))
                    })
                    .unwrap_or_default();
            }
        }
        let result = dp.pop().unwrap().pop().unwrap();
        if result.covered > best.covered
            || (result.covered == best.covered && result.similarity > best.similarity)
        {
            best = result;
        }
    }
    best
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tokenizer = nlprule::Tokenizer::new(std::env::args().nth(1).ok_or("asset path required")?)?;
    let encoder = linn::embed::Encoder::load()?;
    let mut cache = HashMap::new();
    for line in io::stdin().lock().lines() {
        let input: serde_json::Value = serde_json::from_str(&line?)?;
        let qt = input["text"].as_str().ok_or("text")?;
        let st = input["source"].as_str().ok_or("source")?;
        let q = analyze(&tokenizer, qt);
        let s = analyze(&tokenizer, st);
        if q.occurrences.len() > 64 || s.occurrences.len() > 256 {
            println!(
                "{}",
                json!({"text":qt,"status":"cap: shadow not executed","query_tokens":q.occurrences.len(),"source_tokens":s.occurrences.len()})
            );
            continue;
        }
        for (a, text) in [(&q, qt), (&s, st)] {
            for g in &a.groups {
                let surface = text[g.span.clone()].to_owned();
                cache
                    .entry(surface.clone())
                    .or_insert_with(|| encoder.batch(std::slice::from_ref(&surface))[0].clone());
            }
        }
        for mode in ["singleton", "structural", "pos"] {
            for vector in [false, true] {
                let result = solve(&q, &s, qt, st, &cache, mode, vector);
                let unmatched: Vec<_> = q
                    .occurrences
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| {
                        !result
                            .pairs
                            .iter()
                            .any(|(qi, _)| q.groups[*qi].tokens.contains(i))
                    })
                    .map(|(_, t)| json!({"occurrence":t,"text":qt[t.span.clone()]}))
                    .collect();
                let chosen: Vec<_> = result.pairs.iter().map(|(a,b)| json!({"query":q.groups[*a],"source":s.groups[*b],"literal":qt[q.groups[*a].span.clone()].eq_ignore_ascii_case(&st[s.groups[*b].span.clone()])})).collect();
                println!(
                    "{}",
                    json!({"unmatched_material":unmatched,"constraints":["literal identity when vectors off","exact protected identity","possible POS class","POS group-kind compatibility","monotone nonoverlapping partitions","one source unit and sentence"],"query_annotation":q,"source_annotation":s,"provenance":{"record":input.get("record"),"body_hash":input.get("body_hash"),"block":input.get("block"),"base":input.get("base"),"end":input.get("end")},"model":linn::embed::ARTIFACT_MODEL,"text":qt,"source":st,"mode":mode,"vectors":vector,"covered":result.covered,"obligations":q.occurrences.len(),"unmatched":q.occurrences.len()-result.covered,"similarity":result.similarity,"chosen":chosen,"status":"unverified shadow; monotone nonoverlapping capacities"})
                );
            }
        }
    }
    Ok(())
}
