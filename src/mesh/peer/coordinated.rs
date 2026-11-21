//! Job control requests reuse registered tree edges and never fan out on Begin.
use super::*;
mod pressure;
mod reconciliation;
mod registrations;
use crate::mesh::{
    discovery_job,
    scheduler::{Assignment, Routes, Scheduler},
};
use futures_util::{StreamExt, stream::FuturesUnordered};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio::sync::{RwLock, mpsc, oneshot};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Envelope {
    pub id: u64,
    pub route: Vec<String>,
    pub command: Command,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Command {
    Contact {
        assignment: u64,
        target: String,
        routes: Routes,
        replaces_parent: Option<String>,
    },
    Expose {
        round: u8,
        readers: Vec<String>,
    },
    Pull {
        round: u8,
        child: Option<String>,
        delta: bool,
    },
    Detach {
        child: String,
    },
    Stop,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Reply {
    Contact {
        direct: bool,
        free: bool,
        parent: String,
        duplicate: bool,
        via_relay: bool,
        hints: Vec<String>,
        observation: Option<discovery_job::Observation>,
    },
    NoDirectRoute,
    ContactFailed {
        direct: bool,
        kind: Kind,
        message: String,
        observation: Option<discovery_job::Observation>,
    },
    Done {
        changed: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fault {
    kind: Kind,
    message: String,
    peer: Option<String>,
    local: bool,
}
impl Fault {
    fn error(self) -> Error {
        super::super::failure::FailureError {
            kind: self.kind,
            message: self.message,
            peer: self.peer,
            local: self.local,
        }
        .into()
    }
}
impl From<Error> for Fault {
    fn from(error: Error) -> Self {
        Self {
            kind: Kind::of(&error),
            message: bounded(&error),
            peer: match &error {
                Error::Mesh(error) => error.peer.clone(),
                _ => None,
            },
            local: matches!(&error, Error::Mesh(error) if error.local),
        }
    }
}
#[derive(Clone)]
enum Cancellation {
    Forward(String),
    Child(String),
    Source { reader: String, context: u64 },
}
struct Cached {
    fingerprint: String,
    cancellation: Option<Cancellation>,
    reply: RpcResult,
}
impl Cached {
    fn fingerprint(envelope: &Envelope) -> Result<String> {
        Ok(crate::record::hash(&serde_json::to_vec(envelope)?))
    }
    fn new(envelope: &Envelope, reply: RpcResult) -> Result<Self> {
        Ok(Self {
            fingerprint: Self::fingerprint(envelope)?,
            cancellation: Self::cancellation(envelope),
            reply,
        })
    }
    fn cancellation(envelope: &Envelope) -> Option<Cancellation> {
        if let Some(next) = envelope.route.first() {
            Some(Cancellation::Forward(next.clone()))
        } else if let Command::Contact { target, .. } = &envelope.command {
            Some(Cancellation::Child(target.clone()))
        } else if let Command::Expose { readers, .. } = &envelope.command {
            readers.first().cloned().map(|reader| Cancellation::Source {
                reader,
                context: envelope.id,
            })
        } else {
            None
        }
    }
}
type RpcResult = std::result::Result<Reply, Fault>;
type Updates = mpsc::Sender<(u64, discovery_job::Observation)>;
struct AssignmentTask {
    envelope: Envelope,
    task: Option<Pin<Box<dyn Future<Output = Result<Reply>> + Send>>>,
    wake: Option<std::task::Waker>,
}
impl AssignmentTask {
    fn cancel(&mut self) {
        self.task.take();
        if let Some(wake) = self.wake.take() {
            wake.wake();
        }
    }
}
impl Future for AssignmentTask {
    type Output = (Envelope, Option<Result<Reply>>);

    fn poll(
        self: Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        this.wake = Some(context.waker().clone());
        let result = match this.task.as_mut() {
            Some(task) => std::task::ready!(task.as_mut().poll(context)).into(),
            None => None,
        };
        std::task::Poll::Ready((this.envelope.clone(), result))
    }
}
struct Call {
    envelope: Envelope,
    updates: Option<Updates>,
    reply: oneshot::Sender<RpcResult>,
}
struct Session {
    link: Arc<Connected>,
    commands: mpsc::Sender<Call>,
    cancel: mpsc::UnboundedSender<u64>,
    actor: tokio::task::JoinHandle<()>,
    capacity: Arc<capacity::Sources>,
    terminal: Arc<Mutex<Option<Fault>>>,
}
struct Pending {
    reply: oneshot::Sender<RpcResult>,
    updates: Option<Updates>,
}
struct RequestGuard {
    id: u64,
    cancel: mpsc::UnboundedSender<u64>,
    complete: bool,
}
impl Drop for RequestGuard {
    fn drop(&mut self) {
        if !self.complete {
            let _ = self.cancel.send(self.id);
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.capacity
            .forget(&self.link.connection.remote_id().to_string());
        self.actor.abort();
    }
}
impl Session {
    fn new(
        link: Arc<Connected>,
        mut control: Control,
        capacity: Arc<capacity::Sources>,
        cohort: Vec<String>,
    ) -> Arc<Self> {
        let (commands, mut requests) = mpsc::channel::<Call>(16);
        let (cancel, mut cancellations) = mpsc::unbounded_channel();
        let remote = link.connection.remote_id().to_string();
        let observations = capacity.clone();
        let terminal: Arc<Mutex<Option<Fault>>> = Arc::default();
        let ended = terminal.clone();
        let actor = tokio::spawn(async move {
            let mut pending: BTreeMap<u64, Pending> = BTreeMap::new();
            let result = async {
                loop {
                    tokio::select! {
                        Some(id) = cancellations.recv() => {
                            pending.remove(&id);
                            control.send(Frame::Cancel { id }).await?;
                        }
                        request = requests.recv(), if pending.len() < 16 => {
                            let Some(request) = request else { break; };
                            if pending.contains_key(&request.envelope.id) {
                                return Err(protocol("duplicate active control request"));
                            }
                            let id = request.envelope.id;
                            pending.insert(id, Pending { reply: request.reply, updates: request.updates });
                            control.send(Frame::Command(request.envelope)).await?;
                        }
                        frame = control.receive() => match frame? {
                            Frame::Reply { id, result } => {
                                if let Some(request) = pending.remove(&id) { let _ = request.reply.send(result); }
                            }
                            Frame::Capacity(values) => observations.accept(values, &cohort)?,
                            Frame::Observed { id, observation } => {
                                if let Some(updates) = pending.get(&id).and_then(|request| request.updates.as_ref()) {
                                    let _ = updates.try_send((id, observation));
                                }
                            }
                            _ => return Err(protocol("invalid coordinated control response")),
                        }
                    }
                }
                Ok::<_, Error>(())
            }.await;
            let fault: Fault = Kind::on_peer(
                result
                    .err()
                    .unwrap_or_else(|| Kind::Unresolved.error("job control session ended")),
                &remote,
            )
            .into();
            *ended.lock().unwrap() = Some(fault.clone());
            for request in pending.into_values() {
                let _ = request.reply.send(Err(fault.clone()));
            }
        });
        Arc::new(Self {
            link,
            commands,
            cancel,
            actor,
            capacity,
            terminal,
        })
    }
    fn ended(&self, message: &str) -> Error {
        if let Some(fault) = self.terminal.lock().unwrap().clone() {
            return fault.error();
        }
        Kind::on_peer(
            Kind::Unresolved.error(message),
            &self.link.connection.remote_id().to_string(),
        )
    }
    async fn request(&self, envelope: Envelope, updates: Option<Updates>) -> Result<Reply> {
        let id = envelope.id;
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Call {
                envelope,
                updates,
                reply,
            })
            .await
            .map_err(|_| self.ended("job control session ended"))?;
        let mut guard = RequestGuard {
            id,
            cancel: self.cancel.clone(),
            complete: false,
        };
        let response = response
            .await
            .map_err(|_| self.ended("job control response lost"))?;
        guard.complete = true;
        response.map_err(Fault::error)
    }
}
pub(super) struct Node {
    owner: Weak<Peer>,
    startup: Startup,
    job: SignedRun,
    parent: RwLock<Option<Connection>>,
    children: RwLock<BTreeMap<String, Arc<pressure::Edge>>>,
    attached: AtomicBool,
    orphan: AtomicBool,
    reopening: Arc<Mutex<BTreeMap<String, Arc<Mutex<pressure::Budget>>>>>,
    detached: tokio::sync::Notify,
    capacity: Arc<capacity::Sources>,
    cached: Mutex<BTreeMap<u64, Cached>>,
    delta: Mutex<Option<crate::mesh::delta::Delta>>,
    baseline: Mutex<(crate::mesh::delta::Watch, bool)>,
    touched: Mutex<tokio::time::Instant>,
}
impl Node {
    pub(super) fn delta_baseline_valid(&self) -> bool {
        let mut baseline = self.baseline.lock().unwrap();
        baseline.1 && baseline.0.valid()
    }
    pub(super) fn refresh_delta(&self) {
        if let Some(delta) = self.delta.lock().unwrap().as_mut() {
            delta.refresh();
        }
        self.baseline.lock().unwrap().0.valid();
    }
    pub(super) fn delta_eligible(&self, peer: &str) -> bool {
        self.delta
            .lock()
            .unwrap()
            .as_mut()
            .is_some_and(|delta| delta.eligible(peer))
    }
    pub(super) fn delta_revision(&self) -> Option<u64> {
        self.delta.lock().unwrap().as_mut()?.start()
    }
    pub(super) fn delta_page(
        &self,
        peer: &str,
        tombstones: bool,
        after: Option<&str>,
    ) -> Option<(Vec<crate::storage::sync::Entry>, bool)> {
        self.delta
            .lock()
            .unwrap()
            .as_mut()?
            .page(peer, tombstones, after)
    }
    pub(super) fn idle_stamp(&self) -> Option<tokio::time::Instant> {
        (!self.attached.load(Ordering::Acquire)).then(|| *self.touched.lock().unwrap())
    }
    fn peer(&self) -> Result<Arc<Peer>> {
        self.owner
            .upgrade()
            .ok_or_else(|| Kind::Cancelled.error("job endpoint was dropped"))
    }
    async fn reopen(&self, target: &str, purpose: pressure::Purpose) -> Result<Arc<Session>> {
        pressure::reopen(self, target, purpose).await
    }
    async fn release_idle(&self) -> bool {
        let mut edges: Vec<_> = self.children.read().await.values().cloned().collect();
        edges.sort_by_key(|edge| edge.touched());
        edges.into_iter().any(|edge| edge.release_idle())
    }
    async fn cancel(&self, envelope: &Envelope) {
        self.cancel_action(envelope.id, Cached::cancellation(envelope))
            .await;
    }
    async fn cancel_action(&self, id: u64, action: Option<Cancellation>) {
        let Ok(peer) = self.peer() else {
            return;
        };
        match action {
            Some(Cancellation::Forward(next)) => {
                if let Some(edge) = self.children.read().await.get(&next) {
                    edge.cancel(id).await;
                }
            }
            Some(Cancellation::Source { reader, context }) => {
                if let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.job.run.id)
                    && let Some(grant) = entry.reads.get(&reader)
                    && grant.context() == context
                {
                    grant.revoke();
                }
            }
            Some(Cancellation::Child(target)) => {
                self.children.write().await.remove(&target);
                if let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.job.run.id) {
                    entry.children.remove(&target);
                    if let Some(grant) = entry.reads.remove(&target) {
                        grant.revoke();
                    }
                }
            }
            None => (),
        }
    }
    fn expose(
        &self,
        peer: &Peer,
        reader: &str,
        round: u8,
        context: u64,
        connection: usize,
    ) -> Result<Arc<source::Grant>> {
        let mut runs = peer.runs.lock().unwrap();
        let entry = runs
            .get_mut(&self.job.run.id)
            .ok_or_else(|| protocol("expired job"))?;
        if !matches!(round, 1 | 2)
            || entry.terminal_only
            || entry.phase.is_terminal()
            || (entry.parent.as_deref() != Some(reader) && !entry.children.contains(reader))
        {
            return Err(Kind::Authentication.error("source reader is outside its active tree role"));
        }
        entry.advance_round(round)?;
        if entry.reads.get(reader).is_some_and(|grant| grant.active()) {
            return Err(protocol("source operation is already active"));
        }
        let grant = source::Grant::new(round, context, connection);
        if let Some(previous) = entry.reads.insert(reader.into(), grant.clone()) {
            previous.revoke();
        }
        Ok(grant)
    }
    fn execute(
        self: Arc<Self>,
        mut envelope: Envelope,
        updates: Option<Updates>,
    ) -> Pin<Box<dyn Future<Output = Result<Reply>> + Send>> {
        Box::pin(async move {
            let peer = self.peer()?;
            if self.orphan.load(Ordering::Acquire)
                && !(envelope.route.is_empty() && matches!(envelope.command, Command::Stop))
            {
                return Err(
                    Kind::Cancelled.error("failed role has no source or assignment authority")
                );
            }
            peer.verify_job(&self.job).await?;
            peer.progress(&self.job.run.id)?.advance();
            if envelope.route.len() > 255 {
                return Err(protocol("control route exceeds cohort depth"));
            }
            if !envelope.route.is_empty() {
                let purpose = pressure::Purpose::of(&envelope, &peer.local)?;
                let next = envelope.route.remove(0);
                let child = self
                    .children
                    .read()
                    .await
                    .get(&next)
                    .cloned()
                    .ok_or_else(|| Kind::Unresolved.error("recorded child edge is unavailable"))?;
                let terminal_child =
                    envelope.route.is_empty() && matches!(envelope.command, Command::Stop);
                let session = child
                    .acquire(&self, &next, purpose)
                    .await
                    .map_err(|error| Kind::on_peer(error, &next))?;
                // The same lease pins the source connection through durable child acknowledgement.
                let _source = if envelope.route.is_empty() {
                    match &envelope.command {
                        Command::Pull {
                            round, child: None, ..
                        } => Some(source::Operation(self.expose(
                            &peer,
                            &next,
                            *round,
                            envelope.id,
                            session.link.connection.stable_id(),
                        )?)),
                        _ => None,
                    }
                } else {
                    None
                };
                let reply = session
                    .request(envelope, updates)
                    .await
                    .map_err(|error| Kind::on_peer(error, &next))?;
                if terminal_child {
                    self.children.write().await.remove(&next);
                    if let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.job.run.id) {
                        entry.children.remove(&next);
                        if let Some(grant) = entry.reads.remove(&next) {
                            grant.revoke();
                        }
                    }
                }
                return Ok(reply);
            }
            let result = match envelope.command {
                command @ Command::Contact { .. } => {
                    self.contact_command(&peer, envelope.id, command, updates)
                        .await?
                }
                Command::Expose { round, readers } => {
                    self.expose_command(&peer, envelope.id, round, readers)
                        .await?
                }
                Command::Pull {
                    round,
                    child,
                    delta,
                } => {
                    self.pull_command(&peer, envelope.id, round, child, delta)
                        .await?
                }
                Command::Detach { child } => self.detach_command(&peer, child).await?,
                Command::Stop => self.stop_command(&peer).await?,
            };
            peer.progress(&self.job.run.id)?.advance();
            Ok(result)
        })
    }
}

impl Node {
    async fn contact_command(
        self: &Arc<Self>,
        peer: &Arc<Peer>,
        request: u64,
        command: Command,
        updates: Option<Updates>,
    ) -> Result<Reply> {
        let Command::Contact {
            assignment,
            target,
            routes,
            replaces_parent,
        } = command
        else {
            return Err(protocol("contact command expected"));
        };
        if self.job.path.contains(&target) || !self.job.run.cohort.contains(&target) {
            return Err(Kind::Authentication.error("invalid contact target"));
        }
        if !routes.relay && !(routes.direct && peer.has_direct_route(&target)) {
            return Ok(Reply::NoDirectRoute);
        }
        let direct = Arc::new(AtomicBool::new(false));
        let selected = direct.clone();
        let latest = Arc::new(Mutex::new(None));
        let observed = latest.clone();
        let notify = updates.clone();
        let callback: startup::DialUpdate = Arc::new(move |direct, reply| {
            selected.store(direct, Ordering::Release);
            let Some(reply) = reply else {
                return;
            };
            let observation = discovery_job::Observation {
                assignment,
                generation: reply.generation,
                registered: reply.outcome == iroh::endpoint::RelayDialOutcome::Registered,
                connected: false,
            };
            *observed.lock().unwrap() = Some(observation.clone());
            if let Some(notify) = &notify {
                let _ = notify.try_send((request, observation));
            }
        });
        let state = startup::DialState::default();
        state.subscribe(callback.clone());
        let outcome = async {
            let link = match self.startup.adopt(peer, &target, callback).await? {
                Some(link) => link,
                None => {
                    peer.connect_with_routes(&target, routes, &state, Kind::Timeout)
                        .await?
                }
            };
            let presence = discovery_job::Observation {
                assignment,
                generation: 0,
                registered: false,
                connected: true,
            };
            {
                let mut last = latest.lock().unwrap();
                if let Some(last) = last.as_mut() {
                    last.connected = true;
                } else {
                    *last = Some(presence.clone());
                }
            }
            if let Some(updates) = &updates {
                let _ = updates.try_send((request, presence));
            }
            let via_relay = link
                .connection
                .paths()
                .iter()
                .any(|path| path.is_selected() && path.is_relay());
            let admission_deadline = link.admission_deadline;
            let handshake = async {
                let (mut send, mut recv) =
                    link.connection.open_bi().await.map_err(Kind::connection)?;
                peer.handshake_client(&mut send, &mut recv, &target).await?;
                let mut control = Control::new(send, recv);
                control.track_coordinated(peer.progress(&self.job.run.id)?, true);
                control
                    .send(Frame::Begin {
                        job: self.job.clone(),
                        replaces_parent,
                        continuation: false,
                        terminal_only: false,
                    })
                    .await?;
                let Frame::Accepted {
                    free,
                    parent,
                    duplicate,
                    hints,
                } = control.receive().await?
                else {
                    return Err(protocol("coordinated acceptance expected"));
                };
                if hints.len() > self.job.run.cohort.len()
                    || hints
                        .iter()
                        .any(|hint| hint == &target || !self.job.run.cohort.contains(hint))
                {
                    return Err(protocol("address hints exceed the frozen cohort"));
                }
                self.capacity.record(&target, free);
                let parent = parent.ok_or_else(|| protocol("registered peer has no parent"))?;
                if !duplicate {
                    if parent != peer.local {
                        return Err(protocol("accepted parent mismatch"));
                    }
                    self.children.write().await.insert(
                        target.clone(),
                        pressure::Edge::new(
                            Session::new(
                                link,
                                control,
                                self.capacity.clone(),
                                self.job.run.cohort.clone(),
                            ),
                            self.reopening
                                .lock()
                                .unwrap()
                                .entry(target.clone())
                                .or_default()
                                .clone(),
                        ),
                    );
                    if let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.job.run.id) {
                        entry.children.insert(target);
                    }
                }
                Ok::<_, Error>((parent, duplicate, hints, free))
            };
            let (parent, duplicate, hints, free) =
                tokio::time::timeout_at(admission_deadline, handshake)
                    .await
                    .map_err(|_| {
                        Kind::Timeout.error("peer did not accept before the admission deadline")
                    })??;
            Ok::<_, Error>((parent, duplicate, via_relay, hints, free))
        }
        .await;
        let observation = latest.lock().unwrap().clone();
        let direct = direct.load(Ordering::Acquire);
        Ok(match outcome {
            Ok((parent, duplicate, via_relay, hints, free)) => Reply::Contact {
                direct,
                free,
                parent,
                duplicate,
                via_relay,
                hints,
                observation,
            },
            Err(error) => Reply::ContactFailed {
                direct,
                kind: Kind::of(&error),
                message: bounded(&error),
                observation,
            },
        })
    }
    async fn expose_command(
        self: &Arc<Self>,
        peer: &Arc<Peer>,
        request: u64,
        round: u8,
        readers: Vec<String>,
    ) -> Result<Reply> {
        let parent = self
            .parent
            .read()
            .await
            .clone()
            .ok_or_else(|| protocol("root has no parent reader"))?;
        if readers.as_slice() != [parent.remote_id().to_string()] {
            return Err(Kind::Authentication.error("upward source must be its recorded parent"));
        }
        self.expose(peer, &readers[0], round, request, parent.stable_id())?;
        Ok(Reply::Done { changed: 0 })
    }
    async fn pull_command(
        self: &Arc<Self>,
        peer: &Arc<Peer>,
        request: u64,
        round: u8,
        child: Option<String>,
        delta: bool,
    ) -> Result<Reply> {
        if !matches!(round, 1 | 2) {
            return Err(protocol("unsupported data round"));
        }
        let _receiving = source::Receiving::begin(peer, &self.job.run.id, round)?;
        let receiving = child.is_none();
        if delta && (!receiving || round != 2) {
            return Err(protocol("delta requires final downward reconciliation"));
        }
        let delta = delta && receiving && round == 2 && self.delta_baseline_valid();
        let mut held_edge = None;
        let mut source_cancel = None;
        let (connection, remote) = if let Some(child) = child {
            let session = self
                .children
                .read()
                .await
                .get(&child)
                .cloned()
                .ok_or_else(|| Kind::Unresolved.error("child edge is unavailable"))?;
            let session = session
                .acquire(
                    self,
                    &child,
                    pressure::Purpose::Data {
                        round,
                        upward: true,
                    },
                )
                .await?;
            session
                .request(
                    Envelope {
                        id: request,
                        route: vec![],
                        command: Command::Expose {
                            round,
                            readers: vec![peer.local.clone()],
                        },
                    },
                    None,
                )
                .await?;
            source_cancel = Some(RequestGuard {
                id: request,
                cancel: session.cancel.clone(),
                complete: false,
            });
            let connection = session.link.connection.clone();
            held_edge = Some(session);
            (connection, child)
        } else {
            let connection = self
                .parent
                .read()
                .await
                .clone()
                .ok_or_else(|| protocol("root has no parent source"))?;
            let remote = connection.remote_id().to_string();
            (connection, remote)
        };
        let changed = peer
            .pull_round(
                &connection,
                &remote,
                &self.job.run.id,
                (round, request),
                delta,
            )
            .await
            .map_err(|error| Kind::on_peer(error, &remote))?;
        drop(source_cancel);
        drop(held_edge);
        if receiving && round == 1 {
            self.baseline.lock().unwrap().1 = true;
        }
        Ok(Reply::Done { changed })
    }
    async fn detach_command(self: &Arc<Self>, peer: &Arc<Peer>, child: String) -> Result<Reply> {
        self.children.write().await.remove(&child);
        if let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.job.run.id) {
            entry.children.remove(&child);
            if let Some(grant) = entry.reads.remove(&child) {
                grant.revoke();
            }
        }
        Ok(Reply::Done { changed: 0 })
    }
    async fn stop_command(self: &Arc<Self>, peer: &Arc<Peer>) -> Result<Reply> {
        {
            let runs = peer.runs.lock().unwrap();
            let entry = runs
                .get(&self.job.run.id)
                .ok_or_else(|| protocol("expired job"))?;
            if entry.transfers != 0 || entry.reads.values().any(|grant| grant.active()) {
                return Err(protocol("finalization cannot overlap an active data round"));
            }
        }
        if !self.children.read().await.is_empty() {
            return Err(Kind::Cancelled.error("child final acknowledgement is unresolved"));
        }
        peer.store.step(Step::Finish(None)).await?;
        Ok(Reply::Done { changed: 0 })
    }
}

