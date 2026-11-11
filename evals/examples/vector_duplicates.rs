//! Measure byte-identical stored vectors without changing a store or search.
use clap::Parser;
use libsql::Builder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, error::Error, path::PathBuf, time::Instant};

#[derive(Parser)]
struct Args {
    store: PathBuf,
    #[arg(long, default_value = "sentences", value_parser = ["sentences", "forms", "beam"])]
    kind: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !args.store.is_file() {
        return Err("supply an existing copied store".into());
    }
    let started = Instant::now();
    let database = Builder::new_local(&args.store).build().await?;
    let db = database.connect()?;
    db.execute_batch("PRAGMA query_only=ON; PRAGMA mmap_size=268435456;")
        .await?;
    let mut kind = args.kind.clone();
    let mut weights = None;
    if kind == "sentences" {
        let mut manifest = db
            .query(
                "SELECT metadata,rows FROM fallback_artifacts WHERE kind='sentences-dictionary-v1'",
                (),
            )
            .await?;
        if let Some(row) = manifest.next().await? {
            let metadata: serde_json::Value = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
            let order: Vec<usize> = serde_json::from_value(metadata["order"].clone())?;
            let mut counts = vec![0usize; usize::try_from(row.get::<i64>(1)?)?];
            for ordinal in order {
                *counts.get_mut(ordinal).ok_or("invalid sentence ordinal")? += 1;
            }
            weights = Some(counts);
            kind = "sentences-dictionary-v1".into();
        }
    }
    let mut result = db
        .query(
            "SELECT data FROM fallback_vectors WHERE kind=? ORDER BY part",
            [kind.as_str()],
        )
        .await?;
    let mut counts = HashMap::<[u8; 32], usize>::new();
    let mut rows = 0usize;
    let mut stored_rows = 0usize;
    while let Some(row) = result.next().await? {
        let bytes = row.get::<Vec<u8>>(0)?;
        if bytes.len() % (512 * 4) != 0 {
            return Err("invalid vector payload size".into());
        }
        for vector in bytes.chunks_exact(512 * 4) {
            let occurrences = weights
                .as_ref()
                .map_or(Some(&1), |weights| weights.get(stored_rows))
                .ok_or("dictionary row count mismatch")?;
            *counts.entry(Sha256::digest(vector).into()).or_default() += occurrences;
            rows += occurrences;
            stored_rows += 1;
        }
    }
    if rows == 0 {
        return Err("no stored vectors for the requested kind".into());
    }
    if weights
        .as_ref()
        .is_some_and(|weights| weights.len() != stored_rows)
    {
        return Err("dictionary row count mismatch".into());
    }
    println!(
        "{}",
        json!({
            "kind":args.kind,"stored_kind":kind,"rows":rows,"stored_rows":stored_rows,"sha256_distinct":counts.len(),
            "duplicate_fraction":1.-counts.len() as f64/rows as f64,
            "largest_group":counts.values().max(),
            "raw_bytes":rows*512*4,
            "dictionary_and_u32_ordinals_bytes":counts.len()*512*4+rows*4,
            "read_and_hash_ms":started.elapsed().as_secs_f64()*1000.
        })
    );
    Ok(())
}
