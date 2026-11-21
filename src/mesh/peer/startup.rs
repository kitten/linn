//! Job-owned transport-only speculation; membership gates adoption and all data.
//! @ref LLP 0008#manual-mesh-jobs
use super::*;
use tokio::sync::Semaphore;

pub(super) type DialUpdate =
    Arc<dyn Fn(bool, Option<iroh::endpoint::RelayDialObservation>) + Send + Sync>;
#[derive(Default)]
pub(super) struct DialState {
    deadline: Mutex<Option<tokio::time::Instant>>,
    evidence: Mutex<Evidence>,
}
#[derive(Default)]
struct Evidence {
    direct: bool,
    relay: Option<iroh::endpoint::RelayDial>,
    update: Option<DialUpdate>,
}
impl Evidence {
    fn publish(&self) -> Option<iroh::endpoint::RelayDialObservation> {
        let reply = self
            .relay
            .as_ref()
            .and_then(|relay| relay.current_observation());
        if let Some(update) = &self.update {
            update(self.direct, reply);
        }
        reply
    }
}
impl DialState {
    pub(super) fn subscribe(&self, update: DialUpdate) {
        let mut evidence = self.evidence.lock().unwrap();
        assert!(
            evidence.update.is_none(),
            "one owner adopts a dial's evidence"
        );
        evidence.update = Some(update);
        evidence.publish();
    }
    pub(super) fn selected(&self, direct: bool) {
        let mut evidence = self.evidence.lock().unwrap();
        evidence.direct = direct;
        evidence.publish();
    }
    pub(super) fn observed(
        &self,
        relay: iroh::endpoint::RelayDial,
    ) -> Option<iroh::endpoint::RelayDialObservation> {
        let mut evidence = self.evidence.lock().unwrap();
        evidence.relay = Some(relay);
        evidence.publish()
    }
    pub(super) fn started(&self, deadline: tokio::time::Instant) {
        *self.deadline.lock().unwrap() = Some(deadline);
    }
}