pub(super) async fn release_idle(peer: &Peer) -> bool {
    let nodes: Vec<_> = peer
        .runs
        .lock()
        .unwrap()
        .values()
        .filter_map(|entry| entry.coordinated.clone())
        .collect();
    for node in nodes {
        if node.release_idle().await {
            return true;
        }
    }
    false
}
pub(super) async fn connection_permit(
    peer: &Arc<Peer>,
) -> Result<super::super::connections::Permit> {
    let acquire = async {
        // One FIFO admission ticket survives idle wakeups and membership changes.
        let ticket = peer.connections.queue().await;
        loop {
            let changed = peer.connections.notified();
            let idle = peer.connection_idle.notified();
            tokio::pin!(changed, idle);
            changed.as_mut().enable();
            idle.as_mut().enable();
            if let Some(permit) = peer.connections.reserve(&ticket) {
                return Ok(permit);
            }
            if release_idle(peer).await {
                continue;
            }
            tokio::select! { _ = changed => (), _ = idle => (), }
        }
    };
    tokio::time::timeout(wire::WAIT, acquire)
        .await
        .map_err(|_| Kind::Capacity.error("connection queue made no progress"))?
}

pub(super) async fn serve(
    peer: Arc<Peer>,
    connection: Connection,
    mut control: Control,
    mut job: SignedRun,
    replaces_parent: Option<String>,
    continuation: bool,
    terminal_only: bool,
) -> Result<()> {
    let Some((duplicate, node)) = register_session(
        &peer,
        &connection,
        &mut control,
        &mut job,
        replaces_parent,
        continuation,
        terminal_only,
    )
    .await?
    else {
        return Ok(());
    };
    let mut running = Running::new(&peer, &job.run.id, false);
    if terminal_only {
        peer.runs
            .lock()
            .unwrap()
            .get_mut(&job.run.id)
            .unwrap()
            .terminal_only = true;
    }
    control.track_coordinated(peer.progress(&job.run.id)?, false);
    peer.store.step(Step::Finish(None)).await?;
    control
        .send(Frame::Accepted {
            free: peer.outgoing.has_free(),
            parent: Some(connection.remote_id().to_string()),
            duplicate,
            hints: if duplicate { vec![] } else { peer.hints(&job) },
        })
        .await?;
    let outcome = dispatch_session(&peer, &connection, control, &node, terminal_only).await?;
    retire_session(&peer, &connection, &node, &mut running, outcome).await
}

