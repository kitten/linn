//! Bounded mesh framing and per-chunk original-byte delivery barriers.
use super::{
    failure::Kind,
    jobs::SignedRun,
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
mod body;
pub(crate) use body::{BodyResponse, receive_body, send_body};
pub const WAIT: Duration = Duration::from_secs(30);
pub const ALPN: &[u8] = b"linn/mesh/2";

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "body",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Frame {
    Hello {
        version: u8,
        head: String,
    },
    Proof {
        events: Vec<Acceptance>,
        done: bool,
    },
    Ack,
    Authorized,
    Begin {
        job: SignedRun,
        replaces_parent: Option<String>,
        continuation: bool,
        terminal_only: bool,
    },
    Accepted {
        free: bool,
        parent: Option<String>,
        duplicate: bool,
        hints: Vec<String>,
    },
    Capacity(Vec<(String, bool)>),
    Command(super::peer::coordinated::Envelope),
    Reply {
        id: u64,
        result:
            std::result::Result<super::peer::coordinated::Reply, super::peer::coordinated::Fault>,
    },
    Observed {
        id: u64,
        observation: super::discovery_job::Observation,
    },
    Cancel {
        id: u64,
    },
    Replica {
        run: String,
        round: u8,
        context: u64,
        delta: bool,
    },
    DeltaInventory {
        tombstones: bool,
        after: Option<String>,
    },
    DeltaEntries {
        #[serde(deserialize_with = "crate::storage::sync::deserialize_inventory_entries")]
        entries: Vec<crate::storage::sync::InventoryEntry>,
        done: bool,
    },
    InventoryPage(crate::storage::sync::InventoryPage),
    DeltaUnavailable,
    Progress {
        revision: u64,
    },
    Request(Request),
    Response(Response),
}

#[cfg(test)]
pub(crate) mod metrics {
    use std::sync::atomic::{AtomicU64, Ordering};
    static BYTES: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];
    static FRAMES: AtomicU64 = AtomicU64::new(0);
    static REOPENS: std::sync::Mutex<std::collections::BTreeMap<String, [u64; 6]>> =
        std::sync::Mutex::new(std::collections::BTreeMap::new());
    static READS: std::sync::Mutex<std::collections::BTreeMap<String, u64>> =
        std::sync::Mutex::new(std::collections::BTreeMap::new());
    static MILESTONES: std::sync::Mutex<
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, f64>>,
    > = std::sync::Mutex::new(std::collections::BTreeMap::new());
    pub(crate) fn milestone(run: &str, stage: &str, elapsed: std::time::Duration) {
        MILESTONES
            .lock()
            .unwrap()
            .entry(run.into())
            .or_default()
            .insert(stage.into(), elapsed.as_secs_f64() * 1000.0);
    }
    pub(crate) fn milestones()
    -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, f64>> {
        MILESTONES.lock().unwrap().clone()
    }
    pub(crate) fn reopened(peer: &str, purpose: usize) {
        REOPENS.lock().unwrap().entry(peer.into()).or_default()[purpose] += 1;
    }
    pub(crate) fn source_read(peer: &str) {
        *READS.lock().unwrap().entry(peer.into()).or_default() += 1;
    }
    pub(crate) fn resources() -> (
        std::collections::BTreeMap<String, [u64; 6]>,
        std::collections::BTreeMap<String, u64>,
    ) {
        (
            REOPENS.lock().unwrap().clone(),
            READS.lock().unwrap().clone(),
        )
    }
    pub(crate) fn add(class: usize, bytes: usize) {
        BYTES[class].fetch_add(bytes as u64, Ordering::Relaxed);
        FRAMES.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn clear() {
        for counter in &BYTES {
            counter.store(0, Ordering::Relaxed);
        }
        FRAMES.store(0, Ordering::Relaxed);
        REOPENS.lock().unwrap().clear();
        MILESTONES.lock().unwrap().clear();
        READS.lock().unwrap().clear();
    }
    pub(crate) fn snapshot() -> ([u64; 4], u64) {
        (
            std::array::from_fn(|index| BYTES[index].load(Ordering::Relaxed)),
            FRAMES.load(Ordering::Relaxed),
        )
    }
}

#[cfg(not(test))]
pub(crate) mod metrics {
    pub(crate) fn milestone(_: &str, _: &str, _: std::time::Duration) {}
    pub(crate) fn reopened(_: &str, _: usize) {}
}

struct Limited(Vec<u8>);
struct Transmission<'a> {
    send: &'a mut SendStream,
    delivery: Option<tokio::sync::oneshot::Sender<()>>,
    complete: bool,
    failure: Kind,
}
impl Drop for Transmission<'_> {
    fn drop(&mut self) {
        if !self.complete {
            let _ = self.send.reset(self.failure.code().into());
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
pub async fn send(send: &mut SendStream, frame: Frame) -> Result<()> {
    send_inner(send, frame, None).await
}
pub(crate) async fn send_progress(
    send: &mut SendStream,
    frame: Frame,
    progress: &Progress,
) -> Result<()> {
    send_inner(send, frame, Some(progress)).await
}
async fn send_inner(
    send: &mut SendStream,
    frame: Frame,
    progress: Option<&Progress>,
) -> Result<()> {
    if matches!(&frame, Frame::Response(Response::Item { item: Some(_) })) {
        return Err(Kind::Protocol.error("original frames require registered delivery"));
    }
    let mut output = Transmission {
        send,
        delivery: None,
        complete: false,
        failure: Kind::Cancelled,
    };
    let result = async {
        #[cfg(test)]
        let class = match &frame {
            Frame::Response(Response::Item { .. }) => 0,
            Frame::Request(Request::Inventory { .. })
            | Frame::InventoryPage(_)
            | Frame::DeltaInventory { .. }
            | Frame::DeltaEntries { .. } => 1,
            Frame::Hello { .. } | Frame::Proof { .. } | Frame::Ack | Frame::Authorized => 2,
            _ => 3,
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
        #[cfg(test)]
        metrics::add(class, bytes.len());
        let mut offset = 0;
        while offset < bytes.len() {
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
                _ => return Err(Kind::Unresolved.error("mesh write ended or was cancelled")),
            }
        }
        Ok(())
    }
    .await;
    if let Err(error) = &result {
        output.failure = Kind::of(error);
    }
    output.complete = result.is_ok();
    result
}

pub async fn receive_limit(recv: &mut RecvStream, limit: usize) -> Result<Frame> {
    receive_inner(recv, limit, None, false).await
}
pub(crate) async fn receive_progress(recv: &mut RecvStream, progress: &Progress) -> Result<Frame> {
    receive_inner(recv, MAX_FRAME, Some(progress), false).await
}
pub(crate) async fn receive_request(recv: &mut RecvStream) -> Result<Frame> {
    receive_inner(recv, MAX_FRAME, None, true).await
}
async fn receive_inner(
    recv: &mut RecvStream,
    limit: usize,
    progress: Option<&Progress>,
    idle_between_requests: bool,
) -> Result<Frame> {
    let mut header = [0; 4];
    let remaining = if idle_between_requests {
        recv.read_exact(&mut header[..1])
            .await
            .map_err(Kind::read_exact)?;
        &mut header[1..]
    } else {
        &mut header[..]
    };
    tokio::time::timeout(WAIT, recv.read_exact(remaining))
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
