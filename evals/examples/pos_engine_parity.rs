//! Compare the local rule engine with the unchanged registry implementation.
use clap::Parser;
use serde_json::{Value, json};
use std::{error::Error, fs, path::PathBuf, time::Instant};

#[derive(Parser)]
struct Args {
    /// JSONL cases; compare each prepared wording without inspecting gold.
    #[arg(long)]
    cases: Option<PathBuf>,
    /// Original source units exported by pos_units, with a `text` field.
    #[arg(long)]
    units: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let bytes = include_bytes!(env!("LINN_POS_TOKENIZER_BUILD"));
    let old = nlprule::Tokenizer::from_reader(bytes.as_slice())?;
    let compiled = linn::nlp::Tokenizer::from_reader(bytes.as_slice())?.compile_asset()?;
    let new = linn::nlp::Tokenizer::from_compiled(&compiled)?;
    let mut texts = vec![
        "cache invalidation policy".to_owned(),
        "\u{2003}cache invalidation policy".to_owned(),
        "The user does not invalidate the cache. The cache invalidates the user.".to_owned(),
        "café cache.json foo_bar alpha-beta read/local `cache policy`".to_owned(),
        "".to_owned(),
        " ".to_owned(),
        "cache policy ".repeat(200),
    ];
    if let Some(path) = args.cases {
        for line in fs::read_to_string(path)?.lines() {
            let case: Value = serde_json::from_str(line)?;
            texts.push(case["query"].as_str().ok_or("missing query")?.to_owned());
            texts.extend(
                case["alt_queries"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .take(2)
                    .filter_map(Value::as_str)
                    .map(str::to_owned),
            );
        }
    }
    if let Some(path) = args.units {
        for line in fs::read_to_string(path)?.lines() {
            let unit: Value = serde_json::from_str(line)?;
            texts.push(unit["text"].as_str().ok_or("missing text")?.to_owned());
        }
    }
    let mut old_ms = 0.;
    let mut new_ms = 0.;
    for (i, text) in texts.iter().enumerate() {
        let started = Instant::now();
        let a: Vec<_> = old.pipe(text).map(|s| format!("{s:?}")).collect();
        old_ms += started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let b: Vec<_> = new.pipe(text).map(|s| format!("{s:?}")).collect();
        new_ms += started.elapsed().as_secs_f64() * 1000.;
        if a != b {
            return Err(format!("token/tag/chunk/span difference at input {i}").into());
        }
    }
    println!(
        "{}",
        json!({"inputs":texts.len(),"identical":true,"registry_ms":old_ms,"local_ms":new_ms})
    );
    Ok(())
}

#[test]
fn preserves_complete_linguistic_analysis() {
    let bytes = include_bytes!(env!("LINN_POS_TOKENIZER_BUILD"));
    let old = nlprule::Tokenizer::from_reader(bytes.as_slice()).unwrap();
    let compiled = linn::nlp::Tokenizer::from_reader(bytes.as_slice())
        .unwrap()
        .compile_asset()
        .unwrap();
    let new = linn::nlp::Tokenizer::from_compiled(&compiled).unwrap();
    assert_eq!(
        compiled,
        linn::nlp::Tokenizer::from_reader(bytes.as_slice())
            .unwrap()
            .compile_asset()
            .unwrap(),
        "compiled asset must be reproducible"
    );
    for text in [
        "".to_owned(),
        " \r\n\u{2003}".to_owned(),
        "The cache invalidation policy does not invalidate the user.".to_owned(),
        "\u{2003}café cache.json foo_bar alpha-beta read/local `cache policy`".to_owned(),
        "The file that the user said was missing is not the same file. Why?".to_owned(),
        "cache policy ".repeat(200),
    ] {
        let a: Vec<_> = old.pipe(&text).map(|s| format!("{s:?}")).collect();
        let b: Vec<_> = new.pipe(&text).map(|s| format!("{s:?}")).collect();
        assert_eq!(a, b, "token/tag/chunk/span mismatch for {text:?}");
    }
}
