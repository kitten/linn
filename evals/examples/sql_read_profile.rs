//! Diagnose connection settings and query plans using Linn's pinned local libSQL.
use clap::Parser;
use libsql::Builder;
use serde_json::json;
use std::{hint::black_box, path::PathBuf, time::Instant};

#[derive(Parser)]
struct Args {
    store: PathBuf,
    #[arg(long, default_value_t = 2000)]
    cache_kib: u32,
    #[arg(long, default_value_t = 0)]
    mmap_mib: u32,
    #[arg(long, default_value_t = 3)]
    rounds: usize,
    /// Run only phases with this name prefix.
    #[arg(long)]
    only: Option<String>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    for round in 0..args.rounds {
        let started = Instant::now();
        let database = Builder::new_local(&args.store).build().await?;
        let db = database.connect()?;
        db.execute_batch(&format!("PRAGMA query_only=ON; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-{}; PRAGMA mmap_size={};", args.cache_kib, u64::from(args.mmap_mib) * 1024 * 1024)).await?;
        println!(
            "{}",
            json!({"round":round,"phase":"connect","ms":started.elapsed().as_secs_f64()*1000.})
        );
        if round == 0 {
            for pragma in [
                "cache_size",
                "mmap_size",
                "journal_mode",
                "synchronous",
                "page_size",
            ] {
                let mut rows = db.query(&format!("PRAGMA {pragma}"), ()).await?;
                println!(
                    "{}",
                    json!({"pragma":pragma,"value":format!("{:?}",rows.next().await?.unwrap().get_value(0)?)})
                );
            }
        }
        for (name, sql, columns) in [
            (
                "document-count-vocabulary",
                "SELECT count(*) FROM (SELECT DISTINCT doc FROM vocab WHERE term='configuration') v JOIN nav_blocks b ON b.id=v.doc JOIN record_read_index c ON c.record=b.record WHERE (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>'2026-11-05T00:00:00Z')",
                1,
            ),
            (
                "document-count-match",
                "SELECT count(*) FROM fts JOIN nav_blocks b ON b.id=fts.rowid JOIN record_read_index c ON c.record=b.record WHERE fts MATCH '\"configuration\"' AND (c.eligible & 1)!=0 AND (c.expires='' OR c.expires>'2026-11-05T00:00:00Z')",
                1,
            ),
            (
                "headers",
                "SELECT data,body_bytes,body_nonblank,body_hash FROM record_headers ORDER BY record",
                4,
            ),
            (
                "totals",
                "SELECT record,blocks,length FROM record_index_totals ORDER BY record",
                3,
            ),
            (
                "surface",
                "SELECT b.block,s.df FROM nav_lexemes n JOIN nav_surface s ON s.word=n.id JOIN nav_block_words b ON b.word=n.id WHERE n.surface='configuration' ORDER BY b.block",
                2,
            ),
            (
                "trees",
                "SELECT DISTINCT t.kind,t.tree,nr.record FROM nav_lexemes n JOIN nav_block_words b ON b.word=n.id JOIN nav_tree_rows t ON t.block=b.block JOIN nav_trees nr ON nr.kind=t.kind AND nr.id=t.tree WHERE n.norm='configuration' ORDER BY t.kind,t.tree",
                3,
            ),
            (
                "positions",
                "SELECT doc,col,json_group_array(offset ORDER BY offset) FROM vocab WHERE term='configuration' GROUP BY doc,col",
                3,
            ),
            (
                "candidate-block",
                "SELECT b.id,b.record,b.ordinal,b.start,b.end,d.sz,r.body_hash,b.heading,b.path FROM nav_blocks b JOIN fts_docsize d ON d.id=b.id JOIN records r ON r.id=b.record WHERE b.id IN (SELECT value FROM json_each('[1,2,3]')) ORDER BY b.id",
                9,
            ),
            (
                "matching-block-core-hash",
                "SELECT b.id,b.record,b.ordinal,b.start,b.end,d.sz,r.body_hash,b.heading,b.path FROM nav_blocks b JOIN fts_docsize d ON d.id=b.id JOIN records r ON r.id=b.record WHERE b.id IN (SELECT rowid FROM fts WHERE fts MATCH 'memory') ORDER BY b.id",
                9,
            ),
            (
                "matching-block-header-hash",
                "SELECT b.id,b.record,b.ordinal,b.start,b.end,d.sz,r.body_hash,b.heading,b.path FROM nav_blocks b JOIN fts_docsize d ON d.id=b.id JOIN record_headers r ON r.record=b.record WHERE b.id IN (SELECT rowid FROM fts WHERE fts MATCH 'memory') ORDER BY b.id",
                9,
            ),
            (
                "record-sections",
                "SELECT data FROM nav_sections WHERE record='s1-r-0001' ORDER BY ordinal",
                1,
            ),
            (
                "backlinks",
                "SELECT source FROM nav_links WHERE target='s1-r-0001'",
                1,
            ),
            (
                "basename",
                "SELECT record FROM record_basenames WHERE name='README.md' ORDER BY record",
                1,
            ),
            (
                "source-path",
                "SELECT id FROM records WHERE json_extract(CAST(envelope AS TEXT),'$.source_path')='README.md' ORDER BY id",
                1,
            ),
            (
                "pos-block",
                "SELECT data FROM pos_relations WHERE block=1 AND body_hash='' AND identity='' AND generation=X''",
                1,
            ),
            (
                "selected-vectors",
                "SELECT id,emb FROM vectors WHERE id IN (SELECT value FROM json_each('[1,2,3]')) ORDER BY id",
                2,
            ),
            ("blocks", "SELECT id,emb FROM vectors ORDER BY id", 2),
            (
                "beam",
                "SELECT data FROM fallback_vectors WHERE kind='beam' ORDER BY part",
                1,
            ),
        ] {
            if args
                .only
                .as_ref()
                .is_some_and(|prefix| !name.starts_with(prefix))
            {
                continue;
            }
            if round == 0 {
                let mut rows = db.query(&format!("EXPLAIN QUERY PLAN {sql}"), ()).await?;
                while let Some(row) = rows.next().await? {
                    println!("{}", json!({"plan":name,"detail":row.get::<String>(3)?}));
                }
            }
            let started = Instant::now();
            let mut rows = db.query(sql, ()).await?;
            let mut count = 0;
            while let Some(row) = rows.next().await? {
                for col in 0..columns {
                    black_box(row.get_value(col)?);
                }
                count += 1;
            }
            println!(
                "{}",
                json!({"round":round,"phase":name,"rows":count,"ms":started.elapsed().as_secs_f64()*1000.})
            );
        }
    }
    Ok(())
}
