//! @ref LLP 0004#runtime
mod bundle;
mod listing;
pub mod paths;
mod sync;
use crate::{
    error::{Error, Result},
    retrieval::lexical::centered,
    retrieval::query::Query,
    storage::record::{Close, Record, hash},
    storage::sections,
    storage::{Call, Card, Context, Store},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::{
    collections::HashSet,
    io::{self, Read},
    path::PathBuf,
};
#[derive(Parser)]
#[command(
    name = "linn",
    about = "Search private notes and documents. Results are pointers; open a section before relying on it.",
    after_help = "Typical use:\n  linn search '\"git send\" allowed prefixes'\n  linn show <fetch-id>\n  linn search:more <cursor>\n  linn show:outline <ref>\n\nSearches cover every repository. scope: is accepted and does not narrow this store.\nA related topic, a different package or version, or shared words is not an answer.\nNear misses are hints, not results. An honest Nothing qualified is a successful miss.\nSave reusable engineering discoveries; see linn add --help.\n\nExit codes: 0 results/success, 1 no results/not current, 2 command error, 3 damaged index."
)]
struct Args {
    #[arg(long, global = true, help = "Override the account's local store file")]
    store: Option<PathBuf>,
    #[arg(long, global = true, help = "Print stable machine-readable JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    #[command(about = "Export original records as a portable JSONL bundle")]
    Export(bundle::ExportArgs),
    #[command(about = "Merge a trusted Linn bundle, preserving IDs; deletions are permanent")]
    Import { file: String },
    #[command(
        name = "sync:remote",
        about = "Manage trusted SSH peers for this store"
    )]
    SyncRemote {
        #[command(subcommand)]
        command: sync::RemoteCommand,
    },
    #[command(
        name = "sync:serve",
        about = "Serve sync over stdin/stdout; no listener"
    )]
    SyncServe {
        #[arg(long)]
        stdio: bool,
    },
    #[command(
        about = "Pull all SSH remotes, then push the merged records and deletions",
        after_help = "Configure peers with sync:remote pair. A failed pull prevents all pushes.\nRetry interrupted syncs; use index:rebuild to repair incomplete publication."
    )]
    Sync,
    #[command(
        about = "List current records, newest first; no search terms needed",
        after_help = "Examples:\n  linn list --kind note --since 7d\n  linn list --expiring-within 30d\n\nLists metadata across repositories. Use show <record> to read a record.\nClosed, superseded, expired and unaccepted records are excluded."
    )]
    List(listing::ListArgs),
    #[command(
        about = "Find sections. Use specific words expected in the answer; keep identifiers exact.",
        after_help = r#"Add context when needed to distinguish the result.
"double quotes" keep an exact phrase or name together. Do not add the repository name.
Open results with linn show <ref>; cite the fetch id of the result you rely on.
Continue with linn search:more <cursor>, try another query, or stop.

Filters:
  path:<glob>        source path; use when the task names a file or directory
                     * stops at /; ** crosses directories
  kind:note|doc      stored notes or imported documents
  under:"heading"   heading path contains the value
  id:<ref>           the record a reference names
  writer:<email>     exact stored writer
  scope:<org/repo>   accepted; does not narrow this shared-search store

A reference is a record id, a fetch id, or a unique body-hash prefix of at least
8 hex digits. show:outline lists openable sections of that reference's document.
Use --json for cards, related pointers, near misses and the next cursor."#
    )]
    Search { query: String },
    #[command(
        name = "search:more",
        about = "Read the next results of the same search"
    )]
    More { cursor: String },
    #[command(about = "Read original bytes; accepts a record id, fetch id, or unique hash prefix")]
    Show {
        reference: String,
        #[arg(long)]
        all: bool,
    },
    #[command(
        name = "show:outline",
        about = "List a document's sections as openable pointers"
    )]
    Outline { reference: String },
    #[command(
        about = "Import a file unchanged as a document, or stdin (-) as a note. Write specific decisions, gotchas and procedures; link supporting files.",
        after_help = r#"Save useful engineering discoveries automatically: one finding, its context and
