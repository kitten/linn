//! Raw JSONL audit. Input fields: text, optional source; labels are never read.
use linn_eval::{analyze, witnesses};
use sha2::{Digest, Sha256};
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected tokenizer path")?;
    let start = std::time::Instant::now();
    let bytes = std::fs::read(path)?;
    if format!("{:x}", Sha256::digest(&bytes))
        != "e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773"
    {
        return Err("tokenizer is not the audited official English 0.6.4 resource".into());
    }
    let tokenizer = nlprule::Tokenizer::from_reader(bytes.as_slice())?;
    eprintln!(
        "{}",
        serde_json::json!({"load_ms":start.elapsed().as_secs_f64()*1000., "rules":tokenizer.rules().len(), "chunker":tokenizer.chunker().is_some()})
    );
    let mut encoder = None;
    for line in io::stdin().lock().lines() {
        let input: serde_json::Value = serde_json::from_str(&line?)?;
        let text = input["text"].as_str().ok_or("missing raw text")?;
        let start = std::time::Instant::now();
        let query = analyze(&tokenizer, text);
        if input["summary"].as_bool() == Some(true) {
            println!(
                "{}",
                serde_json::json!({"bytes":text.len(),"tokens":query.occurrences.len(),"ambiguous":query.occurrences.iter().filter(|t| t.analyses.iter().map(|(_, p)| p.as_str()).filter(|p| !p.is_empty() && *p != "UNKNOWN" && *p != "SENT_END").collect::<std::collections::BTreeSet<_>>().len()>1).count(),"missing_chunks":query.occurrences.iter().filter(|t| t.chunks.is_empty()).count(),"np":query.groups.iter().filter(|g| g.kind.starts_with("NP")).count(),"vp":query.groups.iter().filter(|g| g.kind.starts_with("VP")).count(),"groups":query.groups.len(),"ms":start.elapsed().as_secs_f64()*1000.})
            );
            continue;
        }
        let source = input["source"].as_str().map(|s| analyze(&tokenizer, s));
        let matrix = if input["embedding"].as_bool() == Some(true) {
            let s = source.as_ref().ok_or("embedding requires source")?;
            let raw = input["source"].as_str().unwrap();
            if encoder.is_none() {
                encoder = Some(linn::embed::Encoder::load()?);
            }
            let encoder = encoder.as_ref().unwrap();
            let qtext: Vec<_> = query
                .groups
                .iter()
                .map(|g| text[g.span.clone()].to_owned())
                .collect();
            let stext: Vec<_> = s
                .groups
                .iter()
                .map(|g| raw[g.span.clone()].to_owned())
                .collect();
            let qv = encoder.batch(&qtext);
            let sv = encoder.batch(&stext);
            Some(
                qv.iter()
                    .map(|q| {
                        sv.iter()
                            .map(|v| linn::embed::Encoder::similarity(q, v))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let support = source.as_ref().map(|s| {
            linn_eval::literal_support(text, &query, input["source"].as_str().unwrap(), s)
        });
        let pairs = source
            .as_ref()
            .map(|s| witnesses(text, &query, input["source"].as_str().unwrap(), s, true));
        println!(
            "{}",
            serde_json::json!({"text":text,"query":query,"source":source,"typed_exact_witnesses":pairs,"matrix":matrix,"support":support,"ms":start.elapsed().as_secs_f64()*1000.})
        );
    }
    Ok(())
}
