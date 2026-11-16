//! @ref LLP 0004#runtime
mod bundle;
mod listing;
mod mesh;
pub mod paths;
use crate::{
    error::{Error, Result},
    retrieval::lexical::centered,
    storage::Call,
};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use serde_json::json;
use std::{
    io::{self},
    path::PathBuf,
};
#[derive(Parser)]
#[command(
    name = "linn",
    about = "Search private notes and documents. Results are pointers; open a section before relying on it.",
    after_help = "Typical use:\n  linn search '\"git send\" allowed prefixes'\n  linn show <fetch-id>\n  linn search:more <cursor>\n  linn show:outline <ref>\n\nSearches cover every repository by default.\nA related topic, a different package or version, or shared words is not an answer.\nNear misses are hints, not results. An honest Nothing qualified is a successful miss.\nSave reusable engineering discoveries; see linn add --help.\n\nExit codes: 0 results/success, 1 no results/not current, 2 command error, 3 damaged index."
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
    #[command(
        name = "sync:pair",
        about = "Create or redeem a whole-store mesh invitation",
        after_help = "Service relays cannot read your private notes or data: transfers are end-to-end encrypted between your devices.\n\nInvitations authorize merging both entire stores and their meshes, including offline members.\nUse --join with a token, piped stdin, or a terminal prompt. Admission is independent of the optional first sync. Use --sync or --no-sync to choose explicitly."
    )]
    SyncPair(mesh::PairArgs),
    #[command(
        name = "sync:status",
        hide = true,
        about = "Inspect local mesh enrollment without contacting peers"
    )]
    SyncStatus {
        #[arg(
            long,
            help = "Check peer transport reachability (at most ten seconds; no record exchange)"
        )]
        check: bool,
    },
    #[command(
        name = "sync:revoke",
        hide = true,
        about = "Permanently revoke a peer by its full ID"
    )]
    SyncRevoke { peer: String },
    #[command(
        name = "sync:leave",
        hide = true,
        about = "Disable local transport and revoke this endpoint when reachable"
    )]
    SyncLeave,
    #[command(
        name = "sync:reset-identity",
        hide = true,
        about = "Reset a copied store's identity; preserve records and deletions"
    )]
    SyncReset,
    #[command(
        name = "daemon:status",
        hide = true,
        about = "Inspect this store’s background process"
    )]
    DaemonStatus,
    #[command(
        name = "daemon:stop",
        hide = true,
        about = "Stop this store’s background process after pending work"
    )]
    DaemonStop,
    #[command(about = "Export original records as a portable JSONL bundle")]
    Export(bundle::ExportArgs),
    #[command(about = "Merge a trusted Linn bundle, preserving IDs; deletions are permanent")]
    Import { file: String },
    #[command(
        about = "Collect and distribute records and deletions through the paired mesh",
        after_help = "Sync administration (existing colon-separated commands):\n  linn sync:pair                  Create or redeem an invitation (-y confirms the notice)\n  linn sync:status [--check]      Inspect enrollment; optionally check peer transport reachability\n  linn sync:revoke <peer-id>      Permanently revoke a peer by its full ID\n  linn sync:leave                 Disable local transport and leave the mesh\n  linn sync:reset-identity        Remove a copied store's identity; preserve records and deletions\n\nPeer IDs identify store instances, not machines. Use each command with -h for details.\nEach invocation runs one finite mesh job; automatic sync is deferred.\nPartial progress is durable. Retry interrupted jobs; use index:rebuild to repair incomplete publication."
    )]
    Sync,
    #[command(
        about = "List current records, newest first; no search terms needed",
        after_help = "Examples:\n  linn list --kind note --since 7d\n  linn list --scope\n  linn list --scope owner/repo\n  linn list --expiring-within 30d\n\nLists metadata across repositories. Use show <record> to read a record.\nClosed, superseded, expired and unaccepted records are excluded."
    )]
    List(listing::ListArgs),
    #[command(
        about = "Find sections in notes and documents",
        after_help = r#"Use specific words expected in the answer; keep identifiers exact.
