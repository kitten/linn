//! @ref LLP 0005#external-document-benchmarks
use super::strings;
use linn::store::{Call, Context, NOW, Store};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::Write,
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
struct Stats {
    cases: usize,
    hit1: usize,
    hit3: usize,
    hit5: usize,
    reciprocal: f64,
}
impl Stats {
    fn add(&mut self, rank: Option<usize>) {
        self.cases += 1;
        if let Some(rank) = rank {
            self.hit1 += usize::from(rank == 1);
            self.hit3 += usize::from(rank <= 3);
            self.hit5 += 1;
            self.reciprocal += 1.0 / rank as f64;
        }
    }
}
fn targets(case: &Value) -> Result<HashSet<String>> {
    let gold = case["gold"]
        .as_array()
        .ok_or("document benchmark requires gold objects")?;
    let mut ids = HashSet::new();
    for g in gold {
        ids.insert(
            g["record"]
                .as_str()
                .ok_or("gold record must be a string")?
                .to_owned(),
        );
        ids.extend(strings(g.get("equivalent")));
    }
    if ids.is_empty() {
        return Err("document benchmark requires a gold document for every case".into());
    }
    Ok(ids)
}
fn rank(calls: &[Call], gold: &HashSet<String>) -> Option<usize> {
    calls
        .first()
        .into_iter()
        .flat_map(|call| call.cards.iter().take(5))
        .position(|card| gold.contains(&card.record))
        .map(|i| i + 1)
}
pub async fn run(store: &mut Store, cases: &[Value], pages: Option<&Path>) -> Result<()> {
    if cases.is_empty() {
        return Err("no document benchmark cases selected".into());
    }
    // Validate all relevance targets before searching or creating an output file.
    let targets: Vec<_> = cases.iter().map(targets).collect::<Result<_>>()?;
    for ids in &targets {
        for id in ids {
            if !store.contains_record(id).await? {
                return Err(format!("gold document not in captured records: {id}").into());
            }
        }
    }
    let mut output = pages.map(File::create).transpose()?;
    let mut all = Stats::default();
    let mut groups = BTreeMap::<String, Stats>::new();
    for (case, gold) in cases.iter().zip(targets) {
        let query = case["query"]
            .as_str()
            .ok_or("case query must be a string")?;
        let id = case["id"].as_str().ok_or("case id must be a string")?;
        let ctx = Context {
            scope: case["scope"].as_str().map(str::to_owned),
            cwd: String::new(),
            now: NOW.into(),
            session: id.into(),
            calls_left: 3,
        };
        // Original question only. No prepared alternatives, intent, notes or gold enter retrieval.
        let calls = store.agent(query, &ctx).await?;
        let result = rank(&calls, &gold);
        all.add(result);
        if let Some(group) = case["benchmark_group"].as_str() {
            groups.entry(group.to_owned()).or_default().add(result);
            if let Some((suite, _)) = group.split_once('/') {
                groups.entry(suite.to_owned()).or_default().add(result);
            }
        }
        if let Some(file) = &mut output {
            let ids: Vec<_> = calls
                .first()
                .into_iter()
                .flat_map(|c| c.cards.iter().take(5))
                .map(|c| &c.record)
                .collect();
            writeln!(file, "{}", json!({"case":id,"wording":0,"records":ids}))?;
        }
    }
    println!("| group | cases | Hit@1 | Hit@3 | Hit@5 | MRR@5 |\n|---|---|---|---|---|---|");
    for (group, s) in std::iter::once(("all".to_owned(), all)).chain(groups) {
        let group = group.replace('|', "\\|").replace(['\n', '\r'], " ");
        println!(
            "| {group} | {} | {}/{} | {}/{} | {}/{} | {:.4} |",
            s.cases,
            s.hit1,
            s.cases,
            s.hit3,
            s.cases,
            s.hit5,
            s.cases,
            s.reciprocal / s.cases as f64
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use linn::store::Card;
    fn call(ids: &[&str]) -> Call {
        Call {
            cards: ids
                .iter()
                .map(|id| Card {
                    record: (*id).into(),
                    ..Card::default()
                })
                .collect(),
            ..Call::default()
        }
    }
    #[test]
    fn first_call_and_actual_card_positions_determine_credit() {
        let gold = HashSet::from(["gold".into()]);
        assert_eq!(rank(&[call(&["x", "x", "gold"])], &gold), Some(3));
        assert_eq!(rank(&[call(&["x"]), call(&["gold"])], &gold), None);
        assert_eq!(
            rank(&[call(&["a", "b", "c", "d", "e", "gold"])], &gold),
            None
        );
        let mut s = Stats::default();
        for rank in [Some(1), Some(3), Some(5), None] {
            s.add(rank);
        }
        assert_eq!((s.cases, s.hit1, s.hit3, s.hit5), (4, 1, 2, 3));
        assert!((s.reciprocal - (1.0 + 1.0 / 3.0 + 1.0 / 5.0)).abs() < 1e-12);
    }
    #[test]
    fn only_gold_object_targets_define_relevance() {
        let case = json!({"gold":[{"record":"a","equivalent":["b"]}],"acceptable":["c"],"equivalent":{"a":["d"]},"intent":"a"});
        assert_eq!(
            targets(&case).unwrap(),
            HashSet::from(["a".into(), "b".into()])
        );
        assert!(targets(&json!({"gold":[]})).is_err());
    }
}