async fn register_session(
    peer: &Arc<Peer>,
    connection: &Connection,
    control: &mut Control,
    job: &mut SignedRun,
    replaces_parent: Option<String>,
    continuation: bool,
    terminal_only: bool,
) -> Result<Option<(bool, Arc<Node>)>> {
    if job.run.version != 2 {
        return Err(Kind::Version.error("unsupported coordinated job profile"));
    }
    peer.verify_job(job).await?;
    let parent = connection.remote_id().to_string();
    if job.path.last() != Some(&parent) || job.path.contains(&peer.local) {
        return Err(protocol("invalid coordinated parent path"));
    }
    job.path.push(peer.local.clone());
    let mut reopening = None;
    if let Some(old_parent) = replaces_parent {
        let ended = {
            let mut runs = peer.runs.lock().unwrap();
            if let Some(entry) = runs.get_mut(&job.run.id) {
                if entry.job.run != job.run || entry.job.signature != job.signature {
                    return Err(Kind::Conflict.error("conflicting recovery identity"));
                }
                if entry.parent.as_ref() == Some(&parent)
                    && !entry
                        .coordinated
                        .as_ref()
                        .is_some_and(|node| node.orphan.load(Ordering::Acquire))
                {
                    None
                } else if entry.parent.as_ref() == Some(&old_parent) {
                    reopening = entry
                        .coordinated
                        .as_ref()
                        .map(|node| node.reopening.clone());
                    let idle = entry
                        .coordinated
                        .as_ref()
                        .is_some_and(|node| !node.attached.load(Ordering::Acquire));
                    let ended = entry.ended.subscribe();
                    entry.phase = Phase::Cancelled;
                    entry.ended.send_replace(true);
                    if idle {
                        runs.remove(&job.run.id);
                    }
                    Some(ended)
                } else {
                    return Err(Kind::Authentication
                        .error("recovery does not match the registered old parent"));
                }
            } else {
                return Err(Kind::Cancelled.error("expired/unknown recovery accounting"));
            }
        };
        if let Some(mut ended) = ended {
            tokio::time::timeout(wire::WAIT, async { while ended.changed().await.is_ok() {} })
                .await
                .map_err(|_| Kind::Timeout.error("old parent work did not finish fencing"))?;
        }
    }
    let (duplicate, node) = if continuation {
        let node = {
            let runs = peer.runs.lock().unwrap();
            let entry = runs
                .get(&job.run.id)
                .ok_or_else(|| Kind::Cancelled.error("expired/unknown recorded job"))?;
            if entry
                .coordinated
                .as_ref()
                .is_some_and(|node| node.orphan.load(Ordering::Acquire))
                && !terminal_only
            {
                return Err(Kind::Cancelled.error("failed role requires fenced parent recovery"));
            }
            if entry.job.run != job.run
                || entry.job.signature != job.signature
                || entry.parent.as_ref() != Some(&parent)
            {
                return Err(
                    Kind::Authentication.error("continuation does not match the recorded parent")
                );
            }
            if entry.terminal_only && !terminal_only {
                return Err(protocol("job is finalizing"));
            }
            entry
                .coordinated
                .clone()
                .ok_or_else(|| protocol("recorded node missing"))?
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let detached = node.detached.notified();
                tokio::pin!(detached);
                detached.as_mut().enable();
                if node
                    .attached
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    break;
                }
                let previous = node.parent.read().await.clone();
                if let Some(previous) = previous {
                    previous.closed().await;
                }
                detached.await;
            }
        })
        .await
        .map_err(|_| Kind::Timeout.error("recorded edge still has an active owner"))?;
        if !peer
            .runs
            .lock()
            .unwrap()
            .get(&job.run.id)
            .and_then(|entry| entry.coordinated.as_ref())
            .is_some_and(|entry| Arc::ptr_eq(entry, &node))
        {
            return Err(Kind::Cancelled.error("expired recorded job"));
        }
        *node.parent.write().await = Some(connection.clone());
        (true, node)
    } else {
        let (_, duplicate) = peer.register(job, Some(&parent))?;
        if duplicate {
            let parent = peer.runs.lock().unwrap()[&job.run.id].parent.clone();
            control
                .send(Frame::Accepted {
                    free: peer.outgoing.has_free(),
                    parent,
                    duplicate: true,
                    hints: vec![],
                })
                .await?;
            return Ok(None);
        }
        let node = Arc::new(Node {
            owner: Arc::downgrade(peer),
            startup: Startup::default(),
            job: job.clone(),
            parent: RwLock::new(Some(connection.clone())),
            children: RwLock::default(),
            attached: AtomicBool::new(true),
            orphan: AtomicBool::new(false),
            reopening: reopening.unwrap_or_default(),
            detached: tokio::sync::Notify::new(),
            capacity: Arc::default(),
            cached: Mutex::default(),
            delta: Mutex::default(),
            baseline: Mutex::new((
                crate::mesh::delta::Watch::new(&peer.store.changes, job.run.id.clone()),
                false,
            )),
            touched: Mutex::new(tokio::time::Instant::now()),
        });
        peer.runs
            .lock()
            .unwrap()
            .get_mut(&job.run.id)
            .unwrap()
            .coordinated = Some(node.clone());
        (false, node)
    };
    Ok(Some((duplicate, node)))
}
struct ControlOutcome {
    // Keep failed control ownership through durable retirement, just like active work.
    control: Option<Control>,
    result: Result<()>,
    idle: bool,
    orphan: bool,
    cache: BTreeMap<u64, Cached>,
}
async fn dispatch_session(
    peer: &Arc<Peer>,
    connection: &Connection,
    mut control: Control,
    node: &Arc<Node>,
    terminal_only: bool,
) -> Result<ControlOutcome> {
    let job = &node.job;
    let (updates, mut observations) = mpsc::channel::<(u64, discovery_job::Observation)>(16);
    let mut work = FuturesUnordered::<AssignmentTask>::new();
    let mut active = BTreeMap::new();
    let mut cache = std::mem::take(&mut *node.cached.lock().unwrap());
    let mut ended = peer.runs.lock().unwrap()[&job.run.id].ended.subscribe();
    let mut local_capacity = peer.outgoing.subscribe();
    node.capacity.record(&peer.local, peer.outgoing.has_free());
    let mut capacity_updates = node.capacity.subscribe();
    let mut sent_capacity = BTreeMap::new();
    control
        .send(Frame::Capacity(node.capacity.delta(&mut sent_capacity)))
        .await?;
    let result = async {
        if *ended.borrow() { return Err(Kind::Cancelled.error("job authority ended before control polling")); }
        loop {
            tokio::select! {
                _ = local_capacity.changed() => { node.capacity.record(&peer.local, peer.outgoing.has_free()); },
                _ = capacity_updates.changed() => {
                    let values = node.capacity.delta(&mut sent_capacity);
                    if !values.is_empty() { control.send(Frame::Capacity(values)).await?; }
                },
                _ = connection.closed() => return Err(Kind::Unresolved.error("active parent connection lost")),
                _ = ended.changed() => return Err(Kind::Authentication.error("job authority ended")),
                Some((id, observation)) = observations.recv() => {
                    control.send(Frame::Observed { id, observation }).await?;
                },
                Some((envelope, reply)) = work.next(), if !work.is_empty() => {
                    active.remove(&envelope.id);
                    if let Some(reply) = reply {
                        let stop = matches!(envelope.command, Command::Stop)
                            && envelope.route.is_empty() && reply.is_ok();
                        let reply = reply.map_err(Fault::from);
                        control.send(Frame::Reply { id: envelope.id, result: reply.clone() }).await?;
                        cache.insert(envelope.id, Cached::new(&envelope, reply)?);
                        if stop { return Ok(()); }
                    }
                }
                frame = control.receive() => match frame? {
                    Frame::Cancel { id } => {
                        if let Some(envelope) = active.remove(&id) {
                            // Drop the operation now: its reserved Apply prefix
                            // must enter the writer queue before cancel/Finish.
                            work.iter_mut().find(|task| task.envelope.id == id).unwrap().cancel();
                            node.cancel(&envelope).await;
                            peer.store.step(Step::Finish(None)).await?;
                            let error = Kind::Cancelled.error("assignment cancelled").into();
                            cache.insert(id, Cached::new(&envelope, Err(error))?);
                        } else if let Some(cached) = cache.get_mut(&id)
                            && !matches!(cached.reply, Ok(Reply::Contact { duplicate: true, .. })) {
                            node.cancel_action(id, cached.cancellation.clone()).await;
                            cached.reply = Err(Kind::Cancelled.error("assignment cancelled").into());
                        }
                    }
                    Frame::Command(envelope) => {
                        if terminal_only && !matches!(envelope.command, Command::Stop) {
                            return Err(protocol("terminal-only continuation cannot exchange records or assign work"));
                        }
                        if let Some(cached) = cache.get(&envelope.id) {
                            if cached.fingerprint != Cached::fingerprint(&envelope)? {
                                return Err(Kind::Conflict.error("control ID reused with different command"));
                            }
                            control.send(Frame::Reply { id: envelope.id, result: cached.reply.clone() }).await?;
                            continue;
                        }
                        if let Some(old) = active.get(&envelope.id) {
                            if old != &envelope { return Err(Kind::Conflict.error("active control ID reused")); }
                            continue;
                        }
                        if active.len() >= 16 {
                            return Err(Kind::Capacity.error("coordinated control capacity reached"));
                        }
                        active.insert(envelope.id, envelope.clone());
                        let task = node.clone().execute(envelope.clone(), Some(updates.clone()));
                        work.push(AssignmentTask { envelope, task: Some(Box::pin(task)), wake: None });
                    }
                    _ => return Err(protocol("coordinated command expected")),
                }
            }
        }
    }.await;
    let (result, control) = if result.is_ok() {
        (control.finish().await, None)
    } else {
        (result, Some(control))
    };
    let idle = active.is_empty()
        && !terminal_only
        && !*ended.borrow()
        && pressure::was_idle_release(connection);
    let orphan = !idle && !*ended.borrow() && result.is_err();
    drop(work);
    Ok(ControlOutcome {
        control,
        result,
        idle,
        orphan,
        cache,
    })
}
async fn retire_session(
    peer: &Arc<Peer>,
    connection: &Connection,
    node: &Arc<Node>,
    running: &mut Running,
    outcome: ControlOutcome,
) -> Result<()> {
    let job = &node.job;
    let ControlOutcome {
        control: _control,
        result,
        idle,
        orphan,
        cache,
    } = outcome;
    peer.store.step(Step::Finish(None)).await?;
    if orphan {
        node.orphan.store(true, Ordering::Release);
        node.children.write().await.clear();
        if let Some(entry) = peer.runs.lock().unwrap().get_mut(&job.run.id) {
            entry.revoke_sources();
            entry.children.clear();
            entry.phase = Phase::Cancelled;
        }
    }
    if idle || orphan {
        for edge in node.children.read().await.values() {
            edge.release_idle();
        }
        if idle {
            *node.cached.lock().unwrap() = cache;
        }
        *node.parent.write().await = None;
        running.retire = false;
        *node.touched.lock().unwrap() = tokio::time::Instant::now();
        node.attached.store(false, Ordering::Release);
        node.detached.notify_waiters();
    }
    if let Err(error) = &result {
        // Preserve the typed terminal failure before the control owner can emit EOF.
        connection.close(Kind::of(error).code().into(), b"mesh stream failed");
    }
    result
}