Add context when needed to distinguish the result.
"double quotes" keep an exact phrase or name together. Do not add the repository name.
Searches cover all repositories by default. Open results with linn show <ref> before relying
on them. Related pointers and near misses are hints; no qualifying result is OK.
Continue with linn search:more <cursor>, try another query, or stop.

Filters:
  path:<glob>        source path; use when the task names a file or directory
                     * stops at /; ** crosses directories
  kind:note|doc      stored notes or imported documents
  under:"heading"   heading path contains the value
  id:<ref>           the record a reference names
  writer:<email>     exact stored writer
  scope:<org/repo>   exact repository scope (case-insensitive)

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
        about = "Import a file unchanged as a document, or stdin (-) as a note.",
        after_help = r#"Save useful engineering discoveries automatically: one finding, its context and
available sources per note. Save new discoveries, not retrieved knowledge or its
summaries. Write original text, not keyword hints.
Keep tasks elsewhere and enduring documentation in the repository.
Correct notes with --parent <ref> or linn add:close <ref> --reason correct.
Use --stale-after only for known expiry."#
    )]
    Add {
        file: PathBuf,
        #[arg(long)]
        label: Option<String>,
        #[arg(long, help = "Omit repository scope from this record")]
        unscoped: bool,
        #[arg(long, help = "Correct an existing record with a new child")]
        parent: Option<String>,
        #[arg(long)]
        stale_after: Option<String>,
        #[arg(long, help = "Source path for a stdin note; derived for files")]
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
        hide = true,
        about = "Verify original bytes, database and index integrity"
    )]
    Check,
    #[command(
        name = "index:rebuild",
        hide = true,
        about = "Rebuild derived indexes from immutable records"
    )]
    Rebuild,
}

async fn enrollment(path: &std::path::Path) -> Result<Option<crate::mesh::pairing::Status>> {
    if !path.exists() {
        return Ok(None);
    }
    let reply = crate::runtime::execute(
        path,
        crate::application::Operation::Mesh(crate::mesh::pairing::Command::Status),
        crate::application::Caller {
            scope: None,
            writer: None,
            diagnostics: Default::default(),
        },
        None,
    )
    .await?;
    match reply.value {
        crate::application::Value::Mesh(crate::mesh::pairing::Output::Status(status)) => {
            Ok(Some(status))
        }
        _ => Err(Error::Invalid("unexpected enrollment response".into())),
    }
}
async fn parse_args() -> Result<Args> {
    let raw: Vec<_> = std::env::args_os().collect();
    let command = Args::command();
    match command.clone().try_get_matches_from(&raw) {
        Ok(matches) => Ok(Args::from_arg_matches(&matches).expect("validated CLI arguments")),
        Err(error) => {
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ) {
                let mut probe = raw.clone();
                let help_index = probe
                    .iter()
                    .rposition(|a| a == "--help" || a == "-h" || a == "help");
                if help_index.is_none() {
                    probe.push("sync:status".into());
                }
                if let Some(index) = help_index {
                    probe[index] = "sync:status".into();
                }
                if let Ok(args) = Args::try_parse_from(probe) {
                    let enrolled = if let Ok(path) = paths::store_path(args.store) {
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(2),
                            enrollment(&path),
                        )
                        .await
                        {
                            Ok(Ok(None)) => Some(false),
                            Ok(Ok(Some(status))) => Some(status.head.is_some()),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let mut command =
                        command.mut_subcommand("sync:pair", |c| c.hide(enrolled != Some(false)));
                    command.print_help()?;
                    println!();
                    std::process::exit(error.exit_code());
                }
            }
            error.exit()
        }
    }
}

