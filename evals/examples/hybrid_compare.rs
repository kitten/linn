//! Local comparison with archived hybrid pages, using matched original document sets.
//! Not a hybrid model runner or a replacement for the full retrieval scorer.
use linn::{
    record::Record,
    store::{Context, Store},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    io::Write,
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[derive(Deserialize)]
struct Hit {
    id: String,
    qi: usize,
    files: Vec<String>,
}
#[derive(Deserialize)]
struct Query {
    id: String,
    qi: usize,
    q: String,
}
#[derive(Default, Serialize)]
struct Stats {
    gold: usize,
    any: usize,
    first: usize,
    mrr_sum: f64,
    xs: usize,
    xs_any: usize,
    xs_first: usize,
    empty: usize,
    empty_correct: usize,
}
fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn wordings(c: &Value) -> Vec<String> {
    std::iter::once(c["query"].as_str().unwrap().to_owned())
        .chain(strings(&c["alt_queries"]))
        .take(3)
        .collect()
}
fn targets(c: &Value) -> BTreeSet<String> {
    c["gold"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|g| {
            std::iter::once(g["record"].as_str().unwrap().to_owned())
                .chain(strings(&g["equivalent"]))
        })
        .collect()
}
fn summarize(
    c: &Value,
    pages: &[Vec<String>],
    shown: &BTreeSet<String>,
    scopes: &BTreeMap<String, String>,
    s: &mut Stats,
) -> Value {
    let gold = targets(c);
    let me = c["scope"].as_str().unwrap_or("").to_lowercase();
    let forbidden: BTreeSet<_> = strings(&c["must_not"]).into_iter().collect();
    let other: BTreeSet<_> = forbidden
        .iter()
        .filter(|id| {
            scopes
                .get(*id)
                .is_some_and(|scope| !scope.is_empty() && scope != &me)
        })
        .cloned()
        .collect();
    let hit = |ids: &BTreeSet<String>, p: &[String]| p.iter().take(5).any(|id| ids.contains(id));
    let any = pages.iter().any(|p| hit(&gold, p));
    let first = hit(&gold, &pages[0]);
    let rank = pages[0]
        .iter()
        .take(5)
        .position(|id| gold.contains(id))
        .map(|i| i + 1);
    if !gold.is_empty() {
        s.gold += 1;
        s.any += usize::from(any);
        s.first += usize::from(first);
        s.mrr_sum += rank.map_or(0., |r| 1. / r as f64);
    }
    let cross = strings(&c["category"]).iter().any(|x| x == "cross-scope") && !other.is_empty();
    let xs_any = pages.iter().any(|p| hit(&other, p));
    let xs_first = hit(&other, &pages[0]);
    if cross {
        s.xs += 1;
        s.xs_any += usize::from(xs_any);
        s.xs_first += usize::from(xs_first);
    }
    let bad = shown
        .iter()
        .any(|id| forbidden.contains(id) && !other.contains(id));
    let empty_correct = if other.is_empty() {
        shown.is_empty() && !bad
    } else {
        !shown.is_disjoint(&other) && !bad
    };
    if gold.is_empty() {
        s.empty += 1;
        s.empty_correct += usize::from(empty_correct);
    }
    json!({"case":c["id"],"gold":!gold.is_empty(),"any":any,"first":first,"rank":rank,"xs":cross,"xs_any":xs_any,"xs_first":xs_first,"empty_correct":gold.is_empty().then_some(empty_correct)})
}
fn load_lines<T: serde::de::DeserializeOwned>(p: &Path) -> Result<Vec<T>> {
    fs::read_to_string(p)?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| Ok(serde_json::from_str(s)?))
        .collect()
}
fn documents(path: &Path, all: &[Record], expected: usize) -> Result<Vec<Record>> {
    let by: BTreeMap<_, _> = all.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(path)? {
        let p = entry?.path();
        if !p.is_file() || p.extension().and_then(|s| s.to_str()) != Some("md") {
            return Err("unexpected archived document entry".into());
        }
        let id = p
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("invalid document name")?;
        let r = by
            .get(id)
            .ok_or("archive document missing from captured corpus")?;
        if fs::read(&p)? != r.body.as_bytes() {
            return Err(format!("archived body differs: {id}").into());
        }
        if !ids.insert(id.to_owned()) {
            return Err("duplicate archived id".into());
        }
    }
    if ids.len() != expected {
        return Err(format!("wrong archive count: {}", ids.len()).into());
    }
    Ok(all
        .iter()
        .filter(|r| ids.contains(&r.id))
        .cloned()
        .collect())
}
async fn compare(
    slice: (&str, &str, &str),
    store: &mut Store,
    cases: &[Value],
    records: &[Record],
    original_records: &[Record],
    archive: &Path,
    output: &Path,
) -> Result<Value> {
    let (name, queries_file, hits_file) = slice;
    let queries: Vec<Query> = serde_json::from_slice(&fs::read(archive.join(queries_file))?)?;
    let hits: Vec<Hit> = load_lines(&archive.join(hits_file))?;
    let mut qs = BTreeMap::new();
    let mut hs = BTreeMap::new();
    for q in queries {
        if qs.insert((q.id, q.qi), q.q).is_some() {
            return Err("duplicate query key".into());
        }
    }
    for h in hits {
        if h.files.len() > 5 {
            return Err("archived page exceeds five results".into());
        }
        if hs.insert((h.id, h.qi), h.files).is_some() {
            return Err("duplicate hit key".into());
        }
    }
    if !qs.keys().eq(hs.keys()) {
        return Err("query/page populations differ".into());
    }
    let scopes: BTreeMap<_, _> = original_records
        .iter()
        .map(|r| {
            (
                r.id.clone(),
                r.scope.as_deref().unwrap_or("").to_lowercase(),
            )
        })
        .collect();
    let indexed: BTreeSet<_> = records.iter().map(|r| &r.id).collect();
    if hs.values().flatten().any(|id| !indexed.contains(id)) {
        return Err("hybrid page contains unindexed record".into());
    }
    let ids: BTreeSet<_> = qs.keys().map(|(id, _)| id.as_str()).collect();
    let selected: Vec<_> = cases
        .iter()
        .filter(|c| ids.contains(c["id"].as_str().unwrap()))
        .collect();
    if selected.len() != ids.len() {
        return Err("missing captured case".into());
    }
    let mut checked = 0;
    for c in &selected {
        let id = c["id"].as_str().unwrap();
        if targets(c)
            .iter()
            .chain(strings(&c["must_not"]).iter())
            .any(|id| !scopes.contains_key(id))
        {
            return Err("case references a record outside the captured corpus".into());
        }
        if Sha256::digest(id.as_bytes())
            .iter()
            .fold(0u32, |a, b| (a * 256 + *b as u32) % 10)
            < 3
        {
            return Err("archive contains a non-dev case".into());
        }
        for (i, q) in wordings(c).iter().enumerate() {
            if qs.get(&(id.to_owned(), i)) != Some(q) {
                return Err("query bytes differ".into());
            }
            checked += 1;
        }
    }
    if checked != qs.len() {
        return Err("extra archived query".into());
    }
    let (mut current, mut hybrid) = (Stats::default(), Stats::default());
    let mut paired = Vec::new();
    let mut export = fs::File::create(output.join(format!("{name}.calls.jsonl")))?;
    let mut latency = Vec::new();
    for c in selected {
        let id = c["id"].as_str().unwrap();
        let mut pages = Vec::new();
        let mut historical = Vec::new();
        let mut shown = BTreeSet::new();
        for (i, q) in wordings(c).iter().enumerate() {
            let ctx = Context {
                scope: c["scope"].as_str().map(str::to_owned),
                session: id.to_owned(),
                ..Context::default()
            };
            let start = std::time::Instant::now();
            let calls = store.agent(q, &ctx).await?;
            latency.push(start.elapsed().as_secs_f64() * 1000.);
            pages.push(calls.first().map_or_else(Vec::new, |call| {
                call.cards
                    .iter()
                    .take(5)
                    .map(|card| card.record.clone())
                    .collect()
            }));
            shown.extend(
                calls
                    .iter()
                    .flat_map(|call| call.cards.iter().map(|card| card.record.clone())),
            );
            historical.push(
                hs[&(id.to_owned(), i)]
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            writeln!(
                export,
                "{}",
                json!({"case":id,"wording":i,"query":q,"calls":calls,"hybrid":historical.last()})
            )?;
        }
        let old_shown = historical.iter().flatten().cloned().collect();
        paired.push(json!({"linn":summarize(c,&pages,&shown,&scopes,&mut current),"hybrid":summarize(c,&historical,&old_shown,&scopes,&mut hybrid)}));
    }
    let expected = match name {
        "small-s1" => (47, 47, 48),
        "large-s1" => (46, 40, 48),
        "large-s5" => (78, 64, 81),
        _ => unreachable!(),
    };
    if (hybrid.any, hybrid.first, hybrid.gold) != expected {
        return Err("historical gold scores do not reproduce".into());
    }
    latency.sort_by(f64::total_cmp);
    Ok(
        json!({"name":name,"records":records.len(),"cases":paired.len(),"wordings":checked,"linn":current,"hybrid":hybrid,"linn_query_ms_median":latency[latency.len()/2],"linn_query_ms_p95":latency[((latency.len()-1) as f64*0.95).floor() as usize],"paired":paired,"hits_sha256":format!("{:x}",Sha256::digest(fs::read(archive.join(hits_file))?))}),
    )
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: hybrid_compare CAPTURED_CORPUS READONLY_ARCHIVE OUTPUT_DIRECTORY".into(),
        );
    }
    let corpus = Path::new(&args[1]);
    let archive = Path::new(&args[2]);
    let output = Path::new(&args[3]);
    fs::create_dir_all(output)?;
    let records: Vec<Record> = load_lines(&corpus.join("records.jsonl"))?;
    let cases: Vec<Value> = load_lines(&corpus.join("cases.jsonl"))?;
    let large = documents(&archive.join("docs-q4"), &records, 1243)?;
    let small = documents(&archive.join("docs-sub"), &records, 90)?;
    for name in ["large.db", "small.db"] {
        if output.join(name).exists() {
            return Err("refuse to reuse an existing comparison store".into());
        }
    }
    let mut results = Vec::new();
    let mut store = Store::build(&output.join("large.db"), large.clone()).await?;
    for (name, queries, hits) in [
        ("large-s1", "queries-sub.json", "hits-q4-hybrid-sub.jsonl"),
        ("large-s5", "queries-s5.json", "hits-q4-hybrid-s5.jsonl"),
    ] {
        results.push(
            compare(
                (name, queries, hits),
                &mut store,
                &cases,
                &large,
                &records,
                archive,
                output,
            )
            .await?,
        );
    }
    drop(store);
    let mut store = Store::build(&output.join("small.db"), small.clone()).await?;
    results.push(
        compare(
            ("small-s1", "queries-sub.json", "hits-sub.jsonl"),
            &mut store,
            &cases,
            &small,
            &records,
            archive,
            output,
        )
        .await?,
    );
    let data = json!({"model":linn::embed::ARTIFACT_MODEL,"body_bytes_verified":true,"query_bytes_verified":true,"fresh_external_run":false,"comparisons":results,
        "records_sha256": format!("{:x}", Sha256::digest(fs::read(corpus.join("records.jsonl"))?)),
        "cases_sha256": format!("{:x}", Sha256::digest(fs::read(corpus.join("cases.jsonl"))?))});
    fs::write(
        output.join("comparison.json"),
        serde_json::to_string_pretty(&data)?,
    )?;
    for r in data["comparisons"].as_array().unwrap() {
        println!(
            "{}",
            json!({"name":r["name"],"records":r["records"],"cases":r["cases"],"linn":r["linn"],"hybrid":r["hybrid"]})
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_wording_rank_and_object_equivalents() {
        let c = json!({"id":"c","scope":"a","gold":[{"record":"g","equivalent":["e"]}],"equivalent":["wrong"]});
        let mut s = Stats::default();
        let pages = vec![vec!["x".into(), "e".into()]];
        summarize(&c, &pages, &BTreeSet::new(), &BTreeMap::new(), &mut s);
        assert_eq!((s.gold, s.any, s.first), (1, 1, 1));
        assert_eq!(s.mrr_sum, 0.5);
    }
    #[test]
    fn other_repository_empty_target_requires_its_card() {
        let c = json!({"scope":"a","gold":[],"must_not":["x","bad"]});
        let scopes = BTreeMap::from([("x".into(), "b".into()), ("bad".into(), "a".into())]);
        for (shown, expected) in [(vec!["x"], 1), (vec![], 0), (vec!["x", "bad"], 0)] {
            let mut s = Stats::default();
            summarize(
                &c,
                &[vec![]],
                &shown.into_iter().map(str::to_owned).collect(),
                &scopes,
                &mut s,
            );
            assert_eq!(s.empty_correct, expected);
        }
    }
}
