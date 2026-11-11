//! Measure store opening or isolate native libSQL row iteration and extraction.
use clap::Parser;
use libsql::Builder;
use linn::store::{Context, Store};
use serde_json::json;
use std::{
    hint::black_box,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Parser)]
struct Args {
    store: PathBuf,
    /// Read database rows without constructing any search structures.
    #[arg(long)]
    raw: bool,
    /// Probe materialized-view syntax in a fresh in-memory database, never the supplied store.
    #[arg(long, conflicts_with_all=["raw","query","cache_kib","mmap_mib"])]
    probe_materialized_views: bool,
    /// Report persisted numerical-array sizes without loading their payloads.
    #[arg(long, conflicts_with_all=["raw","query","probe_materialized_views","cache_kib","mmap_mib"])]
    artifact_stats: bool,
    /// Connection-local page cache budget for the raw diagnostic.
    #[arg(long, requires = "raw")]
    cache_kib: Option<u64>,
    /// Connection-local memory mapping limit for the raw diagnostic.
    #[arg(long, requires = "raw")]
    mmap_mib: Option<u64>,
    /// Include one real agent invocation after store opening.
    #[arg(long, conflicts_with = "raw")]
    query: Option<String>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.probe_materialized_views {
        let database = Builder::new_local(":memory:").build().await?;
        let db = database.connect()?;
        let mut version = db.query("SELECT sqlite_version()", ()).await?;
        let version = version.next().await?.unwrap().get::<String>(0)?;
        db.execute_batch("CREATE TABLE base(id INTEGER PRIMARY KEY,value INTEGER); INSERT INTO base VALUES(1,7); CREATE VIEW ordinary AS SELECT sum(value) AS total FROM base;").await?;
        db.execute_batch("CREATE TABLE snapshot AS SELECT sum(value) AS total FROM base; CREATE TABLE maintained(total INTEGER); INSERT INTO maintained VALUES(7); CREATE TRIGGER maintain_sum AFTER INSERT ON base BEGIN UPDATE maintained SET total=total+new.value; END; INSERT INTO base VALUES(2,11);").await?;
        let mut values=db.query("WITH current AS MATERIALIZED (SELECT sum(value) AS total FROM base) SELECT (SELECT total FROM ordinary),(SELECT total FROM snapshot),(SELECT total FROM maintained),(SELECT total FROM current)",()).await?;
        let values = values.next().await?.unwrap();
        let result = db
            .execute(
                "CREATE MATERIALIZED VIEW cached AS SELECT sum(value) AS total FROM base",
                (),
            )
            .await;
        println!(
            "{}",
            json!({"sqlite_version":version,"ordinary_view_after_insert":values.get::<i64>(0)?,"create_table_as_snapshot_after_insert":values.get::<i64>(1)?,"trigger_maintained_after_insert":values.get::<i64>(2)?,"materialized_cte_after_insert":values.get::<i64>(3)?,"create_materialized_view":match result {Ok(_)=>"supported".to_owned(),Err(e)=>e.to_string()}})
        );
        return Ok(());
    }
    let start = Instant::now();
    if args.artifact_stats {
        let database = Builder::new_local(&args.store).build().await?;
        let db = database.connect()?;
        let mut result=db.query("SELECT a.kind,a.rows,count(v.part),coalesce(sum(length(v.data)),0) FROM fallback_artifacts a LEFT JOIN fallback_vectors v ON v.kind=a.kind GROUP BY a.kind,a.rows ORDER BY a.kind",()).await?;
        while let Some(row) = result.next().await? {
            println!(
                "{}",
                json!({"kind":row.get::<String>(0)?,"rows":row.get::<i64>(1)?,"parts":row.get::<i64>(2)?,"bytes":row.get::<i64>(3)?})
            );
        }
        return Ok(());
    }
    if !args.raw {
        let mut store = Store::open(&args.store).await?;
        println!(
            "{}",
            json!({"phase":"store-open","ms":start.elapsed().as_secs_f64()*1000.,"records":store.record_count(),"resident_headers":store.records.len(),"blocks":store.block_count()})
        );
        if let Some(query) = args.query {
            let start = Instant::now();
            let calls = store.agent(&query, &Context::default()).await?;
            println!(
                "{}",
                json!({"phase":"agent","ms":start.elapsed().as_secs_f64()*1000.,"calls":calls})
            );
        }
        return Ok(());
    }
    let database = Builder::new_local(&args.store).build().await?;
    let db = database.connect()?;
    db.execute_batch("PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000; PRAGMA temp_store=MEMORY;").await?;
    if let Some(kib) = args.cache_kib {
        db.execute_batch(&format!("PRAGMA cache_size=-{kib}"))
            .await?;
    }
    if let Some(mib) = args.mmap_mib {
        db.execute_batch(&format!("PRAGMA mmap_size={}", mib * 1024 * 1024))
            .await?;
    }
    for name in ["cache_size", "mmap_size", "page_size", "journal_mode"] {
        let mut result = db.query(&format!("PRAGMA {name}"), ()).await?;
        let value = result.next().await?.unwrap().get_value(0)?;
        eprintln!("{name}={value:?}");
    }
    for (phase, sql, columns) in [
        ("records", "SELECT envelope FROM records ORDER BY id", 1),
        (
            "sections",
            "SELECT id,record,start,end FROM sections ORDER BY id",
            4,
        ),
        ("vectors", "SELECT id,emb FROM vectors ORDER BY id", 2),
        ("fts-instances", "SELECT term,doc,col,offset FROM vocab", 4),
        (
            "stem-instances",
            "SELECT term,doc,offset FROM stem_vocab",
            3,
        ),
    ] {
        let start = Instant::now();
        let mut result = db.query(sql, ()).await?;
        let prepare = start.elapsed();
        let mut count = 0;
        let mut next = Duration::ZERO;
        let mut extract = Duration::ZERO;
        loop {
            let step = Instant::now();
            let row = result.next().await?;
            next += step.elapsed();
            let Some(row) = row else { break };
            let step = Instant::now();
            for col in 0..columns {
                black_box(row.get_value(col)?);
            }
            extract += step.elapsed();
            count += 1;
        }
        println!(
            "{}",
            json!({"phase":phase,"rows":count,"prepare_ms":prepare.as_secs_f64()*1000.,"next_ms":next.as_secs_f64()*1000.,"extract_ms":extract.as_secs_f64()*1000.,"total_ms":start.elapsed().as_secs_f64()*1000.})
        );
    }
    Ok(())
}
