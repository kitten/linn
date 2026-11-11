mod document_benchmark;
mod neutral;
use clap::Parser;
use linn::{
    record::Record,
    sections::parse_sections,
    store::{Card, Context, NOW, Store},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
#[derive(Parser)]
struct Args {
    #[arg(long, env = "LINN_EVAL_CORPUS", default_value = "evals/corpus/q4")]
    corpus: PathBuf,
    #[arg(long,default_value="dev",value_parser=["dev","all"])]
    mode: String,
    /// Run the neutral fixed prepared-query grader and print its gate table.
    #[arg(long, conflicts_with_all = ["query", "rebuild_vectors", "pages", "compare", "compare_pages"])]
    neutral: bool,
    /// Score original questions by first-call document Hit@1/3/5 and MRR@5.
    #[arg(long, conflicts_with_all = ["neutral", "query", "compare", "compare_pages", "compare_vectors", "rebuild_vectors"])]
    document_benchmark: bool,
    #[arg(long)]
    pages: Option<PathBuf>,
    /// Export complete calls for exact parity checks without verbose rank traces.
    #[arg(long, conflicts_with_all = ["neutral", "document_benchmark", "query", "compare_pages", "compare_vectors", "rebuild_vectors"])]
    responses: Option<PathBuf>,
    /// Export per-wording agent invocation time, excluding ingest and grading.
    #[arg(long, conflicts_with_all = ["neutral", "document_benchmark", "query", "compare_pages", "compare_vectors", "rebuild_vectors"])]
    timings: Option<PathBuf>,
    #[arg(long)]
    compare: Option<PathBuf>,
    /// Compare previously exported pages without rebuilding or searching.
    #[arg(long, requires = "compare")]
    compare_pages: Option<PathBuf>,
    #[arg(long)]
    store: Option<PathBuf>,
    #[arg(long)]
    query: Option<String>,
    #[arg(long)]
    scope: Option<String>,
    /// Copy a local corpus into ignored eval storage: capture <source> <name>.
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long)]
    case: Vec<String>,
    #[arg(long)]
    compare_vectors: Option<PathBuf>,
    #[arg(long)]
    rebuild_vectors: bool,
}
fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    BufReader::new(File::open(path)?)
        .lines()
        .filter_map(|line| match line {
            Ok(s) if s.trim().is_empty() => None,
            x => Some(x),
        })
        .map(|line| Ok(serde_json::from_str(&line?)?))
        .collect()
}
fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn has_category(c: &Value, s: &str) -> bool {
    strings(c.get("category")).iter().any(|v| v == s)
}
fn split(id: &str) -> bool {
    Sha256::digest(id.as_bytes())
        .iter()
        .fold(0u32, |n, b| (n * 256 + *b as u32) % 10)
        >= 3
}
fn section_hit(card: &Card, record: &Record, g: &Value) -> bool {
    let body = &record.body;
    let quote = g["quote"].as_str().unwrap_or("").trim();
    let heading = g["heading"].as_str().unwrap_or("").trim();
    let sections = parse_sections(body);
    let sec = if heading.is_empty() {
        None
    } else {
        sections
            .iter()
            .find(|s| s.heading.trim().to_lowercase() == heading.to_lowercase())
            .or_else(|| {
                sections.iter().find(|s| {
                    !s.path.is_empty()
                        && s.path
                            .join(" > ")
                            .to_lowercase()
                            .ends_with(&heading.to_lowercase())
                })
            })
            .map(|s| s.span.clone())
    };
    if !heading.is_empty() && sec.is_none() && quote.is_empty() {
        return false;
    }
    let mut alts = Vec::new();
    if !quote.is_empty() {
        let mut offset = 0;
        while offset < body.len() {
            if let Some(i) = body[offset..].find(quote) {
                let a = offset + i;
                alts.push(a..a + quote.len());
                offset = a + body[a..].chars().next().unwrap().len_utf8();
            } else {
                break;
            }
        }
        if let Some(s) = &sec {
            alts.retain(|a| s.start <= a.start && a.end <= s.end);
        }
    } else {
        alts.push(sec.clone().unwrap_or(0..body.len()));
    }
    let len = sec
        .map(|s| s.len())
        .unwrap_or_else(|| alts.iter().map(|s| s.len()).max().unwrap_or(0));
    card.span[1] - card.span[0] <= (3 * len).max(1500)
        && alts
            .iter()
            .any(|s| card.span[0] <= s.start && s.end <= card.span[1])
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(actual) = &args.compare_pages {
        let actual = page_map(actual)?;
        let expected = page_map(args.compare.as_ref().unwrap())?;
        if actual.is_empty() || expected.is_empty() {
            return Err("page exports must not be empty".into());
        }
        let mut differences = 0;
        for (key, page) in &actual {
            if expected.get(key) != Some(page) {
                differences += 1;
                eprintln!(
                    "page differs {key}: actual={page} expected={:?}",
                    expected.get(key)
                );
            }
        }
        for key in expected.keys() {
            if !actual.contains_key(key) {
                differences += 1;
                eprintln!("missing actual page {key}");
            }
        }
        eprintln!("compared {} pages; {differences} differences", actual.len());
        if differences > 0 {
            return Err("page parity failed".into());
        }
        return Ok(());
    }
    if let Some(Command::Capture { source, name }) = &args.command {
        capture(source, name)?;
        return Ok(());
    }
    let records: Vec<Record> = load(&args.corpus.join("records.jsonl"))?;
    let by: HashMap<_, _> = records.iter().map(|r| (r.id.clone(), r.clone())).collect();
    let temp = tempfile::tempdir()?;
    let opening = std::time::Instant::now();
    let path = args
        .store
        .clone()
        .unwrap_or_else(|| temp.path().join("eval.db"));
    let mut store = if path.exists() {
        Store::open(&path).await?
    } else {
        Store::build(&path, records.clone()).await?
    };
    if args.rebuild_vectors {
        store.vectors.clear();
        store.rebuild().await?;
    }
    eprintln!(
        "built {} records, {} blocks",
        store.record_count(),
        store.block_count()
    );
    let store_ms = opening.elapsed().as_secs_f64() * 1000.;
    if let Some(path) = args.compare_vectors {
        store.load_vectors().await?;
        let expected = fs::read(path)?;
        let mut ids: Vec<_> = store.vectors.keys().copied().collect();
        ids.sort_unstable();
        let actual: Vec<f32> = ids
            .iter()
            .flat_map(|id| store.vectors[id].iter().copied())
            .collect();
        if expected.len() != actual.len() * 4 {
            return Err("embedding matrix dimensions differ".into());
        }
        let mut bad = Vec::new();
        for (id, chunk) in ids.iter().zip(expected.chunks_exact(512 * 4)) {
            let error = chunk
                .chunks_exact(4)
                .zip(&store.vectors[id])
                .map(|(b, a)| (f32::from_le_bytes(b.try_into().unwrap()) - a).abs())
                .fold(0f32, f32::max);
            if error > 1e-6 {
                bad.push((*id, error));
            }
        }
        eprintln!(
            "embedding rows differing: {} of {}; first {:?}",
            bad.len(),
            store.block_count(),
            &bad[..bad.len().min(10)]
        );
        let error = expected
            .chunks_exact(4)
            .zip(actual)
            .map(|(bytes, actual)| (f32::from_le_bytes(bytes.try_into().unwrap()) - actual).abs())
            .fold(0f32, f32::max);
        eprintln!("embedding matrix maximum absolute error: {error}");
        if error > 1e-6 {
            return Err("embedding matrix differs from expected".into());
        }
    }
    if let Some(q) = args.query {
        let ctx = Context {
            scope: args.scope,
            ..Context::default()
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&store.agent(&q, &ctx).await?)?
        );
        return Ok(());
    }
    let cases: Vec<Value> = load(&args.corpus.join("cases.jsonl"))?;
    let all_cases = cases.clone();
    let cases: Vec<_> = cases
        .into_iter()
        .filter(|c| {
            (args.mode == "all" || split(c["id"].as_str().unwrap()))
                && (args.case.is_empty()
                    || args.case.iter().any(|id| id == c["id"].as_str().unwrap()))
        })
        .collect();
    if args.neutral {
        return neutral::run(&mut store, records, cases, all_cases).await;
    }
    if args.document_benchmark {
        return document_benchmark::run(&mut store, &cases, args.pages.as_deref()).await;
    }
    let mut pages = args.pages.as_ref().map(File::create).transpose()?;
    let mut responses = args.responses.as_ref().map(File::create).transpose()?;
    let mut timings = args.timings.as_ref().map(File::create).transpose()?;
    let mut query_times = Vec::new();
    if let Some(file) = &mut timings {
        writeln!(
            file,
            "{}",
            json!({"event":"store-ready","store_ms":store_ms,"records":store.record_count(),"blocks":store.block_count()})
        )?;
    }
    let comparing = args.compare.is_some();
    let expected = if let Some(path) = &args.compare {
        page_map(path)?
    } else {
        HashMap::new()
    };
    if comparing && expected.is_empty() {
        return Err("page exports must not be empty".into());
    }
    let mut compared = BTreeSet::new();
    let (
        mut ng,
        mut nx,
        mut ne,
        mut ns,
        mut any,
        mut first,
        mut xs,
        mut xs1,
        mut empty,
        mut section,
        mut differences,
    ) = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    for (ci, c) in cases.iter().enumerate() {
        let id = c["id"].as_str().unwrap();
        let scope = c["scope"].as_str().map(str::to_owned);
        let me = scope.as_deref().unwrap_or("").to_lowercase();
        let gold = c["gold"].as_array().cloned().unwrap_or_default();
        let gids: HashSet<String> = gold
            .iter()
            .flat_map(|g| {
                std::iter::once(g["record"].as_str().unwrap().to_owned())
                    .chain(strings(g.get("equivalent")))
            })
            .collect();
        let must_not: HashSet<_> = strings(c.get("must_not")).into_iter().collect();
        let targets: HashSet<_> = must_not
            .iter()
            .filter(|rid| {
                by.get(*rid)
                    .and_then(|r| r.scope.as_ref())
                    .is_some_and(|s| !s.is_empty() && s.to_lowercase() != me)
            })
            .cloned()
            .collect();
        let xt = if has_category(c, "cross-scope") {
            targets.clone()
        } else {
            HashSet::new()
        };
        let doc = has_category(c, "doc-passage") && !gids.is_empty();
        if !gids.is_empty() {
            ng += 1;
        }
        if !xt.is_empty() {
            nx += 1;
        }
        if doc {
            ns += 1;
        }
        let mut wordings = vec![c["query"].as_str().unwrap().to_owned()];
        wordings.extend(strings(c.get("alt_queries")));
        wordings.truncate(3);
        let (mut hit, mut xhit) = (false, false);
        let mut shown = Vec::new();
        for (qi, q) in wordings.iter().enumerate() {
            let ctx = Context {
                scope: scope.clone(),
                cwd: String::new(),
                now: NOW.into(),
                session: id.into(),
                calls_left: 3,
            };
            let started = std::time::Instant::now();
            let calls = store.agent(q, &ctx).await?;
            let query_ms = started.elapsed().as_secs_f64() * 1000.;
            if let Some(file) = &mut responses {
                writeln!(
                    file,
                    "{}",
                    json!({"stage":"calls","case":id,"wording":qi,"query":q,"calls":calls})
                )?;
            }
            if std::env::var_os("LINN_POS_TRACE").is_some() {
                eprintln!(
                    "{}",
                    json!({"stage":"calls","case":id,"wording":qi,"query":q,"calls":calls})
                );
            }
            if let Some(file) = &mut timings {
                query_times.push(query_ms);
                writeln!(
                    file,
                    "{}",
                    json!({"case":id,"wording":qi,"query_ms":query_ms,"calls":calls.len()})
                )?;
            }
            let page: Vec<String> = calls
                .first()
                .into_iter()
                .flat_map(|call| &call.cards)
                .take(5)
                .map(|card| card.record.clone())
                .collect();
            if let Some(file) = pages.as_mut() {
                writeln!(file, "{}", json!({"case":id,"wording":qi,"records":page}))?;
            }
            if comparing {
                let key = format!("{id}:{qi}");
                compared.insert(key.clone());
                match expected.get(&key) {
                    Some(exp) if *exp == json!(page) => {}
                    Some(exp) => {
                        differences += 1;
                        eprintln!("page differs {key}: actual={} expected={exp}", json!(page));
                    }
                    None => {
                        differences += 1;
                        eprintln!("missing expected page {key}");
                    }
                }
            }
            if page.iter().any(|r| gids.contains(r)) {
                hit = true;
                if qi == 0 {
                    first += 1;
                }
            }
            if page.iter().any(|r| xt.contains(r)) {
                xhit = true;
                if qi == 0 {
                    xs1 += 1;
                }
            }
            shown.extend(calls.into_iter().flat_map(|call| call.cards));
        }
        any += usize::from(hit);
        xs += usize::from(xhit);
        if gids.is_empty() {
            ne += 1;
            let got: HashSet<_> = shown.iter().map(|c| c.record.clone()).collect();
            let mn = got.iter().any(|rid| {
                must_not.contains(rid)
                    && by
                        .get(rid)
                        .and_then(|r| r.scope.as_ref())
                        .is_none_or(|s| s.is_empty() || s.to_lowercase() == me)
            });
            empty += usize::from(if targets.is_empty() {
                got.is_empty() && !mn
            } else {
                !got.is_disjoint(&targets) && !mn
            });
        }
        if doc {
            section += usize::from(gold.iter().all(|g| {
                let rid = g["record"].as_str().unwrap();
                by.get(rid).is_some_and(|r| {
                    shown
                        .iter()
                        .any(|card| card.record == rid && section_hit(card, r, g))
                })
            }));
        }
        if (ci + 1) % 50 == 0 {
            eprintln!("scored {}/{}", ci + 1, cases.len());
        }
    }
    println!(
        "any {any}/{ng} first {first}/{ng} xs {xs}/{nx} {xs1}/{nx}\nempty {empty}/{ne} section {section}/{ns}"
    );
    if !query_times.is_empty() {
        query_times.sort_by(f64::total_cmp);
        let n = query_times.len();
        let median = (query_times[(n - 1) / 2] + query_times[n / 2]) / 2.;
        let p95 = query_times[(0.95 * (n - 1) as f64).round_ties_even() as usize];
        eprintln!(
            "agent invocation timings: n={n} median_ms={median:.3} p95_ms={p95:.3} max_ms={:.3}; store_ms={store_ms:.3}; excludes corpus loading, grading and output; not the neutral per-call gate",
            query_times[n - 1]
        );
    }
    for key in expected.keys().filter(|key| !compared.contains(*key)) {
        differences += 1;
        eprintln!("missing actual page {key}");
    }
    if differences > 0 {
        return Err(format!("{differences} first pages differ").into());
    }
    let _ = fs::metadata(&args.corpus)?;
    Ok(())
}

