//! Independent control polling keeps parent loss and heartbeats observable while
//! original-item streams or delegated children are busy.
use super::{
    failure::Kind,
    progress::Progress,
    store::Handle,
    wire::{self, Frame},
};
use crate::error::{Error, Result};
use iroh::endpoint::{RecvStream, SendStream};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

enum Write {
    Frame(Frame, Option<oneshot::Sender<Result<()>>>),
    Reset(Kind, oneshot::Sender<()>),
}
#[derive(Clone)]
struct Tracker {
    progress: Progress,
    forward: bool,
}
pub(crate) struct Control {
    outgoing: mpsc::Sender<Write>,
    incoming: mpsc::Receiver<Result<Frame>>,
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    tracker: Arc<Mutex<Option<Tracker>>>,
    remote: Arc<AtomicU64>,
    observed: (u64, u64),
    last_progress: Instant,
    idle: Duration,
}
impl Control {
    pub(crate) fn new(send: SendStream, recv: RecvStream, store: Handle, peer: String) -> Self {
        Self::with_idle(send, recv, store, peer, Duration::from_secs(60))
    }
    pub(crate) fn with_idle(
        mut send: SendStream,
        mut recv: RecvStream,
        store: Handle,
        peer: String,
        idle: Duration,
    ) -> Self {
        let tracker: Arc<Mutex<Option<Tracker>>> = Arc::default();
        let remote = Arc::new(AtomicU64::new(0));
        let outgoing_tracker = tracker.clone();
        let incoming_tracker = tracker.clone();
        let incoming_remote = remote.clone();
        let (outgoing, mut writes) = mpsc::channel::<Write>(2);
        let (reads, incoming) = mpsc::channel(2);
        let writer = tokio::spawn(async move {
            let mut heartbeat = tokio::time::interval_at(
                tokio::time::Instant::now() + Duration::from_secs(10),
                Duration::from_secs(10),
            );
            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                let write = tokio::select! {
                    write = writes.recv() => match write { Some(write) => write, None => break },
                    _ = heartbeat.tick() => {
                        let revision = outgoing_tracker.lock().unwrap().as_ref()
                            .map_or(0, |tracker| tracker.progress.revision());
                        let frame = Frame::Progress { revision };
                        Write::Frame(frame, None)
                    },
                };
                let (frame, reply) = match write {
                    Write::Frame(frame, reply) => (frame, reply),
                    Write::Reset(kind, reply) => {
                        let _ = send.reset(kind.code().into());
                        let _ = reply.send(());
                        break;
                    }
                };
                let result = wire::send(&mut send, frame, &store, &peer).await;
                let failed = result.is_err();
                if let Some(reply) = reply {
                    let _ = reply.send(result);
                }
                if failed {
                    break;
                }
            }
            let _ = send.reset(0u32.into());
        });
        let reader = tokio::spawn(async move {
            loop {
                match wire::receive_limit(&mut recv, 512 * 1024).await {
                    Ok(Frame::Progress { revision }) => {
                        let previous = incoming_remote.fetch_max(revision, Ordering::Relaxed);
                        if revision > previous
                            && let Some(tracker) = incoming_tracker.lock().unwrap().as_ref()
                            && tracker.forward
                        {
                            // Only child-to-parent progress propagates through the tree.
                            // Echoing parent progress would create a keepalive feedback loop.
                            tracker.progress.advance();
                        }
                    }
                    Ok(frame) => {
                        if reads.send(Ok(frame)).await.is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = reads.send(Err(error)).await;
                        break;
                    }
                }
            }
            let _ = recv.stop(0u32.into());
        });
        Self {
            outgoing,
            incoming,
            reader,
            writer,
            tracker,
            remote,
            observed: (0, 0),
            last_progress: Instant::now(),
            idle,
        }
    }
    pub(crate) async fn send(&self, frame: Frame) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.outgoing
            .send(Write::Frame(frame, Some(reply)))
            .await
            .map_err(|_| Error::Invalid("mesh control writer stopped".into()))?;
        result
            .await
            .map_err(|_| Error::Invalid("mesh control writer stopped".into()))?
    }
    pub(crate) async fn fail(&self, kind: Kind) {
        let (reply, result) = oneshot::channel();
        if self.outgoing.send(Write::Reset(kind, reply)).await.is_ok() {
            let _ = result.await;
        }
    }
    pub(crate) fn track(&self, progress: Progress, forward: bool) {
        *self.tracker.lock().unwrap() = Some(Tracker { progress, forward });
    }
    pub(crate) async fn receive(&mut self) -> Result<Frame> {
        let mut tick = tokio::time::interval(self.idle.min(Duration::from_secs(1)));
        loop {
            let local = self
                .tracker
                .lock()
                .unwrap()
                .as_ref()
                .map_or(0, |t| t.progress.revision());
            let observed = (local, self.remote.load(Ordering::Relaxed));
            if observed != self.observed {
                self.observed = observed;
                self.last_progress = Instant::now();
            } else if self.last_progress.elapsed() >= self.idle {
                return Err(Kind::Timeout.error("mesh job made no application progress"));
            }
            tokio::select! {
                frame = self.incoming.recv() => {
                    return frame.ok_or_else(|| Kind::Offline.error("mesh control stream ended"))?;
                }
                _ = tick.tick() => (),
            }
        }
    }
}
impl Drop for Control {
    fn drop(&mut self) {
        self.reader.abort();
        self.writer.abort();
    }
}
