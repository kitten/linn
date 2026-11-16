//! Fresh-process CLI sampling and paired semantic-output comparisons.
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}
#[derive(Subcommand)]
enum Mode {
    /// Fixed dev sample, with optional comparison and raw output capture.
    Sample(SampleArgs),
    /// Alternating paired rounds, warm-up and a separate diagnostic round.
    Compare(CompareArgs),
}
fn main() -> Result<(), Box<dyn Error>> {
    match Args::parse().mode {
        Mode::Sample(args) => sample(args),
        Mode::Compare(args) => compare(args),
    }
}
fn sample_indices(length: usize, requested: usize) -> impl Iterator<Item = usize> {
    let count = requested.min(length);
    (0..count).map(move |i| i * (length - 1) / (count - 1).max(1))
}
fn category_sample<'a>(
    cases: &[&'a Value],
    per_group: usize,
    require_gold: bool,
) -> Vec<(&'static str, &'a Value)> {
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for group in [
        "identifier",
        "phrase",
        "doc-passage",
        "lateral",
        "paraphrase",
        "no-gold",
    ] {
        let matches: Vec<_> = cases
            .iter()
            .copied()
            .filter(|case| {
                let no_gold = case["gold"].as_array().is_none_or(Vec::is_empty);
                if group == "no-gold" {
                    no_gold
                } else {
                    (!require_gold || !no_gold)
                        && case["category"]
                            .as_array()
                            .is_some_and(|v| v.iter().any(|v| v == group))
                }
            })
            .collect();
        for index in sample_indices(matches.len(), per_group) {
            let case = matches[index];
            if seen.insert(case["id"].as_str().unwrap()) {
                selected.push((group, case));
            }
        }
    }
    selected
}
fn normalize(mut value: Value) -> Value {
    // Ignore clock bytes, while preserving cursor presence and null/string differences.
    if let Some(object) = value.as_object_mut()
        && object.get("cursor").is_some_and(Value::is_string)
    {
        object.insert("cursor".into(), json!("<clock-bearing cursor>"));
    }
    value
}
fn run_cli(
    binary: &Path,
    store: &Path,
    query: &str,
    trace: bool,
) -> Result<(Output, f64), Box<dyn Error>> {
    let started = Instant::now();
    let mut command = Command::new(binary);
    command
        .arg("--store")
        .arg(store)
        .args(["--json", "search", query])
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
        return Err(format!(
            "CLI failed for {query}: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok((output, elapsed))
}
#[derive(clap::Args)]
struct SampleArgs {
    #[arg(long)]
    binary: PathBuf,
    #[arg(long)]
    store: PathBuf,
    #[arg(long, env = "LINN_EVAL_CORPUS")]
    corpus: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 2)]
    per_group: usize,
    /// Add a focused dev case to the fixed category sample.
    #[arg(long)]
    case: Vec<String>,
    /// Run only the explicitly named dev cases.
    #[arg(long, requires = "case")]
    only_cases: bool,
    /// Compare card/hint/related output against another real CLI binary.
    #[arg(long)]
    compare_binary: Option<PathBuf>,
}
fn sample(args: SampleArgs) -> Result<(), Box<dyn Error>> {
    if args.per_group == 0 {
        return Err("per-group must be positive".into());
    }
    fs::create_dir_all(&args.output)?;
    let cases: Vec<Value> = fs::read_to_string(args.corpus.join("cases.jsonl"))?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let dev: Vec<_> = cases
        .iter()
        .filter(|c| {
            Sha256::digest(c["id"].as_str().unwrap().as_bytes())
                .iter()
                .fold(0u32, |n, b| (n * 256 + *b as u32) % 10)
                >= 3
        })
        .collect();
    let mut selected = if args.only_cases {
        Vec::new()
    } else {
        category_sample(&dev, args.per_group, true)
    };
    let mut seen: HashSet<_> = selected
        .iter()
        .map(|(_, c)| c["id"].as_str().unwrap().to_owned())
        .collect();
    for id in &args.case {
        let c = dev
            .iter()
            .find(|c| c["id"] == *id)
            .ok_or_else(|| format!("case {id} is absent or outside dev"))?;
        if seen.insert(id.clone()) {
            selected.push(("focused", *c));
        }
    }
    let mut durations = Vec::new();
    let mut exports = String::new();
    for (ordinal, (group, case)) in selected.into_iter().enumerate() {
        let id = case["id"].as_str().unwrap();
        let query = case["query"].as_str().unwrap();
        let (output, wall_ms) = run_cli(&args.binary, &args.store, query, false)?;
        fs::write(
            args.output.join(format!("{ordinal:04}.stdout")),
            &output.stdout,
        )?;
        fs::write(
            args.output.join(format!("{ordinal:04}.stderr")),
            &output.stderr,
        )?;
        if !matches!(output.status.code(), Some(0 | 1)) {
            return Err(format!(
                "CLI failed for {id}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        let page: Value = serde_json::from_slice(&output.stdout)?;
        if let Some(binary) = &args.compare_binary {
            let (before, _) = run_cli(binary, &args.store, query, false)?;
            if before.status.code() != output.status.code()
                || normalize(serde_json::from_slice(&before.stdout)?) != normalize(page.clone())
            {
                return Err(format!("CLI page differs for {id}").into());
            }
        }
        let event = json!({"case":id,"group":group,"query":query,"wall_ms":wall_ms,"exit":output.status.code(),"records":page["cards"].as_array().into_iter().flatten().map(|c|c["record"].clone()).collect::<Vec<_>>()});
        println!("{event}");
        exports.push_str(&format!("{event}\n"));
        durations.push(wall_ms);
    }
    if durations.is_empty() {
        return Err("no cases selected".into());
    }
    fs::write(args.output.join("timings.jsonl"), exports)?;
    durations.sort_by(f64::total_cmp);
    let n = durations.len();
    eprintln!(
        "fresh CLI invocations: n={n} median_ms={:.3} p95_ms={:.3} max_ms={:.3}; wall clock and caller scope are the actual CLI context; not a retrieval or neutral gate",
        (durations[(n - 1) / 2] + durations[n / 2]) / 2.,
        durations[(0.95 * (n - 1) as f64).round_ties_even() as usize],
        durations[n - 1]
    );
    Ok(())
}

#[derive(clap::Args)]
struct CompareArgs {
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
fn invoke(
    args: &CompareArgs,
    case: &Value,
    after: bool,
    trace: bool,
) -> Result<Value, Box<dyn Error>> {
    let (output, elapsed) = run_cli(
        if after { &args.after } else { &args.before },
        if after {
            &args.store
        } else {
            args.before_store.as_ref().unwrap_or(&args.store)
        },
        case["query"].as_str().unwrap(),
        trace,
    )?;
    let response = normalize(serde_json::from_slice(&output.stdout)?);
    Ok(
        json!({"case":case["id"],"after":after,"trace":trace,"ms":elapsed,"exit":output.status.code(),"response":response,"diagnostics":String::from_utf8_lossy(&output.stderr)}),
    )
}
fn compare(args: CompareArgs) -> Result<(), Box<dyn Error>> {
    if args.per_group == 0 || args.rounds == 0 {
        return Err("per-group and rounds must be positive".into());
    }
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
    let population: Vec<_> = cases.iter().collect();
    let mut selected: BTreeSet<_> = category_sample(&population, args.per_group, false)
        .into_iter()
        .map(|(_, c)| c["id"].as_str().unwrap().to_owned())
        .collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sampling_keeps_endpoints_and_handles_small_populations() {
        assert_eq!(
            sample_indices(0, 2).collect::<Vec<_>>(),
            Vec::<usize>::new()
        );
        assert_eq!(sample_indices(1, 2).collect::<Vec<_>>(), [0]);
        assert_eq!(sample_indices(9, 3).collect::<Vec<_>>(), [0, 4, 8]);
        assert_eq!(sample_indices(3, 9).collect::<Vec<_>>(), [0, 1, 2]);
    }
    #[test]
    fn parity_ignores_clock_bytes_but_preserves_page_and_cursor_shape() {
        let a = json!({"cards":[{"record":"a"}],"cursor":"clock-a"});
        let b = json!({"cards":[{"record":"a"}],"cursor":"clock-b"});
        assert_eq!(normalize(a.clone()), normalize(b));
        for different in [
            json!({"cards":[{"record":"b"}],"cursor":"clock-b"}),
            json!({"cards":[{"record":"a"}],"cursor":null}),
            json!({"cards":[{"record":"a"}]}),
        ] {
            assert_ne!(normalize(a.clone()), normalize(different));
        }
    }
}
