//! @ref LLP 0005#neutral-fixed-loop
use super::{has_category, strings};
use linn::{
    query,
    record::{Record, Validity},
    sections::parse_sections,
    store::{Call, Card, Context, NOW, Store},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn scope(case: &Value) -> &str {
    case["scope"].as_str().unwrap_or("")
}
fn alternatives(case: &Value, g: &Value) -> Vec<String> {
    std::iter::once(g["record"].as_str().unwrap().to_owned())
        .chain(strings(
            case.get("equivalent")
                .and_then(|v| v.get(g["record"].as_str().unwrap())),
        ))
        .collect()
}
fn found(calls: &[Call]) -> Vec<&Card> {
    calls.iter().flat_map(|c| c.cards.iter().take(5)).collect()
}
fn ids(calls: &[Call]) -> Vec<String> {
    calls
        .iter()
        .flat_map(|c| &c.cards)
        .map(|c| c.record.clone())
        .collect()
}
fn first(calls: &[Call]) -> Vec<String> {
    calls
        .first()
        .into_iter()
        .flat_map(|c| c.cards.iter().take(5))
        .map(|c| c.record.clone())
        .collect()
}
fn hint_ids(calls: &[Call]) -> Vec<(String, [usize; 2])> {
    calls
        .first()
        .into_iter()
        .flat_map(|c| &c.hints)
        .map(|p| (p.record.clone(), p.span))
        .collect()
}
fn forbidden(case: &Value, calls: &[Call], by: &HashMap<String, Record>) -> bool {
    let mn: HashSet<_> = strings(case.get("must_not")).into_iter().collect();
    calls.iter().flat_map(|c| &c.cards).any(|c| {
        mn.contains(&c.record)
            && by
                .get(&c.record)
                .and_then(|r| r.scope.as_deref())
                .is_none_or(|s| s.is_empty() || s.to_lowercase() == scope(case).to_lowercase())
    })
}
fn empty_ok(case: &Value, calls: &[Call], by: &HashMap<String, Record>) -> bool {
    let targets: HashSet<_> = strings(case.get("must_not"))
        .into_iter()
        .filter(|id| {
            by.get(id)
                .and_then(|r| r.scope.as_deref())
                .is_some_and(|s| !s.is_empty() && s.to_lowercase() != scope(case).to_lowercase())
        })
        .collect();
    let shown: HashSet<_> = found(calls).iter().map(|c| c.record.clone()).collect();
    !forbidden(case, calls, by)
        && if targets.is_empty() {
            shown.is_empty()
        } else {
            !targets.is_disjoint(&shown)
        }
}
fn eligible(r: &Record, validity: &Validity) -> bool {
    validity.eligible(r, NOW) && !r.body.trim().is_empty()
}
fn lines(body: &str) -> Vec<&str> {
    let mut lines: Vec<_> = body
        .split([
            '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}',
            '\u{2029}',
        ])
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)))
        .filter(|s| s.chars().count() >= 30)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    lines.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));
    lines.truncate(3);
    lines
}
fn violations(calls: &[Call], by: &HashMap<String, Record>, validity: &Validity) -> bool {
    let direct = calls.iter().any(|c| {
        c.cards
            .iter()
            .map(|c| &c.record)
            .chain(
                c.hints
                    .iter()
                    .take(3)
                    .chain(c.related.iter().take(2))
                    .map(|p| &p.record),
            )
            .any(|id| by.get(id).is_none_or(|r| !eligible(r, validity)))
    });
    if direct {
        return true;
    }
    let text = calls
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    by.values()
        .filter(|r| !eligible(r, validity) && r.close.is_none() && !r.body.trim().is_empty())
        .any(|r| {
            lines(&r.body).iter().any(|line| {
                text.contains(line)
                    && !by
                        .values()
                        .any(|r| eligible(r, validity) && r.body.contains(line))
            })
        })
}
type Target = (Vec<std::ops::Range<usize>>, Option<std::ops::Range<usize>>);
fn target(r: &Record, g: &Value) -> Option<Target> {
    let heading = g["heading"].as_str().unwrap_or("").trim().to_lowercase();
    let quote = g["quote"].as_str().unwrap_or("").trim();
    let sections = parse_sections(&r.body);
    let section = if heading.is_empty() {
        None
    } else {
        sections
            .iter()
            .find(|s| s.heading.trim().to_lowercase() == heading)
            .or_else(|| {
                sections.iter().find(|s| {
                    !s.path.is_empty() && s.path.join(" > ").to_lowercase().ends_with(&heading)
                })
            })
            .map(|s| s.span.clone())
    };
    if !heading.is_empty() && section.is_none() && quote.is_empty() {
        return None;
    }
    let mut spans = Vec::new();
    if !quote.is_empty() {
        let mut pos = 0;
        while pos < r.body.len() {
            let Some(i) = r.body[pos..].find(quote) else {
                break;
            };
            let a = pos + i;
            spans.push(a..a + quote.len());
            pos = a + r.body[a..].chars().next().unwrap().len_utf8();
        }
        if let Some(s) = &section {
            spans.retain(|r| s.start <= r.start && r.end <= s.end);
        }
        if spans.is_empty() {
            return None;
        }
    } else {
        spans.push(section.clone().unwrap_or(0..r.body.len()));
    }
    Some((spans, section))
}
fn overlap(card: &Card, a: usize, b: usize, bound: usize) -> bool {
    card.span[1].min(b).saturating_sub(card.span[0].max(a)) as f64 >= 0.5 * (b - a) as f64
        && card.span[1] - card.span[0] <= bound
}
fn satisfies(card: &Card, case: &Value, g: &Value, r: &Record) -> bool {
    let original = g["record"].as_str().unwrap();
    let mut spec = g.clone();
    if r.id != original {
        let judged = case.get("equivalent_targets").and_then(|v| v.get(&r.id));
        if let Some(span) = judged.and_then(|v| v["span"].as_array()) {
            let (a, b) = (
                span[0].as_u64().unwrap() as usize,
                span[1].as_u64().unwrap() as usize,
            );
            return overlap(card, a, b, (3 * (b - a)).max(1500));
        }
        if let Some(h) = judged.and_then(|v| v["heading"].as_str()) {
            let hspec = json!({"heading":h});
            if let Some((spans, sec)) = target(r, &hspec) {
                return contains_target(card, case, r, &spans, sec.as_ref(), false);
            }
        }
        if target(r, &spec).is_none() && r.body.len() <= 2400 {
            return true;
        }
    }
    spec["record"] = json!(r.id);
    let Some((spans, sec)) = target(r, &spec) else {
        return false;
    };
    let record_level = spec["quote"].as_str().unwrap_or("").trim().is_empty()
        && spec["heading"].as_str().unwrap_or("").trim().is_empty();
    contains_target(card, case, r, &spans, sec.as_ref(), record_level)
}
fn contains_target(
    card: &Card,
    case: &Value,
    r: &Record,
    spans: &[std::ops::Range<usize>],
    sec: Option<&std::ops::Range<usize>>,
    record_level: bool,
) -> bool {
    if record_level {
        return true;
    }
    let gold_len = sec
        .map(|s| s.len())
        .unwrap_or_else(|| spans.iter().map(|s| s.len()).max().unwrap_or(0));
    let bound = (3 * gold_len).max(1500);
    if card.span[1] - card.span[0] <= bound
        && spans
            .iter()
            .any(|s| card.span[0] <= s.start && s.end <= card.span[1])
    {
        return true;
    }
    case.get("alt_sections")
        .and_then(|v| v.get(&r.id))
        .and_then(Value::as_array)
        .is_some_and(|alts| {
            alts.iter().any(|s| {
                let (a, b) = (
                    s[0].as_u64().unwrap() as usize,
                    s[1].as_u64().unwrap() as usize,
                );
                overlap(card, a, b, bound.max(b - a))
            })
        })
}
#[derive(Default)]
struct Cost {
    tokens: Vec<f64>,
    ms: Vec<f64>,
}
impl Cost {
    async fn run(
        &mut self,
        store: &mut Store,
        case: &Value,
        query: &str,
        cwd: &str,
    ) -> Result<Vec<Call>> {
        let ctx = Context {
            scope: case["scope"].as_str().map(str::to_owned),
            cwd: cwd.into(),
            now: NOW.into(),
            session: case["id"].as_str().unwrap().into(),
            calls_left: 3,
        };
        let start = Instant::now();
        let calls = store.agent(query, &ctx).await?;
        let ms = start.elapsed().as_secs_f64() * 1000.0 / calls.len().max(1) as f64;
        for c in &calls {
            self.tokens.push(c.text.chars().count().div_ceil(4) as f64);
            self.ms.push(ms);
        }
        Ok(calls)
    }
}
fn rounded(x: f64) -> f64 {
    (x * 1000.0).round_ties_even() / 1000.0
}
fn percentile(xs: &[f64], p: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut x = xs.to_vec();
    x.sort_by(f64::total_cmp);
    x[(p * (x.len() - 1) as f64).round_ties_even() as usize]
}
fn median(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut x = xs.to_vec();
    x.sort_by(f64::total_cmp);
    (x[(x.len() - 1) / 2] + x[x.len() / 2]) / 2.0
}
fn gate(name: &str, value: f64, limit: f64, at_least: bool, n: Option<usize>) {
    let status = if n.is_some_and(|n| n < 10) {
        "INSUFFICIENT-N"
    } else if value.is_finite()
        && if at_least {
            value >= limit
        } else {
            value <= limit
        }
    {
        "pass"
    } else {
        "FAIL"
    };
    println!(
        "| {name} | {value:.3} | {} {limit} | {} | {status} |",
        if at_least { ">=" } else { "<=" },
        n.map(|n| n.to_string()).unwrap_or_default()
    );
}
fn rate(rows: &[(Value, Value)], key: &str, cats: &[&str], exclude: &[&str]) -> (f64, usize) {
    let sel: Vec<_> = rows
        .iter()
        .filter(|(c, r)| {
            r[key].is_boolean()
                && (cats.is_empty() || cats.iter().any(|s| has_category(c, s)))
                && !exclude.iter().any(|s| has_category(c, s))
        })
        .collect();
    (
        if sel.is_empty() {
            f64::NAN
        } else {
            rounded(sel.iter().filter(|(_, r)| r[key] == true).count() as f64 / sel.len() as f64)
        },
        sel.len(),
    )
}

