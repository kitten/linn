//! Resident endpoint and explicitly delegated collection/distribution jobs.
//! @ref LLP 0008#manual-mesh-jobs
use super::{
    control_stream::Control,
    discovery::LocalLookup,
    failure::Kind,
    jobs::{Failure, Phase, Report, SignedRun},
    membership::{Membership, unhex},
    progress::Progress,
    store::{self, ControlStore, Handle, Step, Value},
    wire::{self, ALPN, Frame},
};
use crate::{
    error::{Error, Result},
    storage::mesh::Identity,
    sync::protocol::{Request, Response},
};
use iroh::{
    Endpoint, EndpointAddr, PublicKey, RelayMode, RelayUrl,
    address_lookup::memory::MemoryLookup,
    endpoint::{Connection, RecvStream, SendStream, presets},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{sync::Semaphore, task::JoinSet};

struct Entry {
    job: SignedRun,
    parent: Option<String>,
    children: BTreeSet<String>,
    phase: Phase,
    report: Report,
    touched: Instant,
    progress: Progress,
    ended: tokio::sync::watch::Sender<bool>,
}
impl Entry {
    fn serves(&self, peer: &str) -> bool {
        (self.phase == Phase::Collected && self.parent.as_deref() == Some(peer))
            || (self.phase == Phase::Distributing && self.children.contains(peer))
    }
}
pub(crate) struct Peer {
    pub endpoint: Endpoint,
    pub local_error: Option<String>,
    local: String,
    key: iroh::SecretKey,
    relay: RelayUrl,
    store: Handle,
    lookup: MemoryLookup,
    discovery: Option<LocalLookup>,
    runs: Mutex<BTreeMap<String, Entry>>,
    outgoing: Semaphore,
    incoming: Semaphore,
    handshakes: Semaphore,
    connections: Arc<Semaphore>,
    stopped: AtomicBool,
    roots: AtomicUsize,
    roots_changed: tokio::sync::Notify,
}
struct Running {
    peer: Weak<Peer>,
    run: String,
    root: bool,
    finished: bool,
}
impl Running {
    fn new(peer: &Arc<Peer>, run: &str, root: bool) -> Self {
        if root {
            peer.roots.fetch_add(1, Ordering::AcqRel);
        }
        Self {
            peer: Arc::downgrade(peer),
            run: run.into(),
            root,
            finished: false,
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Some(peer) = self.peer.upgrade() {
            if !self.finished
                && let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.run)
            {
                entry.phase = Phase::Cancelled;
                entry.touched = Instant::now();
                entry.ended.send_replace(true);
            }
            if self.root {
                peer.roots.fetch_sub(1, Ordering::AcqRel);
                peer.roots_changed.notify_waiters();
            }
        }
    }
}
struct Connected {
    connection: Connection,
    server: tokio::task::JoinHandle<()>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}
impl Drop for Connected {
    fn drop(&mut self) {
        self.connection.close(0u32.into(), b"manual link ended");
        self.server.abort();
    }
}
struct Child {
    peer: String,
    link: Arc<Connected>,
    control: Control,
}

fn protocol(message: &str) -> Error {
    Kind::Protocol.error(format!("mesh peer: {message}"))
}
fn bounded(error: &Error) -> String {
    let mut message = error.to_string();
    if message.len() > 512 {
        let mut end = 512;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
    }
    message
}
fn failure(peer: &str, phase: Phase, error: &Error) -> Failure {
    Failure {
        peer: peer.into(),
        phase,
        kind: Kind::of(error),
        message: bounded(error),
    }
}

impl Peer {
    pub(crate) async fn bind(
        store: Handle,
        identity: Identity,
        membership: Membership,
    ) -> Result<Arc<Self>> {
        let relay: RelayUrl = identity
            .service
            .parse()
            .map_err(|_| protocol("invalid relay URL"))?;
        let lookup = MemoryLookup::new();
        let endpoint = Endpoint::builder(presets::Minimal)
            .secret_key(identity.key.clone())
            .alpns(vec![ALPN.to_vec()])
            .clear_address_lookup()
            .address_lookup(lookup.clone())
            .relay_mode(RelayMode::custom([relay.clone()]))
            .bind()
            .await
            .map_err(|_| protocol("could not bind peer endpoint"))?;
        let discovery = LocalLookup::start(&endpoint, lookup.clone(), &membership);
        let local_error = discovery.as_ref().err().map(bounded);
        Ok(Arc::new(Self {
            local: endpoint.id().to_string(),
            endpoint,
            local_error,
            key: identity.key,
            relay,
            store,
            lookup,
            discovery: discovery.ok(),
            runs: Mutex::new(BTreeMap::new()),
            outgoing: Semaphore::new(2),
            incoming: Semaphore::new(2),
            handshakes: Semaphore::new(8),
            connections: Arc::new(Semaphore::new(32)),
            stopped: AtomicBool::new(false),
            roots: AtomicUsize::new(0),
            roots_changed: tokio::sync::Notify::new(),
        }))
    }

    pub(crate) async fn update(&self, membership: &Membership) -> Result<()> {
        for entry in self.runs.lock().unwrap().values_mut() {
            if !membership.members.contains(&entry.job.run.initiator)
                || entry
                    .parent
                    .as_ref()
                    .is_some_and(|parent| !membership.members.contains(parent))
            {
                entry.phase = Phase::Cancelled;
                entry.touched = Instant::now();
                entry.ended.send_replace(true);
            }
        }
        if let Some(discovery) = &self.discovery {
            discovery.update_membership(membership)?;
        }
        Ok(())
    }
    pub(crate) fn running(&self) -> bool {
        !self.stopped.load(Ordering::Acquire)
    }
    pub(crate) fn status(&self) -> super::status::Transport {
        use iroh::Watcher;
        let runs = self.runs.lock().unwrap();
        let mut jobs: Vec<_> = runs
            .iter()
            .filter(|(_, entry)| !entry.phase.is_terminal())
            .map(|(id, entry)| super::status::Job {
                id: id.clone(),
                phase: entry.phase,
                report: entry.report.clone(),
            })
            .collect();
        if let Some((id, entry)) = runs
            .iter()
            .filter(|(_, entry)| entry.phase.is_terminal())
            .max_by_key(|(_, entry)| entry.touched)
        {
            jobs.push(super::status::Job {
                id: id.clone(),
                phase: entry.phase,
                report: entry.report.clone(),
            });
        }
        super::status::Transport {
            ready: !self.stopped.load(Ordering::Acquire),
            relay_connected: self
                .endpoint
                .home_relay_status()
                .get()
                .iter()
                .any(|relay| relay.is_connected()),
            local_discovery: self.discovery.as_ref().is_some_and(LocalLookup::is_running),
            addresses: self
                .endpoint
                .bound_sockets()
                .iter()
                .map(ToString::to_string)
                .collect(),
            error: self
                .local_error
                .clone()
                .or_else(|| self.discovery.as_ref().and_then(LocalLookup::error)),
            jobs,
        }
    }
    pub(crate) async fn shutdown(&self) -> Result<()> {
        self.stopped.store(true, Ordering::Release);
        if let Some(discovery) = &self.discovery {
            discovery.stop();
        }
        self.endpoint.close().await;
        tokio::time::timeout(wire::WAIT, async {
            loop {
                let changed = self.roots_changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                if self.roots.load(Ordering::Acquire) == 0 {
                    break;
                }
                changed.await;
            }
        })
        .await
        .map_err(|_| {
            Kind::Timeout.error("mesh peer: manual jobs did not drain before the shutdown deadline")
        })?;
        Ok(())
    }

    pub(crate) async fn listen(self: Arc<Self>) {
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                incoming = self.endpoint.accept() => {
                    let Some(incoming) = incoming else { break; };
                    let Ok(permit) = self.connections.clone().try_acquire_owned() else { incoming.refuse(); continue; };
                    let peer = self.clone();
                    connections.spawn(async move {
                        let _permit = permit;
                        if let Ok(Ok(connection)) = tokio::time::timeout(wire::WAIT, incoming).await { peer.clone().serve_connection(connection, true).await; }
                    });
                }
                _ = connections.join_next(), if !connections.is_empty() => (),
            }
        }
        connections.abort_all();
        while connections.join_next().await.is_some() {}
    }

    async fn authorized(&self, peer: &str) -> Result<()> {
        self.store
            .step(Step::Peer {
                peer: peer.into(),
                request: Request::Needs { entries: vec![] },
            })
            .await?;
        Ok(())
    }
    async fn send(&self, send: &mut SendStream, frame: Frame, peer: &str) -> Result<()> {
        wire::send(send, frame, &self.store, peer).await
    }

    async fn send_proof(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
    ) -> Result<()> {
        let mut cursor = 0;
        loop {
            let Value::History(events) = self.store.step(Step::History(cursor)).await? else {
                return Err(protocol("unexpected history reply"));
            };
            let done = events.len() < 16;
            if let Some((last, _)) = events.last() {
                cursor = *last;
            }
            self.send(
                send,
                Frame::Proof {
                    events: events.into_iter().map(|(_, event)| event).collect(),
                    done,
                },
                peer,
            )
            .await?;
            if !matches!(wire::receive_limit(recv, 4096).await?, Frame::Ack) {
                return Err(protocol("history acknowledgement expected"));
            }
            if done {
                return Ok(());
            }
        }
    }
    async fn receive_proof(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
    ) -> Result<()> {
        let mut proof = super::proof::Staging::new(&self.store).await?;
        loop {
            let Frame::Proof { events, done } = wire::receive_limit(recv, 512 * 1024).await? else {
                return Err(protocol("membership proof expected"));
            };
            if events.len() > 16 || (!done && events.is_empty()) {
                return Err(protocol("invalid membership proof page"));
            }
            for event in events {
                proof.accept(&self.store, event).await?;
            }
            self.send(send, Frame::Ack, peer).await?;
            if done {
                return proof.finish();
            }
        }
    }
    async fn handshake_client(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
    ) -> Result<()> {
        let head = store::membership(&self.store, None)
            .await?
            .ok_or_else(|| Kind::Authentication.error("no trusted mesh membership"))?
            .head;
        self.send(send, Frame::Hello { version: 1, head }, peer)
            .await?;
        match wire::receive_limit(recv, 4096).await? {
            Frame::Authorized => return self.authorized(peer).await,
            Frame::Ack => (),
            _ => return Err(protocol("membership negotiation expected")),
        }
        self.send_proof(send, recv, peer).await?;
        if !matches!(wire::receive_limit(recv, 4096).await?, Frame::Authorized) {
            return Err(Kind::Authentication.error("mesh peer: membership authorization denied"));
        }
        self.receive_proof(send, recv, peer).await?;
        self.authorized(peer).await?;
        self.send(send, Frame::Authorized, peer).await
    }
    async fn handshake_server(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
        head: &str,
    ) -> Result<()> {
        unhex::<32>(head)?;
        let current = store::membership(&self.store, None)
            .await?
            .ok_or_else(|| Kind::Authentication.error("no trusted mesh membership"))?;
        if current.revoked.contains(peer) {
            return Err(Kind::Authentication.error("mesh peer: peer revoked"));
        }
        if current.head == head {
            self.authorized(peer).await?;
            return self.send(send, Frame::Authorized, peer).await;
        }
        self.send(send, Frame::Ack, peer).await?;
        self.receive_proof(send, recv, peer).await?;
        self.authorized(peer).await?;
        self.send(send, Frame::Authorized, peer).await?;
        self.send_proof(send, recv, peer).await?;
        if !matches!(wire::receive_limit(recv, 4096).await?, Frame::Authorized) {
            return Err(
                Kind::Authentication.error("mesh peer: peer did not authorize this endpoint")
            );
        }
        Ok(())
    }

    async fn connect(self: &Arc<Self>, peer: &str) -> Result<Arc<Connected>> {
        let permit = self
            .connections
            .clone()
            .try_acquire_owned()
            .map_err(|_| Kind::Capacity.error("mesh connection capacity reached"))?;
        let id = PublicKey::from_bytes(&unhex(peer)?).map_err(|_| protocol("invalid peer key"))?;
        if self.discovery.as_ref().is_some_and(LocalLookup::is_running) {
            use iroh::Watcher;
            for _ in 0..50 {
                if self.lookup.get_endpoint_info(id).is_some()
                    || self
                        .endpoint
                        .home_relay_status()
                        .get()
                        .iter()
                        .any(|relay| relay.is_connected())
                {
                    break;
                }
                if self.stopped.load(Ordering::Acquire) {
                    return Err(Kind::Cancelled.error("mesh peer: peer transport stopped"));
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        let mut last = Kind::Offline.error("peer unreachable after bounded connection attempts");
        for attempt in 0..2 {
            if self.stopped.load(Ordering::Acquire) {
                return Err(Kind::Cancelled.error("mesh peer: peer transport stopped"));
            }
            let mut address = EndpointAddr::new(id).with_relay_url(self.relay.clone());
            if let Some(info) = self.lookup.get_endpoint_info(id) {
                address.addrs.extend(info.to_endpoint_addr().addrs);
            }
            let result = tokio::time::timeout(
                Duration::from_secs(10),
                self.endpoint.connect(address, ALPN),
            )
            .await;
            if let Ok(Ok(connection)) = result {
                let server_peer = self.clone();
                let server_connection = connection.clone();
                let server = tokio::spawn(async move {
                    server_peer.serve_connection(server_connection, false).await;
                });
                return Ok(Arc::new(Connected {
                    connection,
                    server,
                    _permit: permit,
                }));
            }
            last = match result {
                Ok(Err(error)) => Kind::connect(error),
                _ => Kind::Offline.error("peer unreachable before connection deadline"),
            };
            if !matches!(Kind::of(&last), Kind::Offline | Kind::Timeout) {
                return Err(last);
            }
            if attempt == 0 {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
        Err(last)
    }

    async fn serve_connection(self: Arc<Self>, connection: Connection, inbound: bool) {
        let mut streams = JoinSet::new();
        let mut idle_since = tokio::time::Instant::now();
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(idle_since + wire::WAIT), if inbound && streams.is_empty() => {
                    connection.close(Kind::Timeout.code().into(), b"idle mesh connection"); break;
                }
                next = connection.accept_bi(), if streams.len() < 8 => {
                    let Ok((mut send, mut recv)) = next else { break; };
                    let peer = self.clone(); let connection = connection.clone();
                    idle_since = tokio::time::Instant::now();
                    streams.spawn(async move {
                        let remote = connection.remote_id().to_string();
                        let result = async {
                            let handshake = tokio::time::timeout(wire::WAIT, peer.handshakes.acquire()).await
                                .map_err(|_| Kind::Capacity.error("mesh handshake capacity reached"))?
                                .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
                            match wire::receive_limit(&mut recv, 16 * 1024).await? {
                                Frame::Hello { version: 1, head } => {
                                    tokio::time::timeout(Duration::from_secs(300), peer.handshake_server(&mut send, &mut recv, &remote, &head)).await.map_err(|_| Kind::Timeout.error("membership handshake timed out"))??;
                                    drop(handshake);
                                    let mut control = Control::new(send, recv, peer.store.clone(), remote.clone());
                                    let Frame::Begin(job) = tokio::time::timeout(wire::WAIT, control.receive()).await.map_err(|_| Kind::Timeout.error("mesh job begin timed out"))?? else { return Err(protocol("manual job begin expected")); };
                                    peer.serve_job(connection.clone(), control, job).await
                                }
                                Frame::Replica { run } => { drop(handshake); peer.serve_replica(&mut send, &mut recv, &remote, &run).await }
                                Frame::Hello { .. } => Err(Kind::Version.error("unsupported mesh version")),
                                _ => Err(protocol("unsupported mesh stream")),
                            }
                        }.await;
                        if let Err(error) = result { connection.close(Kind::of(&error).code().into(), b"mesh stream failed"); }
                    });
                }
                _ = streams.join_next(), if !streams.is_empty() => idle_since = tokio::time::Instant::now(),
            }
        }
        streams.abort_all();
        while streams.join_next().await.is_some() {}
    }

    fn register(&self, job: &SignedRun, parent: Option<&str>) -> Result<(Phase, bool)> {
        let mut runs = self.runs.lock().unwrap();
        runs.retain(|_, entry| {
            matches!(
                entry.phase,
                Phase::Collecting | Phase::Collected | Phase::Distributing
            ) || entry.touched.elapsed() < Duration::from_secs(600)
        });
        if let Some(entry) = runs.get(&job.run.id) {
            if entry.job.run != job.run || entry.job.signature != job.signature {
                return Err(
                    Kind::Conflict.error("mesh peer: manual run ID reused with different bytes")
                );
            }
            return Ok((entry.phase, true));
        }
        if runs
            .values()
            .filter(|entry| !entry.phase.is_terminal())
            .count()
            >= 4
        {
            return Err(Kind::Capacity.error("mesh peer: manual job capacity reached"));
        }
        while runs.len() >= 68 {
            let oldest = runs
                .iter()
                .filter(|(_, entry)| entry.phase.is_terminal())
                .min_by_key(|(_, entry)| entry.touched)
                .map(|(id, _)| id.clone());
            if let Some(id) = oldest {
                runs.remove(&id);
            } else {
                break;
            }
        }
        runs.insert(
            job.run.id.clone(),
            Entry {
                job: job.clone(),
                parent: parent.map(str::to_owned),
                children: BTreeSet::new(),
                phase: Phase::Collecting,
                report: Report::new(&job.run, &self.local),
                touched: Instant::now(),
                progress: Progress::default(),
                ended: tokio::sync::watch::channel(false).0,
            },
        );
        Ok((Phase::Collecting, false))
    }
    fn phase(&self, run: &str, phase: Phase, report: &mut Report) {
        if let Some(entry) = self.runs.lock().unwrap().get_mut(run) {
            // Accepted items update the run immediately, even if a later pull
            // fails before its aggregate result returns to the phase coordinator.
            report.received = entry.report.received;
            entry.phase = phase;
            entry.report = report.clone();
            entry.touched = Instant::now();
            if phase.is_terminal() {
                entry.ended.send_replace(true);
            }
        }
    }
    fn progress(&self, run: &str) -> Result<Progress> {
        self.runs
            .lock()
            .unwrap()
            .get(run)
            .map(|entry| entry.progress.clone())
            .ok_or_else(|| Kind::Authentication.error("unknown mesh job"))
    }
    async fn verify_job(&self, job: &SignedRun) -> Result<()> {
        let frozen = store::membership(&self.store, Some(job.run.head.clone()))
            .await?
            .ok_or_else(|| protocol("manual job head is not verified"))?;
        let current = store::membership(&self.store, None)
            .await?
            .ok_or_else(|| protocol("membership is not initialized"))?;
        if !job.run.cohort.contains(&self.local) {
            return Err(
                Kind::Authentication.error("mesh peer: endpoint is outside the frozen job cohort")
            );
        }
        job.verify(&frozen, &current)
    }

    async fn child(self: &Arc<Self>, peer: &str, job: &SignedRun) -> Result<Option<Child>> {
        let link = self.connect(peer).await?;
        let (mut send, mut recv) = link.connection.open_bi().await.map_err(Kind::connection)?;
        tokio::time::timeout(
            Duration::from_secs(300),
            self.handshake_client(&mut send, &mut recv, peer),
        )
        .await
        .map_err(|_| Kind::Timeout.error("membership handshake timed out"))??;
        let mut control = Control::new(send, recv, self.store.clone(), peer.into());
        control.track(self.progress(&job.run.id)?, true);
        control.send(Frame::Begin(job.clone())).await?;
        match control.receive().await? {
            Frame::Accepted {
                duplicate: true, ..
            } => Ok(None),
            Frame::Accepted {
                phase: Phase::Collecting,
                duplicate: false,
            } => Ok(Some(Child {
                peer: peer.into(),
                link,
                control,
            })),
            _ => Err(protocol("invalid manual job acceptance")),
        }
    }

    fn collect<'a>(
        self: &'a Arc<Self>,
        job: &'a SignedRun,
    ) -> Pin<Box<dyn Future<Output = (Report, Vec<Child>)> + Send + 'a>> {
        Box::pin(async move {
            let mut report = Report::new(&job.run, &self.local);
            let mut children = Vec::new();
            for peer in &job.run.cohort {
                if let Ok(progress) = self.progress(&job.run.id) {
                    progress.advance();
                }
                if self.stopped.load(Ordering::Acquire) {
                    report.failures.push(failure(
                        &self.local,
                        Phase::Cancelled,
                        &Kind::Cancelled.error("mesh peer: transport stopped"),
                    ));
                    break;
                }
                if job.path.contains(peer) {
                    continue;
                }
                if job.path.len() > usize::from(job.run.max_hops) {
                    report.failures.push(failure(
                        peer,
                        Phase::Collecting,
                        &Kind::HopLimit.error("mesh peer: propagation hop limit reached"),
                    ));
                    continue;
                }
                let result = async {
                    let Some(mut child) = self.child(peer, job).await? else {
                        return Ok(None);
                    };
                    if let Some(entry) = self.runs.lock().unwrap().get_mut(&job.run.id) {
                        entry.children.insert(peer.clone());
                    }
                    let Frame::Collected(mut child_report) = child.control.receive().await? else {
                        return Err(protocol("collection result expected"));
                    };
                    self.validate_report(&child_report, job)?;
                    child_report.received =
                        self.pull(&child.link.connection, peer, &job.run.id).await?;
                    Ok::<_, Error>(Some((child, child_report)))
                }
                .await;
                match result {
                    Ok(Some((child, child_report))) => {
                        report.received += child_report.received;
                        report.merge_collected(&child_report);
                        children.push(child);
                    }
                    Ok(None) => (),
                    Err(error) => report
                        .failures
                        .push(failure(peer, Phase::Collecting, &error)),
                }
            }
            report.normalize();
            (report, children)
        })
    }

    fn validate_report(&self, report: &Report, job: &SignedRun) -> Result<()> {
        if report.run != job.run.id
            || report.cohort != job.run.cohort
            || report.collected.len() > 256
            || report.distributed.len() > 256
            || report.failures.len() > 512
            || report.failures.iter().any(|failure| {
                !job.run.cohort.contains(&failure.peer) || failure.message.len() > 512
            })
            || report
                .collected
                .iter()
                .chain(&report.distributed)
                .any(|peer| !job.run.cohort.contains(peer))
        {
            return Err(protocol("invalid delegated job report"));
        }
        Ok(())
    }

    async fn distribute(&self, job: &SignedRun, report: &mut Report, children: &mut [Child]) {
        report.distributed.push(self.local.clone());
        for child in children {
            let result = async {
                child.control.send(Frame::Distribute).await?;
                let Frame::Completed(child_report) = child.control.receive().await? else {
                    return Err(protocol("distribution result expected"));
                };
                self.validate_report(&child_report, job)?;
                Ok(child_report)
            }
            .await;
            match result {
                Ok(child_report) => report.merge_distributed(&child_report),
                Err(error) => {
                    report
                        .failures
                        .push(failure(&child.peer, Phase::Distributing, &error))
                }
            }
        }
        report.normalize();
    }

    pub(crate) async fn sync(self: Arc<Self>, run: String) -> Result<Report> {
        self.store.step(Step::Finish).await?;
        let membership = store::membership(&self.store, None)
            .await?
            .ok_or_else(|| protocol("store is not enrolled"))?;
        let job = SignedRun::new(&self.key, &membership, run)?;
        self.verify_job(&job).await?;
        if self.stopped.load(Ordering::Acquire) {
            return Err(Kind::Cancelled.error("mesh peer: peer transport stopped"));
        }
        self.register(&job, None)?;
        let mut running = Running::new(&self, &job.run.id, true);
        let (mut report, mut children) = self.collect(&job).await;
        self.phase(&job.run.id, Phase::Distributing, &mut report);
        self.distribute(&job, &mut report, &mut children).await;
        self.store.step(Step::Finish).await?;
        self.phase(
            &job.run.id,
            if report.complete {
                Phase::Complete
            } else {
                Phase::Failed
            },
            &mut report,
        );
        running.finished = true;
        Ok(report)
    }

    async fn serve_job(
        self: &Arc<Self>,
        connection: Connection,
        mut control: Control,
        mut job: SignedRun,
    ) -> Result<()> {
        let parent = connection.remote_id().to_string();
        self.verify_job(&job).await?;
        if job.path.last() != Some(&parent) || job.path.len() > usize::from(job.run.max_hops) {
            return Err(protocol("invalid forwarding path or hop limit"));
        }
        if !job.path.contains(&self.local) {
            job.path.push(self.local.clone());
        } else if !self.runs.lock().unwrap().contains_key(&job.run.id) {
            return Err(protocol("cyclic forwarding path without a registered run"));
        }
        let (phase, duplicate) = self.register(&job, Some(&parent))?;
        control.track(self.progress(&job.run.id)?, false);
        let mut running = (!duplicate).then(|| Running::new(self, &job.run.id, false));
        control.send(Frame::Accepted { phase, duplicate }).await?;
        if duplicate {
            return Ok(());
        }
        let mut ended = self
            .runs
            .lock()
            .unwrap()
            .get(&job.run.id)
            .unwrap()
            .ended
            .subscribe();
        let result = async {
            let collection = self.collect(&job);
            tokio::pin!(collection);
            let (mut report, mut children) = loop {
                tokio::select! {
                    result = &mut collection => break result,
                    request = control.receive() => match request? {
                        Frame::Distribute => control.send(Frame::Accepted { phase: Phase::Collecting, duplicate: true }).await?,
                        _ => return Err(protocol("unexpected collection control message")),
                    }
                }
            };
            self.store.step(Step::Finish).await?;
            self.phase(&job.run.id, Phase::Collected, &mut report);
            control.send(Frame::Collected(report.clone())).await?;
            if !matches!(control.receive().await?, Frame::Distribute) {
                return Err(protocol("distribution request expected"));
            }
            self.phase(&job.run.id, Phase::Distributing, &mut report);
            {
                let distribution = async {
                    report.received += self.pull(&connection, &parent, &job.run.id).await?;
                    self.distribute(&job, &mut report, &mut children).await;
                    self.store.step(Step::Finish).await?;
                    Ok::<_, Error>(())
                };
                tokio::pin!(distribution);
                loop {
                    tokio::select! {
                        result = &mut distribution => { result?; break; }
                        request = control.receive() => match request? {
                                Frame::Distribute => control.send(Frame::Accepted { phase: Phase::Distributing, duplicate: true }).await?,
                            _ => return Err(protocol("unexpected distribution control message")),
                        }
                    }
                }
            }
            control.send(Frame::Completed(report.clone())).await?;
            self.phase(&job.run.id, Phase::Complete, &mut report);
            Ok(())
        };
        let result = tokio::select! {
            biased;
            _ = ended.changed() => Err(Kind::Authentication.error("mesh peer: manual job authorization ended")),
            result = result => result,
        };
        if let Err(error) = &result {
            control.fail(Kind::of(error)).await;
            let mut report = Report::new(&job.run, &self.local);
            report
                .failures
                .push(failure(&self.local, Phase::Cancelled, error));
            self.phase(&job.run.id, Phase::Cancelled, &mut report);
            let _ = self.store.step(Step::Finish).await;
        }
        if result.is_ok()
            && let Some(running) = &mut running
        {
            running.finished = true;
        }
        result
    }

    async fn pull(&self, connection: &Connection, peer: &str, run: &str) -> Result<u64> {
        let _permit = tokio::time::timeout(wire::WAIT, self.outgoing.acquire())
            .await
            .map_err(|_| Kind::Capacity.error("outgoing mesh transfers are busy"))?
            .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
        self.authorized(peer).await?;
        let progress = self.progress(run)?;
        progress.advance();
        let (mut send, mut recv) = connection.open_bi().await.map_err(Kind::connection)?;
        self.send(&mut send, Frame::Replica { run: run.into() }, peer)
            .await?;
        let mut changed = 0;
        let mut batch_items = 0;
        let mut batch_bytes = 0;
        let result = async {
            for tombstones in [true, false] {
                let mut after = None;
                loop {
                    self.send(
                        &mut send,
                        Frame::Request(Request::Inventory {
                            tombstones,
                            after: after.clone(),
                            scope: None,
                        }),
                        peer,
                    )
                    .await?;
                    let Frame::Response(Response::Entries { entries }) =
                        wire::receive_progress(&mut recv, &progress).await?
                    else {
                        return Err(protocol("inventory reply expected"));
                    };
                    if entries.len() > 256
                        || entries.windows(2).any(|p| p[0].id >= p[1].id)
                        || entries.iter().any(|entry| {
                            after.as_ref().is_some_and(|after| &entry.id <= after)
                                || entry.hash.is_none() != tombstones
                        })
                    {
                        return Err(protocol("invalid inventory page"));
                    }
                    let done = entries.len() < 256;
                    after = entries.last().map(|entry| entry.id.clone());
                    for entry in entries {
                        let Value::Replica(Response::Needed { ids }) = self
                            .store
                            .step(Step::Peer {
                                peer: peer.into(),
                                request: Request::Needs {
                                    entries: vec![entry.clone()],
                                },
                            })
                            .await?
                        else {
                            return Err(protocol("needs reply expected"));
                        };
                        if ids.is_empty() {
                            continue;
                        }
                        self.send(
                            &mut send,
                            Frame::Request(Request::Get {
                                id: entry.id.clone(),
                                scope: None,
                            }),
                            peer,
                        )
                        .await?;
                        let Frame::Response(Response::Item { item }) =
                            wire::receive_progress(&mut recv, &progress).await?
                        else {
                            return Err(protocol("item reply expected"));
                        };
                        let Some(item) = item else {
                            continue;
                        };
                        if item.id != entry.id || item.envelope_hash != entry.hash {
                            return Err(
                                Kind::Conflict.error("mesh peer: item does not match inventory")
                            );
                        }
                        let item_bytes = item.envelope.as_ref().map_or(0, String::len);
                        let Value::Replica(Response::Applied { changed: applied }) = self
                            .store
                            .step(Step::Peer {
                                peer: peer.into(),
                                request: Request::Apply { item, scope: None },
                            })
                            .await?
                        else {
                            return Err(protocol("apply reply expected"));
                        };
                        changed += u64::from(applied);
                        if applied && let Some(entry) = self.runs.lock().unwrap().get_mut(run) {
                            entry.report.received += 1;
                        }
                        batch_items += usize::from(applied);
                        batch_bytes += if applied { item_bytes } else { 0 };
                        if batch_items >= 256 || batch_bytes >= 4 * 1024 * 1024 {
                            self.store.step(Step::Finish).await?;
                            batch_items = 0;
                            batch_bytes = 0;
                        }
                    }
                    if done {
                        break;
                    }
                }
            }
            self.send(&mut send, Frame::Request(Request::Finish), peer)
                .await?;
            if !matches!(
                wire::receive_progress(&mut recv, &progress).await?,
                Frame::Response(Response::Done)
            ) {
                return Err(protocol("replica completion expected"));
            }
            Ok(())
        }
        .await;
        // Accepted originals publish even when a later item or branch failed.
        let published = self.store.step(Step::Finish).await;
        published?;
        result?;
        Ok(changed)
    }

    async fn serve_replica(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
        run: &str,
    ) -> Result<()> {
        let _permit = tokio::time::timeout(wire::WAIT, self.incoming.acquire())
            .await
            .map_err(|_| Kind::Capacity.error("incoming mesh transfers are busy"))?
            .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
        self.authorized(peer).await?;
        let progress = self.progress(run)?;
        let mut ended = {
            let runs = self.runs.lock().unwrap();
            let entry = runs
                .get(run)
                .ok_or_else(|| Kind::Authentication.error("mesh peer: unknown manual run"))?;
            if !entry.serves(peer) {
                return Err(Kind::Authentication
                    .error("mesh peer: replica request is outside the active job phase"));
            }
            entry.ended.subscribe()
        };
        if *ended.borrow() {
            return Err(Kind::Cancelled.error("mesh peer: manual job ended"));
        }
        self.store.step(Step::Finish).await?;
        let Value::SourceBudget(mut remaining) =
            self.store.step(Step::SourceBudget(peer.into())).await?
        else {
            return Err(protocol("source budget expected"));
        };
        let serve = async {
            loop {
                let Frame::Request(request) = wire::receive(recv).await? else {
                    return Err(protocol("replica request expected"));
                };
                {
                    let runs = self.runs.lock().unwrap();
                    let entry = runs.get(run).ok_or_else(|| {
                        Kind::Authentication.error("mesh peer: unknown manual run")
                    })?;
                    if !entry.serves(peer) {
                        return Err(Kind::Authentication
                            .error("mesh peer: replica request is outside the active job phase"));
                    }
                }
                if !matches!(
                    request,
                    Request::Inventory { scope: None, .. }
                        | Request::Get { scope: None, .. }
                        | Request::Finish
                ) {
                    return Err(protocol("mesh source streams are read-only"));
                }
                let inventory = match &request {
                    Request::Inventory { tombstones, .. } => Some(usize::from(*tombstones)),
                    _ => None,
                };
                let finish = matches!(request, Request::Finish);
                let Value::Replica(mut response) = self
                    .store
                    .step(Step::Peer {
                        peer: peer.into(),
                        request,
                    })
                    .await?
                else {
                    return Err(protocol("unexpected source replica reply"));
                };
                if let (Some(class), Response::Entries { entries }) = (inventory, &mut response) {
                    entries.truncate(remaining[class].min(entries.len() as u64) as usize);
                    remaining[class] -= entries.len() as u64;
                }
                wire::send_progress(
                    send,
                    Frame::Response(response),
                    &self.store,
                    peer,
                    &progress,
                )
                .await?;
                if finish {
                    return Ok(());
                }
            }
        };
        tokio::select! {
            biased;
            _ = ended.changed() => Err(Kind::Cancelled.error("mesh peer: manual job ended")),
            result = serve => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replica_sources_are_bound_to_the_current_phase_and_tree_role() {
        let job = SignedRun {
            run: super::super::jobs::Run {
                version: 1,
                id: "run".into(),
                initiator: "root".into(),
                head: "head".into(),
                cohort: vec![],
                max_hops: 16,
            },
            signature: String::new(),
            path: vec![],
        };
        let mut entry = Entry {
            report: Report::new(&job.run, "local"),
            job,
            parent: Some("parent".into()),
            children: BTreeSet::from(["child".into()]),
            phase: Phase::Collecting,
            touched: Instant::now(),
            progress: Progress::default(),
            ended: tokio::sync::watch::channel(false).0,
        };
        for phase in [
            Phase::Collecting,
            Phase::Collected,
            Phase::Distributing,
            Phase::Complete,
            Phase::Failed,
            Phase::Cancelled,
        ] {
            entry.phase = phase;
            assert_eq!(entry.serves("parent"), phase == Phase::Collected);
            assert_eq!(entry.serves("child"), phase == Phase::Distributing);
            assert!(!entry.serves("other"));
        }
    }
    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn equal_heads_skip_proofs_without_authorizing_an_unknown_endpoint() {
        let fixtures: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../tests/fixtures/mesh-membership-v1.json"))
                .unwrap();
        let membership: Membership = serde_json::from_value(fixtures[2]["state"].clone()).unwrap();
        let (store, mut jobs) = store::channel();
        let state = membership.clone();
        let dispatcher = tokio::spawn(async move {
            while let Some(job) = jobs.recv().await {
                let result = match job.step {
                    Step::Membership(None) => Ok(Value::Membership(Some(state.clone()))),
                    Step::Peer {
                        peer,
                        request: Request::Needs { entries },
                    } if entries.is_empty() => {
                        if state.members.contains(&peer) {
                            Ok(Value::Replica(Response::Needed { ids: vec![] }))
                        } else {
                            Err(Kind::Authentication.error("unknown peer"))
                        }
                    }
                    _ => panic!("equal heads must not read or import proof history"),
                };
                let _ = job.reply.send(result);
            }
        });
        let identity = Identity {
            key: iroh::SecretKey::from_bytes(&[1; 32]),
            service: "http://127.0.0.1:1".into(),
            receipt_key: Some(membership.receipt_key.clone()),
            head: Some(membership.head.clone()),
            enabled: true,
            pending_leave: false,
            conflict: false,
        };
        let peer = Peer::bind(store.clone(), identity, membership.clone())
            .await
            .unwrap();
        let port = peer
            .endpoint
            .bound_sockets()
            .iter()
            .find(|s| s.is_ipv4())
            .unwrap()
            .port();
        for known in [true, false] {
            let key = if known {
                iroh::SecretKey::from_bytes(&[2; 32])
            } else {
                iroh::SecretKey::generate()
            };
            let remote = Endpoint::builder(presets::Minimal)
                .secret_key(key)
                .clear_ip_transports()
                .bind_addr("127.0.0.1:0")
                .unwrap()
                .clear_address_lookup()
                .relay_mode(RelayMode::Disabled)
                .alpns(vec![ALPN.to_vec()])
                .bind()
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                let (client, server) = tokio::join!(
                    remote.connect(
                        EndpointAddr::new(peer.endpoint.id())
                            .with_ip_addr((std::net::Ipv4Addr::LOCALHOST, port).into()),
                        ALPN
                    ),
                    async { peer.endpoint.accept().await.unwrap().await }
                );
                let client = client.unwrap();
                let server = server.unwrap();
                let (mut send, mut recv) = client.open_bi().await.unwrap();
                wire::send(
                    &mut send,
                    Frame::Hello {
                        version: 1,
                        head: membership.head.clone(),
                    },
                    &store,
                    &peer.local,
                )
                .await
                .unwrap();
                let (mut reply, mut request) = server.accept_bi().await.unwrap();
                let Frame::Hello { head, .. } = wire::receive(&mut request).await.unwrap() else {
                    panic!("hello expected");
                };
                let outcome = peer
                    .handshake_server(&mut reply, &mut request, &remote.id().to_string(), &head)
                    .await;
                if known {
                    outcome.unwrap();
                    assert!(matches!(
                        wire::receive(&mut recv).await.unwrap(),
                        Frame::Authorized
                    ));
                } else {
                    assert_eq!(Kind::of(&outcome.err().unwrap()), Kind::Authentication);
                }
            })
            .await
            .unwrap();
            remote.close().await;
        }
        peer.shutdown().await.unwrap();
        dispatcher.abort();
        let _ = dispatcher.await;
    }
}
