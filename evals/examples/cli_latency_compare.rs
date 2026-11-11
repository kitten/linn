//! Paired fresh-CLI latency and semantic-output parity across query categories.
use clap::Parser;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet, error::Error, fs, path::PathBuf, process::Command, time::Instant,
};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    corpus: PathBuf,
    #[arg(long)]
    pages: PathBuf,
    #[arg(long)]
    before: PathBuf,
    #[arg(long)]
    after: PathBuf,
    #[arg(long)]
    store: PathBuf,
    /// Separate identical baseline store, avoiding alternating schema migrations.
    #[arg(long)]
    before_store: Option<PathBuf>,
    #[arg(long, default_value_t = 3)]
    rounds: usize,
    #[arg(long, default_value_t = 2)]
    per_group: usize,
}
fn normalize(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        // Cursor embeds wall clock; presence and the rest of the response still match.
        if object.get("cursor").is_some_and(Value::is_string) {
            object.insert("cursor".into(), json!("<clock-bearing cursor>"));
        }
    }
    value
}
fn invoke(args: &Args, case: &Value, after: bool, trace: bool) -> Result<Value, Box<dyn Error>> {
    let started = Instant::now();
    let mut command = Command::new(if after { &args.after } else { &args.before });
    command
        .arg("--store")
        .arg(if after {
            &args.store
        } else {
            args.before_store.as_ref().unwrap_or(&args.store)
        })
        .args(["--json", "search", case["query"].as_str().unwrap()])
        .env_remove("LINN_STARTUP_TIMINGS")
        .env_remove("LINN_POS_TIMINGS");
    if trace {
        command
            .env("LINN_STARTUP_TIMINGS", "1")
            .env("LINN_POS_TIMINGS", "1");
    }
    let output = command.output()?;
    let elapsed = started.elapsed().as_secs_f64() * 1000.;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let response = normalize(serde_json::from_slice(&output.stdout)?);
    Ok(
        json!({"case":case["id"],"after":after,"trace":trace,"ms":elapsed,"exit":output.status.code(),"response":response,"diagnostics":String::from_utf8_lossy(&output.stderr)}),
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let eligible: BTreeSet<String> = fs::read_to_string(&args.pages)?
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).unwrap();
            value["case"].as_str().unwrap().to_owned()
        })
        .collect();
    let cases: Vec<Value> = fs::read_to_string(args.corpus.join("cases.jsonl"))?
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|c| eligible.contains(c["id"].as_str().unwrap()))
        .collect();
    let mut selected = BTreeSet::new();
    for category in [
        "identifier",
        "phrase",
        "doc-passage",
        "lateral",
        "paraphrase",
        "no-gold",
    ] {
        let group: Vec<_> = cases
            .iter()
            .filter(|c| {
                if category == "no-gold" {
                    c["gold"].as_array().is_none_or(Vec::is_empty)
                } else {
                    c["category"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|v| v == category))
                }
            })
            .collect();
        let count = args.per_group.min(group.len());
        for i in 0..count {
            let case = group[i * (group.len() - 1) / (count - 1).max(1)];
            selected.insert(case["id"].as_str().unwrap().to_owned());
        }
    }
    if selected.is_empty() {
        return Err("no benchmark cases".into());
    }
    // The store corpus also exercises prefixes outside the product CLI grammar.
    let excluded_cli_queries: BTreeSet<_> = cases
        .iter()
        .filter(|c| {
            selected.contains(c["id"].as_str().unwrap())
                && !linn::query::Query::parse(c["query"].as_str().unwrap()).is_ok_and(|q| {
                    q.filters.iter().all(|(key, _)| {
                        ["scope", "under", "path", "kind", "id"].contains(&key.as_str())
                    })
                })
        })
        .map(|c| c["id"].as_str().unwrap().to_owned())
        .collect();
    selected.retain(|id| !excluded_cli_queries.contains(id));
    if selected.is_empty() {
        return Err("no CLI-supported benchmark cases".into());
    }
    // One unmeasured pair performs any one-time derived-table upgrade.
    let first = cases
        .iter()
        .find(|c| selected.contains(c["id"].as_str().unwrap()))
        .unwrap();
    invoke(&args, first, false, false)?;
    invoke(&args, first, true, false)?;
    let mut rows = Vec::new();
    for round in 0..=args.rounds {
        for case in cases
            .iter()
            .filter(|c| selected.contains(c["id"].as_str().unwrap()))
        {
            let mut pair = Vec::new();
            for after in if round % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let mut row = invoke(&args, case, after, round == args.rounds)?;
                row["round"] = json!(round);
                pair.push(row);
            }
            if pair[0]["exit"] != pair[1]["exit"] || pair[0]["response"] != pair[1]["response"] {
                return Err(format!("semantic output differs for {}", case["id"]).into());
            }
            rows.extend(pair);
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"cases":selected,"excluded_cli_queries":excluded_cli_queries,"rows":rows})
        )?
    );
    Ok(())
}
