//! Final reconciliation and terminal acknowledgements over the registered tree.
//! @ref LLP 0008#bounded-forwarding-and-duplicate-control
use super::*;

struct FinalRound<'a> {
    network: &'a Network,
    scheduler: Scheduler,
    origin: tokio::time::Instant,
    request: u64,
    contributions: BTreeMap<String, BTreeSet<String>>,
    pending_up: BTreeSet<String>,
    report: Report,
}
pub(super) async fn finish(
    network: &Network,
    initial: InitialCollection,
    mut report: Report,
    origin: tokio::time::Instant,
    mut request: u64,
) -> Result<Report> {
    let peer = network.node.peer()?;
    let job = &network.node.job;
    let discovery = initial.discovery;
    for (target, kind) in discovery.failures {
        report.failures.push(failure(
            &target,
            Phase::Collecting,
            &kind.error(if kind == Kind::Version {
                format!("update linn on peer {target}, then retry")
            } else {
                "peer contact did not complete".into()
            }),
        ));
    }
    let mut scheduler = discovery.scheduler;
    let mut snapshot = scheduler.snapshot(origin.elapsed());
    snapshot.round = 2;
    report.traversal = Some(Box::new(snapshot));
    peer.phase(&job.run.id, Phase::Collecting, &mut report);
    let contributions = initial.data.contributions;
    let mut pending_up: BTreeSet<_> = scheduler
        .reached()
        .keys()
        .filter(|id| {
            *id != &peer.local
                && initial
                    .data
                    .sent
                    .get(*id)
                    .is_none_or(|sent| !contributions[*id].is_subset(sent))
        })
        .cloned()
        .collect();
    for target in pending_up.clone() {
        let mut parent = scheduler.reached()[&target].parent.clone();
        while let Some(id) = parent.filter(|id| id != &peer.local) {
            pending_up.insert(id.clone());
            parent = scheduler.reached()[&id].parent.clone();
        }
    }
    report
        .distributed
        .retain(|target| initial.data.baselines.get(target) == Some(&initial.revision));
    // Retire all old-role results before a recovery creates new obligations.
    for (target, phase, error) in &initial.data.failures {
        report.failures.push(failure(target, *phase, error));
        if *phase == Phase::Collecting {
            pending_up.remove(target);
        }
    }
    for (_, _, error) in initial.data.failures {
        recover(
            network,
            &mut scheduler,
            &error,
            &mut request,
            origin,
            &mut pending_up,
            &mut report,
        )
        .await?;
    }
    FinalRound {
        network,
        scheduler,
        origin,
        request,
        contributions,
        pending_up,
        report,
    }
    .run()
    .await
}

