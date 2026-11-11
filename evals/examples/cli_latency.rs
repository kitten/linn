//! Fresh-process CLI timing on fixed dev originals, independent of gold scoring.
use clap::Parser;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::PathBuf, process::Command, time::Instant};

#[derive(Parser)]
struct Args {
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
fn semantic(mut value: Value) -> Value {
    // A cursor includes wall-clock eligibility time. Its encoded bytes are not a page.
    if let Some(object) = value.as_object_mut() {
        object.remove("cursor");
    }
    value
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
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
        if args.only_cases {
            continue;
        }
        let matches: Vec<_> = dev
            .iter()
            .copied()
            .filter(|c| {
                if group == "no-gold" {
                    c["gold"].as_array().is_none_or(Vec::is_empty)
                } else {
                    c["gold"].as_array().is_some_and(|g| !g.is_empty())
                        && c["category"]
                            .as_array()
                            .is_some_and(|v| v.iter().any(|v| v == group))
                }
            })
            .collect();
        let count = args.per_group.min(matches.len());
        for i in 0..count {
            let c = matches[i * (matches.len() - 1) / (count - 1).max(1)];
            if seen.insert(c["id"].as_str().unwrap().to_owned()) {
                selected.push((group, c));
            }
        }
    }
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
        let start = Instant::now();
        let output = Command::new(&args.binary)
            .arg("--store")
            .arg(&args.store)
            .args(["--json", "search", query])
            .env_remove("LINN_STARTUP_TIMINGS")
            .output()?;
        let wall_ms = start.elapsed().as_secs_f64() * 1000.;
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
            let before = Command::new(binary)
                .arg("--store")
                .arg(&args.store)
                .args(["--json", "search", query])
                .env_remove("LINN_STARTUP_TIMINGS")
                .output()?;
            if before.status.code() != output.status.code()
                || semantic(serde_json::from_slice(&before.stdout)?) != semantic(page.clone())
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