struct Dial {
    state: Arc<DialState>,
    task: tokio::task::JoinHandle<Result<Arc<Connected>>>,
}
impl Drop for Dial {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub(crate) struct Startup {
    pub(super) verified_at: Option<tokio::time::Instant>,
    peer: Weak<Peer>,
    slots: Arc<Semaphore>,
    dials: Mutex<BTreeMap<String, Dial>>,
}
impl Default for Startup {
    fn default() -> Self {
        Self {
            verified_at: None,
            peer: Weak::new(),
            slots: Arc::new(Semaphore::new(8)),
            dials: Mutex::default(),
        }
    }
}
impl Startup {
    pub(crate) async fn start(peer: Arc<Peer>) -> Result<Self> {
        let membership = store::membership(&peer.store, None)
            .await?
            .ok_or_else(|| protocol("membership missing"))?;
        let slots = Arc::new(Semaphore::new(8));
        let mut dials = BTreeMap::new();
        for target in membership
            .members
            .iter()
            .filter(|id| *id != &peer.local && !membership.revoked.contains(*id))
            .take(8)
        {
            let target = target.clone();
            let owner = peer.clone();
            // Reserve before spawning: verified contacts must not consume every
            // permit while awaiting an unpolled speculative task's adoption.
            let slot = slots
                .clone()
                .try_acquire_owned()
                .expect("eight speculative slots");
            let remote = target.clone();
            let state = Arc::new(DialState::default());
            let started = state.clone();
            let task = tokio::spawn(async move {
                let _slot = slot;
                owner
                    .dial(
                        &remote,
                        super::super::scheduler::Routes {
                            direct: true,
                            relay: true,
                        },
                        &started,
                        Kind::Timeout,
                        false,
                    )
                    .await
            });
            dials.insert(target, Dial { task, state });
        }
        Ok(Self {
            verified_at: None,
            peer: Arc::downgrade(&peer),
            slots,
            dials: Mutex::new(dials),
        })
    }
    pub(crate) async fn verified(
        &mut self,
        peer: &Arc<Peer>,
        at: tokio::time::Instant,
    ) -> Result<()> {
        let membership = store::membership(&peer.store, None)
            .await?
            .ok_or_else(|| protocol("membership missing"))?;
        self.verified_at = Some(at);
        if !self.peer.ptr_eq(&Arc::downgrade(peer)) {
            self.dials.lock().unwrap().clear();
            self.peer = Weak::new();
            return Ok(());
        }
        self.dials.lock().unwrap().retain(|target, _| {
            membership.members.contains(target) && !membership.revoked.contains(target)
        });
        Ok(())
    }
    pub(super) async fn slot(&self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        self.slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Kind::Cancelled.error("contact scheduler stopped"))
    }
    pub(super) async fn adopt(
        &self,
        peer: &Arc<Peer>,
        target: &str,
        update: DialUpdate,
    ) -> Result<Option<Arc<Connected>>> {
        let dial = self.dials.lock().unwrap().remove(target);
        let Some(mut dial) = dial else {
            return Ok(None);
        };
        // Admission is forbidden before this verified adoption. The pending dial
        // retains its original timeout inside its task.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        let unusable = dial.task.is_finished()
            || dial
                .state
                .deadline
                .lock()
                .unwrap()
                .is_some_and(|end| end <= tokio::time::Instant::now());
        if self.verified_at.is_none() {
            return Err(protocol(
                "speculative adoption requires verified membership",
            ));
        }
        peer.authorized(target).await?;
        if !unusable {
            dial.state.subscribe(update.clone());
        }
        let result = (&mut dial.task)
            .await
            .map_err(|_| Kind::Cancelled.error("speculative contact stopped"))?;
        match result {
            Err(_) if unusable => Ok(None),
            Err(error) => Err(error),
            Ok(link) if link.connection.close_reason().is_some() => {
                if unusable {
                    Ok(None)
                } else {
                    Err(Kind::connection(link.connection.close_reason().unwrap()))
                }
            }
            Ok(mut link) => {
                if unusable {
                    dial.state.subscribe(update);
                }
                let link_mut =
                    Arc::get_mut(&mut link).expect("speculative transport has one owner");
                // A speculative task still queued on local capacity has not issued
                // contact yet. Queue time stays outside the admission allowance.
                link_mut.admission_deadline = deadline.max(link_mut.admission_deadline);
                link_mut.server = Some(peer.start_server(&link_mut.connection));
                Ok(Some(link))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    #[tokio::test]
    #[ignore = "requires native loopback UDP"]
    async fn native_unpolled_speculation_reserves_contact_capacity_before_verified_adoption() {
        let key = iroh::SecretKey::from_bytes(&[1; 32]);
        let membership = Membership {
            head: "0".repeat(64),
            service: "http://127.0.0.1:1".into(),
            receipt_key: "0".repeat(64),
            roots: BTreeSet::from(["1".repeat(64)]),
            members: (1..=10)
                .map(|index| {
                    iroh::SecretKey::from_bytes(&[index; 32])
                        .public()
                        .to_string()
                })
                .collect(),
            revoked: Default::default(),
            invitations: Default::default(),
        };
        let (store, mut requests) = store::channel();
        let state = membership.clone();
        let dispatcher = tokio::spawn(async move {
            while let Some(request) = requests.recv().await {
                assert!(matches!(request.step, Step::Membership(None)));
                let _ = request
                    .reply
                    .send(Ok(Value::Membership(Some(state.clone()))));
            }
        });
        let peer = Peer::bind(
            store,
            Identity {
                key,
                service: membership.service.clone(),
                receipt_key: Some(membership.receipt_key.clone()),
                head: Some(membership.head.clone()),
                enabled: true,
                pending_leave: false,
                conflict: false,
            },
            membership,
        )
        .await
        .unwrap();
        let startup = Startup::start(peer.clone()).await.unwrap();
        // This current-thread runtime has not polled any spawned dial yet.
        // Verified contacts cannot take the slots and then await those dials.
        assert_eq!(startup.dials.lock().unwrap().len(), 8);
        assert_eq!(startup.slots.available_permits(), 0);
        assert!(startup.slot().now_or_never().is_none());
        let slots = startup.slots.clone();
        drop(startup);
        tokio::task::yield_now().await;
        assert_eq!(slots.available_permits(), 8);
        peer.shutdown().await.unwrap();
        dispatcher.abort();
    }

    async fn relay_source() -> (
        Arc<Peer>,
        String,
        tokio::net::UdpSocket,
        tokio::task::JoinHandle<()>,
    ) {
        let service = std::env::var("LINN_RELAY_TEST_URL").expect("local Worker relay required");
        let keys = [iroh::SecretKey::generate(), iroh::SecretKey::generate()];
        super::super::relay_fixture::enroll(service.trim_end_matches('/'), &[&keys[0], &keys[1]])
            .await
            .unwrap();
        let target = keys[1].public().to_string();
        let state = Membership {
            head: "0".repeat(64),
            service,
            receipt_key: "0".repeat(64),
            roots: BTreeSet::from([keys[0].public().to_string()]),
            members: keys.iter().map(|key| key.public().to_string()).collect(),
            revoked: Default::default(),
            invitations: Default::default(),
        };
        let (store, mut requests) = store::channel();
        let current = state.clone();
        let dispatcher = tokio::spawn(async move {
            while let Some(request) = requests.recv().await {
                let result = match request.step {
                    Step::Membership(_) => Value::Membership(Some(current.clone())),
                    Step::Authorize(_) | Step::Finish(None) => Value::Done,
                    _ => panic!("an offline target cannot exchange records"),
                };
                let _ = request.reply.send(Ok(result));
            }
        });
        let peer = Peer::bind(
            store,
            Identity {
                key: keys[0].clone(),
                service: state.service.clone(),
                receipt_key: Some(state.receipt_key.clone()),
                head: Some(state.head.clone()),
                enabled: true,
                pending_leave: false,
                conflict: false,
            },
            state,
        )
        .await
        .unwrap();
        if let Some(discovery) = &peer.discovery {
            discovery.stop();
        }
        tokio::time::timeout(Duration::from_secs(10), peer.endpoint.online())
            .await
            .unwrap();
        let blackhole = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        peer.lookup.set_endpoint_info(
            EndpointAddr::new(keys[1].public()).with_ip_addr(blackhole.local_addr().unwrap()),
        );
        (peer, target, blackhole, dispatcher)
    }

    #[tokio::test]
    #[ignore = "requires a configured local Worker relay"]
    async fn native_speculative_relay_evidence_before_and_after_verified_adoption() {
        for before in [true, false] {
            let (peer, target, _blackhole, dispatcher) = relay_source().await;
            let held = if before {
                vec![]
            } else {
                let ticket = peer.connections.queue().await;
                (0..super::super::super::connections::capacity(2))
                    .map(|_| peer.connections.reserve(&ticket).unwrap())
                    .collect::<Vec<_>>()
            };
            let mut startup = Startup::start(peer.clone()).await.unwrap();
            let state = startup.dials.lock().unwrap()[&target].state.clone();
            if before {
                tokio::time::timeout(Duration::from_secs(2), async {
                    loop {
                        if state
                            .evidence
                            .lock()
                            .unwrap()
                            .relay
                            .as_ref()
                            .and_then(|relay| relay.current_observation())
                            .is_some()
                        {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert!(
                    state.evidence.lock().unwrap().update.is_none(),
                    "no job evidence before verification"
                );
                assert!(!startup.dials.lock().unwrap()[&target].task.is_finished());
            }
            startup
                .verified(&peer, tokio::time::Instant::now())
                .await
                .unwrap();
            let (report, ()) = tokio::join!(
                peer.clone()
                    .sync_started(super::super::super::membership::random_id(), startup),
                async {
                    if !before {
                        tokio::time::timeout(Duration::from_secs(2), async {
                            while state.evidence.lock().unwrap().update.is_none() {
                                tokio::task::yield_now().await;
                            }
                        })
                        .await
                        .unwrap();
                    }
                    drop(held);
                }
            );
            let report = report.unwrap();
            assert!(report.unavailable_only(), "before={before}: {report:?}");
            assert_eq!(
                report.collected.as_slice(),
                std::slice::from_ref(&peer.local)
            );
            assert_eq!(
                report.distributed.as_slice(),
                std::slice::from_ref(&peer.local)
            );
            assert!(
                report
                    .failures
                    .iter()
                    .all(|failure| failure.peer == target && failure.kind == Kind::Offline)
            );
            assert!(!report.traversal.as_ref().unwrap().discovery_expired);
            peer.shutdown().await.unwrap();
            dispatcher.abort();
        }
    }

    #[tokio::test]
    #[ignore = "requires a configured local Worker relay and speculative expiry"]
    async fn native_speculative_discard_does_not_forward_cached_relay_evidence() {
        let (peer, target, _blackhole, dispatcher) = relay_source().await;
        let mut startup = Startup::start(peer.clone()).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while !startup.dials.lock().unwrap()[&target].task.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let state = startup.dials.lock().unwrap()[&target].state.clone();
        assert!(
            state
                .evidence
                .lock()
                .unwrap()
                .relay
                .as_ref()
                .and_then(|relay| relay.current_observation())
                .is_some()
        );
        startup
            .verified(&peer, tokio::time::Instant::now())
            .await
            .unwrap();
        let forwarded = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = forwarded.clone();
        let adopted = startup
            .adopt(
                &peer,
                &target,
                Arc::new(move |_, reply| {
                    if reply.is_some() {
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                }),
            )
            .await
            .unwrap();
        assert!(adopted.is_none());
        assert_eq!(forwarded.load(Ordering::Relaxed), 0);
        assert!(state.evidence.lock().unwrap().update.is_none());
        let report = peer
            .clone()
            .sync_started(super::super::super::membership::random_id(), startup)
            .await
            .unwrap();
        assert!(
            report.unavailable_only(),
            "a fresh verified dial supplies its own evidence: {report:?}"
        );
        peer.shutdown().await.unwrap();
        dispatcher.abort();
    }
}
