//! Compare numerical and source-annotation publications byte for byte.
use clap::Parser;
use libsql::Builder;
use serde_json::json;
use std::{error::Error, path::PathBuf};

#[derive(Parser)]
struct Args {
    before: PathBuf,
    after: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if !args.before.is_file() || !args.after.is_file() {
        return Err("supply two existing stores".into());
    }
    let before = Builder::new_local(&args.before).build().await?;
    let after = Builder::new_local(&args.after).build().await?;
    let a = before.connect()?;
    let b = after.connect()?;
    a.execute_batch("PRAGMA query_only=ON;").await?;
    b.execute_batch("PRAGMA query_only=ON;").await?;
    for (name, sql) in [
        ("blocks", "SELECT id,record,emb FROM vectors ORDER BY id"),
        (
            "fallback-metadata",
            "SELECT kind,model,rows,metadata FROM fallback_artifacts WHERE kind NOT LIKE 'sentences%' ORDER BY kind",
        ),
        (
            "fallback-vectors",
            "SELECT kind,part,data FROM fallback_vectors WHERE kind NOT LIKE 'sentences%' ORDER BY kind,part",
        ),
        (
            "pos",
            "SELECT block,record,body_hash,identity,data FROM pos_relations ORDER BY block",
        ),
    ] {
        let mut left = a.query(sql, ()).await?;
        let mut right = b.query(sql, ()).await?;
        let mut count = 0;
        loop {
            match (left.next().await?, right.next().await?) {
                (None, None) => break,
                (Some(l), Some(r)) => {
                    if l.column_count() != r.column_count() {
                        return Err(format!("{name}: column count differs").into());
                    }
                    for column in 0..l.column_count() {
                        if l.get_value(column)? != r.get_value(column)? {
                            return Err(
                                format!("{name}: row {count}, column {column} differs").into()
                            );
                        }
                    }
                    count += 1;
                }
                _ => return Err(format!("{name}: row count differs after {count} rows").into()),
            }
        }
        println!(
            "{}",
            json!({"population":name,"rows":count,"identical":true})
        );
    }
    let mut left = Sentences::load(&a).await?;
    let mut right = Sentences::load(&b).await?;
    if left.blocks != right.blocks || left.model != right.model {
        return Err("sentence metadata differs".into());
    }
    for index in 0..left.blocks.len() {
        if left.vector(index).await? != right.vector(index).await? {
            return Err(format!("sentence {index} differs").into());
        }
    }
    println!(
        "{}",
        json!({"population":"sentences","rows":left.blocks.len(),"identical":true})
    );
    Ok(())
}

struct Sentences {
    model: String,
    blocks: Vec<usize>,
    order: Option<Vec<usize>>,
    data: Vec<u8>,
    chunks: libsql::Rows,
    chunk_start: usize,
}
impl Sentences {
    async fn load(db: &libsql::Connection) -> Result<Self, Box<dyn Error>> {
        let mut manifest = db.query("SELECT kind,model,rows,metadata FROM fallback_artifacts WHERE kind IN ('sentences','sentences-dictionary-v1') ORDER BY kind DESC LIMIT 1", ()).await?;
        let row = manifest.next().await?.ok_or("no sentence manifest")?;
        let kind: String = row.get(0)?;
        let model = row.get(1)?;
        let rows = usize::try_from(row.get::<i64>(2)?)?;
        let metadata: serde_json::Value = serde_json::from_slice(&row.get::<Vec<u8>>(3)?)?;
        let (blocks, order): (Vec<usize>, Option<Vec<usize>>) = if kind == "sentences" {
            (serde_json::from_value(metadata)?, None)
        } else {
            (
                serde_json::from_value(metadata["blocks"].clone())?,
                Some(serde_json::from_value(metadata["order"].clone())?),
            )
        };
        let mut chunks = db
            .query(
                "SELECT data FROM fallback_vectors WHERE kind=? ORDER BY part",
                [kind],
            )
            .await?;
        let mut data = Vec::new();
        if let Some(order) = &order {
            if order.len() != blocks.len() || order.iter().any(|i| *i >= rows) {
                return Err("invalid sentence ordinals".into());
            }
            while let Some(row) = chunks.next().await? {
                data.extend(row.get::<Vec<u8>>(0)?);
            }
            if data.len() != rows.checked_mul(2048).ok_or("oversized dictionary")? {
                return Err("invalid dictionary length".into());
            }
        } else if blocks.len() != rows {
            return Err("invalid sentence row count".into());
        }
        Ok(Self {
            model,
            blocks,
            order,
            data,
            chunks,
            chunk_start: 0,
        })
    }
    async fn vector(&mut self, index: usize) -> Result<&[u8], Box<dyn Error>> {
        if let Some(order) = &self.order {
            let start = order[index] * 2048;
            return Ok(&self.data[start..start + 2048]);
        }
        let start = index * 2048;
        while start >= self.chunk_start + self.data.len() {
            self.chunk_start += self.data.len();
            self.data = self
                .chunks
                .next()
                .await?
                .ok_or("missing sentence bytes")?
                .get(0)?;
            if self.data.is_empty() || !self.data.len().is_multiple_of(2048) {
                return Err("invalid sentence chunk".into());
            }
        }
        let offset = start - self.chunk_start;
        Ok(&self.data[offset..offset + 2048])
    }
}