struct Network {
    registrations: tokio::sync::Mutex<registrations::Registrations>,
    node: Arc<Node>,
    paths: Arc<RwLock<BTreeMap<String, Vec<String>>>>,
    recoveries: RwLock<BTreeMap<String, String>>,
}
impl discovery_job::Dialer for Network {
    fn capacity(&self) -> ((u64, bool), BTreeMap<String, bool>) {
        let ready = self
            .node
            .peer()
            .ok()
            .is_some_and(|peer| peer.outgoing.has_free());
        if let Ok(peer) = self.node.peer() {
            self.node.capacity.record(&peer.local, ready);
        }
        let (revision, values) = self.node.capacity.snapshot();
        ((revision, ready), values)
    }
    async fn capacity_changed(&self, revision: (u64, bool)) {
        if let Ok(peer) = self.node.peer() {
            tokio::select! {
                _ = self.node.capacity.changed_after(revision.0) => (),
                _ = peer.outgoing.changed_after(revision.1) => (),
            }
        }
    }
    async fn watch_targets(&self, unresolved: Vec<String>, targets: Vec<String>) {
        if let Ok(peer) = self.node.peer() {
            self.registrations
                .lock()
                .await
                .update(&peer, unresolved, targets);
        }
    }
    async fn registration(&self) -> discovery_job::Registration {
        self.registrations.lock().await.next().await
    }

