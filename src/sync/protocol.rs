//! @ref LLP 0006#wire-protocol
use crate::{
    error::{Error, Result},
    storage::sync::{Entry, Item, PAGE, Replica},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{BufRead, Write};
const VERSION: u32 = 1;
pub const MAX_FRAME: usize = 16 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Hello {
        version: u32,
    },
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
    Hello { version: u32 },
    Entries { entries: Vec<Entry> },
    Item { item: Option<Item> },
    Needed { ids: Vec<String> },
    Applied { changed: bool },
    Done,
    Error { message: String },
}
pub fn read<T: DeserializeOwned>(reader: &mut impl BufRead) -> Result<Option<T>> {
    let mut frame = Vec::new();
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            return if frame.is_empty() {
                Ok(None)
            } else {
                Err(Error::Invalid("truncated sync frame".into()))
            };
        }
        let end = bytes.iter().position(|&b| b == b'\n').map(|p| p + 1);
        let n = end.unwrap_or(bytes.len());
        if frame.len() + n > MAX_FRAME {
            return Err(Error::Invalid("sync frame exceeds 16 MiB".into()));
        }
        frame.extend_from_slice(&bytes[..n]);
        reader.consume(n);
        if end.is_some() {
            return Ok(Some(serde_json::from_slice(&frame)?));
        }
    }
}
pub fn write<T: Serialize>(writer: &mut impl Write, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() + 1 > MAX_FRAME {
        return Err(Error::Invalid("sync frame exceeds 16 MiB".into()));
    }
    writer.write_all(&bytes)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
pub fn exchange(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    request: &Request,
) -> Result<Response> {
    write(writer, request)?;
    reply(reader)
}
pub(super) fn reply(reader: &mut impl BufRead) -> Result<Response> {
    match read(reader)?.ok_or_else(|| Error::Invalid("sync peer disconnected".into()))? {
        Response::Error { message } => Err(Error::Invalid(format!("sync peer: {message}"))),
        reply => Ok(reply),
    }
}
pub fn handshake(reader: &mut impl BufRead, writer: &mut impl Write) -> Result<()> {
    match exchange(reader, writer, &Request::Hello { version: VERSION })? {
        Response::Hello { version: VERSION } => Ok(()),
        _ => Err(Error::Invalid("incompatible sync protocol".into())),
    }
}
pub async fn serve(
    replica: &Replica,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<()> {
    match read::<Request>(reader)? {
        Some(Request::Hello { version: VERSION }) => {
            write(writer, &Response::Hello { version: VERSION })?
        }
        _ => {
            write(
                writer,
                &Response::Error {
                    message: "incompatible sync protocol".into(),
                },
            )?;
            return Err(Error::Invalid("incompatible sync protocol".into()));
        }
    }
    while let Some(request) = read::<Request>(reader)? {
        let reply = handle(replica, request).await;
        match reply {
            Ok(response) => write(writer, &response)?,
            Err(e) => {
                write(
                    writer,
                    &Response::Error {
                        message: e.to_string(),
                    },
                )?;
                return Err(e);
            }
        }
    }
    Ok(())
}
pub(crate) async fn handle(replica: &Replica, request: Request) -> Result<Response> {
    Ok(match request {
        Request::Hello { .. } => return Err(Error::Invalid("duplicate handshake".into())),
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
