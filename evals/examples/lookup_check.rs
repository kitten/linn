//! Compare complete calls and indexed SQL/in-memory record lookups on a copied store.
//! @ref LLP 0005#lookup-optimization-diagnostic
use linn::store::Store;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    hint::black_box,
    io::Write,
    path::Path,
    process::Command,
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err(
            "usage: lookup_check CORPUS COPIED_STORE BEFORE_BINARY AFTER_BINARY OUTPUT_DIRECTORY"
                .into(),
        );
    }
    let corpus = Path::new(&args[0]);
    let output = Path::new(&args[4]);
    fs::create_dir_all(output)?;
    let mut store = Store::open(Path::new(&args[1])).await?;
    let mut rows = store
        .db
        .query("SELECT id FROM records ORDER BY id LIMIT 128", ())
        .await?;
    let mut ids = Vec::new();
    while let Some(row) = rows.next().await? {
        ids.push(row.get::<String>(0)?);
    }
    drop(rows);
    if ids.is_empty() {
        return Err("lookup benchmark needs records".into());
    }
    for id in &ids {
        store.load_record_blocks(id).await?;
    }
    // Require the same ordered rows before timing either implementation.
    for id in &ids {
        let expected: Vec<_> = store.record_blocks(id).map(|b| b.id).collect();
        let mut rows = store
            .db
            .query(
                "SELECT id FROM sections WHERE record=? ORDER BY id",
                [(*id).clone()],
            )
            .await?;
        let mut actual = Vec::new();
        while let Some(row) = rows.next().await? {
            actual.push(row.get::<i64>(0)? as usize);
        }
        if actual != expected {
            return Err(format!("SQL and memory lookup differ for {id}").into());
        }
    }
    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        for id in &ids {
            black_box(store.record_blocks(id).map(|b| b.id).collect::<Vec<_>>());
        }
    }
    let memory_ns = start.elapsed().as_nanos() as f64 / (ids.len() * iterations) as f64;
    let start = Instant::now();
    for _ in 0..iterations {
        for id in &ids {
            let mut rows = store
                .db
                .query(
                    "SELECT id FROM sections WHERE record=? ORDER BY id",
                    [(*id).clone()],
                )
                .await?;
            let mut values = Vec::new();
            while let Some(row) = rows.next().await? {
                values.push(row.get::<i64>(0)? as usize);
            }
            black_box(values);
        }
    }
    let sql_ns = start.elapsed().as_nanos() as f64 / (ids.len() * iterations) as f64;
    println!(
        "record lookup ns/op: memory {memory_ns:.0}, indexed SQL {sql_ns:.0}; {} lookups each; ordered rows identical",
        ids.len() * iterations
    );
    drop(store);
    let cases: Vec<Value> = fs::read_to_string(corpus.join("cases.jsonl"))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let dev: Vec<_> = cases
        .iter()
        .filter(|c| {
            Sha256::digest(c["id"].as_str().unwrap().as_bytes())
                .iter()
                .fold(0u32, |r, b| (r * 256 + *b as u32) % 10)
                >= 3
        })
        .collect();
    if dev.is_empty() {
        return Err("no dev cases".into());
    }
    let count = dev.len().min(24);
    let mut files = [
        File::create(output.join("before.calls.jsonl"))?,
        File::create(output.join("after.calls.jsonl"))?,
    ];
    let mut timings = [0.; 2];
    for i in 0..count {
        let c = dev[i * (dev.len() - 1) / (count - 1).max(1)];
        let mut responses = Vec::new();
        for (j, binary) in args[2..4].iter().enumerate() {
            let start = Instant::now();
            let mut command = Command::new(binary);
            command
                .arg("--corpus")
                .arg(corpus)
                .arg("--store")
                .arg(&args[1])
                .arg(format!("--query={}", c["query"].as_str().unwrap()));
            if let Some(scope) = c["scope"].as_str() {
                command.arg("--scope").arg(scope);
            }
            let result = command.output()?;
            timings[j] += start.elapsed().as_secs_f64();
            if !result.status.success() {
                return Err(String::from_utf8_lossy(&result.stderr).into_owned().into());
            }
            let calls: Value = serde_json::from_slice(&result.stdout)?;
            writeln!(files[j], "{}", json!({"case":c["id"],"calls":calls}))?;
            responses.push(calls);
        }
        if responses[0] != responses[1] {
            return Err(format!("complete calls differ for {}", c["id"]).into());
        }
    }
    println!(
        "complete calls identical on {count} fixed, evenly spaced dev queries (all calls, cards, spans, excerpts, hints and related pointers)"
    );
    println!(
        "full-command wall seconds including process startup and store open: before {:.3}, after {:.3}; not a search-latency gate",
        timings[0], timings[1]
    );
    Ok(())
}