    fn snapshot(&self, mut snapshot: crate::mesh::status::Traversal) {
        if let Ok(peer) = self.node.peer() {
            let mut runs = peer.runs.lock().unwrap();
            if let Some(entry) = runs.get_mut(&self.node.job.run.id) {
                snapshot.round = snapshot
                    .round
                    .max(entry.report.traversal.as_ref().map_or(1, |s| s.round));
                if entry.report.traversal.as_deref() != Some(&snapshot) {
                    entry.report.traversal = Some(Box::new(snapshot));
                    entry.progress.advance();
                }
            }
        }
    }
    type Link = String;
    async fn contact(
        &self,
        assignment: Assignment,
        updates: mpsc::Sender<discovery_job::Observation>,
    ) -> discovery_job::Attempt<String> {
        let (observations, mut observed) = mpsc::channel(8);
        let route = self.paths.read().await.get(&assignment.source).cloned();
        let replaces_parent = self
            .recoveries
            .read()
            .await
            .get(&assignment.target)
            .cloned();
        let result = async {
            let _slot = self.node.startup.slot().await?;
            let route = route.ok_or_else(|| protocol("source has no admitted tree route"))?;
            let command = Command::Contact {
                assignment: assignment.id,
                target: assignment.target.clone(),
                routes: assignment.routes,
                replaces_parent,
            };
            let envelope = Envelope { id: assignment.id, route, command };
            let operation = self.node.clone().execute(envelope, Some(observations));
            tokio::pin!(operation);
            loop {
                tokio::select! {
                    result = &mut operation => return result,
                    Some((_, observation)) = observed.recv() => { let _ = updates.try_send(observation); }
                }
            }
        }.await;
        if let Ok(Reply::Contact { free, .. }) = &result {
            self.node.capacity.record(&assignment.target, *free);
        }
        match result {
            Ok(Reply::Contact {
                direct,
                free: _,
                parent,
                duplicate,
                via_relay,
                hints,
                observation,
            }) => discovery_job::Attempt {
                direct: if direct {
                    discovery_job::Direct::Attempted
                } else {
                    discovery_job::Direct::Unavailable
                },
                observation,
                result: if duplicate {
                    Ok(discovery_job::Admission::Registered { parent })
                } else {
                    let mut path = self
                        .paths
                        .read()
                        .await
                        .get(&assignment.source)
                        .cloned()
                        .unwrap_or_default();
                    path.push(assignment.target.clone());
                    self.paths
                        .write()
                        .await
                        .insert(assignment.target.clone(), path);
                    Ok(discovery_job::Admission::Accepted {
                        link: assignment.target,
                        via_relay,
                        hints,
                    })
                },
            },
            Ok(Reply::NoDirectRoute) => discovery_job::Attempt {
                direct: discovery_job::Direct::Unavailable,
                observation: None,
                result: Ok(discovery_job::Admission::NoDirectRoute),
            },
            Ok(Reply::ContactFailed {
                direct,
                kind,
                observation,
                ..
            }) => discovery_job::Attempt {
                direct: if direct {
                    discovery_job::Direct::Attempted
                } else {
                    discovery_job::Direct::Unavailable
                },
                observation,
                result: Err(kind),
            },
            Ok(_) => discovery_job::Attempt {
                direct: discovery_job::Direct::Unknown,
                observation: None,
                result: Err(Kind::Protocol),
            },
            Err(error) => discovery_job::Attempt {
                direct: discovery_job::Direct::Unknown,
                observation: None,
                result: Err(Kind::of(&error)),
            },
        }
    }
}

