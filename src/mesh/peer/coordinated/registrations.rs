//! Job-owned unresolved-target subscriptions, independent of data-round progress.
use super::*;
use discovery_job::Registration;
use iroh::endpoint::{TargetWatch, TargetWatchEvent};

#[derive(Default)]
pub(super) struct Registrations {
    armed: BTreeSet<String>,
    unknown_targets: BTreeSet<String>,
    active: BTreeMap<String, TargetWatch>,
    unknown: std::collections::VecDeque<String>,
}
impl Registrations {
    pub(super) fn update(&mut self, peer: &Peer, unresolved: Vec<String>, targets: Vec<String>) {
        self.armed.retain(|target| unresolved.contains(target));
        self.unknown_targets
            .retain(|target| unresolved.contains(target));
        self.active.retain(|target, _| unresolved.contains(target));
        self.unknown.retain(|target| unresolved.contains(target));
        for target in targets {
            if !self.armed.insert(target.clone()) {
                continue;
            }
            let Ok(bytes) = unhex::<32>(&target) else {
                continue;
            };
            let Ok(id) = PublicKey::from_bytes(&bytes) else {
                continue;
            };
            if let Some(watch) = peer
                .endpoint
                .watch_target_registration(peer.relay.clone(), id)
            {
                self.active.insert(target, watch);
            } else {
                self.unknown.push_back(target);
            }
        }
    }
    pub(super) fn qualify(&self, report: &mut Report) {
        for failure in &mut report.failures {
            if failure.kind == Kind::Offline && self.unknown_targets.contains(&failure.peer) {
                failure.kind = Kind::Timeout;
                failure.message = "unresolved: relay registration notifications unavailable".into();
            }
        }
    }
    pub(super) async fn next(&mut self) -> Registration {
        if let Some(target) = self.unknown.pop_front() {
            self.unknown_targets.insert(target.clone());
            return Registration {
                target,
                registered: false,
            };
        }
        let event = {
            let mut pending = FuturesUnordered::new();
            for (target, watch) in &mut self.active {
                pending.push(async move { (target.clone(), watch.event().await) });
            }
            match pending.next().await {
                Some(event) => event,
                None => std::future::pending().await,
            }
        };
        if matches!(event.1, TargetWatchEvent::Unknown) {
            self.active.remove(&event.0);
            self.unknown_targets.insert(event.0.clone());
        }
        Registration {
            target: event.0,
            registered: matches!(event.1, TargetWatchEvent::Registered { .. }),
        }
    }
}

/// Keep progressing transfers alive while registration-triggered discovery runs.
pub(super) async fn reconcile(
    network: &Network,
    scheduler: &mut Scheduler,
    origin: tokio::time::Instant,
    operation: impl Future<Output = Result<Reply>>,
    pending_up: &mut BTreeSet<String>,
    contributions: &mut BTreeMap<String, BTreeSet<String>>,
    report: &mut Report,
) -> Result<Reply> {
    use crate::mesh::discovery_job::Dialer;
    use futures_util::FutureExt;
    let before: BTreeSet<_> = scheduler.reached().keys().cloned().collect();
    if scheduler.discovery_open(origin.elapsed()) {
        network
            .watch_targets(
                scheduler.missing().map(str::to_owned).collect(),
                scheduler.watch_targets(origin.elapsed()),
            )
            .await;
    }
    tokio::pin!(operation);
    let mut result = None;
    let mut discovery: Option<
        Pin<Box<dyn Future<Output = discovery_job::Discovery<String>> + Send + '_>>,
    > = None;
    let mut admissions = None;
    loop {
        let cap = scheduler.open_until();
        if discovery.is_none() && cap.is_some_and(|cap| origin.elapsed() >= cap) {
            scheduler.close_discovery();
            network.watch_targets(vec![], vec![]).await;
            network.snapshot(scheduler.snapshot(origin.elapsed()));
        }
        let cap = scheduler.open_until();
        let event = if result.is_some() && discovery.is_none() {
            match network.registration().now_or_never() {
                Some(event) => Some(event),
                None => break,
            }
        } else {
            tokio::select! {
                biased;
                _ = async { tokio::time::sleep_until(origin + cap.unwrap()).await },
                    if cap.is_some() && discovery.is_none() => None,
                event = network.registration(), if discovery.is_none() => Some(event),
                found = async { discovery.as_mut().unwrap().await }, if discovery.is_some() => {
                    wire::metrics::milestone(&network.node.job.run.id, "quiescent", origin.elapsed());
                    *scheduler = found.scheduler;
                    for (target, kind) in found.failures {
                        report.failures.push(failure(&target, Phase::Collecting,
                            &kind.error(if kind == Kind::Version { format!("update linn on peer {target}, then retry") }
                            else { "registration-triggered contact did not complete".into() })));
                    }
                    discovery = None;
                    admissions = None;
                    None
                },
                completed = &mut operation, if result.is_none() => { result = Some(completed); None },
            }
        };
        if let Some(event) = event {
            let mut failures = BTreeMap::new();
            let retry =
                discovery_job::registration(scheduler, &mut failures, event, origin.elapsed());
            for (target, kind) in failures {
                report.failures.retain(|failure| {
                    failure.peer != target
                        || !matches!(
                            failure.kind,
                            Kind::Offline | Kind::Timeout | Kind::Cancelled
                        )
                });
                report.failures.push(failure(
                    &target,
                    Phase::Collecting,
                    &kind.error("unresolved pending verified contact"),
                ));
            }
            network.snapshot(scheduler.snapshot(origin.elapsed()));
            let peer = network.node.peer()?;
            let phase = peer
                .runs
                .lock()
                .unwrap()
                .get(&network.node.job.run.id)
                .map(|entry| entry.phase);
            if let Some(phase) = phase {
                peer.phase(&network.node.job.run.id, phase, report);
            }
            if retry {
                let (future, receiver) = discovery_job::start(scheduler.clone(), network, origin);
                discovery = Some(Box::pin(future));
                admissions = Some(receiver);
            }
        }
    }
    // Keep the bounded admission receiver owned until its producer has settled.
    drop(admissions);
    let peer = network.node.peer()?;
    for target in scheduler
        .reached()
        .keys()
        .filter(|target| !before.contains(*target))
    {
        contributions
            .entry(target.clone())
            .or_insert_with(|| BTreeSet::from([target.clone()]));
        let mut parent = Some(target.clone());
        while let Some(id) = parent.filter(|id| id != &peer.local) {
            pending_up.insert(id.clone());
            parent = scheduler.reached()[&id].parent.clone();
        }
    }
    network.registrations.lock().await.qualify(report);
    result.unwrap()
}