impl FinalRound<'_> {
    async fn run(mut self) -> Result<Report> {
        loop {
            let mut nodes: Vec<_> = self
                .scheduler
                .reached()
                .iter()
                .filter(|(id, _)| *id != &self.network.node.job.run.initiator)
                .map(|(id, state)| (id.clone(), state.clone()))
                .collect();
            nodes.sort_by_key(|(id, state)| (std::cmp::Reverse(state.level), id.clone()));
            if self.collect(&nodes).await? {
                continue;
            }
            if !self.distribute(&nodes).await? && self.pending_up.is_empty() {
                break;
            }
        }
        self.finish().await
    }
    async fn collect(&mut self, nodes: &[(String, crate::mesh::scheduler::Node)]) -> Result<bool> {
        let peer = self.network.node.peer()?;
        let node = &self.network.node;
        let job = &node.job;
        for (target, state) in nodes {
            if !self.pending_up.contains(target) || !self.scheduler.reached().contains_key(target) {
                continue;
            }
            let parent = state.parent.as_ref().unwrap();
            peer.phase(&job.run.id, Phase::Collecting, &mut self.report);
            self.request += 1;
            let route = self.network.paths.read().await[parent].clone();
            match registrations::reconcile(
                self.network,
                &mut self.scheduler,
                self.origin,
                node.clone().execute(
                    Envelope {
                        id: self.request,
                        route,
                        command: Command::Pull {
                            round: 2,
                            child: Some(target.clone()),
                            delta: false,
                        },
                    },
                    None,
                ),
                &mut self.pending_up,
                &mut self.contributions,
                &mut self.report,
            )
            .await
            {
                Ok(Reply::Done { changed }) => {
                    let received = self
                        .contributions
                        .entry(target.clone())
                        .or_insert_with(|| BTreeSet::from([target.clone()]))
                        .clone();
                    self.contributions
                        .entry(parent.clone())
                        .or_default()
                        .extend(received);
                    self.pending_up.remove(target);
                    if parent == &peer.local && changed > 0 {
                        self.report.distributed.clear();
                    }
                }
                Ok(_) => return Err(protocol("collection acknowledgement expected")),
                Err(error) => {
                    self.report
                        .failures
                        .push(failure(target, Phase::Collecting, &error));
                    self.pending_up.remove(target);
                    if recover(
                        self.network,
                        &mut self.scheduler,
                        &error,
                        &mut self.request,
                        self.origin,
                        &mut self.pending_up,
                        &mut self.report,
                    )
                    .await?
                    {
                        return Ok(true);
                    }
                }
            }
        }
        self.report.collected = self.contributions[&peer.local].iter().cloned().collect();
        peer.phase(&job.run.id, Phase::Collecting, &mut self.report);

        self.pending_up
            .retain(|id| self.scheduler.reached().contains_key(id));
        Ok(false)
    }
    async fn distribute(
        &mut self,
        nodes: &[(String, crate::mesh::scheduler::Node)],
    ) -> Result<bool> {
        let peer = self.network.node.peer()?;
        let node = &self.network.node;
        let job = &node.job;
        self.report.distributed.push(peer.local.clone());
        peer.phase(&job.run.id, Phase::Distributing, &mut self.report);
        for (target, state) in nodes.iter().rev() {
            if !self.scheduler.reached().contains_key(target)
                || self.report.distributed.contains(target)
            {
                continue;
            }
            let parent = state.parent.as_ref().unwrap();
            if !self.report.distributed.contains(parent) {
                self.report.failures.push(failure(
                    target,
                    Phase::Distributing,
                    &Kind::Unresolved
                        .error("parent did not acknowledge the final contribution set"),
                ));
                continue;
            }
            self.request += 1;
            let route = self.network.paths.read().await[target].clone();
            let operation = node.clone().execute(
                Envelope {
                    id: self.request,
                    route,
                    command: Command::Pull {
                        round: 2,
                        child: None,
                        delta: parent == &peer.local && node.delta_eligible(target),
                    },
                },
                None,
            );
            let result = registrations::reconcile(
                self.network,
                &mut self.scheduler,
                self.origin,
                operation,
                &mut self.pending_up,
                &mut self.contributions,
                &mut self.report,
            )
            .await;
            match result {
                Ok(_) => self.report.distributed.push(target.clone()),
                Err(error) => {
                    self.report
                        .failures
                        .push(failure(target, Phase::Distributing, &error));
                    if recover(
                        self.network,
                        &mut self.scheduler,
                        &error,
                        &mut self.request,
                        self.origin,
                        &mut self.pending_up,
                        &mut self.report,
                    )
                    .await?
                    {
                        return Ok(true);
                    }
                }
            }
            peer.phase(&job.run.id, Phase::Distributing, &mut self.report);
        }
        Ok(false)
    }
    async fn finish(mut self) -> Result<Report> {
        let peer = self.network.node.peer()?;
        let node = &self.network.node;
        let job = &node.job;
        self.scheduler.close_discovery();
        discovery_job::Dialer::snapshot(
            self.network,
            self.scheduler.snapshot(self.origin.elapsed()),
        );
        self.network
            .registrations
            .lock()
            .await
            .qualify(&mut self.report);
        discovery_job::Dialer::watch_targets(self.network, vec![], vec![]).await;
        let mut nodes: Vec<_> = self
            .scheduler
            .reached()
            .iter()
            .filter(|(id, _)| *id != &peer.local)
            .collect();
        nodes.sort_by_key(|(id, state)| (state.level, (*id).clone()));
        // A parent's final acknowledgement also covers its retained children.
        for (id, _) in nodes.iter().rev() {
            if !self.report.distributed.contains(id) {
                invalidate_coverage(&mut self.report, &self.scheduler, id);
            }
        }
        // Retire children bottom-up only after all final replies have arrived.
        nodes.reverse();
        for (target, _) in nodes {
            self.request += 1;
            let route = self.network.paths.read().await[target].clone();
            if let Err(error) = node
                .clone()
                .execute(
                    Envelope {
                        id: self.request,
                        route,
                        command: Command::Stop,
                    },
                    None,
                )
                .await
            {
                invalidate_coverage(&mut self.report, &self.scheduler, target);
                let error = if Kind::of(&error) == Kind::Offline {
                    Kind::Timeout.error("terminal acknowledgement unresolved after connection loss")
                } else {
                    error
                };
                self.report
                    .failures
                    .push(failure(target, Phase::Distributing, &error));
            }
            peer.phase(&job.run.id, Phase::Distributing, &mut self.report);
        }
        peer.store.step(Step::Finish(None)).await?;
        for failure in &mut self.report.failures {
            if failure.kind == Kind::Offline && !self.scheduler.absence_complete(&failure.peer) {
                failure.kind = Kind::Timeout;
            }
        }
        let fatal: Vec<_> = self
            .report
            .failures
            .iter()
            .filter(|failure| {
                !matches!(
                    failure.kind,
                    Kind::Offline | Kind::Timeout | Kind::Unresolved | Kind::Cancelled
                ) || self.scheduler.failed_direct(&failure.peer)
            })
            .cloned()
            .collect();
        self.report.normalize();
        for failure in fatal {
            if !self
                .report
                .failures
                .iter()
                .any(|other| other.peer == failure.peer && other.phase == failure.phase)
            {
                self.report.failures.push(failure);
            }
        }
        self.report.complete &= self.report.failures.is_empty();
        peer.phase(
            &job.run.id,
            if self.report.complete {
                Phase::Complete
            } else {
                Phase::Failed
            },
            &mut self.report,
        );
        Ok(self.report)
    }
}

