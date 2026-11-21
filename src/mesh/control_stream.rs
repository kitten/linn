//! Independent control polling keeps parent loss and heartbeats observable while
//! original-item streams or delegated children are busy.
use super::{
    failure::Kind,
    progress::Progress,
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
    Finish(oneshot::Sender<Result<()>>),
}
#[derive(Clone)]
struct Tracker {
    progress: Progress,
    forward: bool,
    prompt: bool,
}
pub(crate) struct Control {
    outgoing: mpsc::Sender<Write>,
    incoming: mpsc::Receiver<Result<Frame>>,
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    tracker: Arc<Mutex<Option<Tracker>>>,
    remote: Arc<AtomicU64>,
    tracked: Arc<tokio::sync::Notify>,
    observed: (u64, u64),
    last_progress: Instant,
    idle: Duration,
}
impl Control {
    pub(crate) fn new(send: SendStream, recv: RecvStream) -> Self {
        Self::with_idle(send, recv, Duration::from_secs(60))
    }
    pub(crate) fn with_idle(mut send: SendStream, mut recv: RecvStream, idle: Duration) -> Self {
        let tracker: Arc<Mutex<Option<Tracker>>> = Arc::default();
        let remote = Arc::new(AtomicU64::new(0));
        let tracked = Arc::new(tokio::sync::Notify::new());
        let outgoing_tracked = tracked.clone();
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
            let mut sent_revision = 0;
            let mut sent_at = tokio::time::Instant::now() - Duration::from_millis(100);
            loop {
                let tracking = outgoing_tracker.lock().unwrap().clone();
                let activity = async {
                    if let Some(tracker) = &tracking
                        && tracker.prompt
                    {
                        tracker.progress.changed_since(sent_revision).await;
                        tokio::time::sleep_until(sent_at + Duration::from_millis(100)).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                };
                let write = tokio::select! {
                    _ = outgoing_tracked.notified() => continue,
                    _ = activity => {
                        Write::Frame(Frame::Progress { revision: tracking.as_ref().unwrap().progress.revision() }, None)
                    }
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
                    Write::Finish(reply) => {
                        let result = send
                            .finish()
                            .map_err(|error| Kind::Unresolved.error(error.to_string()));
                        let _ = reply.send(result);
                        return;
                    }
                };
                if let Frame::Progress { revision } = &frame {
                    sent_revision = *revision;
                    sent_at = tokio::time::Instant::now();
                }
                let result = wire::send(&mut send, frame).await;
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
                        let error =
                            Kind::of(&error).error(format!("mesh control response: {error}"));
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
            tracked,
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
    pub(crate) async fn finish(self) -> Result<()> {
        let (reply, result) = oneshot::channel();
        self.outgoing
            .send(Write::Finish(reply))
            .await
            .map_err(|_| {
                Kind::Unresolved.error("mesh control writer stopped before finalization")
            })?;
        result
            .await
            .map_err(|_| Kind::Unresolved.error("mesh control finalization stopped"))?
    }
    pub(crate) fn track_coordinated(&self, progress: Progress, forward: bool) {
        // Prompt activity goes child-to-parent. Downstream siblings keep the
        // ordinary heartbeat instead of rebroadcasting each root revision.
        self.track_mode(progress, forward, !forward);
    }
    fn track_mode(&self, progress: Progress, forward: bool, prompt: bool) {
        *self.tracker.lock().unwrap() = Some(Tracker {
            progress,
            forward,
            prompt,
        });
        self.tracked.notify_one();
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
                    return frame.ok_or_else(|| Kind::Unresolved.error("mesh control stream ended"))?;
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
