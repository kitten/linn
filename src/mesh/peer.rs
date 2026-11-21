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
    Endpoint, EndpointAddr, NetReportConfig, PublicKey, RelayConfig, RelayMap, RelayMode, RelayUrl,
    address_lookup::memory::MemoryLookup,
    endpoint::{Connection, RecvStream, SendStream, presets},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{sync::Semaphore, task::JoinSet};

mod capacity;
pub(super) mod coordinated;
mod pipeline;
#[cfg(test)]
#[path = "../../tests/support/relay.rs"]
mod relay_fixture;
mod replica;
pub(crate) mod source;
mod startup;
pub(crate) use startup::Startup;

struct Entry {
    job: SignedRun,
    parent: Option<String>,
    children: BTreeSet<String>,
    phase: Phase,
    report: Report,
    progress: Progress,
    ended: tokio::sync::watch::Sender<bool>,
    reads: BTreeMap<String, Arc<source::Grant>>,
    coordinated: Option<Arc<coordinated::Node>>,
    terminal_only: bool,
    data_round: u8,
    transfers: usize,
}
impl Drop for Entry {
    fn drop(&mut self) {
        self.revoke_sources();
    }
}
impl Entry {
    fn advance_round(&mut self, round: u8) -> Result<()> {
        if !matches!(round, 1 | 2)
            || round < self.data_round
            || (round > self.data_round
                && (self.transfers != 0 || self.reads.values().any(|grant| grant.active())))
        {
            return Err(protocol(
                "data rounds cannot overlap or replay an earlier phase",
            ));
        }
        if round > self.data_round {
            self.revoke_sources();
            self.data_round = round;
        }
        Ok(())
    }
    fn revoke_sources(&mut self) {
        for grant in self.reads.values() {
            grant.revoke();
        }
        self.reads.clear();
    }
    fn serves_round(&self, peer: &str, round: u8) -> bool {
        matches!(round, 1 | 2)
            && !self.terminal_only
            && self
                .reads
                .get(peer)
                .is_some_and(|grant| grant.round == round)
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
    outgoing: capacity::Transfers,
    incoming: Semaphore,
    handshakes: Semaphore,
    connections: Arc<super::connections::Connections>,
    stopped: AtomicBool,
    roots: AtomicUsize,
    roots_changed: tokio::sync::Notify,
    connection_idle: tokio::sync::Notify,
    readers: tokio::sync::Mutex<JoinSet<()>>,
    #[cfg(test)]
    replica_rounds: [AtomicUsize; 2],
    #[cfg(test)]
    delta_transfers: [AtomicUsize; 2],
}
struct Running {
    peer: Weak<Peer>,
    run: String,
    root: bool,
    retire: bool,
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
            retire: true,
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if !self.retire {
            return;
        }
        if let Some(peer) = self.peer.upgrade() {
            if let Some(entry) = peer.runs.lock().unwrap().remove(&self.run) {
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
    admission_deadline: tokio::time::Instant,
    connection: Connection,
    server: Option<tokio::task::JoinHandle<()>>,
    _permit: super::connections::Permit,
}
impl Drop for Connected {
    fn drop(&mut self) {
        self.connection.close(0u32.into(), b"manual link ended");
        if let Some(server) = &self.server {
            server.abort();
        }
    }
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
            .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
                relay.clone(),
                None,
            )])))
            .net_report_config(NetReportConfig::minimal())
            .bind()
            .await
            .map_err(|_| protocol("could not bind peer endpoint"))?;
        Ok(Self::with_endpoint(
            store, identity, membership, endpoint, lookup, relay,
        ))
    }
    fn with_endpoint(
        store: Handle,
        identity: Identity,
        membership: Membership,
        endpoint: Endpoint,
        lookup: MemoryLookup,
        relay: RelayUrl,
    ) -> Arc<Self> {
        let discovery = LocalLookup::start(&endpoint, lookup.clone(), &membership);
        let local_error = discovery.as_ref().err().map(bounded);
        Arc::new(Self {
            local: endpoint.id().to_string(),
            endpoint,
            local_error,
            key: identity.key,
            relay,
            store,
            lookup,
            discovery: discovery.ok(),
            runs: Mutex::new(BTreeMap::new()),
            outgoing: capacity::Transfers::new(),
            incoming: Semaphore::new(2),
            handshakes: Semaphore::new(8),
            connections: super::connections::Connections::new(membership.members.len()),
            stopped: AtomicBool::new(false),
            roots: AtomicUsize::new(0),
            roots_changed: tokio::sync::Notify::new(),
            connection_idle: tokio::sync::Notify::new(),
            readers: tokio::sync::Mutex::new(JoinSet::new()),
            #[cfg(test)]
            replica_rounds: std::array::from_fn(|_| AtomicUsize::new(0)),
            #[cfg(test)]
            delta_transfers: std::array::from_fn(|_| AtomicUsize::new(0)),
        })
    }

    pub(crate) async fn update(&self, membership: &Membership) -> Result<()> {
        {
            let mut runs = self.runs.lock().unwrap();
            for entry in runs.values_mut() {
                if !membership.members.contains(&entry.job.run.initiator)
                    || entry
                        .parent
                        .as_ref()
                        .is_some_and(|parent| !membership.members.contains(parent))
                {
                    entry.phase = Phase::Cancelled;
                    entry.revoke_sources();
                    entry.ended.send_replace(true);
                }
            }
            runs.retain(|_, entry| {
                !(*entry.ended.borrow()
                    && entry
                        .coordinated
                        .as_ref()
                        .is_some_and(|node| node.idle_stamp().is_some()))
            });
        }
        self.connections.membership(membership.members.len());
        while self.connections.excess() && coordinated::release_idle(self).await {}
        if let Some(discovery) = &self.discovery {
            discovery.update_membership(membership)?;
        }
        Ok(())
    }
    pub(crate) fn uses_service(&self, service: &str) -> bool {
        service
            .parse::<RelayUrl>()
            .is_ok_and(|relay| relay == self.relay)
    }
    pub(crate) fn running(&self) -> bool {
        !self.stopped.load(Ordering::Acquire)
    }
    pub(crate) fn job_revision(&self, run: &str) -> Option<u64> {
        self.runs
            .lock()
            .unwrap()
            .get(run)
            .map(|entry| entry.progress.revision())
    }
    pub(crate) fn status(&self) -> super::status::Transport {
        use iroh::Watcher;
        let runs = self.runs.lock().unwrap();
        let jobs: Vec<_> = runs
            .iter()
            .filter(|(_, entry)| {
                !entry.phase.is_terminal()
                    && entry
                        .coordinated
                        .as_ref()
                        .is_none_or(|node| node.idle_stamp().is_none())
            })
            .map(|(id, entry)| super::status::Job {
                id: id.clone(),
                phase: entry.phase,
                report: entry.report.clone(),
            })
            .collect();
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
        {
            let mut runs = self.runs.lock().unwrap();
            for entry in runs.values_mut() {
                entry.phase = Phase::Cancelled;
                entry.ended.send_replace(true);
            }
            runs.retain(|_, entry| {
                !entry
                    .coordinated
                    .as_ref()
                    .is_some_and(|node| node.idle_stamp().is_some())
            });
        }
        self.endpoint.close().await;
        self.readers.lock().await.shutdown().await;
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
                        if let Ok(Ok(connection)) = tokio::time::timeout(wire::WAIT, incoming).await {
                            peer.clone().serve_connection(connection, true).await;
                        }
                    });
                }
                _ = connections.join_next(), if !connections.is_empty() => (),
            }
        }
        while connections.join_next().await.is_some() {}
    }

    async fn authorized(&self, peer: &str) -> Result<()> {
        self.store.step(Step::Authorize(peer.into())).await?;
        Ok(())
    }
    async fn send_proof(&self, send: &mut SendStream, recv: &mut RecvStream) -> Result<()> {
        let mut cursor = 0;
        loop {
            let Value::History(events) = self.store.step(Step::History(cursor)).await? else {
                return Err(protocol("unexpected history reply"));
            };
            let done = events.len() < 16;
            if let Some((last, _)) = events.last() {
                cursor = *last;
            }
            wire::send(
                send,
                Frame::Proof {
                    events: events.into_iter().map(|(_, event)| event).collect(),
                    done,
                },
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
    async fn receive_proof(&self, send: &mut SendStream, recv: &mut RecvStream) -> Result<()> {
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
            wire::send(send, Frame::Ack).await?;
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
        wire::send(send, Frame::Hello { version: 2, head }).await?;
        match wire::receive_limit(recv, 4096).await? {
            Frame::Authorized => return self.authorized(peer).await,
            Frame::Ack => (),
            _ => return Err(protocol("membership negotiation expected")),
        }
        self.send_proof(send, recv).await?;
        if !matches!(wire::receive_limit(recv, 4096).await?, Frame::Authorized) {
            return Err(Kind::Authentication.error("mesh peer: membership authorization denied"));
        }
        self.receive_proof(send, recv).await?;
        self.authorized(peer).await?;
        wire::send(send, Frame::Authorized).await
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
            return wire::send(send, Frame::Authorized).await;
        }
        wire::send(send, Frame::Ack).await?;
        self.receive_proof(send, recv).await?;
        self.authorized(peer).await?;
        wire::send(send, Frame::Authorized).await?;
        self.send_proof(send, recv).await?;
        if !matches!(wire::receive_limit(recv, 4096).await?, Frame::Authorized) {
            return Err(
                Kind::Authentication.error("mesh peer: peer did not authorize this endpoint")
            );
        }
        Ok(())
    }

    pub(crate) async fn check_transport(self: &Arc<Self>, peer: &str) -> Result<()> {
        let link = self.connect(peer).await?;
        drop(link);
        Ok(())
    }
    async fn connect(self: &Arc<Self>, peer: &str) -> Result<Arc<Connected>> {
        self.connect_with_routes(
            peer,
            super::scheduler::Routes {
                direct: true,
                relay: true,
            },
            &startup::DialState::default(),
            Kind::Timeout,
        )
        .await
    }
    async fn connect_with_routes(
        self: &Arc<Self>,
        peer: &str,
        routes: super::scheduler::Routes,
        state: &startup::DialState,
        timeout_kind: Kind,
    ) -> Result<Arc<Connected>> {
        self.dial(peer, routes, state, timeout_kind, true).await
    }
    fn has_direct_route(&self, peer: &str) -> bool {
        !self.endpoint.bound_sockets().is_empty()
            && unhex(peer)
                .ok()
                .and_then(|bytes| PublicKey::from_bytes(&bytes).ok())
                .and_then(|id| self.lookup.get_endpoint_info(id))
                .is_some_and(|info| info.to_endpoint_addr().ip_addrs().next().is_some())
    }
    async fn dial(
        self: &Arc<Self>,
        peer: &str,
        routes: super::scheduler::Routes,
        state: &startup::DialState,
        timeout_kind: Kind,
        serving: bool,
    ) -> Result<Arc<Connected>> {
        use iroh::endpoint::{ConnectError, ConnectOptions, InitialPath};
        if !routes.relay && !(routes.direct && self.has_direct_route(peer)) {
            return Err(Kind::Unresolved.error("no known direct route to target"));
        }
        let permit = coordinated::connection_permit(self).await?;
        let admission_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
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
        let mut address = EndpointAddr::new(id).with_relay_url(self.relay.clone());
        if let Some(info) = self.lookup.get_endpoint_info(id) {
            address.addrs.extend(info.to_endpoint_addr().addrs);
        }
        // Read the direct candidates after passive readiness, including any
        // mDNS address learned while a cold relay session was still starting.
        let direct = routes.direct
            && !self.endpoint.bound_sockets().is_empty()
            && address.ip_addrs().next().is_some();
        state.selected(direct);
        let initial_path = match (direct, routes.relay) {
            (true, true) => InitialPath::All,
            (true, false) => InitialPath::DirectOnly,
            (false, true) => InitialPath::RelayOnly,
            (false, false) => return Err(Kind::Unresolved.error("no known direct route to target")),
        };
        let mut observation = routes
            .relay
            .then(|| self.endpoint.observe_relay_dial(self.relay.clone(), id))
            .flatten();
        let dial = async {
            let connecting = self
                .endpoint
                .connect_with_opts(
                    address,
                    ALPN,
                    ConnectOptions::new().with_initial_path(initial_path),
                )
                .await
                .map_err(ConnectError::from)?;
            connecting.await.map_err(ConnectError::from)
        };
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        state.started(deadline);
        let connecting = tokio::time::timeout_at(deadline, dial);
        tokio::pin!(connecting);
        let connection = loop {
            tokio::select! {
                biased;
                result = &mut connecting => break result.map_err(|_| {
                    timeout_kind.error("peer is not reachable now; it may be asleep or offline")
                })?.map_err(Kind::connect)?,
                _ = async { observation.as_mut().unwrap().observation().await }, if observation.is_some() => {
                    let outcome = state.observed(observation.take().unwrap());
                    if outcome.is_some_and(|reply| reply.outcome == iroh::endpoint::RelayDialOutcome::NotRegistered)
                        && !direct {
                            tokio::select! {
                                biased;
                                result = &mut connecting => break result.map_err(|_| {
                                    timeout_kind.error("peer is not reachable now; it may be asleep or offline")
                                })?.map_err(Kind::connect)?,
                                _ = std::future::ready(()) => {
                                    return Err(Kind::Timeout.error(
                                        "peer is not registered at its relay for this dial attempt",
                                    ));
                                },
                            }
                    }
                }
            }
        };
        let server = serving.then(|| self.start_server(&connection));
        Ok(Arc::new(Connected {
            admission_deadline,
            connection,
            server,
            _permit: permit,
        }))
    }

    fn hints(&self, job: &SignedRun) -> Vec<String> {
        job.run
            .cohort
            .iter()
            .filter(|target| *target != &self.local && !job.path.contains(target))
            .filter(|target| {
                unhex::<32>(target)
                    .ok()
                    .and_then(|bytes| PublicKey::from_bytes(&bytes).ok())
                    .and_then(|id| self.lookup.get_endpoint_info(id))
                    .is_some_and(|info| info.to_endpoint_addr().ip_addrs().next().is_some())
            })
            .cloned()
            .collect()
    }

    fn start_server(self: &Arc<Self>, connection: &Connection) -> tokio::task::JoinHandle<()> {
        let peer = self.clone();
        let connection = connection.clone();
        tokio::spawn(async move {
            peer.serve_connection(connection, false).await;
        })
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
                    let Ok((send, recv)) = next else { break; };
                    let peer = self.clone(); let connection = connection.clone();
                    idle_since = tokio::time::Instant::now();
                    streams.spawn(peer.serve_stream(connection, send, recv));
                }
                _ = streams.join_next(), if !streams.is_empty() => idle_since = tokio::time::Instant::now(),
            }
        }
        // Job owners must observe closure, fence work and settle their
        // publication/idle-role state before their stream future is dropped.
        while streams.join_next().await.is_some() {}
    }

    async fn serve_stream(
        self: Arc<Self>,
        connection: Connection,
        mut send: SendStream,
        mut recv: RecvStream,
    ) {
        let remote = connection.remote_id().to_string();
        let mut replica = false;
        let result = async {
            let handshake = tokio::time::timeout(wire::WAIT, self.handshakes.acquire())
                .await
                .map_err(|_| Kind::Capacity.error("mesh handshake capacity reached"))?
                .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
            match wire::receive_limit(&mut recv, 16 * 1024).await? {
                Frame::Hello { version: 2, head } => {
                    tokio::time::timeout(
                        Duration::from_secs(300),
                        self.handshake_server(&mut send, &mut recv, &remote, &head),
                    )
                    .await
                    .map_err(|_| Kind::Timeout.error("membership handshake timed out"))??;
                    drop(handshake);
                    let mut control = Control::new(send, recv);
                    let begin = tokio::time::timeout(wire::WAIT, control.receive())
                        .await
                        .map_err(|_| Kind::Timeout.error("mesh job begin timed out"))??;
                    match begin {
                        Frame::Begin {
                            job,
                            replaces_parent,
                            continuation,
                            terminal_only,
                        } => {
                            coordinated::serve(
                                self.clone(),
                                connection.clone(),
                                control,
                                job,
                                replaces_parent,
                                continuation,
                                terminal_only,
                            )
                            .await
                        }
                        _ => Err(protocol("manual job begin expected")),
                    }
                }
                Frame::Replica {
                    run,
                    round,
                    context,
                    delta,
                } => {
                    replica = true;
                    drop(handshake);
                    self.serve_replica_stream(
                        &mut send,
                        &mut recv,
                        &remote,
                        &run,
                        (round, context, delta),
                        connection.stable_id(),
                    )
                    .await
                }
                Frame::Hello { .. } => Err(Kind::Version.error("unsupported mesh version")),
                _ => Err(protocol("unsupported mesh stream")),
            }
        }
        .await;
        if let Err(error) = result
            && !replica
        {
            connection.close(Kind::of(&error).code().into(), b"mesh stream failed");
        }
    }

    fn register(&self, job: &SignedRun, parent: Option<&str>) -> Result<(Phase, bool)> {
        let mut runs = self.runs.lock().unwrap();
        if let Some(entry) = runs.get(&job.run.id) {
            if entry.job.run != job.run || entry.job.signature != job.signature {
                return Err(
                    Kind::Conflict.error("mesh peer: manual run ID reused with different bytes")
                );
            }
            return Ok((entry.phase, true));
        }
        let ended: Vec<_> = runs
            .iter()
            .filter(|(_, entry)| {
                *entry.ended.borrow()
                    && entry
                        .coordinated
                        .as_ref()
                        .is_some_and(|node| node.idle_stamp().is_some())
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ended {
            if let Some(entry) = runs.remove(&id) {
                entry.ended.send_replace(true);
            }
        }
        if runs.len() >= 4 {
            let candidate = runs
                .iter()
                .filter(|(_, entry)| entry.parent.is_some())
                .filter_map(|(id, entry)| {
                    entry
                        .coordinated
                        .as_ref()
                        .and_then(|node| node.idle_stamp())
                        .map(|at| (at, id.clone()))
                })
                .min()
                .map(|(_, id)| id);
            if let Some(id) = candidate
                && let Some(entry) = runs.remove(&id)
            {
                entry.ended.send_replace(true);
            }
        }
        if runs.len() >= 4 {
            return Err(Kind::Capacity.error("mesh peer: role metadata capacity reached"));
        }
        runs.insert(
            job.run.id.clone(),
            Entry {
                job: job.clone(),
                parent: parent.map(str::to_owned),
                children: BTreeSet::new(),
                phase: Phase::Collecting,
                report: Report::new(&job.run, &self.local),
                progress: Progress::default(),
                ended: tokio::sync::watch::channel(false).0,
                reads: BTreeMap::new(),
                coordinated: None,
                terminal_only: false,
                data_round: 0,
                transfers: 0,
            },
        );
        Ok((Phase::Collecting, false))
    }
    fn phase(&self, run: &str, phase: Phase, report: &mut Report) {
        if let Some(entry) = self.runs.lock().unwrap().get_mut(run) {
            // Accepted items update the run immediately, even if a later pull
            // fails before its aggregate result returns to the phase coordinator.
            report.received = entry.report.received;
            report.sent = entry.report.sent;
            if let Some(current) = &entry.report.traversal
                && report
                    .traversal
                    .as_ref()
                    .is_none_or(|incoming| current.round >= incoming.round)
            {
                report.traversal = Some(current.clone());
            }
            entry.progress.advance();
            entry.phase = phase;
            entry.report = report.clone();
            if phase.is_terminal() {
                entry.revoke_sources();
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

    pub(crate) async fn sync_started(
        self: Arc<Self>,
        run: String,
        startup: Startup,
    ) -> Result<Report> {
        coordinated::synchronize_started(self, run, startup).await
    }
    async fn pull_round(
        &self,
        connection: &Connection,
        peer: &str,
        run: &str,
        round: (u8, u64),
        delta: bool,
    ) -> Result<u64> {
        let _permit = tokio::time::timeout(wire::WAIT, self.outgoing.acquire())
            .await
            .map_err(|_| Kind::Capacity.error("outgoing mesh transfers are busy"))?
            .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
        self.authorized(peer).await?;
        #[cfg(test)]
        if delta {
            self.delta_transfers[0].fetch_add(1, Ordering::Relaxed);
        }
        let (changed, fallback) = self
            .pull_pipeline(connection, peer, run, round, delta)
            .await?;
        if fallback {
            #[cfg(test)]
            self.delta_transfers[1].fetch_add(1, Ordering::Relaxed);
            let (more, _) = self
                .pull_pipeline(connection, peer, run, round, false)
                .await?;
            Ok(changed + more)
        } else {
            Ok(changed)
        }
    }
    async fn serve_replica_stream(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
        run: &str,
        round: (u8, u64, bool),
        connection: usize,
    ) -> Result<()> {
        let result = self
            .serve_replica_round(send, recv, peer, run, round, connection)
            .await;
        if let Err(error) = &result {
            let code = Kind::of(error).code();
            let _ = send.reset(code.into());
            let _ = recv.stop(code.into());
        }
        result
    }
    async fn serve_replica_round(
        &self,
        send: &mut SendStream,
        recv: &mut RecvStream,
        peer: &str,
        run: &str,
        operation: (u8, u64, bool),
        connection: usize,
    ) -> Result<()> {
        let (round, context, delta) = operation;
        #[cfg(test)]
        if let round @ 1..=2 = round {
            self.replica_rounds[usize::from(round - 1)].fetch_add(1, Ordering::Relaxed);
        }
        let _permit = tokio::time::timeout(wire::WAIT, self.incoming.acquire())
            .await
            .map_err(|_| Kind::Capacity.error("incoming mesh transfers are busy"))?
            .map_err(|_| Kind::Cancelled.error("mesh peer: endpoint closing"))?;
        self.authorized(peer).await?;
        let progress = self.progress(run)?;
        let (mut ended, job, source_stream, scope) = {
            let runs = self.runs.lock().unwrap();
            let entry = runs
                .get(run)
                .ok_or_else(|| Kind::Authentication.error("mesh peer: unknown manual run"))?;
            if !entry.serves_round(peer, round) {
                return Err(Kind::Authentication
                    .error("mesh peer: replica request is outside the active job phase"));
            }
            let source_stream = entry
                .reads
                .get(peer)
                .ok_or_else(|| Kind::Authentication.error("no current source operation"))?
                .claim(round, context, connection, delta)?;
            let scope = source_stream.scope.clone();
            (
                entry.ended.subscribe(),
                Arc::new(entry.job.clone()),
                source_stream,
                scope,
            )
        };
        if *ended.borrow() {
            return Err(Kind::Cancelled.error("mesh peer: manual job ended"));
        }
        self.store.step(Step::Finish(None)).await?;
        let Value::SourceSnapshot(cutoff) =
            self.store.step(Step::SourceSnapshot(peer.into())).await?
        else {
            return Err(protocol("source snapshot expected"));
        };
        let mut source = replica::ReplicaSource {
            peer: self,
            remote: peer,
            run,
            round,
            delta,
            job,
            stream: source_stream,
            cutoff,
            progress,
            delta_started: false,
            revision: None,
        };
        let serve = source.serve(send, recv);
        tokio::select! {
            biased;
            _ = ended.changed() => Err(Kind::Cancelled.error("mesh peer: manual job ended")),
            _ = scope.ended() => Err(Kind::Cancelled.error("source operation ended")),
            result = serve => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                    Step::Authorize(peer) => {
                        if state.members.contains(&peer) {
                            Ok(Value::Done)
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
                        version: 2,
                        head: membership.head.clone(),
                    },
                )
                .await
                .unwrap();
                let (mut reply, mut request) = server.accept_bi().await.unwrap();
                let Frame::Hello { head, .. } =
                    wire::receive_limit(&mut request, crate::sync::protocol::MAX_FRAME)
                        .await
                        .unwrap()
                else {
                    panic!("hello expected");
                };
                let outcome = peer
                    .handshake_server(&mut reply, &mut request, &remote.id().to_string(), &head)
                    .await;
                if known {
                    outcome.unwrap();
                    assert!(matches!(
                        wire::receive_limit(&mut recv, crate::sync::protocol::MAX_FRAME)
                            .await
                            .unwrap(),
                        Frame::Authorized
                    ));
                } else {
                    assert_eq!(Kind::of(&outcome.err().unwrap()), Kind::Authentication);
                }
            })
            .await
            .unwrap();
            // A transport probe authenticates the endpoint but opens no record or
            // membership stream, even when the endpoint is not a mesh member.
            let address = remote
                .bound_sockets()
                .into_iter()
                .find(|s| s.is_ipv4())
                .unwrap();
            peer.lookup
                .set_endpoint_info(EndpointAddr::new(remote.id()).with_ip_addr(address));
            tokio::time::timeout(Duration::from_secs(5), async {
                let remote_id = remote.id().to_string();
                let (probe, ()) = tokio::join!(peer.check_transport(&remote_id), async {
                    let connection = remote.accept().await.unwrap().await.unwrap();
                    assert!(
                        connection.accept_bi().await.is_err(),
                        "probe must not open application streams"
                    );
                });
                probe.unwrap();
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
