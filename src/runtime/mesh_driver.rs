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
                listener
                    .await
                    .map_err(|_| Error::Invalid("mesh listener did not drain".into()))?;
            }
            result?;
            self.store.step(Step::Finish(None)).await?;
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
        // The membership reply may include revocation committed after the identity read.
        let local = identity.key.public().to_string();
        if !state.members.contains(&local) || state.revoked.contains(&local) {
            self.close().await?;
            return Ok(None);
        }
        if let Some(peer) = &self.peer
            && peer.running()
            && peer.endpoint.id() == identity.key.public()
            && peer.uses_service(&identity.service)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mesh::membership::Membership, mesh::store::Value, storage::mesh::Identity};
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test]
    async fn reload_checks_membership_after_reading_identity() {
        for case in 0..3 {
            let key = iroh::SecretKey::from_bytes(&[71; 32]);
            let local = key.public().to_string();
            let other = iroh::SecretKey::from_bytes(&[72; 32]).public().to_string();
            let service = if case == 2 {
                "http://["
            } else {
                "http://127.0.0.1:1"
            };
            let membership = Membership {
                head: "1".repeat(64),
                service: service.into(),
                receipt_key: "0".repeat(64),
                roots: BTreeSet::from(["2".repeat(64)]),
                members: match case {
                    0 => BTreeSet::new(),
                    1 => BTreeSet::from([other]),
                    _ => BTreeSet::from([local.clone()]),
                },
                revoked: if case < 2 {
                    BTreeSet::from([local])
                } else {
                    BTreeSet::new()
                },
                invitations: BTreeMap::new(),
            };
            let (store, mut requests) = store::channel();
            // A commit between the two requests can return these coherent snapshots.
            let worker = tokio::spawn(async move {
                let mut observed = Vec::new();
                while let Some(request) = requests.recv().await {
                    let value = match request.step {
                        Step::Identity => {
                            observed.push("identity");
                            Ok(Value::Identity(Some(Identity {
                                key: key.clone(),
                                service: service.into(),
                                receipt_key: Some("0".repeat(64)),
                                head: Some("0".repeat(64)),
                                enabled: true,
                                pending_leave: false,
                                conflict: false,
                            })))
                        }
                        Step::Membership(None) => {
                            observed.push("membership");
                            Ok(Value::Membership(Some(membership.clone())))
                        }
                        _ => {
                            observed.push("unexpected");
                            Err(Error::Invalid("unexpected reload store request".into()))
                        }
                    };
                    let _ = request.reply.send(value);
                }
                observed
            });
            let mut owner = Owner {
                store,
                allowed: true,
                peer: None,
                listener: None,
                view: Arc::new(Mutex::new(View::default())),
            };
            let result = owner.reload(false).await;
            if case < 2 {
                assert!(result.is_ok_and(|peer| peer.is_none()));
            } else {
                // Local membership must reach binding; an invalid URL stops before I/O.
                assert!(matches!(result, Err(Error::Mesh(error))
                    if error.kind == crate::mesh::failure::Kind::Protocol
                    && error.message == "mesh peer: invalid relay URL"));
            }
            assert!(owner.peer.is_none());
            drop(owner);
            assert_eq!(worker.await.unwrap(), ["identity", "membership"]);
        }
    }
}