async fn recover(
    network: &Network,
    scheduler: &mut Scheduler,
    error: &Error,
    request: &mut u64,
    origin: tokio::time::Instant,
    pending_up: &mut BTreeSet<String>,
    report: &mut Report,
) -> Result<bool> {
    let Error::Mesh(error) = error else {
        return Ok(false);
    };
    if error.local
        || !matches!(
            error.kind,
            Kind::Offline | Kind::Timeout | Kind::Unresolved | Kind::Cancelled
        )
    {
        return Ok(false);
    }
    let Some(failed) = error.peer.as_ref() else {
        return Ok(false);
    };
    let Some(state) = scheduler.reached().get(failed) else {
        return Ok(false);
    };
    let Some(parent) = state.parent.clone() else {
        return Ok(false);
    };
    if !scheduler.discovery_open(origin.elapsed()) {
        return Ok(false);
    }
    let route = network.paths.read().await[&parent].clone();
    *request += 1;
    if network
        .node
        .clone()
        .execute(
            Envelope {
                id: *request,
                route,
                command: Command::Detach {
                    child: failed.clone(),
                },
            },
            None,
        )
        .await
        .is_err()
    {
        return Ok(false);
    }
    let Some(removed) = scheduler.recover_subtree(failed, origin.elapsed()) else {
        return Ok(false);
    };
    {
        let mut paths = network.paths.write().await;
        let mut replacements = network.recoveries.write().await;
        for (id, state) in &removed {
            network.node.capacity.forget(id);
            paths.remove(id);
            if let Some(parent) = &state.parent {
                replacements.insert(id.clone(), parent.clone());
            }
            report.distributed.retain(|peer| peer != id);
            pending_up.remove(id);
        }
    }
    let before: BTreeSet<_> = scheduler.reached().keys().cloned().collect();
    let (discover, _admissions) = discovery_job::start(scheduler.clone(), network, origin);
    let discovery = discover.await;
    *scheduler = discovery.scheduler;
    for (target, kind) in discovery.failures {
        report.failures.push(failure(
            &target,
            Phase::Distributing,
            &kind.error("failed branch could not be recovered"),
        ));
    }
    let added: Vec<_> = scheduler
        .reached()
        .keys()
        .filter(|id| !before.contains(*id))
        .cloned()
        .collect();
    for id in &added {
        let mut next = Some(id.clone());
        while let Some(id) = next {
            if id == network.node.peer()?.local {
                break;
            }
            pending_up.insert(id.clone());
            next = scheduler.reached()[&id].parent.clone();
        }
    }
    for id in removed
        .keys()
        .filter(|id| !scheduler.reached().contains_key(*id))
    {
        report.failures.push(failure(
            id,
            Phase::Distributing,
            &Kind::Timeout.error("failed subtree has no completed replacement route"),
        ));
    }
    Ok(!added.is_empty())
}