available sources per note. Write original text, not keyword hints.
Keep tasks elsewhere and enduring documentation in the repository.
Correct notes with --parent <ref> or linn add:close <ref> --reason correct.
Use --stale-after only for known expiry."#
    )]
    Add {
        file: String,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        unscoped: bool,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        stale_after: Option<String>,
        #[arg(long)]
        source_path: Option<String>,
    },
    #[command(
        name = "add:close",
        about = "Write a close envelope; leave the target bytes immutable"
    )]
    Close {
        reference: String,
        #[arg(long,value_parser=["supersede","correct","expire"])]
        reason: String,
        #[arg(long)]
        replacement: Option<String>,
    },
    #[command(
        name = "add:forget",
        about = "Forget a record with a tombstone and secure deletion"
    )]
    Forget { reference: String },
    #[command(
        name = "index:check",
        about = "Verify original bytes, database and index integrity"
    )]
    Check,
    #[command(
        name = "index:rebuild",
        about = "Rebuild derived indexes from immutable records"
    )]
    Rebuild,
}
fn now() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zero nanoseconds")
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn cursor(q: &str, scope: &Option<String>, next: usize, now: &str) -> String {
    format!(
        "s1.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&(q, scope, next, now)).unwrap())
    )
}
pub async fn run() -> Result<i32> {
    let timing = std::env::var_os("LINN_STARTUP_TIMINGS").is_some();
    let mut phase = std::time::Instant::now();
    let mut mark = |name: &str| {
        if timing {
            eprintln!(
                "linn cli: {}",
                json!({"phase":name,"ms":phase.elapsed().as_secs_f64()*1000.})
            );
        }
        phase = std::time::Instant::now();
    };
    let args = Args::parse();
    let path = paths::store_path(args.store)?;
    mark("arguments-and-path");
    if matches!(
        &args.command,
        Command::SyncRemote { .. } | Command::SyncServe { .. } | Command::Sync
    ) {
        return sync::run(&path, args.command, args.json).await;
    }
    if let Command::Import { file } = &args.command {
        return bundle::import(&path, file, args.json).await;
    }
    let opened = if matches!(&args.command, Command::Rebuild) {
        Store::open_for_rebuild(&path).await
    } else {
        Store::open(&path).await
    };
    let mut store = match opened {
        Ok(store) => store,
        Err(error) if matches!(&args.command, Command::Check) => {
            eprintln!("Index damaged: {error}");
            return Ok(3);
        }
        Err(error) => return Err(error),
    };
    mark("store-open");
    let scope = if matches!(&args.command, Command::List(_)) {
        None
    } else {
        paths::scope()
    };
    let ctx = Context {
        scope: scope.clone(),
        now: now(),
        ..Context::default()
    };
    mark("scope-and-context");
    store.set_query_clock(&ctx.now);
    let result = match args.command {
        Command::SyncRemote { .. } | Command::SyncServe { .. } | Command::Sync => unreachable!(),
        Command::Import { .. } => unreachable!(),
        Command::Export(options) => {
            bundle::export(&mut store, options, &ctx.now, scope.as_deref()).await
        }
        Command::List(options) => listing::run(&store, options, &ctx.now, args.json).await,
        Command::Search { query } => search(&mut store, &query, &ctx, 0, args.json).await,
        Command::More { cursor: token } => {
            let bytes = URL_SAFE_NO_PAD
                .decode(
                    token
                        .strip_prefix("s1.")
                        .ok_or_else(|| Error::Invalid("invalid cursor".into()))?,
                )
                .map_err(|_| Error::Invalid("invalid cursor".into()))?;
            let (q, scope, next, now): (String, Option<String>, usize, String) =
                serde_json::from_slice(&bytes)?;
            search(
                &mut store,
                &q,
                &Context { scope, now, ..ctx },
                next,
                args.json,
            )
            .await
        }
        Command::Show { reference, all } => {
            let rid = store
                .resolve_metadata(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.load_record_sections(&rid).await?;
            let lifecycle = store.lifecycle_references(&rid).await?;
            let (record, span) = store.resolve(&reference, scope.as_deref())?;
            let fetch = format!("{}:v1:{}:{}", record.body_hash(), span[0], span[1]);
            if !store.validity.eligible(record, &ctx.now) {
                let child = lifecycle.iter().map(|id| &store.records[id]).find(|r| {
                    r.state == "accepted"
                        && r.close.is_none()
                        && r.parent.as_ref() == Some(&record.id)
                });
                let close = lifecycle.iter().map(|id| &store.records[id]).find(|r| {
                    r.state == "accepted" && r.close.as_ref().is_some_and(|c| c.target == record.id)
                });
                let reason = if let Some(child) = child {
                    format!("superseded by {}", child.id)
                } else if let Some(close) = close {
                    format!("closed by {}", close.id)
                } else {
                    "expired or not accepted".into()
                };
                if args.json {
                    println!(
                        "{}",
                        json!({"fetch":fetch,"record":record.id,"label":record.label,"heading":"","text":"","truncated":false,"superseded_by":child.map(|r|&r.id),"status":"not_current"})
                    );
                } else {
                    println!("Not current: {reason}.");
                }
                return Ok(1);
            }
            let (shown, characters) = store.display_span(&rid, span, all).await?;
            let truncated = !all && characters > 1500;
            let heading = store.sections[&record.id]
                .iter()
                .find(|s| s.span.start <= span[0] && span[0] < s.span.end)
                .map(|s| s.path.join(" > "))
                .unwrap_or_default();
            if args.json {
                println!(
                    "{}",
                    json!({"fetch":fetch,"record":record.id,"label":record.label,"heading":heading,"text":shown,"truncated":truncated,"superseded_by":null})
                );
            } else {
                println!(
                    "{}{}",
                    record.label,
                    if heading.is_empty() {
                        String::new()
                    } else {
                        format!(" — {heading}")
                    }
                );
                print!("{shown}");
                if truncated {
                    println!(
                        "… ({} more characters; --all to print the rest)",
                        characters - 1500
                    );
                }
            }
            Ok(0)
        }
        Command::Outline { reference } => {
            let rid = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.load_record_sections(&rid).await?;
            let (record, _) = store.resolve(&reference, scope.as_deref())?;
            if !store.validity.eligible(record, &ctx.now) {
                println!("Nothing qualified.");
                return Ok(1);
            }
            let sections = &store.sections[&record.id];
            let top = sections
                .iter()
                .filter(|s| s.level > 0)
                .map(|s| s.level)
                .min()
                .unwrap_or(1);
            let mut cards = Vec::new();
            for sec in sections
                .iter()
                .filter(|s| s.level <= top + 1 && !s.span.is_empty())
                .take(20)
            {
                let span = if sec.span.len() > 1500 {
                    sections::hint_spans(record.body.as_bytes(), sec.span.clone())[0].clone()
                } else {
                    sec.span.clone()
                };
                let text = String::from_utf8_lossy(&record.body.as_bytes()[span.clone()]);
                let excerpt = text
                    .lines()
                    .find(|s| !s.trim().is_empty() && !s.trim_start().starts_with('#'))
                    .unwrap_or("")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(160)
                    .collect();
                cards.push(Card {
                    record: record.id.clone(),
                    heading: sec.path.join(" > "),
                    span: [span.start, span.end],
                    text: excerpt,
                    fetch: Some(format!(
                        "{}:v1:{}:{}",
                        record.body_hash(),
                        span.start,
                        span.end
                    )),
                    ..Card::default()
                });
            }
            if args.json {
                println!("{}", json!({"cards":cards}));
            } else {
                for (i, c) in cards.iter().enumerate() {
                    println!(
                        "[o{}] {}\n    {}\n    fetch {}",
                        i + 1,
                        if c.heading.is_empty() {
                            "(document start)"
                        } else {
                            &c.heading
                        },
                        c.text,
                        c.fetch.as_deref().unwrap()
                    );
                }
            }
            Ok(if cards.is_empty() { 1 } else { 0 })
        }
        Command::Add {
            file,
            label,
            unscoped,
            parent,
            stale_after,
            source_path,
        } => {
            let mut body = String::new();
            let (kind, path) = if file == "-" {
                io::stdin().read_to_string(&mut body)?;
                ("note", source_path)
            } else {
                if source_path.is_some() {
                    return Err(Error::Invalid("--source-path is only for notes".into()));
                }
                body = std::fs::read_to_string(&file)?;
                ("doc", Some(paths::source_path(&PathBuf::from(&file))?))
            };
            let parent = match parent {
                Some(p) => Some(
                    store
                        .resolve_loaded(&p, scope.as_deref())
                        .await?
                        .0
                        .id
                        .clone(),
                ),
                None => None,
            };
            let label = label.unwrap_or_else(|| {
                if kind == "doc" {
                    let sections = sections::parse_sections(&body);
                    let level = sections
                        .iter()
                        .filter(|s| s.level > 0)
                        .map(|s| s.level)
                        .min()
                        .unwrap_or(1);
                    let top = sections
                        .iter()
                        .filter(|s| s.level == level)
                        .collect::<Vec<_>>();
                    if top.len() == 1 {
                        return top[0].heading.clone();
                    }
                    PathBuf::from(&file)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into()
                } else {
                    body.lines().next().unwrap_or("").chars().take(80).collect()
                }
            });
            let stamp = now();
            let id = format!(
                "{}-{}",
                &hash(body.as_bytes())[..16],
                time::OffsetDateTime::now_utc().unix_timestamp_nanos()
            );
            let source_ref = if kind == "doc" {
                Some(paths::evidence(&PathBuf::from(&file))?)
            } else {
                None
            };
            let link = regex::Regex::new(r"\]\(linn:([^)]+)\)").unwrap();
            for captures in link.captures_iter(&body) {
                if store
                    .resolve_loaded(&captures[1], scope.as_deref())
                    .await
                    .is_err()
                {
                    eprintln!("Unresolved link: linn:{}", &captures[1]);
                }
            }
            let record = Record {
                id: id.clone(),
                body,
                scope: if unscoped { None } else { scope },
                state: "accepted".into(),
                kind: kind.into(),
                label,
                source_path: path,
                parent,
                close: None,
                stale_after,
                writer: Some(paths::writer_email()),
                source: json!({"type":if kind=="doc"{"repo-file"}else{"derived-from-thread"},"ref":source_ref}),
                created: stamp,
                stored_body: None,
                projected_eligible: None,
            };
            store.add(record).await?;
            if args.json {
                println!("{}", json!({"record":id}));
            } else {
                println!("{id}");
            }
            Ok(0)
        }
        Command::Close {
            reference,
            reason,
            replacement,
        } => {
            let target = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            let replacement = match replacement {
                Some(p) => Some(
                    store
                        .resolve_loaded(&p, scope.as_deref())
                        .await?
                        .0
                        .id
                        .clone(),
                ),
                None => None,
            };
            let id = format!(
                "close-{}",
                time::OffsetDateTime::now_utc().unix_timestamp_nanos()
            );
            let r: Record = serde_json::from_value(
                json!({"id":id,"body":"","kind":"note","state":"accepted","writer":paths::writer_email(),"created":now(),"scope":scope,"close":Close{target,reason,replacement}}),
            )?;
            store.add(r).await?;
            if args.json {
                println!("{}", json!({"record":id}));
            } else {
                println!("{id}");
            }
            Ok(0)
        }
        Command::Forget { reference } => {
            let id = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.forget(&id).await?;
            if args.json {
                println!("{}", json!({"forgotten":id}));
            } else {
                println!("Forgotten: {id}");
            }
            Ok(0)
        }
        Command::Check => match store.integrity().await {
            Ok(()) => {
                if args.json {
                    println!("{}", json!({"integrity":"ok"}));
                } else {
                    println!("Index integrity: ok");
                }
                Ok(0)
            }
            Err(e) => {
                eprintln!("Index damaged: {e}");
                Ok(3)
            }
        },
        Command::Rebuild => {
            store.rebuild().await?;
            if args.json {
                println!("{}", json!({"rebuilt":true}));
            } else {
                println!("Index rebuilt.");
            }
            Ok(0)
        }
    };
    mark("command-and-output");
    result
}
async fn search(
    store: &mut Store,
    q: &str,
    ctx: &Context,
    start: usize,
    json_output: bool,
) -> Result<i32> {
    let parsed = Query::parse(q).map_err(Error::Invalid)?;
    if parsed
        .filters
        .iter()
        .any(|(k, _)| !["scope", "under", "path", "kind", "id", "writer"].contains(&k.as_str()))
    {
        return Err(Error::Invalid("Query error: Unknown prefix".into()));
    }
    let original_query = q;
    let mut replacements = std::collections::HashMap::new();
    for (k, v) in &parsed.filters {
        if k == "id" {
            let rid = store
                .resolve_loaded(v, ctx.scope.as_deref())
                .await?
                .0
                .id
                .clone();
            if rid.contains('"') {
                return Err(Error::Invalid("record ID contains a quote and cannot be used as a query filter; use show instead".into()));
            }
            replacements.insert((k.clone(), v.clone()), rid);
        }
    }
    let q = crate::retrieval::query::rewrite_filter_values(original_query, &replacements);
    let parsed = Query::parse(&q).map_err(Error::Invalid)?;
    let mut calls = store.agent(&q, ctx).await?;
    let pointers = calls
        .iter()
        .flat_map(|call| call.hints.iter().chain(&call.related))
        .map(|p| (p.record.clone(), p.span[0]))
        .collect::<Vec<_>>();
    store.load_pointer_blocks(&pointers).await?;
    let mut seen = HashSet::new();
    for call in &mut calls {
        call.cards.retain(|c| {
            let yes = store
                .rows
                .get(&c.section)
                .is_some_and(|b| store.allowed(b, &parsed.filters));
            yes && seen.insert((c.record.clone(), c.span))
        });
        call.hints.retain(|p| pointer_allowed(store, p, &parsed));
        call.related.retain(|p| pointer_allowed(store, p, &parsed));
    }
    let selected =
        (start..calls.len()).find(|i| !calls[*i].cards.is_empty() || (*i == 0 && start == 0));
    let Some(i) = selected else {
        if json_output {
            println!(
                "{}",
                json!({"status":"nothing_qualified","cards":[],"more":false})
            );
        } else {
            println!("No more results for this search.");
        }
        return Ok(1);
    };
    let call = &calls[i];
    let next = (i + 1..calls.len()).find(|j| !calls[*j].cards.is_empty());
    let token = next.map(|n| cursor(original_query, &ctx.scope, n, &ctx.now));
    if json_output {
        let cards:Vec<_>=call.cards.iter().enumerate().map(|(i,c)|{let r=&store.records[&c.record];json!({"n":i+1,"record":c.record,"label":r.label,"heading":c.heading,"excerpt":c.text,"fetch":format!("{}:v1:{}:{}",&r.body_hash()[..12],c.span[0],c.span[1]),"labels":c.labels})}).collect();
        println!(
            "{}",
            json!({"cursor":token,"call":i+1,"status":if cards.is_empty(){"nothing_qualified"}else{"results"},"cards":cards,"related":pointer_json(store,&call.related),"hints":pointer_json(store,&call.hints),"more":next.is_some()})
        );
    } else {
        print_call(store, call);
        if let Some(token) = token {
            println!("more: linn search:more {token}");
        } else {
            println!("No more results for this search.");
        }
    }
    Ok(if call.cards.is_empty() { 1 } else { 0 })
}
fn pointer_allowed(store: &Store, p: &crate::storage::Pointer, q: &Query) -> bool {
    store
        .record_blocks(&p.record)
        .find(|b| b.start <= p.span[0] && p.span[0] < b.end)
        .is_some_and(|b| store.allowed(b, &q.filters))
}
fn print_call(store: &Store, call: &Call) {
    if call.cards.is_empty() {
        println!("Nothing qualified.");
    }
    for (i, c) in call.cards.iter().enumerate() {
        let r = &store.records[&c.record];
        println!(
            "[{}] {}{}{}\n    {}\n    fetch {}:v1:{}:{}",
            i + 1,
            r.label,
            if r.kind == "doc" { " (doc)" } else { "" },
            if c.heading.is_empty() {
                String::new()
            } else {
                format!(" — {}", c.heading)
            },
            c.text,
            &r.body_hash()[..12],
            c.span[0],
            c.span[1]
        );
    }
    for (title, prefix, list) in [
        ("Related (heading match, not results):", "r", &call.related),
        ("Near misses (not results):", "h", &call.hints),
    ] {
        if !list.is_empty() {
            println!("{title}");
            for (i, p) in list.iter().enumerate() {
                let r = &store.records[&p.record];
                println!(
                    "[{prefix}{}] {}{}{}\n    {}{}\n    fetch {}:v1:{}:{}",
                    i + 1,
                    p.label,
                    if r.kind == "doc" { " (doc)" } else { "" },
                    if p.heading.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", p.heading)
                    },
                    if prefix == "h" {
                        format!(
                            "matched: {}; missing: {}\n    ",
                            p.matched.join(", "),
                            if p.missing.is_empty() {
                                "-".into()
                            } else {
                                p.missing.join(", ")
                            }
                        )
                    } else {
                        String::new()
                    },
                    centered(&p.excerpt, &[]),
                    &r.body_hash()[..12],
                    p.span[0],
                    p.span[1]
                );
            }
        }
    }
}

fn pointer_json(store: &Store, pointers: &[crate::storage::Pointer]) -> Vec<serde_json::Value> {
    pointers.iter().enumerate().map(|(i,p)|{let r=&store.records[&p.record];json!({"n":i+1,"record":p.record,"label":p.label,"heading":p.heading,"excerpt":p.excerpt,"fetch":format!("{}:v1:{}:{}",&r.body_hash()[..12],p.span[0],p.span[1]),"matched":p.matched,"missing":p.missing})}).collect()
}
