use super::Command;
use crate::{
    error::{Error, Result},
    storage::sync::{Remote, Replica},
    sync,
};
use clap::Subcommand;
use std::{io, path::Path};
#[derive(Subcommand)]
pub enum RemoteCommand {
    #[command(
        about = "Save an SSH remote; sync checks the connection",
        after_help = "Origin: ssh://[user@]host[:port][/absolute/store/path]\nBoth hosts need linn on PATH. SSH keys and known hosts must already work.\nAll records are exchanged unless --scope selects an original repository.\nTombstone IDs always propagate. Pairing grants no access beyond your SSH account."
    )]
    Pair {
        name: String,
        origin: String,
        #[arg(long)]
        scope: Option<String>,
    },
    Remove {
        name: String,
    },
    List,
}
pub(super) async fn run(path: &Path, command: Command, json: bool) -> Result<i32> {
    let replica = Replica::open(path).await?;
    match command {
        Command::SyncRemote { command } => match command {
            RemoteCommand::Pair {
                name,
                origin,
                scope,
            } => {
                if name.is_empty()
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                    || scope.as_ref().is_some_and(|s| s.trim().is_empty())
                {
                    return Err(Error::Invalid("invalid remote name or empty scope".into()));
                }
                sync::ssh::Origin::parse(&origin)?;
                replica
                    .pair(&Remote {
                        name: name.clone(),
                        origin,
                        scope,
                    })
                    .await?;
                if json {
                    println!("{}", serde_json::json!({"paired":name}));
                } else {
                    println!("Paired: {name}");
                }
            }
            RemoteCommand::Remove { name } => {
                replica.remove(&name).await?;
                if json {
                    println!("{}", serde_json::json!({"removed":name}));
                } else {
                    println!("Removed: {name}");
                }
            }
            RemoteCommand::List => {
                let remotes = replica.remotes().await?;
                if json {
                    println!("{}", serde_json::json!({"remotes":remotes}));
                } else {
                    for r in remotes {
                        println!(
                            "{}\t{}{}",
                            r.name,
                            r.origin,
                            r.scope.map(|s| format!("\t{s}")).unwrap_or_default()
                        );
                    }
                }
            }
        },
        Command::SyncServe { .. } => {
            sync::protocol::serve(&replica, &mut io::stdin().lock(), &mut io::stdout().lock())
                .await?
        }
        Command::Sync => {
            let counts = sync::run(&replica).await?;
            if json {
                println!("{}", serde_json::to_string(&counts)?);
            } else {
                println!(
                    "Synced {} remotes: {} pulled, {} pushed.",
                    counts.remotes, counts.pulled, counts.pushed
                );
            }
        }
        _ => unreachable!(),
    }
    Ok(0)
}