fn invalidate_coverage(report: &mut Report, scheduler: &Scheduler, target: &str) {
    let mut failed = Some(target.to_owned());
    while let Some(id) = failed {
        report.distributed.retain(|covered| covered != &id);
        failed = scheduler.reached()[&id].parent.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lost_terminal_ack_stays_unresolved_after_report_normalization() {
        let mut scheduler = Scheduler::new(
            "root".into(),
            vec!["child".into(), "root".into()],
            "relay".into(),
        )
        .unwrap();
        let attempt = scheduler.schedule(Duration::ZERO).remove(0);
        scheduler.accept(attempt.id, false, Duration::ZERO).unwrap();
        let mut report = Report {
            cohort: vec!["child".into(), "root".into()],
            collected: vec!["child".into(), "root".into()],
            distributed: vec!["child".into(), "root".into()],
            complete: true,
            ..Report::default()
        };
        invalidate_coverage(&mut report, &scheduler, "child");
        report.failures.push(failure(
            "child",
            Phase::Distributing,
            &Kind::Timeout.error("terminal acknowledgement unresolved"),
        ));
        report.normalize();
        assert!(!report.complete);
        assert!(!report.unavailable_only());
        assert!(report.distributed.is_empty());
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].kind, Kind::Timeout);
    }
}
