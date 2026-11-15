//! Bounded mesh framing and per-chunk original-byte delivery barriers.
use super::{
    failure::Kind,
    jobs::{Phase, Report, SignedRun},
    membership::Acceptance,
    progress::Progress,
    store::{Handle, Step},
};
use crate::{
    error::{Error, Result},
    sync::protocol::{MAX_FRAME, Request, Response},
};
use iroh::endpoint::{RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use std::{io::Write, time::Duration};
pub const WAIT: Duration = Duration::from_secs(30);
pub const ALPN: &[u8] = b"linn/mesh/1";

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "body",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Frame {
    Hello { version: u8, head: String },
    Proof { events: Vec<Acceptance>, done: bool },
    Ack,
    Authorized,
    Begin(SignedRun),
    Accepted { phase: Phase, duplicate: bool },
    Collected(Report),
    Distribute,
    Completed(Report),
    Progress { revision: u64 },
    Replica { run: String },
    Request(Request),
    Response(Response),
}

struct Limited(Vec<u8>);
struct Transmission<'a> {
    send: &'a mut SendStream,
    delivery: Option<tokio::sync::oneshot::Sender<()>>,
    complete: bool,
}
impl Drop for Transmission<'_> {
    fn drop(&mut self) {
        if !self.complete {
            let _ = self.send.reset(0u32.into());
        }
        // Reset a partial frame before releasing its original-byte delivery lease.
        drop(self.delivery.take());
    }
}
impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len() + bytes.len() > MAX_FRAME + 4 {
            return Err(std::io::Error::other("mesh frame too large"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub async fn send(send: &mut SendStream, frame: Frame, store: &Handle, peer: &str) -> Result<()> {
    send_inner(send, frame, store, peer, None).await
}
pub(crate) async fn send_progress(
    send: &mut SendStream,
    frame: Frame,
    store: &Handle,
    peer: &str,
    progress: &Progress,
) -> Result<()> {
    send_inner(send, frame, store, peer, Some(progress)).await
}
async fn send_inner(
    send: &mut SendStream,
    frame: Frame,
    store: &Handle,
    peer: &str,
    progress: Option<&Progress>,
) -> Result<()> {
    let mut output = Transmission {
        send,
        delivery: None,
        complete: false,
    };
    let original = match &frame {
        Frame::Response(Response::Item { item: Some(item) }) => {
            Some((item.id.clone(), item.envelope_hash.clone()))
        }
        _ => None,
    };
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        let mut bytes = Limited(vec![0; 4]);
        serde_json::to_writer(&mut bytes, &frame).map_err(|_| {
            Error::Invalid("mesh frame could not be encoded within its limit".into())
        })?;
        let len = (bytes.0.len() - 4) as u32;
        bytes.0[..4].copy_from_slice(&len.to_be_bytes());
        Ok(bytes.0)
    })
    .await
    .map_err(|_| Error::Invalid("mesh encoder stopped".into()))??;
    let mut offset = 0;
    while offset < bytes.len() {
        output.delivery = if let Some((id, hash)) = &original {
            Some(
                store
                    .lease(Step::Chunk {
                        peer: peer.into(),
                        id: id.clone(),
                        hash: hash.clone(),
                    })
                    .await?
                    .1,
            )
        } else {
            None
        };
        let end = (offset + 64 * 1024).min(bytes.len());
        let written = tokio::time::timeout(WAIT, output.send.write(&bytes[offset..end])).await;
        match written {
            Ok(Ok(n)) if n > 0 => {
                offset += n;
                if let Some(progress) = progress {
                    progress.advance();
                }
            }
            Err(_) => return Err(Kind::Timeout.error("mesh write made no progress")),
            Ok(Err(error)) => return Err(Kind::write(error)),
            _ => return Err(Kind::Offline.error("mesh write ended or was cancelled")),
        }
        drop(output.delivery.take());
    }
    output.complete = true;
    Ok(())
}

pub async fn receive(recv: &mut RecvStream) -> Result<Frame> {
    receive_limit(recv, MAX_FRAME).await
}
pub async fn receive_limit(recv: &mut RecvStream, limit: usize) -> Result<Frame> {
    receive_inner(recv, limit, None).await
}
pub(crate) async fn receive_progress(recv: &mut RecvStream, progress: &Progress) -> Result<Frame> {
    receive_inner(recv, MAX_FRAME, Some(progress)).await
}
async fn receive_inner(
    recv: &mut RecvStream,
    limit: usize,
    progress: Option<&Progress>,
) -> Result<Frame> {
    let mut header = [0; 4];
    tokio::time::timeout(WAIT, recv.read_exact(&mut header))
        .await
        .map_err(|_| Kind::Timeout.error("mesh frame header timed out"))?
        .map_err(Kind::read_exact)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 || len > limit.min(MAX_FRAME) {
        return Err(Kind::Protocol.error("invalid mesh frame size"));
    }
    let mut body = vec![0; len];
    let mut offset = 0;
    while offset < len {
        let end = (offset + 64 * 1024).min(len);
        let n = tokio::time::timeout(WAIT, recv.read(&mut body[offset..end]))
            .await
            .map_err(|_| Kind::Timeout.error("mesh read made no progress"))?
            .map_err(Kind::read)?
            .filter(|n| *n > 0)
            .ok_or_else(|| Kind::Protocol.error("truncated mesh frame"))?;
        offset += n;
        if let Some(progress) = progress {
            progress.advance();
        }
    }
    tokio::task::spawn_blocking(move || {
        serde_json::from_slice(&body).map_err(|_| Kind::Protocol.error("malformed mesh frame"))
    })
    .await
    .map_err(|_| Error::Invalid("mesh decoder stopped".into()))?
}

#[cfg(test)]
mod tests;
