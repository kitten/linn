//! Compare the local rule engine with the unchanged registry implementation.
#[allow(dead_code)]
#[path = "../../src/nlp/compiler.rs"]
mod nlp;

use clap::Parser;
use serde_json::{Value, json};
use std::{error::Error, fs, path::PathBuf, time::Instant};

#[derive(Parser)]
struct Args {
    /// JSONL cases; compare each prepared wording without inspecting gold.
    #[arg(long)]
    cases: Option<PathBuf>,
    /// Original source units as JSONL, with a `text` field.
    #[arg(long)]
    units: Option<PathBuf>,
}

// Keep the two engines' public structures independent; compare a stable explicit
// schema rather than their Debug implementation or their private storage layout.
macro_rules! annotation {
    ($sentence:expr) => {{
        let sentence = $sentence;
        json!({
            "text": sentence.text(),
            "byte_span": sentence.span().byte(),
            "char_span": sentence.span().char(),
            "tokens": sentence.tokens().iter().map(|token| json!({
                "text": token.word().text().as_str(),
                "byte_span": token.span().byte(),
                "char_span": token.span().char(),
                "has_space_before": token.has_space_before(),
                "chunks": token.chunks(),
                "tags": token.word().tags().iter().map(|tag| json!({
                    "lemma": tag.lemma().as_str(),
                    "pos": tag.pos().as_str(),
                    "frozen": tag.frozen(),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })
    }};
}

fn fixtures() -> Vec<String> {
    [
        "",
        " ",
        " \r\n\u{2003}",
        "cache invalidation policy",
        "\u{2003}cache invalidation policy",
        "The user does not invalidate the cache. The cache invalidates the user.",
        "The cache invalidation policy does not invalidate the user.",
        "café cache.json foo_bar alpha-beta read/local `cache policy`",
        "\u{2003}café cache.json foo_bar alpha-beta read/local `cache policy`",
        "The file that the user said was missing is not the same file. Why?",
        "Élodie’s naïve café 😀. e\u{301}lan\u{00a0}cache\tpolicy\r\nnext.",
        "US us Us USA iPhone cachePolicy CACHE cache Cache caches",
        "qxzvplugh flibbertigibbet123 foo_bar HTTPServer x-y-z",
        "mother-in-law ice cream New York as well as in spite of each other",
        "https://example.org/a?q=cache#policy user@example.org www.example.org.",
        "(cache) [policy] {file} \"cache\" cache+policy ^cache$ a.*b 1.25 12:30",
        "I saw her duck. They can fish. Time flies like an arrow; fruit flies like a banana.",
        "They record a record and object to the object. The old man the boats.",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(["cache policy ".repeat(200)])
    .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let bytes = include_bytes!(env!("LINN_POS_TOKENIZER_BUILD"));
    let old = nlprule::Tokenizer::from_reader(bytes.as_slice())?;
    let compiled = nlp::compile_asset(bytes)?;
    let new = linn::nlp::Tokenizer::from_compiled(&compiled)?;
    let mut texts = fixtures();
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
        let a: Vec<_> = old.pipe(text).map(|s| annotation!(s)).collect();
        old_ms += started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let b: Vec<_> = new.pipe(text).map(|s| annotation!(s)).collect();
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
    let compiled = nlp::compile_asset(bytes).unwrap();
    let new = linn::nlp::Tokenizer::from_compiled(&compiled).unwrap();
    assert!(
        compiled == nlp::compile_asset(bytes).unwrap(),
        "compiled asset must be reproducible"
    );
    for text in fixtures() {
        let a: Vec<_> = old.pipe(&text).map(|s| annotation!(s)).collect();
        let b: Vec<_> = new.pipe(&text).map(|s| annotation!(s)).collect();
        assert_eq!(a, b, "token/tag/chunk/span mismatch for {text:?}");
    }
}