struct InitialCollection {
    discovery: discovery_job::Discovery<String>,
    data: InitialData,
    revision: u64,
}
struct InitialData {
    contributions: BTreeMap<String, BTreeSet<String>>,
    sent: BTreeMap<String, BTreeSet<String>>,
    baselines: BTreeMap<String, u64>,
    failures: Vec<(String, Phase, Error)>,
}
struct InitialTransfer {
    source: String,
    target: String,
    upward: bool,
}
struct InitialOutcome {
    transfer: InitialTransfer,
    contribution: BTreeSet<String>,
    revision: u64,
    delta_baseline: Option<u64>,
    result: Result<Reply>,
}
impl InitialTransfer {
    fn start_baseline(&self, peer: &Peer, network: &Network) -> (u64, Option<u64>) {
        let revision = peer.store.changes.revision();
        let delta = if !self.upward {
            network
                .node
                .delta
                .lock()
                .unwrap()
                .get_or_insert_with(|| {
                    crate::mesh::delta::Delta::new(
                        &peer.store.changes,
                        network.node.job.run.id.clone(),
                    )
                })
                .start()
        } else {
            None
        };
        (revision, delta)
    }

    async fn execute(&self, network: &Network, id: u64) -> Result<Reply> {
        let destination = if self.upward {
            &self.source
        } else {
            &self.target
        };
        let route = network
            .paths
            .read()
            .await
            .get(destination)
            .cloned()
            .ok_or_else(|| protocol("admitted peer has no tree route"))?;
        network
            .node
            .clone()
            .execute(
                Envelope {
                    id,
                    route,
                    command: Command::Pull {
                        round: 1,
                        child: self.upward.then(|| self.target.clone()),
                        delta: false,
                    },
                },
                None,
            )
            .await
    }
}
impl InitialData {
    fn completed(
        &mut self,
        outcome: InitialOutcome,
        network: &Network,
        peer: &Peer,
        report: &mut Report,
    ) {
        let transfer = outcome.transfer;
        match outcome.result {
            Ok(Reply::Done { .. }) => {
                if transfer.upward {
                    self.contributions
                        .entry(transfer.source)
                        .or_default()
                        .extend(outcome.contribution.clone());
                    self.sent.insert(transfer.target, outcome.contribution);
                    report.collected = self.contributions[&peer.local].iter().cloned().collect();
                } else {
                    if let Some(start) = outcome.delta_baseline {
                        network
                            .node
                            .delta
                            .lock()
                            .unwrap()
                            .as_mut()
                            .unwrap()
                            .baseline(transfer.target.clone(), start);
                    }
                    self.baselines
                        .insert(transfer.target.clone(), outcome.revision);
                    report.distributed.push(transfer.target);
                }
                peer.phase(&network.node.job.run.id, Phase::Collecting, report);
            }
            result => self.failures.push((
                transfer.target,
                if transfer.upward {
                    Phase::Collecting
                } else {
                    Phase::Distributing
                },
                match result {
                    Err(error) => error,
                    _ => protocol("data acknowledgement expected"),
                },
            )),
        }
    }
}
async fn collect_initial(
    network: &Network,
    scheduler: Scheduler,
    origin: tokio::time::Instant,
    request: &mut u64,
    report: &mut Report,
) -> Result<InitialCollection> {
    use discovery_job::Dialer;
    let peer = network.node.peer()?;
    let (discovery, mut admissions) = discovery_job::start(scheduler, network, origin);
    tokio::pin!(discovery);
    let mut finished: Option<discovery_job::Discovery<String>> = None;
    let mut retained_failures = BTreeMap::new();
    let mut closed = false;
    let mut sealed = false;
    let mut distributing = false;
    let mut l1 = BTreeSet::<String>::new();
    let mut l1_pending = BTreeSet::new();
    let mut queue = std::collections::VecDeque::<InitialTransfer>::new();
    let mut active = FuturesUnordered::new();
    let mut per_parent = BTreeMap::<(String, bool), usize>::new();
    let mut data = InitialData {
        contributions: BTreeMap::from([(peer.local.clone(), BTreeSet::from([peer.local.clone()]))]),
        sent: BTreeMap::new(),
        baselines: BTreeMap::new(),
        failures: Vec::new(),
    };
    loop {
        if sealed && l1_pending.is_empty() && !distributing {
            distributing = true;
            for target in l1
                .iter()
                .rev()
                .filter(|target| data.sent.contains_key(*target))
            {
                queue.push_front(InitialTransfer {
                    source: peer.local.clone(),
                    target: target.clone(),
                    upward: false,
                });
            }
        }
        while active.len() < 8 {
            let Some(index) = queue.iter().position(|transfer| {
                per_parent
                    .get(&(transfer.source.clone(), transfer.upward))
                    .copied()
                    .unwrap_or(0)
                    < 2
            }) else {
                break;
            };
            let transfer = queue.remove(index).unwrap();
            *per_parent
                .entry((transfer.source.clone(), transfer.upward))
                .or_default() += 1;
            let snapshot = data.contributions[&transfer.target].clone();
            let (baseline, delta_baseline) = transfer.start_baseline(&peer, network);
            *request += 1;
            let id = *request;
            active.push(async move {
                let result = transfer.execute(network, id).await;
                InitialOutcome {
                    transfer,
                    contribution: snapshot,
                    revision: baseline,
                    delta_baseline,
                    result,
                }
            });
        }
        let cap = finished
            .as_ref()
            .and_then(|result| result.scheduler.open_until());
        tokio::select! {
            biased;
            _ = async { tokio::time::sleep_until(origin + cap.unwrap()).await }, if cap.is_some() => {
                let current = finished.as_mut().unwrap();
                current.scheduler.close_discovery();
                network.watch_targets(vec![], vec![]).await;
                network.snapshot(current.scheduler.snapshot(origin.elapsed()));
            }
            event = discovery_job::Dialer::registration(network), if finished.is_some() => {
                let mut current = finished.take().unwrap();
                let retry = discovery_job::registration(
                    &mut current.scheduler, &mut current.failures, event, origin.elapsed(),
                );
                if retry {
                    retained_failures.extend(current.failures);
                    let (resumed, receiver) = discovery_job::start(current.scheduler, network, origin);
                    discovery.set(resumed);
                    admissions = receiver;
                    closed = false;
                } else { finished = Some(current); }
            },
            _ = std::future::ready(()),
                if finished.is_some() && closed && queue.is_empty() && active.is_empty() => break,
            mut result = &mut discovery, if finished.is_none() => {
                wire::metrics::milestone(&network.node.job.run.id, "quiescent", origin.elapsed());
                for (target, kind) in std::mem::take(&mut retained_failures) {
                    let fatal = !matches!(kind, Kind::Offline | Kind::Timeout | Kind::Cancelled)
                        || result.scheduler.failed_direct(&target);
                    if fatal || !result.scheduler.reached().contains_key(&target) {
                        result.failures.entry(target).and_modify(|old| {
                            if matches!(*old, Kind::Offline | Kind::Timeout | Kind::Unresolved | Kind::Cancelled) {
                                *old = kind;
                            }
                        }).or_insert(kind);
                    }
                }
                finished = Some(result);
            },
            event = admissions.recv(), if !closed => match event {
                Some(discovery_job::Event::Accepted(admission)) => {
                    let assignment = admission.assignment;
                    if assignment.source == peer.local && !assignment.refresh && !sealed {
                        l1.insert(assignment.target.clone());
                        l1_pending.insert(assignment.target.clone());
                    }
                    data.contributions.entry(assignment.target.clone())
                        .or_insert_with(|| BTreeSet::from([assignment.target.clone()]));
                    queue.push_back(InitialTransfer {
                        source: assignment.source, target: assignment.target, upward: true,
                    });
                }
                Some(discovery_job::Event::InitialComplete) => {
                    wire::metrics::milestone(&network.node.job.run.id, "initial_attempts_settled", origin.elapsed());
                    sealed = true;
                },
                None => closed = true,
            },
            Some(outcome) = active.next(), if !active.is_empty() => {
                *per_parent.get_mut(&(outcome.transfer.source.clone(), outcome.transfer.upward)).unwrap() -= 1;
                if outcome.transfer.upward { l1_pending.remove(&outcome.transfer.target); }
                data.completed(outcome, network, &peer, report);
            },
        }
    }
    Ok(InitialCollection {
        discovery: finished.unwrap(),
        data,
        revision: peer.store.changes.revision(),
    })
}