pub async fn run(
    store: &mut Store,
    records: Vec<Record>,
    cases: Vec<Value>,
    all_cases: Vec<Value>,
) -> Result<()> {
    let by: HashMap<_, _> = records.iter().map(|r| (r.id.clone(), r.clone())).collect();
    let validity = Validity::project(&records);
    let mut cost = Cost::default();
    let mut rows = Vec::new();
    let integrity_build = store.integrity().await.is_ok();
    let mut exposure: BTreeMap<String, Vec<HashMap<String, f64>>> = BTreeMap::new();
    for (i, c) in cases.iter().enumerate() {
        let mut calls = Vec::new();
        let mut queries = vec![c["query"].as_str().unwrap().to_owned()];
        queries.extend(strings(c.get("alt_queries")));
        for q in queries.iter().take(3) {
            calls.extend(
                cost.run(store, c, q, c["cwd"].as_str().unwrap_or(""))
                    .await?,
            );
        }
        let gold = c["gold"].as_array().cloned().unwrap_or_default();
        let found = found(&calls);
        let mut row = json!({"first":first(&calls),"violations":violations(&calls,&by,&validity),"must_not":forbidden(c,&calls,&by)});
        let mut excluded: HashSet<_> = strings(c.get("acceptable")).into_iter().collect();
        for g in &gold {
            excluded.extend(alternatives(c, g));
        }
        if gold.is_empty() {
            row["empty"] = json!(empty_ok(c, &calls, &by));
        } else {
            row["found"] = json!(found.iter().any(|card| {
                excluded.contains(&card.record)
                    && gold
                        .iter()
                        .any(|g| alternatives(c, g).contains(&card.record))
            }));
            row["section"] = json!(gold.iter().all(|g| found.iter().any(|card| {
                alternatives(c, g).contains(&card.record)
                    && by
                        .get(&card.record)
                        .is_some_and(|r| satisfies(card, c, g, r))
            })));
        }
        let mut chars: HashMap<String, usize> = HashMap::new();
        for call in &calls {
            for card in call.cards.iter().take(5) {
                *chars.entry(card.record.clone()).or_default() += card.text.chars().count();
            }
            for p in call.hints.iter().take(3).chain(call.related.iter().take(2)) {
                *chars.entry(p.record.clone()).or_default() +=
                    p.excerpt.chars().count() + p.heading.chars().count();
            }
        }
        exposure.entry(scope(c).to_owned()).or_default().push(
            chars
                .into_iter()
                .filter(|(id, _)| !excluded.contains(id))
                .map(|(id, n)| (id, (n as f64 / 1500.0).min(1.0)))
                .collect(),
        );
        rows.push((c.clone(), row));
        if (i + 1) % 50 == 0 {
            eprintln!("neutral scored {}/{}", i + 1, cases.len());
        }
    }
    // Independent full rebuild; never forget from the measured input store.
    eprintln!("neutral diagnostics: independent rebuild, determinism and location");
    let scratch = tempfile::tempdir()?;
    let mut rebuilt = Store::build(&scratch.path().join("checks.db"), records.clone()).await?;
    let mut det = 0;
    let mut inv = 0;
    let mut inv_n = 0;
    for (c, row) in &rows {
        let q = c["query"].as_str().unwrap();
        let cwd = c["cwd"].as_str().unwrap_or("");
        let again = cost.run(&mut rebuilt, c, q, cwd).await?;
        let orig = cost.run(store, c, q, cwd).await?;
        det += usize::from(
            json!(first(&again)) == row["first"] && hint_ids(&again) == hint_ids(&orig),
        );
    }
    for c in &cases {
        if c["gold"].as_array().is_some_and(|g| !g.is_empty()) {
            for q in strings(c.get("alt_queries")) {
                cost.run(store, c, &q, c["cwd"].as_str().unwrap_or(""))
                    .await?;
            }
        }
    }
    for c in &cases {
        let q = c["query"].as_str().unwrap();
        let cwd = c["cwd"].as_str().unwrap_or("");
        let parsed = query::Query::parse(q);
        if !parsed.as_ref().is_ok_and(|q| {
            q.filters
                .iter()
                .any(|(k, _)| ["path", "under", "cites"].contains(&k.as_str()))
        }) {
            inv_n += 1;
            let a = cost.run(store, c, q, "").await?;
            let orig = cost.run(store, c, q, cwd).await?;
            let b = cost.run(store, c, q, "zz-unrelated/sub/dir").await?;
            inv += usize::from(ids(&a) == ids(&orig) && ids(&a) == ids(&b));
        }
    }
    eprintln!("neutral diagnostics: self-retrieval");
    // R1: first broadest section, rarest three tokens, seeded shuffle per scope.
    let mut df: HashMap<String, usize> = HashMap::new();
    let mut rec_words = HashMap::new();
    for r in &records {
        for w in fts_tokens(&r.body).into_iter().collect::<HashSet<_>>() {
            *df.entry(w).or_default() += 1;
        }
        let mut best = BTreeSet::new();
        for s in parse_sections(&r.body) {
            let ws: BTreeSet<_> = fts_tokens(&r.body[s.span])
                .into_iter()
                .filter(|w| {
                    let stripped = w.trim_matches(['-', '_']);
                    stripped.chars().count() >= 4
                        && !stripped.chars().all(|c| c.is_numeric())
                        && !query::stop(w)
                })
                .collect();
            if ws.len() > best.len() {
                best = ws;
            }
        }
        rec_words.insert(r.id.clone(), best);
    }
    let scopes: BTreeSet<_> = all_cases
        .iter()
        .map(|c| c["scope"].as_str().map(str::to_owned))
        .collect();
    let mut rng = Shuffle::new();
    let (mut ki, mut ki_n) = (0, 0);
    for scope in &scopes {
        let mut pool: Vec<_> = records
            .iter()
            .filter(|r| r.scope == *scope && eligible(r, &validity))
            .collect();
        rng.shuffle(&mut pool);
        for r in pool.into_iter().take(120) {
            let mut ws: Vec<_> = rec_words[&r.id].iter().cloned().collect();
            ws.sort_by_key(|w| (df[w], w.clone()));
            ws.truncate(3);
            if ws.len() < 2 {
                continue;
            }
            let q = ws
                .iter()
                .map(|w| {
                    if w.starts_with('-') {
                        format!("\"{w}\"")
                    } else {
                        w.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            let c = json!({"id":format!("ki-{}",r.id),"scope":scope});
            let calls = cost.run(store, &c, &q, "").await?;
            ki_n += 1;
            if ki_n % 100 == 0 {
                eprintln!("neutral self-retrieval: {ki_n} records");
            }
            ki += usize::from(first(&calls).contains(&r.id));
        }
    }
    eprintln!("neutral diagnostics: exposure and scratch forget");
    // Include the standalone exposure diagnostic searches in cost.
    for c in &cases {
        let mut qs = vec![c["query"].as_str().unwrap().to_owned()];
        qs.extend(strings(c.get("alt_queries")));
        for q in qs {
            cost.run(store, c, &q, c["cwd"].as_str().unwrap_or(""))
                .await?;
        }
    }
    let mut unique_df: HashMap<String, usize> = HashMap::new();
    let mut unique_words = HashMap::new();
    for r in &records {
        let ws: BTreeSet<_> = query::surface(&r.body)
            .into_iter()
            .filter(|w| w.chars().count() >= 5 && w.chars().all(char::is_alphabetic))
            .collect();
        for w in &ws {
            *unique_df.entry(w.clone()).or_default() += 1;
        }
        unique_words.insert(r.id.clone(), ws);
    }
    let candidates: Vec<_> = records
        .iter()
        .filter(|r| {
            scopes.contains(&r.scope)
                && eligible(r, &validity)
                && unique_words[&r.id].iter().any(|w| unique_df[w] == 1)
        })
        .take(5)
        .collect();
    let mut forget = !candidates.is_empty();
    for r in candidates {
        let term = unique_words[&r.id]
            .iter()
            .find(|w| unique_df[*w] == 1)
            .unwrap();
        let c = json!({"id":format!("forget-{}",r.id),"scope":r.scope});
        let before = cost.run(&mut rebuilt, &c, term, "").await?;
        rebuilt.forget(&r.id).await?;
        let after = cost.run(&mut rebuilt, &c, term, "").await?;
        let text = after
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        forget &= ids(&before).contains(&r.id)
            && !ids(&after).contains(&r.id)
            && !lines(&r.body).iter().any(|l| text.contains(l));
    }
    let g4 = integrity_build && rebuilt.integrity().await.is_ok() && forget;
    let mut worst = 0f64;
    for loops in exposure.values().filter(|v| v.len() >= 20) {
        let mut totals: HashMap<&str, f64> = HashMap::new();
        for row in loops {
            for (id, w) in row {
                *totals.entry(id).or_default() += w;
            }
        }
        for total in totals.values() {
            worst = worst.max(total / loops.len() as f64);
        }
    }
    println!("| gate | value | need | n | status |\n|---|---|---|---|---|");
    gate(
        "G1 eligibility violations",
        rows.iter().filter(|(_, r)| r["violations"] == true).count() as f64,
        0.0,
        false,
        None,
    );
    gate(
        "G2 must-not rate",
        rounded(
            rows.iter().filter(|(_, r)| r["must_not"] == true).count() as f64
                / rows.len().max(1) as f64,
        ),
        0.02,
        false,
        None,
    );
    gate(
        "G3 location invariance",
        if inv_n == 0 {
            1.0
        } else {
            inv as f64 / inv_n as f64
        },
        1.0,
        true,
        Some(inv_n),
    );
    gate("G4 integrity and forget", f64::from(g4), 1.0, true, None);
    gate(
        "G5 determinism after rebuild",
        if rows.is_empty() {
            1.0
        } else {
            det as f64 / rows.len() as f64
        },
        1.0,
        true,
        None,
    );
    for (name, key, cats, exclude, threshold) in [
        ("Q1 empty-correct", "empty", vec![], vec![], 0.90),
        (
            "Q2 core found",
            "found",
            vec!["identifier", "phrase", "doc-passage"],
            vec![],
            0.85,
        ),
        (
            "Q3 overall found",
            "found",
            vec![],
            vec!["paraphrase", "lateral"],
            0.75,
        ),
        (
            "Q4 section hit",
            "section",
            vec!["doc-passage"],
            vec![],
            0.75,
        ),
        ("Q5 lateral found", "found", vec!["lateral"], vec![], 0.60),
        (
            "Q6 paraphrase found",
            "found",
            vec!["paraphrase"],
            vec![],
            0.35,
        ),
    ] {
        let (v, n) = rate(&rows, key, &cats, &exclude);
        gate(name, v, threshold, true, Some(n));
    }
    gate("C1 tokens median", median(&cost.tokens), 600.0, false, None);
    gate(
        "C1 tokens p95",
        percentile(&cost.tokens, 0.95),
        1500.0,
        false,
        None,
    );
    gate(
        "C2 latency p95 ms",
        (percentile(&cost.ms, 0.95) * 100.0).round_ties_even() / 100.0,
        50.0,
        false,
        None,
    );
    gate(
        "R1 self-retrieval@5",
        if ki_n == 0 {
            f64::NAN
        } else {
            ki as f64 / ki_n as f64
        },
        0.90,
        true,
        Some(ki_n),
    );
    gate(
        "R2 maximum exposure share",
        rounded(worst),
        0.10,
        false,
        None,
    );
    Ok(())
}
fn fts_tokens(text: &str) -> Vec<String> {
    query::WORD
        .find_iter(text)
        .filter_map(|m| {
            let mut start = m.start();
            while start > 0 && text.as_bytes()[start - 1] == b'-' {
                start -= 1;
            }
            let token = &text[start..m.end()];
            token
                .chars()
                .any(char::is_alphanumeric)
                .then(|| token.to_lowercase())
        })
        .collect()
}
// MT19937 integer-seed initialization and rejection-based getrandbits shuffle.
struct Shuffle {
    state: [u32; 624],
    index: usize,
}
impl Shuffle {
    fn new() -> Self {
        let mut s = Self {
            state: [0; 624],
            index: 624,
        };
        s.state[0] = 19650218;
        for i in 1..624 {
            s.state[i] = 1812433253u32
                .wrapping_mul(s.state[i - 1] ^ (s.state[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let mut i = 1;
        for _ in 0..624 {
            s.state[i] = (s.state[i]
                ^ ((s.state[i - 1] ^ (s.state[i - 1] >> 30)).wrapping_mul(1664525)))
            .wrapping_add(7);
            i += 1;
            if i >= 624 {
                s.state[0] = s.state[623];
                i = 1;
            }
        }
        for _ in 0..623 {
            s.state[i] = (s.state[i]
                ^ ((s.state[i - 1] ^ (s.state[i - 1] >> 30)).wrapping_mul(1566083941)))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                s.state[0] = s.state[623];
                i = 1;
            }
        }
        s.state[0] = 0x80000000;
        s
    }
    fn next(&mut self) -> u32 {
        if self.index >= 624 {
            for i in 0..624 {
                let y = (self.state[i] & 0x80000000) | (self.state[(i + 1) % 624] & 0x7fffffff);
                self.state[i] = self.state[(i + 397) % 624]
                    ^ (y >> 1)
                    ^ if y & 1 != 0 { 0x9908b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
    fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let n = i + 1;
            let bits = usize::BITS - n.leading_zeros();
            let j = loop {
                let v = (self.next() >> (32 - bits)) as usize;
                if v < n {
                    break v;
                }
            };
            x.swap(i, j);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeded_shuffle_matches_integer_seed_stream() {
        let mut rng = Shuffle::new();
        assert_eq!(rng.next(), 1390851128);
    }
    #[test]
    fn neutral_scope_self_test() {
        let r: Record =
            serde_json::from_value(json!({"id":"r","body":"answer","scope":"other/repo"})).unwrap();
        let mut by = HashMap::from([("r".into(), r)]);
        let c = json!({"scope":"my/repo","gold":[],"must_not":["r"]});
        let page = vec![Call {
            cards: vec![Card {
                record: "r".into(),
                ..Card::default()
            }],
            ..Call::default()
        }];
        assert!(empty_ok(&c, &page, &by));
        assert!(!empty_ok(&c, &[], &by));
        assert!(!forbidden(&c, &page, &by));
        by.get_mut("r").unwrap().scope = Some("MY/REPO".into());
        assert!(forbidden(&c, &page, &by));
        assert!(!empty_ok(&c, &page, &by));
    }
    #[test]
    fn author_fields_are_unread_and_ineligible_text_is_detected() {
        let r:Record=serde_json::from_value(json!({"id":"bad","state":"candidate","body":"This distinctive forbidden line is longer than thirty characters.","terms":{"not":"a term list"},"section_terms":false})).unwrap();
        assert!(serde_json::to_value(&r).unwrap().get("terms").is_none());
        let by = HashMap::from([("bad".into(), r.clone())]);
        let validity = Validity::project(std::slice::from_ref(&r));
        assert!(violations(
            &[Call {
                text: r.body.clone(),
                ..Call::default()
            }],
            &by,
            &validity
        ));
        assert!(violations(
            &[Call {
                cards: vec![Card {
                    record: "bad".into(),
                    ..Card::default()
                }],
                ..Call::default()
            }],
            &by,
            &validity
        ));
    }
    #[test]
    fn percentile_uses_nearest_even_and_unicode_token_cost() {
        assert_eq!(percentile(&[1., 2., 3., 4.], 0.5), 3.);
        assert_eq!(median(&[1., 2., 3., 4.]), 2.5);
        assert_eq!("ééééé".chars().count().div_ceil(4), 2);
    }
}
