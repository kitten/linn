//! @ref LLP 0008#replica-wire-contract
use crate::{
    error::{Error, Result},
    storage::sync::{Entry, Item, PAGE, Replica},
};
use serde::{Deserialize, Serialize};
pub const MAX_FRAME: usize = 16 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Inventory {
        tombstones: bool,
        after: Option<String>,
        scope: Option<String>,
    },
    Get {
        id: String,
        scope: Option<String>,
    },
    Needs {
        entries: Vec<Entry>,
    },
    Apply {
        item: Item,
        scope: Option<String>,
    },
    Finish,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Response {
    Entries { entries: Vec<Entry> },
    Item { item: Option<Item> },
    Needed { ids: Vec<String> },
    Applied { changed: bool },
    Done,
}
pub(crate) async fn handle(replica: &Replica, request: Request) -> Result<Response> {
    Ok(match request {
        Request::Inventory {
            tombstones,
            after,
            scope,
        } => Response::Entries {
            entries: replica
                .inventory(tombstones, after.as_deref(), scope.as_deref())
                .await?,
        },
        Request::Get { id, scope } => Response::Item {
            item: replica.get(&id, scope.as_deref()).await?,
        },
        Request::Needs { entries } => {
            if entries.len() > PAGE {
                return Err(Error::Invalid("oversized inventory".into()));
            }
            let mut ids = Vec::new();
            for entry in entries {
                if replica.needs(&entry).await? {
                    ids.push(entry.id);
                }
            }
            Response::Needed { ids }
        }
        Request::Apply { item, scope } => Response::Applied {
            changed: replica.apply(&item, scope.as_deref()).await?,
        },
        Request::Finish => {
            replica.finish().await?;
            Response::Done
        }
    })
}
