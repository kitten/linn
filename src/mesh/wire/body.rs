//! Registered original buffers retain secure-forget ownership across network waits.
use super::*;
use crate::{
    mesh::{
        buffers::{Body, Payload},
        jobs::SignedRun,
    },
    storage::sync::InventoryEntry,
};
use std::{
    future::{Future, poll_fn},
    sync::Arc,
    task::Poll,
};

#[derive(Serialize)]
#[serde(tag = "type", content = "body", rename_all = "snake_case")]
enum OriginalFrame<'a> {
    Response(OriginalReply<'a>),
}
#[derive(Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
enum OriginalReply<'a> {
    Item {
        item: &'a crate::storage::sync::Item,
    },
}
struct OriginalBuffer(Body);
impl Drop for OriginalBuffer {
    fn drop(&mut self) {
        self.0.discard();
    }
}

pub(crate) async fn send_body(
    send: &mut SendStream,
    body: Body,
    store: &Handle,
    peer: &str,
    job: &Arc<SignedRun>,
    progress: &Progress,
    scope: &Arc<crate::mesh::peer::source::Scope>,
) -> Result<()> {
    // Also fence a blocking encoder when its async caller is cancelled.
    let _original = OriginalBuffer(body.clone());
    let mut output = Transmission {
        send,
        delivery: None,
        settled: false,
        failure: Kind::Cancelled,
    };
    let result = async {
        let small = body.with(|payload| {
            let Payload::Item(item) = payload else {
                return Err(Kind::Protocol.error("original expected"));
            };
            let bytes = item.id.len()
                + item.envelope.as_ref().map_or(0, String::len)
                + item.envelope_hash.as_ref().map_or(0, String::len)
                + item.body_hash.as_ref().map_or(0, String::len);
            Ok(bytes <= (INLINE_FRAME - 256) / 6)
        })?;
        let len = if small {
            encode_original(&body)?
        } else {
            let encoded = body.clone();
            tokio::task::spawn_blocking(move || encode_original(&encoded))
                .await
                .map_err(|_| Kind::Protocol.error("mesh encoder stopped"))??
        };
        #[cfg(test)]
        metrics::add(0, len);
        let entry = body.entry();
        let mut offset = 0;
        while offset < len {
            output.delivery = Some(
                store
                    .lease(Step::BodyChunk {
                        scope: scope.clone(),
                        peer: peer.into(),
                        job: job.clone(),
                        id: entry.id.clone(),
                        hash: entry.hash.clone(),
                    })
                    .await?
                    .1,
            );
            let chunk = body.with(|payload| {
                let Payload::Encoded(bytes) = payload else {
                    return Err(Kind::Protocol.error("encoded original expected"));
                };
                Ok(bytes[offset..(offset + 64 * 1024).min(len)].to_vec())
            })?;
            let written = match tokio::time::timeout(WAIT, output.send.write(&chunk)).await {
                Ok(Ok(n)) if n > 0 => Ok(n),
                Ok(Err(error)) => Err(Kind::write(error)),
                Err(_) => Err(Kind::Timeout.error("mesh write made no progress")),
                _ => Err(Kind::Unresolved.error("mesh write ended")),
            };
            // No application copy survives release, including a timed-out attempt.
            drop(chunk);
            output.release_attempt(&written);
            offset += written?;
            progress.advance();
        }
        Ok(())
    }
    .await;
    let result = result.map_err(|error| {
        if matches!(&error, Error::Mesh(failure) if failure.local) {
            Kind::Authentication.error("original was forgotten before delivery")
        } else {
            error
        }
    });
    if let Err(error) = &result {
        output.failure = Kind::of(error);
    }
    output.settled |= result.is_ok();
    result
}

pub(crate) enum BodyResponse {
    Item,
    Missing,
    Unavailable,
    Mismatch,
}

pub(crate) async fn receive_body(
    recv: &mut RecvStream,
    body: Body,
    entry: &InventoryEntry,
    progress: &Progress,
    delta: bool,
) -> Result<BodyResponse> {
    tokio::select! {
        biased;
        _ = body.erased() => {
            let _ = recv.stop(Kind::Cancelled.code().into());
            Err(Body::erased_error())
        },
        result = read_body(recv, body.clone(), entry, progress, delta) => result,
    }
}