pub(super) async fn synchronize_started(
    peer: Arc<Peer>,
    run: String,
    startup: Startup,
) -> Result<Report> {
    Coordinator::prepare(peer, run, startup).await?.run().await
}

struct Coordinator {
    _running: Running,
    peer: Arc<Peer>,
    network: Network,
    scheduler: Scheduler,
    origin: tokio::time::Instant,
}
impl Coordinator {
    async fn prepare(peer: Arc<Peer>, run: String, startup: Startup) -> Result<Self> {
        peer.store.step(Step::Finish(None)).await?;
        let membership = store::membership(&peer.store, None)
            .await?
            .ok_or_else(|| protocol("membership missing"))?;
        let origin = startup
            .verified_at
            .unwrap_or_else(tokio::time::Instant::now);
        let job = SignedRun::new(&peer.key, &membership, run)?;
        peer.verify_job(&job).await?;
        peer.register(&job, None)?;
        let _running = Running::new(&peer, &job.run.id, true);
        let node = Arc::new(Node {
            owner: Arc::downgrade(&peer),
            startup,
            job: job.clone(),
            parent: RwLock::new(None),
            children: RwLock::default(),
            attached: AtomicBool::new(true),
            orphan: AtomicBool::new(false),
            reopening: Default::default(),
            detached: tokio::sync::Notify::new(),
            capacity: Arc::default(),
            cached: Mutex::default(),
            delta: Mutex::default(),
            baseline: Mutex::new((
                crate::mesh::delta::Watch::new(&peer.store.changes, job.run.id.clone()),
                false,
            )),
            touched: Mutex::new(tokio::time::Instant::now()),
        });
        peer.runs
            .lock()
            .unwrap()
            .get_mut(&job.run.id)
            .unwrap()
            .coordinated = Some(node.clone());
        let network = Network {
            registrations: Default::default(),
            node: node.clone(),
            recoveries: RwLock::default(),
            paths: Arc::new(RwLock::new(BTreeMap::from([(peer.local.clone(), vec![])]))),
        };
        let scheduler = Scheduler::new(
            peer.local.clone(),
            job.run.cohort.clone(),
            peer.relay.to_string(),
        )
        .ok_or_else(|| protocol("invalid coordinator cohort"))?;
        Ok(Self {
            _running,
            peer,
            network,
            scheduler,
            origin,
        })
    }
    async fn run(self) -> Result<Report> {
        let mut report = Report::new(&self.network.node.job.run, &self.peer.local);
        let mut request = 1u64 << 32;
        let initial = collect_initial(
            &self.network,
            self.scheduler,
            self.origin,
            &mut request,
            &mut report,
        )
        .await?;
        reconciliation::finish(&self.network, initial, report, self.origin, request).await
    }
}

#[cfg(test)]
mod tests;
