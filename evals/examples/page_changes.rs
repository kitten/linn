//! Compare saved pages; no searches or changes to retrieval scoring.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

#[derive(Deserialize)]
struct Page {
    case: String,
    wording: usize,
    records: Vec<String>,
}
type Pages = BTreeMap<(String, usize), Vec<String>>;
fn pages(path: &str) -> Result<Pages, Box<dyn Error>> {
    let mut out = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let p: Page = serde_json::from_str(line)?;
        if out.insert((p.case, p.wording), p.records).is_some() {
            return Err("duplicate page".into());
        }
    }
    Ok(out)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: page_changes CORPUS BEFORE AFTER".into());
    }
    let before = pages(&args[2])?;
    let after = pages(&args[3])?;
    if !before.keys().eq(after.keys()) {
        return Err("page populations differ".into());
    }
    let records: BTreeMap<String, String> =
        fs::read_to_string(Path::new(&args[1]).join("records.jsonl"))?
            .lines()
            .map(|l| {
                let v: Value = serde_json::from_str(l).unwrap();
                (
                    v["id"].as_str().unwrap().to_owned(),
                    v["scope"].as_str().unwrap_or("").to_lowercase(),
                )
            })
            .collect();
    let mut changes = BTreeMap::<String, Vec<String>>::new();
    for line in fs::read_to_string(Path::new(&args[1]).join("cases.jsonl"))?.lines() {
        let c: Value = serde_json::from_str(line)?;
        let id = c["id"].as_str().ok_or("case id missing")?;
        if !before.contains_key(&(id.to_owned(), 0)) {
            continue;
        }
        let mut gold = BTreeSet::new();
        for g in c["gold"].as_array().into_iter().flatten() {
            gold.insert(g["record"].as_str().unwrap().to_owned());
            for e in g["equivalent"].as_array().into_iter().flatten() {
                gold.insert(e.as_str().unwrap().to_owned());
            }
        }
        let scope = c["scope"].as_str().unwrap_or("").to_lowercase();
        let mut xs = BTreeSet::new();
        if c["category"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v == "cross-scope"))
        {
            for v in c["must_not"].as_array().into_iter().flatten() {
                let r = v.as_str().unwrap();
                if records.get(r).is_some_and(|s| !s.is_empty() && s != &scope) {
                    xs.insert(r.to_owned());
                }
            }
        }
        for (label, targets) in [("gold", gold), ("xs", xs)] {
            if targets.is_empty() {
                continue;
            }
            for first in [false, true] {
                let hit = |data: &BTreeMap<(String, usize), Vec<String>>| {
                    data.iter()
                        .filter(|((cid, w), _)| cid == id && (!first || *w == 0))
                        .any(|(_, rows)| rows.iter().take(5).any(|r| targets.contains(r)))
                };
                let (a, b) = (hit(&before), hit(&after));
                if a != b {
                    changes
                        .entry(format!(
                            "{label}_{}_{}",
                            if first { "first" } else { "any" },
                            if b { "gain" } else { "loss" }
                        ))
                        .or_default()
                        .push(id.to_owned());
                }
            }
        }
    }
    let changed: Vec<_> = before
        .iter()
        .filter(|(key, rows)| after.get(*key) != Some(*rows))
        .map(|(key, rows)| json!({"case":key.0,"wording":key.1,"before":rows,"after":after[key]}))
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"pages":before.len(),"changed_pages":changed.len(),"case_changes":changes,"pages_changed":changed})
        )?
    );
    Ok(())
}