pub async fn run() -> Result<i32> {
    use crate::application::{AddOptions, Caller, Operation as Op};
    let args = parse_args().await?;
    let path = paths::store_path(args.store)?;
    if matches!(args.command, Command::DaemonStatus | Command::DaemonStop) {
        let state =
            crate::runtime::control(&path, matches!(args.command, Command::DaemonStop)).await?;
        if args.json {
            println!("{}", serde_json::to_value(&state)?);
        } else {
            println!("Store: {}", path.display());
            match state.pid.filter(|_| state.running) {
                Some(pid) => println!("Background process: running (PID {pid})."),
                None => println!(
                    "Background process: {}.",
                    if state.running { "running" } else { "stopped" }
                ),
            }
        }
        return Ok(0);
    }
    let caller = Caller {
        diagnostics: crate::diagnostics::Diagnostics::from_environment(),
        scope: if matches!(
            &args.command,
            Command::Search { .. }
                | Command::Show { .. }
                | Command::Outline { .. }
                | Command::Add { .. }
                | Command::Close { .. }
                | Command::Forget { .. }
                | Command::Export(_)
        ) {
            paths::scope()
        } else {
            None
        },
        writer: if matches!(&args.command, Command::Add { .. } | Command::Close { .. }) {
            paths::writer_email()
        } else {
            None
        },
    };
    if let Command::SyncPair(options) = &args.command {
        let enrolled = if options.needs_confirmation() && path.exists() {
            enrollment(&path).await?.is_some_and(|s| s.head.is_some())
        } else {
            false
        };
        options.confirm(enrolled)?;
    }
    let mesh_context = match &args.command {
        Command::SyncRevoke { peer } => Some(format!("Revoked peer ID: {peer}")),
        Command::SyncPair(options) => options.cancel_context(),
        _ => None,
    };
    let first_sync = if let Command::SyncPair(options) = &args.command {
        options.first_sync()
    } else {
        Some(false)
    };
    let mut input = None;
    let mut output_path = None;
    let check = matches!(args.command, Command::Check);
    let op = match args.command {
        Command::Search { query } => Op::Search { query },
        Command::More { cursor } => Op::More { cursor },
        Command::Show { reference, all } => Op::Show { reference, all },
        Command::Outline { reference } => Op::Outline { reference },
        Command::Close {
            reference,
            reason,
            replacement,
        } => Op::Close {
            reference,
            reason,
            replacement,
        },
        Command::Forget { reference } => Op::Forget { reference },
        Command::Check => Op::Check,
        Command::Rebuild => Op::Rebuild,
        Command::SyncPair(options) => Op::Mesh(options.resolve()?),
        Command::SyncStatus { check } => Op::Mesh(if check {
            crate::mesh::pairing::Command::CheckStatus
        } else {
            crate::mesh::pairing::Command::Status
        }),
        Command::SyncRevoke { peer } => Op::Mesh(crate::mesh::pairing::Command::Revoke { peer }),
        Command::SyncLeave => Op::Mesh(crate::mesh::pairing::Command::Leave),
        Command::SyncReset => Op::Mesh(crate::mesh::pairing::Command::Reset),
        Command::Sync => Op::Mesh(crate::mesh::pairing::Command::Sync),
        Command::List(options) => Op::List(options.resolve()?),
        Command::Export(options) => {
            let (op, path) = options.resolve();
            output_path = path;
            Op::Export(op)
        }
        Command::Import { file } => {
            let mut bytes = tempfile::tempfile()?;
            if file == "-" {
                io::copy(&mut io::stdin().lock(), &mut bytes)?;
            } else {
                io::copy(&mut std::fs::File::open(file)?, &mut bytes)?;
            }
            input = Some(bytes);
            Op::Import
        }
        Command::Add {
            file,
            label,
            unscoped,
            parent,
            stale_after,
            source_path,
        } => {
            let mut bytes = tempfile::tempfile()?;
            let task = if file == std::path::Path::new("-") {
                io::copy(&mut io::stdin().lock(), &mut bytes)?;
                crate::application::InputTask::Note { source_path }
            } else {
                if source_path.is_some() {
                    return Err(Error::Invalid("--source-path is only for notes".into()));
                }
                io::copy(&mut std::fs::File::open(&file)?, &mut bytes)?;
                crate::application::InputTask::File(paths::file_input(&file)?)
            };
            input = Some(bytes);
            Op::Add(AddOptions {
                label,
                input_name: file.file_name().map(|s| s.to_string_lossy().into_owned()),
                input: task,
                parent,
                stale_after,
                unscoped,
            })
        }
        _ => unreachable!(),
    };
    use std::io::{Seek, SeekFrom};
    if let Some(input) = &mut input {
        input.seek(SeekFrom::Start(0))?;
    }
    let mut progress = mesh::ProgressRenderer::default();
    let reply = match crate::runtime::execute_with_progress(&path, op, caller, input, |event| {
        progress.render(event)
    })
    .await
    {
        Ok(r) => r,
        Err(e)
            if check
                && !matches!(&e, Error::Application(error)
            if matches!(error.code, crate::error::ErrorCode::Schema)) =>
        {
            eprintln!("Index damaged: {e}");
            return Ok(3);
        }
        Err(e) => {
            progress.finish();
            if let Error::Application(error) = &e {
                for line in &error.diagnostics {
                    eprintln!("{line}");
                }
            }
            return Err(e);
        }
    };
    progress.finish();
    if !args.json
        && matches!(
            &reply.value,
            crate::application::Value::Mesh(crate::mesh::pairing::Output::Changed { .. })
        )
        && let Some(context) = mesh_context
    {
        println!("{context}");
    }
    for warning in reply.warnings {
        eprintln!("{warning}");
    }
    if let Some(mut output) = reply.output {
        bundle::write_output(&mut output, output_path)?;
    }
    let joined = matches!(
        &reply.value,
        crate::application::Value::Mesh(crate::mesh::pairing::Output::Changed {
            operation: crate::mesh::pairing::Change::Paired,
            ..
        })
    );
    if joined && first_sync.is_none() && !args.json && mesh::offer_sync()? {
        render(reply.value, false)?;
        let reply = crate::runtime::execute_with_progress(
            &path,
            Op::Mesh(crate::mesh::pairing::Command::Sync),
            Caller {
                scope: None,
                writer: None,
                diagnostics: Default::default(),
            },
            None,
            |event| progress.render(event),
        )
        .await?;
        progress.finish();
        return render(reply.value, false);
    }
    render(reply.value, args.json)
}
fn render(value: crate::application::Value, json_output: bool) -> Result<i32> {
    use crate::application::Value;
    match value {
        Value::Search(page) => {
            let Some(call) = &page.call else {
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
            if json_output {
                let cards:Vec<_>=call.cards.iter().enumerate().map(|(i,c)|{let r=&page.records[&c.record];json!({"n":i+1,"record":c.record,"label":r.label,"heading":c.heading,"excerpt":c.text,"fetch":c.fetch,"labels":c.labels})}).collect();
                println!(
                    "{}",
                    json!({"cursor":page.cursor,"call":page.number,"status":if cards.is_empty(){"nothing_qualified"}else{"results"},"cards":cards,"related":pointer_json(&call.related),"hints":pointer_json(&call.hints),"more":page.more})
                );
            } else {
                print_call(&page, call);
                if let Some(token) = page.cursor {
                    println!("more: linn search:more {token}");
                } else {
                    println!("No more results for this search.");
                }
            }
            return Ok(if call.cards.is_empty() { 1 } else { 0 });
        }
        Value::Show(s) => {
            if json_output {
                let mut value = json!({"fetch":s.fetch,"record":s.record,"label":s.label,"heading":s.heading,"text":s.text,"truncated":s.truncated,"superseded_by":s.superseded_by});
                if s.reason.is_some() {
                    value["status"] = json!("not_current");
                }
                println!("{value}");
            } else if let Some(reason) = &s.reason {
                println!("Not current: {reason}.");
            } else {
                println!(
                    "{}{}",
                    s.label,
                    if s.heading.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", s.heading)
                    }
                );
                print!("{}", s.text);
                if s.truncated {
                    println!(
                        "… ({} more characters; --all to print the rest)",
                        s.characters - 1500
                    );
                }
            }
            return Ok(if s.reason.is_some() { 1 } else { 0 });
        }
        Value::OutlineNotCurrent => {
            if json_output {
                println!("{}", json!({"status":"not_current","entries":[]}));
            } else {
                println!("Outline unavailable: the record is not current.");
            }
            return Ok(1);
        }
        Value::Outline(cards) => {
            if json_output {
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
            return Ok(if cards.is_empty() { 1 } else { 0 });
        }
        Value::List { records, next } => {
            let empty = records.is_empty();
            if json_output {
                println!("{}", json!({"records":records,"next":next}));
            } else {
                for r in records {
                    println!(
                        "{}  {}  created: {}  expires: {}\n  {}",
                        r.record,
                        r.kind,
                        if r.created.is_empty() {
                            "unknown"
                        } else {
                            &r.created
                        },
                        r.stale_after.as_deref().unwrap_or("never"),
                        r.label.replace(['\n', '\r', '\t'], " ")
                    );
                }
                if empty {
                    println!("No records.");
                }
                if let Some(next) = next {
                    println!("Next: linn list --cursor {next}");
                }
            }
            return Ok(if empty { 1 } else { 0 });
        }
        Value::Added(id) => {
            if json_output {
                println!("{}", json!({"record":id}));
            } else {
                println!("{id}");
            }
        }
        Value::Forgotten(id) => {
            if json_output {
                println!("{}", json!({"forgotten":id}));
            } else {
                println!("Forgotten: {id}");
            }
        }
        Value::Checked => {
            if json_output {
                println!("{}", json!({"integrity":"ok"}));
            } else {
                println!("Index integrity: ok");
            }
        }
        Value::Rebuilt => {
            if json_output {
                println!("{}", json!({"rebuilt":true}));
            } else {
                println!("Index rebuilt.");
            }
        }
        Value::Exported => {}
        Value::Imported(c) => {
            if json_output {
                println!("{}", serde_json::to_string(&c)?);
            } else {
                println!(
                    "Import complete: {} record/deletion changes applied; {} items already present.",
                    c.changed, c.unchanged
                );
            }
        }
        Value::Mesh(output) => {
            let incomplete = !output.complete();
            mesh::render(output, json_output)?;
            if incomplete {
                if json_output {
                    eprintln!(
                        "Sync incomplete; accepted partial progress is durable. Inspect the JSON report for peer failures and retry with linn sync."
                    );
                }
                return Ok(2);
            }
        }
    };
    Ok(0)
}
fn print_call(store: &crate::application::SearchPage, call: &Call) {
    if call.cards.is_empty() {
        println!("Nothing qualified.");
    }
    for (i, c) in call.cards.iter().enumerate() {
        let r = &store.records[&c.record];
        println!(
            "[{}] {}{}{}\n    {}\n    fetch {}",
            i + 1,
            r.label,
            if r.kind == "doc" { " (doc)" } else { "" },
            if c.heading.is_empty() {
                String::new()
            } else {
                format!(" — {}", c.heading)
            },
            c.text,
            c.fetch.as_deref().expect("application fetch reference")
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
                    "[{prefix}{}] {}{}{}\n    {}{}\n    fetch {}",
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
                    p.fetch.as_deref().expect("application fetch reference")
                );
            }
        }
    }
}

fn pointer_json(pointers: &[crate::storage::Pointer]) -> Vec<serde_json::Value> {
    pointers.iter().enumerate().map(|(i,p)|{json!({"n":i+1,"record":p.record,"label":p.label,"heading":p.heading,"excerpt":p.excerpt,"fetch":p.fetch,"matched":p.matched,"missing":p.missing})}).collect()
}
