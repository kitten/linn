//! Passive peer transport lifecycle; it never schedules a root record job.
use crate::{
    error::{Error, Result},
    mesh::{
        peer::Peer,
        status::Transport,
        store::{self, ControlStore, Step},
    },
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::{
    sync::{Notify, mpsc, oneshot},
    task::JoinHandle,
};

enum Command {
    Reload {
        reply: oneshot::Sender<Result<Option<Arc<Peer>>>>,
    },
    Suspend {
        reply: oneshot::Sender<Result<()>>,
    },
    Quit {
        reply: oneshot::Sender<Result<()>>,
    },
}
#[derive(Default)]
struct View {
    peer: Option<Arc<Peer>>,
    error: Option<String>,
}
#[derive(Clone)]
pub(super) struct Driver {
    commands: mpsc::Sender<Command>,
    reload: Arc<Notify>,
    resume_requested: Arc<AtomicBool>,
    view: Arc<Mutex<View>>,
    cancellation: Arc<Cancellation>,
    control: Arc<tokio::sync::Mutex<()>>,
}
struct Cancellation {
    gate: AtomicBool,
    quitting: AtomicBool,
    active: AtomicUsize,
    changed: Notify,
    epoch: tokio::sync::watch::Sender<u64>,
}
pub(super) struct Work {
    cancellation: Arc<Cancellation>,
}
impl Drop for Work {
    fn drop(&mut self) {
        self.cancellation.active.fetch_sub(1, Ordering::AcqRel);
        self.cancellation.changed.notify_waiters();
    }
}
impl Cancellation {
    async fn drained(&self) -> Result<()> {
        tokio::time::timeout(crate::runtime::WAIT, async {
            loop {
                let changed = self.changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                if self.active.load(Ordering::Acquire) == 0 {
                    break;
                }
                changed.await;
            }
        })
        .await
        .map_err(|_| {
            Error::Invalid("mesh network jobs did not drain before identity change".into())
        })
    }
    fn cancel(&self) {
        self.gate.store(true, Ordering::Release);
        self.epoch
            .send_modify(|epoch| *epoch = epoch.wrapping_add(1));
    }
}
struct Owner {
    store: crate::mesh::store::Handle,
    allowed: bool,
    peer: Option<Arc<Peer>>,
    listener: Option<JoinHandle<()>>,
    view: Arc<Mutex<View>>,
}
impl Driver {
    pub(super) fn start(
        store: crate::mesh::store::Handle,
        network: &tokio::runtime::Handle,
        startup: bool,
    ) -> (Self, JoinHandle<Result<()>>) {
        let (commands, mut requests) = mpsc::channel(16);
        let reload = Arc::new(Notify::new());
        let resume_requested = Arc::new(AtomicBool::new(false));
        let view = Arc::new(Mutex::new(View::default()));
        let cancellation = Arc::new(Cancellation {
            gate: AtomicBool::new(false),
            quitting: AtomicBool::new(false),
            active: AtomicUsize::new(0),
            changed: Notify::new(),
            epoch: tokio::sync::watch::channel(0).0,
        });
        let result = Self {
            commands,
            reload: reload.clone(),
            resume_requested: resume_requested.clone(),
            view: view.clone(),
            cancellation: cancellation.clone(),
            control: Arc::new(tokio::sync::Mutex::new(())),
        };
        let task = network.spawn(async move {
            let mut owner = Owner {
                store,
                allowed: startup,
                peer: None,
                listener: None,
                view,
            };
            if startup {
                let _ = owner.reload(false).await;
            }
            loop {
                tokio::select! {
                    _ = reload.notified() => {
                        let _ = owner.reload(resume_requested.swap(false, Ordering::AcqRel)).await;
                    }
                    request = requests.recv() => {
                        let Some(request) = request else { return owner.close().await; };
                        if matches!(request, Command::Suspend { .. } | Command::Quit { .. }) {
                            resume_requested.store(false, Ordering::Release);
                        }
                        match request {
                            Command::Reload { reply } => {
                                if !reply.is_closed() {
                                    let resume = resume_requested.swap(false, Ordering::AcqRel);
                                    let _ = reply.send(owner.reload(resume).await);
                                }
                            }
                            Command::Suspend { reply } => {
                                let _ = reply.send(owner.suspend(&cancellation).await);
                            }
                            Command::Quit { reply } => {
                                let _ = reply.send(owner.suspend(&cancellation).await);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        });
        (result, task)
    }
    pub(super) fn changed(&self) {
        self.reload.notify_one();
    }
    pub(super) fn resume(&self) {
        self.resume_requested.store(true, Ordering::Release);
        self.changed();
    }
    pub(super) async fn control(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.control.clone().lock_owned().await
    }
    pub(super) fn cancellation(&self) -> tokio::sync::watch::Receiver<u64> {
        self.cancellation.epoch.subscribe()
    }
    pub(super) fn begin_work(&self) -> Result<Work> {
        if self.cancellation.gate.load(Ordering::Acquire) {
            return Err(Error::Invalid(
                "mesh transport is stopping; request not executed".into(),
            ));
        }
        self.cancellation.active.fetch_add(1, Ordering::AcqRel);
        let work = Work {
            cancellation: self.cancellation.clone(),
        };
        if self.cancellation.gate.load(Ordering::Acquire) {
            return Err(Error::Invalid(
                "mesh transport is stopping; request not executed".into(),
            ));
        }
        Ok(work)
    }
    pub(super) fn release_gate(&self) {
        if !self.cancellation.quitting.load(Ordering::Acquire) {
            self.cancellation.gate.store(false, Ordering::Release);
        }
    }
    pub(super) fn status(&self) -> Transport {
        let view = self.view.lock().unwrap();
        if let Some(peer) = &view.peer {
            let mut status = peer.status();
            if view.error.is_some() {
                status.error = view.error.clone();
            }
            status
        } else {
            Transport {
                error: view.error.clone(),
                ..Default::default()
            }
        }
    }
    pub(super) async fn ensure(&self) -> Result<Option<Arc<Peer>>> {
        let (reply, result) = oneshot::channel();
        self.commands
            .send(Command::Reload { reply })
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?;
        result
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?
    }
    pub(super) async fn suspend(&self) -> Result<()> {
        self.cancellation.cancel();
        let (reply, result) = oneshot::channel();
        self.commands
            .send(Command::Suspend { reply })
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?;
        result
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?
    }
    pub(super) async fn quit(&self) -> Result<()> {
        self.cancellation.quitting.store(true, Ordering::Release);
        self.cancellation.cancel();
        let (reply, result) = oneshot::channel();
        self.commands
            .send(Command::Quit { reply })
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?;
        result
            .await
            .map_err(|_| Error::Invalid("peer transport stopped".into()))?
    }
}
impl Owner {
    async fn suspend(&mut self, cancellation: &Cancellation) -> Result<()> {
        self.allowed = false;
        self.close().await?;
        cancellation.drained().await
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(peer) = &self.peer {
            let result = peer.shutdown().await;
            if let Some(listener) = self.listener.take() {
                listener.abort();
                let _ = listener.await;
            }
            result?;
            self.store.step(Step::Finish).await?;
        }
        self.peer = None;
        self.view.lock().unwrap().peer = None;
        Ok(())
    }
    async fn reload(&mut self, resume: bool) -> Result<Option<Arc<Peer>>> {
        self.allowed |= resume;
        let result = self.reload_inner().await;
        if let Err(error) = &result {
            self.view.lock().unwrap().error = Some(error.to_string().chars().take(256).collect());
            let _ = self.close().await;
        } else {
            self.view.lock().unwrap().error = None;
        }
        result
    }
    async fn reload_inner(&mut self) -> Result<Option<Arc<Peer>>> {
        if !self.allowed {
            self.close().await?;
            return Ok(None);
        }
        let identity = store::identity(&self.store).await?;
        let Some(identity) = identity.filter(|id| id.enabled && !id.pending_leave && !id.conflict)
        else {
            self.close().await?;
            return Ok(None);
        };
        let state = store::membership(&self.store, None)
            .await?
            .ok_or_else(|| Error::Invalid("missing verified mesh state".into()))?;
        if let Some(peer) = &self.peer
            && peer.running()
            && peer.endpoint.id() == identity.key.public()
        {
            peer.update(&state).await?;
            return Ok(Some(peer.clone()));
        }
        self.close().await?;
        let peer = Peer::bind(self.store.clone(), identity, state).await?;
        self.listener = Some(tokio::spawn(peer.clone().listen()));
        self.peer = Some(peer.clone());
        self.view.lock().unwrap().peer = Some(peer.clone());
        Ok(Some(peer))
    }
}