async fn read_body(
    recv: &mut RecvStream,
    body: Body,
    entry: &InventoryEntry,
    progress: &Progress,
    delta: bool,
) -> Result<BodyResponse> {
    let mut header = [0; 4];
    tokio::time::timeout(WAIT, recv.read_exact(&mut header))
        .await
        .map_err(|_| Kind::Timeout.error("mesh item frame header timed out"))?
        .map_err(Kind::read_exact)?;
    let len = u32::from_be_bytes(header) as usize;
    if !valid_frame_size(len, MAX_FRAME) {
        return Err(Kind::Protocol.error("item exceeds the absolute frame limit"));
    }
    if (len + 4) as u64 > entry.frame_bytes {
        let _ = recv.stop(Kind::Protocol.code().into());
        body.with(|payload| {
            *payload = Payload::Empty;
            Ok(())
        })?;
        return Ok(BodyResponse::Mismatch);
    }
    body.with(|payload| {
        let Payload::Receiving(bytes) = payload else {
            return Err(Kind::Protocol.error("receive buffer expected"));
        };
        bytes.reserve_exact(len);
        Ok(())
    })?;
    let mut remaining = len;
    while remaining != 0 {
        let read = recv.read_chunk(remaining.min(64 * 1024));
        tokio::pin!(read);
        let receive = poll_fn(|context| {
            // Poll once under the ownership lock, never hold it across Pending.
            // A ready transport chunk is copied and dropped before forget can
            // acknowledge; no application chunk lives outside the registry.
            match body.with(|payload| {
                let Payload::Receiving(buffer) = payload else {
                    return Err(Kind::Protocol.error("receive buffer expected"));
                };
                Ok(match read.as_mut().poll(context) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Ok(Some(bytes))) if !bytes.is_empty() => {
                        buffer.extend_from_slice(&bytes);
                        Poll::Ready(Ok(bytes.len()))
                    }
                    Poll::Ready(Err(error)) => Poll::Ready(Err(Kind::read(error))),
                    _ => Poll::Ready(Err(Kind::Protocol.error("truncated mesh frame"))),
                })
            }) {
                Ok(result) => result,
                Err(error) => Poll::Ready(Err(error)),
            }
        });
        remaining -= tokio::time::timeout(WAIT, receive)
            .await
            .map_err(|_| Kind::Timeout.error("mesh read made no progress"))??;
        progress.advance();
    }
    if len <= INLINE_FRAME {
        return decode_original(&body, entry, delta);
    }
    let decoded = body.clone();
    let entry = entry.clone();
    tokio::task::spawn_blocking(move || decode_original(&decoded, &entry, delta))
        .await
        .map_err(|_| Kind::Protocol.error("mesh decoder stopped"))?
}

// Keep bounded small frames on the async task; large CPU work remains offloaded.
const INLINE_FRAME: usize = 64 * 1024;
fn encode_original(body: &Body) -> Result<usize> {
    body.with(|payload| {
        let Payload::Item(item) = payload else {
            return Err(Kind::Protocol.error("original expected"));
        };
        let mut bytes = Limited(vec![0; 4]);
        serde_json::to_writer(
            &mut bytes,
            &OriginalFrame::Response(OriginalReply::Item { item }),
        )
        .map_err(|_| Kind::Protocol.error("original exceeds frame limit"))?;
        let len = bytes.0.len();
        bytes.0[..4].copy_from_slice(&((len - 4) as u32).to_be_bytes());
        *payload = Payload::Encoded(bytes.0);
        Ok(len)
    })
}
fn decode_original(body: &Body, entry: &InventoryEntry, delta: bool) -> Result<BodyResponse> {
    body.with(|payload| {
        let Payload::Receiving(bytes) = payload else {
            return Err(Kind::Protocol.error("receive buffer expected"));
        };
        let frame: Frame = serde_json::from_slice(bytes)
            .map_err(|_| Kind::Protocol.error("malformed mesh frame"))?;
        let response = match frame {
            Frame::Response(Response::Item { item: Some(item) }) => {
                if item.id != entry.entry.id || item.envelope_hash != entry.entry.hash {
                    *payload = Payload::Empty;
                    return Ok(BodyResponse::Mismatch);
                }
                if item.envelope.as_ref().map_or(0, String::len) as u64 != entry.envelope_bytes {
                    return Err(Kind::Protocol.error("original envelope size changed"));
                }
                *payload = Payload::Item(item);
                return Ok(BodyResponse::Item);
            }
            Frame::Response(Response::Item { item: None }) => BodyResponse::Missing,
            Frame::DeltaUnavailable => BodyResponse::Unavailable,
            _ => return Err(Kind::Protocol.error("item reply expected")),
        };
        *payload = if matches!(response, BodyResponse::Missing) && !delta {
            Payload::Missing
        } else {
            Payload::Empty
        };
        Ok(response)
    })
}
