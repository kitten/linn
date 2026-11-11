//! @ref LLP 0006#commands
pub mod protocol;
pub mod ssh;
use crate::{
    error::{Error, Result},
    storage::sync::{PAGE, Remote, Replica},
};
use protocol::{Request, Response, exchange};
use serde::Serialize;
use std::io::{BufRead, Write};
#[derive(Default, Serialize)]
pub struct Counts {
    pub pulled: usize,
    pub pushed: usize,
    pub remotes: usize,
}
pub async fn pull(
    replica: &Replica,
    remote: &Remote,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<usize> {
    protocol::handshake(reader, writer)?;
    let mut count = 0;
    for tombstones in [true, false] {
        let mut after: Option<String> = None;
        loop {
            let Response::Entries { entries } = exchange(
                reader,
                writer,
                &Request::Inventory {
                    tombstones,
                    after: after.clone(),
                    scope: remote.scope.clone(),
                },
            )?
            else {
                return Err(Error::Invalid("expected sync inventory".into()));
            };
            if entries.len() > PAGE
                || entries.windows(2).any(|w| w[0].id >= w[1].id)
                || entries
                    .first()
                    .is_some_and(|e| after.as_ref().is_some_and(|a| e.id <= *a))
                || entries.iter().any(|e| e.hash.is_none() != tombstones)
            {
                return Err(Error::Invalid("invalid sync inventory order".into()));
            }
            if entries.is_empty() {
                break;
            }
            after = entries.last().map(|e| e.id.clone());
            for entry in entries {
                if !replica.needs(&entry).await? {
                    continue;
                }
                let Response::Item { item } = exchange(
                    reader,
                    writer,
                    &Request::Get {
                        id: entry.id.clone(),
                        scope: remote.scope.clone(),
                    },
                )?
                else {
                    return Err(Error::Invalid("expected sync item".into()));
                };
                let item =
                    item.ok_or_else(|| Error::Invalid("sync inventory changed; retry".into()))?;
                if item.id != entry.id
                    || (item.envelope.is_some() && item.envelope_hash != entry.hash)
                {
                    return Err(Error::Invalid("sync inventory changed; retry".into()));
                }
                count += usize::from(replica.apply(&item, remote.scope.as_deref()).await?);
            }
        }
    }
    Ok(count)
}
pub async fn push(
    replica: &Replica,
    remote: &Remote,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<usize> {
    protocol::handshake(reader, writer)?;
    let mut count = 0;
    for tombstones in [true, false] {
        let mut after: Option<String> = None;
        loop {
            let entries = replica
                .inventory(tombstones, after.as_deref(), remote.scope.as_deref())
                .await?;
            if entries.is_empty() {
                break;
            }
            after = entries.last().map(|e| e.id.clone());
            let Response::Needed { ids } = exchange(
                reader,
                writer,
                &Request::Needs {
                    entries: entries.clone(),
                },
            )?
            else {
                return Err(Error::Invalid("expected missing sync IDs".into()));
            };
            if ids.len() > entries.len()
                || ids.iter().any(|id| !entries.iter().any(|e| &e.id == id))
            {
                return Err(Error::Invalid("peer requested unoffered record".into()));
            }
            for id in ids {
                let item = replica
                    .get(&id, remote.scope.as_deref())
                    .await?
                    .ok_or_else(|| Error::Invalid("sync inventory changed; retry".into()))?;
                let Response::Applied { changed } = exchange(
                    reader,
                    writer,
                    &Request::Apply {
                        item,
                        scope: remote.scope.clone(),
                    },
                )?
                else {
                    return Err(Error::Invalid("expected sync acknowledgement".into()));
                };
                count += usize::from(changed);
            }
        }
    }
    if !matches!(exchange(reader, writer, &Request::Finish)?, Response::Done) {
        return Err(Error::Invalid(
            "expected sync publication acknowledgement".into(),
        ));
    }
    Ok(count)
}
pub async fn run(replica: &Replica) -> Result<Counts> {
    let remotes = replica.remotes().await?;
    let mut counts = Counts {
        remotes: remotes.len(),
        ..Counts::default()
    };
    for remote in &remotes {
        eprintln!("Pulling {}…", remote.name);
        let result = async {
            let mut session = ssh::Session::open(&ssh::Origin::parse(&remote.origin)?)?;
            let count = pull(
                replica,
                remote,
                &mut session.reader,
                session.writer.as_mut().unwrap(),
            )
            .await?;
            session.close()?;
            Ok::<_, Error>(count)
        }
        .await;
        match result {
            Ok(count) => counts.pulled += count,
            Err(error) => {
                let recovery = match replica.finish().await {
                    Ok(()) => "received records are available locally".to_owned(),
                    Err(recovery) => {
                        format!("local publication failed: {recovery}; run linn index:rebuild")
                    }
                };
                return Err(Error::Invalid(format!(
                    "pull from {} failed: {error}; {recovery}; no remotes were pushed",
                    remote.name
                )));
            }
        }
    }
    replica.finish().await?;
    for remote in &remotes {
        eprintln!("Pushing {}…", remote.name);
        let result = async {
            let mut session = ssh::Session::open(&ssh::Origin::parse(&remote.origin)?)?;
            let count = push(
                replica,
                remote,
                &mut session.reader,
                session.writer.as_mut().unwrap(),
            )
            .await?;
            session.close()?;
            Ok::<_, Error>(count)
        }
        .await;
        counts.pushed += result.map_err(|error| Error::Invalid(format!("push to {} failed: {error}; retry sync; incomplete remote publication can be repaired with index:rebuild on that host", remote.name)))?;
    }
    Ok(counts)
}