fn page_map(path: &Path) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let mut pages = HashMap::new();
    for v in load::<Value>(path)? {
        let key = format!(
            "{}:{}",
            v["case"].as_str().ok_or("page is missing case id")?,
            v["wording"]
                .as_u64()
                .ok_or("page is missing wording index")?
        );
        if !v["records"]
            .as_array()
            .is_some_and(|ids| ids.iter().all(Value::is_string))
        {
            return Err("page records must be an array of strings".into());
        }
        let page = v["records"].clone();
        if pages.insert(key.clone(), page).is_some() {
            return Err(format!("duplicate page key {key}").into());
        }
    }
    Ok(pages)
}

#[derive(clap::Subcommand)]
enum Command {
    Capture { source: PathBuf, name: String },
}
fn capture(source: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("capture name must contain only letters, digits, - or _".into());
    }
    let destination = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join(name);
    for file in ["records.jsonl", "cases.jsonl"] {
        let target = destination.join(file);
        let output = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&target)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()?;
        if output.status.success() {
            return Err(format!("refusing tracked path {}", target.display()).into());
        }
        if output.status.code() != Some(1) {
            return Err("cannot verify capture destination is untracked".into());
        }
        if target.exists() {
            return Err(format!("capture destination already exists: {}", target.display()).into());
        }
        if !source.join(file).is_file() {
            return Err(format!("missing corpus file {file}").into());
        }
    }
    fs::create_dir_all(&destination)?;
    for file in ["records.jsonl", "cases.jsonl"] {
        fs::copy(source.join(file), destination.join(file))?;
    }
    eprintln!("Captured corpus at {}", destination.display());
    Ok(())
}
