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
trait Reset {
    fn reset_kind(&mut self, kind: Kind);
}
impl Reset for SendStream {
    fn reset_kind(&mut self, kind: Kind) {
        let _ = self.reset(kind.code().into());
    }
}
struct Transmission<'a, T: Reset = SendStream> {
    send: &'a mut T,
    delivery: Option<tokio::sync::oneshot::Sender<()>>,
    settled: bool,
    failure: Kind,
}
impl<T: Reset> Transmission<'_, T> {
    fn release_attempt(&mut self, result: &Result<usize>) {
        if let Err(error) = result {
            self.failure = Kind::of(error);
            self.send.reset_kind(self.failure);
            self.settled = true;
        }
        // A failed write is cancelled at the transport before forget may proceed.
        drop(self.delivery.take());
    }
}
impl<T: Reset> Drop for Transmission<'_, T> {
    fn drop(&mut self) {
        if !self.settled {
            self.send.reset_kind(self.failure);
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
        settled: false,
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
    output.settled |= result.is_ok();
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
    if !valid_frame_size(len, limit) {
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

fn valid_frame_size(len: usize, limit: usize) -> bool {
    len != 0 && len <= limit.min(MAX_FRAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::sync::oneshot::error::TryRecvError;

    #[tokio::test]
    async fn failed_write_resets_before_the_delivery_lease_closes() {
        struct Probe {
            receiver: std::sync::Arc<Mutex<tokio::sync::oneshot::Receiver<()>>>,
            leased_at_reset: Vec<bool>,
        }
        impl Reset for Probe {
            fn reset_kind(&mut self, _: Kind) {
                self.leased_at_reset.push(matches!(
                    self.receiver.lock().unwrap().try_recv(),
                    Err(TryRecvError::Empty)
                ));
            }
        }
        let (tx, rx) = tokio::sync::oneshot::channel();
        let receiver = std::sync::Arc::new(Mutex::new(rx));
        let mut probe = Probe {
            receiver: receiver.clone(),
            leased_at_reset: vec![],
        };
        let mut output = Transmission {
            send: &mut probe,
            delivery: Some(tx),
            settled: false,
            failure: Kind::Cancelled,
        };
        let result = Err(Kind::Timeout.error("stalled write"));
        output.release_attempt(&result);
        drop(output);
        assert_eq!(probe.leased_at_reset, vec![true]);
        assert!(matches!(
            receiver.lock().unwrap().try_recv(),
            Err(TryRecvError::Closed)
        ));
    }
    use crate::{
        mesh::{control_stream::Control, store},
        storage::sync::Item,
    };
    use iroh::{
        Endpoint, EndpointAddr, RelayMode,
        endpoint::{Connection, presets},
    };
    use std::sync::Arc;

    async fn endpoint(alpn: &[u8]) -> Endpoint {
        Endpoint::builder(presets::Minimal)
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .clear_address_lookup()
            .relay_mode(RelayMode::Disabled)
            .alpns(vec![alpn.to_vec()])
            .bind()
            .await
            .unwrap()
    }

    async fn connection() -> (Endpoint, Endpoint, Connection, Connection) {
        let a = endpoint(ALPN).await;
        let b = endpoint(ALPN).await;
        let address = EndpointAddr::new(b.id()).with_ip_addr(b.bound_sockets()[0]);
        let (client, server) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(a.connect(address, ALPN), async {
                b.accept().await.unwrap().await
            })
        })
        .await
        .unwrap();
        (a, b, client.unwrap(), server.unwrap())
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn forget_fences_a_partial_received_body_before_acknowledging_and_drops_late_bytes() {
        use crate::{
            mesh::buffers::{Buffers, Payload},
            storage::sync::{Entry, InventoryEntry},
        };
        let (a, b, client, server) = connection().await;
        let envelope = "x".repeat(256 * 1024);
        let item = Item {
            id: "forgotten".into(),
            envelope_hash: Some("hash".into()),
            body_hash: None,
            envelope: Some(envelope),
        };
        let bytes = serde_json::to_vec(&Frame::Response(Response::Item {
            item: Some(item.clone()),
        }))
        .unwrap();
        let expected = InventoryEntry {
            entry: Entry {
                id: item.id,
                hash: item.envelope_hash,
            },
            frame_bytes: (bytes.len() + 4) as u64,
            envelope_bytes: item.envelope.unwrap().len() as u64,
        };
        let buffers = Arc::new(Buffers::default());
        let body = buffers
            .receive(buffers.epoch(), expected.entry.clone())
            .unwrap();
        let release = Arc::new(tokio::sync::Notify::new());
        let source_release = release.clone();
        let source = tokio::spawn(async move {
            let (mut send, _) = server.open_bi().await.unwrap();
            send.write_all(&(bytes.len() as u32).to_be_bytes())
                .await
                .unwrap();
            send.write_all(&bytes[..64 * 1024]).await.unwrap();
            source_release.notified().await;
            let _ = send.write_all(&bytes[64 * 1024..]).await;
        });
        let (_, mut recv) = client.accept_bi().await.unwrap();
        let received = body.clone();
        let reader = tokio::spawn(async move {
            receive_body(&mut recv, received, &expected, &Progress::default(), false).await
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if body
                    .with(|payload| {
                        Ok(matches!(payload, Payload::Receiving(bytes) if !bytes.is_empty()))
                    })
                    .unwrap()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            let erased = buffers.clone();
            tokio::task::spawn_blocking(move || erased.forget("forgotten"))
                .await
                .unwrap();
            assert!(
                body.take().is_err(),
                "a queued Apply cannot resurrect the received body"
            );
            assert_eq!(
                buffers.live_count(),
                0,
                "forget acknowledgement must leave no local original"
            );
            release.notify_one();
            let error = reader.await.unwrap().err().unwrap();
            assert!(
                matches!(error, Error::Mesh(error) if error.local && error.kind == Kind::Cancelled)
            );
            source.await.unwrap();
            assert_eq!(buffers.live_count(), 0);
        })
        .await
        .unwrap();
        a.close().await;
        b.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn partial_original_frames_reset_on_forget_or_cancellation_between_leases() {
        for cancel in [false, true] {
            let (a, b, client, server) = connection().await;
            let (handle, mut jobs) = store::channel();
            let (mut send_stream, _) = client.open_bi().await.unwrap();
            let envelope =
                serde_json::json!({"id":"original", "body":"x".repeat(256 * 1024)}).to_string();
            let item = Item {
                id: "original".into(),
                envelope_hash: Some(crate::record::hash(envelope.as_bytes())),
                body_hash: None,
                envelope: Some(envelope),
            };
            let membership = crate::mesh::membership::Membership {
                head: "0".repeat(64),
                service: "http://127.0.0.1:1".into(),
                receipt_key: "0".repeat(64),
                roots: std::collections::BTreeSet::from(["1".repeat(64)]),
                members: std::collections::BTreeSet::from([a.id().to_string(), b.id().to_string()]),
                revoked: Default::default(),
                invitations: Default::default(),
            };
            let job = std::sync::Arc::new(
                crate::mesh::jobs::SignedRun::new(
                    a.secret_key(),
                    &membership,
                    crate::mesh::membership::random_id(),
                )
                .unwrap(),
            );
            let error = send(
                &mut send_stream,
                Frame::Response(Response::Item {
                    item: Some(item.clone()),
                }),
            )
            .await
            .unwrap_err();
            assert_eq!(Kind::of(&error), Kind::Protocol);
            let remote = b.id().to_string();
            let buffers = handle.buffers.clone();
            let body = buffers.original(item);
            let tracked = (!cancel).then(|| body.clone());
            let sender = tokio::spawn(async move {
                send_body(
                    &mut send_stream,
                    body,
                    &handle,
                    &remote,
                    &job,
                    &Progress::default(),
                    &Arc::default(),
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(5), async {
                let first = jobs.recv().await.unwrap();
                assert!(matches!(first.step, Step::BodyChunk { .. }));
                assert!(first.reply.send(Ok(store::Value::Done)).is_ok());
                first.delivered.await.unwrap_err();
                let next = jobs.recv().await.unwrap();
                // The receiver has seen a partial frame before the authorization changes.
                let (_, mut recv) = server.accept_bi().await.unwrap();
                let mut prefix = [0; 64];
                recv.read_exact(&mut prefix).await.unwrap();
                if cancel {
                    sender.abort();
                } else {
                    buffers.forget("original");
                    assert!(tracked.as_ref().unwrap().with(|_| Ok(())).is_err());
                    assert!(
                        next.reply
                            .send(Err(Kind::Authentication.error("original forgotten")))
                            .is_ok()
                    );
                }
                let result = sender.await;
                assert!(if cancel {
                    result.unwrap_err().is_cancelled()
                } else {
                    result.unwrap().is_err()
                });
                next.delivered.await.unwrap_err();
                assert!(recv.read_to_end(MAX_FRAME).await.is_err());
                drop(tracked);
                assert_eq!(buffers.live_count(), 0);
            })
            .await
            .unwrap();
            a.close().await;
            b.close().await;
        }
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP and a real heartbeat interval"]
    async fn idle_control_streams_stay_live_without_application_messages() {
        let (a, b, client, server) = connection().await;
        let (send, recv) = client.open_bi().await.unwrap();
        let mut outgoing = Control::new(send, recv);
        outgoing.send(Frame::Ack).await.unwrap();
        let (send, recv) = server.accept_bi().await.unwrap();
        let mut incoming = Control::new(send, recv);
        assert!(matches!(incoming.receive().await.unwrap(), Frame::Ack));
        // Longer than the no-progress read deadline: only heartbeats keep it alive.
        tokio::time::sleep(WAIT + Duration::from_secs(5)).await;
        incoming.send(Frame::Ack).await.unwrap();
        assert!(matches!(outgoing.receive().await.unwrap(), Frame::Ack));
        drop(outgoing);
        drop(incoming);
        a.close().await;
        b.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn remote_failures_keep_their_categories_on_control_streams() {
        use crate::mesh::failure::Kind;
        for kind in [
            Kind::Authentication,
            Kind::Version,
            Kind::Conflict,
            Kind::Publication,
            Kind::Timeout,
            Kind::Unresolved,
        ] {
            for connection_close in [false, true] {
                let (a, b, client, server) = connection().await;
                let (mut send, mut recv) = client.open_bi().await.unwrap();
                super::send(&mut send, Frame::Ack).await.unwrap();
                let (mut server_send, mut server_recv) = server.accept_bi().await.unwrap();
                assert!(matches!(
                    receive_limit(&mut server_recv, MAX_FRAME).await.unwrap(),
                    Frame::Ack
                ));
                if connection_close {
                    server.close(kind.code().into(), b"typed failure");
                } else {
                    server_send.reset(kind.code().into()).unwrap();
                }
                let error = receive_limit(&mut recv, MAX_FRAME).await.err().unwrap();
                assert_eq!(Kind::of(&error), kind);
                a.close().await;
                b.close().await;
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn heartbeats_cannot_replace_application_progress() {
        use crate::mesh::{failure::Kind, progress::Progress};
        let (a, b, client, server) = connection().await;
        let (send, recv) = client.open_bi().await.unwrap();
        let mut outgoing = Control::with_idle(send, recv, Duration::from_millis(100));
        outgoing.send(Frame::Ack).await.unwrap();
        let (send, recv) = server.accept_bi().await.unwrap();
        let mut incoming = Control::new(send, recv);
        assert!(matches!(incoming.receive().await.unwrap(), Frame::Ack));
        let local = Progress::default();
        outgoing.track_coordinated(local.clone(), false);
        // Parent progress must not become local progress and echo back indefinitely.
        incoming
            .send(Frame::Progress { revision: 1 })
            .await
            .unwrap();
        let keepalive = tokio::spawn(async move {
            for _ in 0..100 {
                incoming
                    .send(Frame::Progress { revision: 1 })
                    .await
                    .unwrap();
                incoming.send(Frame::Ack).await.unwrap();
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            incoming
        });
        let error = tokio::time::timeout(Duration::from_millis(350), async {
            loop {
                match outgoing.receive().await {
                    Ok(Frame::Ack) => (),
                    Ok(_) => panic!("unexpected control frame"),
                    Err(error) => break error,
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(Kind::of(&error), Kind::Timeout);
        assert_eq!(local.revision(), 0);
        keepalive.abort();
        let _ = keepalive.await;
        drop(outgoing);
        a.close().await;
        b.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn application_progress_extends_the_control_deadline() {
        use crate::mesh::progress::Progress;
        let (a, b, client, server) = connection().await;
        let (send, recv) = client.open_bi().await.unwrap();
        let mut outgoing = Control::with_idle(send, recv, Duration::from_millis(100));
        outgoing.send(Frame::Ack).await.unwrap();
        let (send, recv) = server.accept_bi().await.unwrap();
        let mut incoming = Control::new(send, recv);
        assert!(matches!(incoming.receive().await.unwrap(), Frame::Ack));
        let local = Progress::default();
        outgoing.track_coordinated(local.clone(), true);
        let progressing = tokio::spawn(async move {
            for revision in 1..=8 {
                incoming.send(Frame::Progress { revision }).await.unwrap();
                tokio::time::sleep(Duration::from_millis(30)).await;
            }
            incoming.send(Frame::Ack).await.unwrap();
            incoming
        });
        assert!(matches!(outgoing.receive().await.unwrap(), Frame::Ack));
        assert_eq!(local.revision(), 8);
        let incoming = progressing.await.unwrap();
        drop(incoming);
        drop(outgoing);
        a.close().await;
        b.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn coordinated_progress_is_prompt_upstream_without_a_sibling_broadcast() {
        use crate::mesh::progress::Progress;
        let (a, b, client, server) = connection().await;
        let (send, recv) = client.open_bi().await.unwrap();
        let parent = Control::new(send, recv);
        parent.send(Frame::Ack).await.unwrap();
        let (send, mut recv) = server.accept_bi().await.unwrap();
        assert!(matches!(
            receive_limit(&mut recv, MAX_FRAME).await.unwrap(),
            Frame::Ack
        ));
        let upstream = Progress::default();
        parent.track_coordinated(upstream.clone(), true);
        upstream.advance();
        assert!(
            tokio::time::timeout(
                Duration::from_millis(250),
                receive_limit(&mut recv, MAX_FRAME)
            )
            .await
            .is_err(),
            "parent activity waits for the ordinary heartbeat"
        );
        let child = Control::new(send, recv);
        let downstream = Progress::default();
        child.track_coordinated(downstream.clone(), false);
        downstream.advance();
        tokio::time::timeout(Duration::from_secs(2), async {
            while upstream.revision() < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            downstream.revision(),
            1,
            "parent progress must not echo as child activity"
        );
        drop(parent);
        drop(child);
        a.close().await;
        b.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn mixed_cohort_alpn_rejection_is_a_version_failure() {
        let current = endpoint(ALPN).await;
        let old = endpoint(b"linn/mesh/1").await;
        let address = EndpointAddr::new(old.id()).with_ip_addr(old.bound_sockets()[0]);
        let (result, rejected) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(current.connect(address, ALPN), async {
                old.accept().await.unwrap().await
            })
        })
        .await
        .unwrap();
        let error = crate::mesh::failure::Kind::connect(result.unwrap_err());
        assert_eq!(
            crate::mesh::failure::Kind::of(&error),
            crate::mesh::failure::Kind::Version,
            "{error}"
        );
        assert!(rejected.is_err());
        current.close().await;
        old.close().await;
    }

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn final_control_reply_survives_delayed_reader_and_prompt_writer_teardown() {
        let (a, b, client, server) = connection().await;
        let (send, recv) = client.open_bi().await.unwrap();
        let mut caller = Control::new(send, recv);
        caller.send(Frame::Ack).await.unwrap();
        let (send, recv) = server.accept_bi().await.unwrap();
        let mut responder = Control::new(send, recv);
        assert!(matches!(responder.receive().await.unwrap(), Frame::Ack));
        let progress = Progress::default();
        responder.track_coordinated(progress.clone(), false);
        progress.advance();
        responder
            .send(Frame::Reply {
                id: 9,
                result: Ok(crate::mesh::peer::coordinated::Reply::Done { changed: 0 }),
            })
            .await
            .unwrap();
        progress.advance();
        responder.finish().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(
            caller.receive().await.unwrap(),
            Frame::Reply {
                id: 9,
                result: Ok(_)
            }
        ));
        assert_eq!(
            Kind::of(&caller.receive().await.err().unwrap()),
            Kind::Unresolved
        );
        drop(caller);
        a.close().await;
        b.close().await;
    }
}
