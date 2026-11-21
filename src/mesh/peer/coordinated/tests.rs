use super::*;

#[tokio::test]
async fn cancelling_an_assignment_drops_its_operation_before_control_cleanup() {
    use futures_util::FutureExt;
    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let guard = Dropped(dropped.clone());
    let mut work = FuturesUnordered::new();
    work.push(AssignmentTask {
        envelope: Envelope {
            id: 7,
            route: vec![],
            command: Command::Stop,
        },
        task: Some(Box::pin(async move {
            let _guard = guard;
            std::future::pending::<Result<Reply>>().await
        })),
        wake: None,
    });
    assert!(work.next().now_or_never().is_none());
    work.iter_mut().next().unwrap().cancel();
    assert!(dropped.load(Ordering::Acquire));
    let (envelope, result) = work.next().await.unwrap();
    assert_eq!(envelope.id, 7);
    assert!(result.is_none());
    assert!(work.is_empty());
}

#[tokio::test]
#[ignore = "requires native loopback UDP and 61 seconds of idle retention"]
async fn native_idle_role_has_no_elapsed_expiry_and_entry_pressure_is_explicit() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let mut peers = Vec::new();
    let mut dispatchers = Vec::new();
    let mut membership_reads = Vec::new();
    for key in keys {
        let (store, mut requests) = store::channel();
        let state = state.clone();
        let membership = state.clone();
        let reads = Arc::new(AtomicUsize::new(0));
        membership_reads.push(reads.clone());
        dispatchers.push(tokio::spawn(async move {
            while let Some(request) = requests.recv().await {
                let reply = match request.step {
                    Step::Membership(_) => {
                        reads.fetch_add(1, Ordering::Relaxed);
                        Value::Membership(Some(state.clone()))
                    }
                    Step::Finish(None) => Value::Done,
                    Step::Authorize(_) => Value::Done,
                    _ => panic!("metadata continuation must not exchange records"),
                };
                let _ = request.reply.send(Ok(reply));
            }
        }));
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
        if let Some(discovery) = &peer.discovery {
            discovery.stop();
        }
        peers.push(peer);
    }
    let root = &peers[0];
    let target = &peers[1];
    let listening = target.clone();
    let listener = tokio::spawn(async move {
        listening.listen().await;
    });
    let address = target
        .endpoint
        .bound_sockets()
        .into_iter()
        .find(|address| address.is_ipv4())
        .unwrap();
    root.lookup.set_endpoint_info(
        EndpointAddr::new(target.endpoint.id())
            .with_ip_addr((std::net::Ipv4Addr::LOCALHOST, address.port()).into()),
    );
    let mut nodes = Vec::new();
    let mut guards = Vec::new();
    for i in 0..4 {
        let job = SignedRun::new(&root.key, &state, format!("{i:064x}")).unwrap();
        root.register(&job, None).unwrap();
        guards.push(Some(Running::new(root, &job.run.id, true)));
        let node = Arc::new(Node {
            owner: Arc::downgrade(root),
            startup: Startup::default(),
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
                crate::mesh::delta::Watch::new(&root.store.changes, job.run.id.clone()),
                false,
            )),
            touched: Mutex::new(tokio::time::Instant::now()),
        });
        root.runs
            .lock()
            .unwrap()
            .get_mut(&job.run.id)
            .unwrap()
            .coordinated = Some(node.clone());
        let reply = node
            .clone()
            .execute(
                Envelope {
                    id: 1,
                    route: vec![],
                    command: Command::Contact {
                        assignment: 1,
                        target: target.local.clone(),
                        routes: Routes {
                            direct: true,
                            relay: false,
                        },
                        replaces_parent: None,
                    },
                },
                None,
            )
            .await
            .unwrap();
        assert!(
            matches!(
                reply,
                Reply::Contact {
                    duplicate: false,
                    ..
                }
            ),
            "{reply:?}"
        );
        let premature = node
            .clone()
            .execute(
                Envelope {
                    id: 16,
                    route: vec![],
                    command: Command::Stop,
                },
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(
            Kind::of(&premature),
            Kind::Cancelled,
            "a parent cannot finalize before its child"
        );
        let edge = node.children.read().await[&target.local].clone();
        let current = edge
            .acquire(
                &node,
                &target.local,
                pressure::Purpose::Data {
                    round: 1,
                    upward: true,
                },
            )
            .await
            .unwrap();
        current
            .request(
                Envelope {
                    id: 17,
                    route: vec![],
                    command: Command::Expose {
                        round: 1,
                        readers: vec![root.local.clone()],
                    },
                },
                None,
            )
            .await
            .unwrap();
        drop(current);
        assert!(edge.release_idle());
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if target
                    .runs
                    .lock()
                    .unwrap()
                    .get(&job.run.id)
                    .and_then(|entry| entry.coordinated.as_ref())
                    .is_some_and(|node| node.idle_stamp().is_some())
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            panic!(
                "idle transition missing: registry={:?}",
                target
                    .runs
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(id, entry)| (
                        id.clone(),
                        entry.phase,
                        entry
                            .coordinated
                            .as_ref()
                            .map(|node| node.attached.load(Ordering::Acquire))
                    ))
                    .collect::<Vec<_>>()
            )
        });
        nodes.push(node);
    }
    assert!(
        target.status().jobs.is_empty(),
        "retained idle metadata is not a job carried into status"
    );
    let fifth = SignedRun::new(&root.key, &state, format!("{:064x}", 4)).unwrap();
    assert_eq!(
        Kind::of(&root.register(&fifth, None).unwrap_err()),
        Kind::Capacity,
        "executing roots are never evicted"
    );
    tokio::time::sleep(Duration::from_secs(61)).await;
    assert_eq!(
        target.runs.lock().unwrap().len(),
        4,
        "elapsed time is not an eviction event"
    );
    // Model the other connecting reservations without changing C(N).
    let edge = nodes[3].children.read().await[&target.local].clone();
    let owned = edge
        .acquire(
            &nodes[3],
            &target.local,
            pressure::Purpose::Data {
                round: 1,
                upward: true,
            },
        )
        .await
        .unwrap();
    let before = membership_reads[1].load(Ordering::Relaxed);
    let duplicate = owned
        .request(
            Envelope {
                id: 17,
                route: vec![],
                command: Command::Expose {
                    round: 1,
                    readers: vec![root.local.clone()],
                },
            },
            None,
        )
        .await
        .unwrap();
    assert!(matches!(duplicate, Reply::Done { changed: 0 }));
    assert_eq!(
        membership_reads[1].load(Ordering::Relaxed),
        before,
        "a cached duplicate never restarts authority/store work"
    );
    let reservations: Vec<_> = (0..31)
        .map(|_| root.connections.clone().try_acquire_owned().unwrap())
        .collect();
    let probe = root.check_transport(&target.local);
    tokio::pin!(probe);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut probe)
            .await
            .is_err(),
        "an owned connection cannot be evicted"
    );
    drop(owned);
    tokio::time::timeout(Duration::from_secs(3), &mut probe)
        .await
        .unwrap()
        .unwrap();
    drop(reservations);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if target.runs.lock().unwrap()[&nodes[3].job.run.id]
                .coordinated
                .as_ref()
                .unwrap()
                .idle_stamp()
                .is_some()
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    guards[0].take(); // Simulate a root that ended without delivering remote cleanup.
    root.register(&fifth, None).unwrap();
    let fifth_guard = Running::new(root, &fifth.run.id, true);
    let node = Arc::new(Node {
        owner: Arc::downgrade(root),
        startup: Startup::default(),
        job: fifth.clone(),
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
            crate::mesh::delta::Watch::new(&root.store.changes, fifth.run.id.clone()),
            false,
        )),
        touched: Mutex::new(tokio::time::Instant::now()),
    });
    let reply = node
        .clone()
        .execute(
            Envelope {
                id: 1,
                route: vec![],
                command: Command::Contact {
                    assignment: 1,
                    target: target.local.clone(),
                    routes: Routes {
                        direct: true,
                        relay: false,
                    },
                    replaces_parent: None,
                },
            },
            None,
        )
        .await
        .unwrap();
    assert!(
        matches!(
            reply,
            Reply::Contact {
                duplicate: false,
                ..
            }
        ),
        "{reply:?}"
    );
    assert_eq!(target.runs.lock().unwrap().len(), 4);
    assert!(
        !target
            .runs
            .lock()
            .unwrap()
            .contains_key(&nodes[0].job.run.id)
    );
    let expired = pressure::reopen(&nodes[0], &target.local, pressure::Purpose::Terminal).await;
    assert!(
        expired.is_err(),
        "a continued Begin cannot recreate evicted state"
    );
    assert!(
        !target
            .runs
            .lock()
            .unwrap()
            .contains_key(&nodes[0].job.run.id)
    );
    let other_edge = nodes[2].children.read().await[&target.local].clone();
    let other = other_edge
        .acquire(
            &nodes[2],
            &target.local,
            pressure::Purpose::Data {
                round: 1,
                upward: true,
            },
        )
        .await
        .unwrap();
    let conflict = other
        .request(
            Envelope {
                id: 17,
                route: vec![],
                command: Command::Expose {
                    round: 2,
                    readers: vec![root.local.clone()],
                },
            },
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(
        Kind::of(&conflict),
        Kind::Conflict,
        "changed command fields cannot reuse an old ID"
    );
    drop(other);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if target
                .runs
                .lock()
                .unwrap()
                .get(&nodes[2].job.run.id)
                .and_then(|entry| entry.coordinated.as_ref())
                .is_some_and(|node| {
                    node.orphan.load(Ordering::Acquire) && node.idle_stamp().is_some()
                })
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(
        pressure::reopen(&nodes[2], &target.local, pressure::Purpose::Expansion)
            .await
            .is_err(),
        "an ordinary continuation cannot revive a failed role"
    );
    nodes[2]
        .clone()
        .execute(
            Envelope {
                id: 18,
                route: vec![],
                command: Command::Detach {
                    child: target.local.clone(),
                },
            },
            None,
        )
        .await
        .unwrap();
    let recovered = nodes[2]
        .clone()
        .execute(
            Envelope {
                id: 19,
                route: vec![],
                command: Command::Contact {
                    assignment: 19,
                    target: target.local.clone(),
                    routes: Routes {
                        direct: true,
                        relay: false,
                    },
                    replaces_parent: Some(root.local.clone()),
                },
            },
            None,
        )
        .await
        .unwrap();
    assert!(
        matches!(
            recovered,
            Reply::Contact {
                duplicate: false,
                ..
            }
        ),
        "{recovered:?}"
    );
    let recovered = nodes[2].children.read().await[&target.local].clone();
    assert!(recovered.release_idle());
    let exhausted = recovered
        .acquire(
            &nodes[2],
            &target.local,
            pressure::Purpose::Data {
                round: 1,
                upward: true,
            },
        )
        .await;
    assert!(
        matches!(exhausted, Err(ref error) if Kind::of(error) == Kind::Capacity),
        "detaching/readmitting an edge cannot replenish a consumed reopen allowance"
    );
    let resumed = recovered
        .acquire(
            &nodes[2],
            &target.local,
            pressure::Purpose::Data {
                round: 2,
                upward: true,
            },
        )
        .await
        .unwrap();
    resumed
        .link
        .connection
        .close(0u32.into(), b"test failed parent transport");
    drop(resumed);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if target.runs.lock().unwrap()[&nodes[2].job.run.id]
                .coordinated
                .as_ref()
                .is_some_and(|node| {
                    node.orphan.load(Ordering::Acquire) && node.idle_stamp().is_some()
                })
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(recovered.release_idle());
    let cleanup = recovered
        .acquire(&nodes[2], &target.local, pressure::Purpose::Terminal)
        .await
        .unwrap();
    cleanup
        .request(
            Envelope {
                id: 25,
                route: vec![],
                command: Command::Stop,
            },
            None,
        )
        .await
        .unwrap();
    drop(cleanup);
    tokio::time::timeout(Duration::from_secs(2), async {
        while target
            .runs
            .lock()
            .unwrap()
            .contains_key(&nodes[2].job.run.id)
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let terminal = edge
        .acquire(&nodes[3], &target.local, pressure::Purpose::Terminal)
        .await
        .unwrap();
    let rejected = terminal
        .request(
            Envelope {
                id: 99,
                route: vec![],
                command: Command::Pull {
                    round: 2,
                    child: None,
                    delta: false,
                },
            },
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(Kind::of(&rejected), Kind::Protocol);
    drop(terminal);
    drop(fifth_guard);
    drop(guards);
    drop(node);
    drop(nodes);
    root.shutdown().await.unwrap();
    target.shutdown().await.unwrap();
    assert!(target.runs.lock().unwrap().is_empty());
    listener.abort();
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_five_peers_finish_without_recursive_begin() {
    run_scenario(Scenario::devices(5)).await;
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_gateway_carries_all_originals_to_earlier_peers() {
    run_scenario(Scenario {
        topology: Topology::Gateway,
        ..Scenario::devices(5)
    })
    .await;
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_final_parent_loss_recovers_reachable_children() {
    run_scenario(Scenario {
        topology: Topology::Gateway,
        loss: ParentLoss::Final,
        ..Scenario::devices(5)
    })
    .await;
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_collection_parent_loss_recovers_reachable_originals() {
    run_scenario(Scenario {
        topology: Topology::Gateway,
        loss: ParentLoss::Collection,
        ..Scenario::devices(5)
    })
    .await;
}
#[derive(Clone, Copy)]
enum Topology {
    Clique,
    Star,
    Gateway,
}
#[derive(Clone, Copy)]
enum ParentLoss {
    None,
    Collection,
    Initial,
    Final,
}
#[derive(Clone, Copy)]
enum PathState {
    Cold,
    Warm,
}
enum Network {
    Loopback,
    Relay(String),
}
impl Network {
    async fn traffic(&self, reset: bool) -> Option<serde_json::Value> {
        let Self::Relay(url) = self else {
            return None;
        };
        let url = format!(
            "{}/_inspect{}",
            url.trim_end_matches('/'),
            if reset { "?resetTraffic=1" } else { "" }
        );
        let value: serde_json::Value = reqwest::get(url)
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        Some(value["relayTraffic"].clone())
    }
}
#[derive(Default)]
enum StartupCase {
    #[default]
    None,
    Hold(Duration),
    Offline(Duration),
    Cancel(Duration),
    Closed(Duration),
}
struct RegistrationProbe {
    final_round: bool,
    root: Mutex<Weak<Peer>>,
    target: Mutex<Weak<Peer>>,
    replacement: Mutex<Option<(Arc<Peer>, tokio::task::JoinHandle<()>)>>,
    calls: AtomicUsize,
    fired: AtomicBool,
}
impl RegistrationProbe {
    fn new(final_round: bool) -> Arc<Self> {
        Arc::new(Self {
            final_round,
            root: Mutex::default(),
            target: Mutex::default(),
            replacement: Mutex::default(),
            calls: AtomicUsize::new(0),
            fired: AtomicBool::new(false),
        })
    }
    async fn reconnect(&self) {
        let root = self.root.lock().unwrap().upgrade().unwrap();
        let target = self.target.lock().unwrap().upgrade().unwrap();
        let (run, progress) = {
            let runs = root.runs.lock().unwrap();
            let entry = runs.values().next().unwrap();
            (entry.job.run.id.clone(), entry.progress.clone())
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let revision = progress.revision();
                if root
                    .status()
                    .jobs
                    .iter()
                    .find(|job| job.id == run)
                    .and_then(|job| job.report.traversal.as_deref())
                    .is_some_and(|state| state.pending.is_empty() && state.reached == 4)
                {
                    break;
                }
                progress.changed_since(revision).await;
            }
        })
        .await
        .expect("ordinary discovery must become quiescent while the transfer remains blocked");
        let state = store::membership(&target.store, None)
            .await
            .unwrap()
            .unwrap();
        let lookup = MemoryLookup::new();
        let endpoint = Endpoint::builder(presets::Minimal)
            .secret_key(target.key.clone())
            .clear_ip_transports()
            .clear_address_lookup()
            .address_lookup(lookup.clone())
            .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
                target.relay.clone(),
                None,
            )])))
            .alpns(vec![wire::ALPN.to_vec()])
            .net_report_config(NetReportConfig::minimal())
            .bind()
            .await
            .unwrap();
        endpoint.online().await;
        let replacement = Peer::with_endpoint(
            target.store.clone(),
            Identity {
                key: target.key.clone(),
                service: state.service.clone(),
                receipt_key: Some(state.receipt_key.clone()),
                head: Some(state.head.clone()),
                enabled: true,
                pending_leave: false,
                conflict: false,
            },
            state,
            endpoint,
            lookup,
            target.relay.clone(),
        );
        let serving = replacement.clone();
        let listener = tokio::spawn(async move {
            serving.listen().await;
        });
        *self.replacement.lock().unwrap() = Some((replacement, listener));
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let revision = progress.revision();
                if root
                    .status()
                    .jobs
                    .iter()
                    .find(|job| job.id == run)
                    .and_then(|job| job.report.traversal.as_deref())
                    .is_some_and(|state| state.reached == 5)
                {
                    break;
                }
                progress.changed_since(revision).await;
            }
        })
        .await
        .expect("registration must trigger actual admission while reconciliation is blocked");
        self.fired.store(true, Ordering::Release);
    }
}
struct Scenario {
    keys: Vec<iroh::SecretKey>,
    jobs: usize,
    records: bool,
    common: usize,
    topology: Topology,
    loss: ParentLoss,
    paths: PathState,
    overlap: Option<Arc<OverlapTrace>>,
    delta_fault: Option<Arc<DeltaFault>>,
    bulk: Option<Bulk>,
    pipeline: Option<Arc<PipelineProbe>>,
    network: Network,
    startup: StartupCase,
    registration: Option<Arc<RegistrationProbe>>,
}
impl Default for Scenario {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            jobs: 1,
            records: false,
            common: 0,
            topology: Topology::Clique,
            loss: ParentLoss::None,
            paths: PathState::Cold,
            overlap: None,
            delta_fault: None,
            bulk: None,
            pipeline: None,
            network: Network::Loopback,
            startup: StartupCase::None,
            registration: None,
        }
    }
}
impl Scenario {
    fn devices(count: usize) -> Self {
        Self {
            keys: (0..count).map(|_| iroh::SecretKey::generate()).collect(),
            records: true,
            ..Self::default()
        }
    }
}
#[derive(Clone, Copy)]
struct Bulk {
    count: usize,
    bytes: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PipelineProbeKind {
    Prefetch,
    SparseFlush,
    CommitOverlap,
    SlowPublication,
    SourceShift,
}
struct PipelineProbe {
    kind: PipelineProbeKind,
    used: AtomicBool,
    gets: AtomicUsize,
    first_group: AtomicUsize,
    progress_at_block: std::sync::atomic::AtomicU64,
    receiver: Mutex<Weak<Peer>>,
    blocked: tokio::sync::watch::Sender<bool>,
    entered: tokio::sync::Notify,
    activity: tokio::sync::Notify,
    release: tokio::sync::Notify,
    sparse_committed: AtomicBool,
    prefetched: AtomicBool,
}
impl PipelineProbe {
    fn new(kind: PipelineProbeKind) -> Arc<Self> {
        Arc::new(Self {
            kind,
            used: AtomicBool::new(false),
            gets: AtomicUsize::new(0),
            first_group: AtomicUsize::new(0),
            progress_at_block: std::sync::atomic::AtomicU64::new(0),
            receiver: Mutex::default(),
            blocked: tokio::sync::watch::channel(false).0,
            entered: Default::default(),
            activity: Default::default(),
            release: Default::default(),
            sparse_committed: AtomicBool::new(false),
            prefetched: AtomicBool::new(false),
        })
    }
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_pipeline_prefetches_the_next_page_while_current_needs_is_blocked() {
    let probe = PipelineProbe::new(PipelineProbeKind::Prefetch);
    run_scenario(Scenario {
        bulk: Some(Bulk {
            count: 320,
            bytes: 512,
        }),
        pipeline: Some(probe.clone()),
        ..Scenario::devices(5)
    })
    .await;
    assert!(probe.used.load(Ordering::Acquire));
    assert!(probe.prefetched.load(Ordering::Acquire));
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_pipeline_sparse_group_flushes_before_the_next_response_arrives() {
    let probe = PipelineProbe::new(PipelineProbeKind::SparseFlush);
    run_scenario(Scenario {
        bulk: Some(Bulk {
            count: 40,
            bytes: 512,
        }),
        pipeline: Some(probe.clone()),
        ..Scenario::devices(5)
    })
    .await;
    assert!(probe.used.load(Ordering::Acquire));
    assert_eq!(probe.first_group.load(Ordering::Acquire), 1);
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_pipeline_credits_and_reader_continue_while_a_group_commits() {
    let probe = PipelineProbe::new(PipelineProbeKind::CommitOverlap);
    let fixture = tokio::spawn(run_scenario(Scenario {
        bulk: Some(Bulk {
            count: 96,
            bytes: 512,
        }),
        pipeline: Some(probe.clone()),
        ..Scenario::devices(5)
    }));
    tokio::time::timeout(Duration::from_secs(10), probe.entered.notified())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while probe.gets.load(Ordering::Acquire) <= 32 {
            probe.activity.notified().await;
        }
    })
    .await
    .expect("a committing group must not stop fetch credits");
    let receiver = probe.receiver.lock().unwrap().upgrade().unwrap();
    let progress = receiver
        .runs
        .lock()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .progress
        .clone();
    tokio::time::timeout(
        Duration::from_secs(5),
        progress.changed_since(probe.progress_at_block.load(Ordering::Acquire)),
    )
    .await
    .expect("credited responses must drain while the store is blocked");
    assert!(
        probe.gets.load(Ordering::Acquire) <= 64,
        "at most two count-bounded groups may be fetched"
    );
    probe.release.notify_one();
    fixture.await.unwrap();
}
#[tokio::test]
#[ignore = "native publication pause exceeds the ordinary 30-second frame timeout"]
async fn native_pipeline_source_survives_a_31_second_publication_pause() {
    let probe = PipelineProbe::new(PipelineProbeKind::SlowPublication);
    run_scenario(Scenario {
        bulk: Some(Bulk {
            count: 512,
            bytes: 512,
        }),
        pipeline: Some(probe.clone()),
        ..Scenario::devices(5)
    })
    .await;
    assert!(probe.used.load(Ordering::Acquire));
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_pipeline_snapshot_keeps_preexisting_items_when_an_earlier_id_is_inserted() {
    let probe = PipelineProbe::new(PipelineProbeKind::SourceShift);
    run_scenario(Scenario {
        pipeline: Some(probe.clone()),
        ..Scenario::devices(5)
    })
    .await;
    assert!(probe.used.load(Ordering::Acquire));
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DeltaFaultKind {
    Untracked,
    CountOverflow,
    ByteOverflow,
    Mismatch,
    Missing,
    ReceiverMutation,
    Truncated,
    Failure(Kind),
}
struct DeltaFault {
    kind: DeltaFaultKind,
    root: Mutex<Weak<Peer>>,
    armed: AtomicBool,
    injected: AtomicBool,
    chunks: AtomicUsize,
    prefix_published: AtomicBool,
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_delta_fallbacks_preserve_full_bytes_and_valid_prefixes() {
    for kind in [
        DeltaFaultKind::Untracked,
        DeltaFaultKind::CountOverflow,
        DeltaFaultKind::ByteOverflow,
        DeltaFaultKind::Mismatch,
        DeltaFaultKind::Missing,
        DeltaFaultKind::ReceiverMutation,
        DeltaFaultKind::Truncated,
        DeltaFaultKind::Failure(Kind::Conflict),
        DeltaFaultKind::Failure(Kind::Protocol),
        DeltaFaultKind::Failure(Kind::Publication),
    ] {
        let fault = Arc::new(DeltaFault {
            kind,
            root: Mutex::default(),
            armed: AtomicBool::new(false),
            injected: AtomicBool::new(false),
            chunks: AtomicUsize::new(0),
            prefix_published: AtomicBool::new(false),
        });
        run_scenario(Scenario {
            topology: Topology::Gateway,
            loss: ParentLoss::None,
            paths: PathState::Cold,

            keys: (0..5).map(|_| iroh::SecretKey::generate()).collect(),
            jobs: 1,
            records: true,
            overlap: None,
            delta_fault: Some(fault.clone()),
            common: 0,
            ..Scenario::default()
        })
        .await;
        assert!(fault.injected.load(Ordering::Acquire), "{kind:?}");
        if kind == DeltaFaultKind::Truncated || matches!(kind, DeltaFaultKind::Failure(_)) {
            assert!(
                fault.prefix_published.load(Ordering::Acquire),
                "valid bytes publish before failed-parent recovery"
            );
        }
    }
}
#[derive(Default)]
struct OverlapTrace {
    active: AtomicBool,
    used: AtomicBool,
    observed: AtomicBool,
    ready: tokio::sync::Notify,
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_initial_distribution_parent_loss_recovers_children() {
    run_scenario(Scenario {
        topology: Topology::Gateway,
        loss: ParentLoss::Initial,
        paths: PathState::Cold,

        keys: (0..5).map(|_| iroh::SecretKey::generate()).collect(),
        jobs: 1,
        records: true,
        overlap: None,
        delta_fault: None,
        common: 0,
        ..Scenario::default()
    })
    .await;
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_coordinated_island_collection_overlaps_slow_l1() {
    let trace = Arc::new(OverlapTrace::default());
    run_scenario(Scenario {
        topology: Topology::Gateway,
        loss: ParentLoss::None,
        paths: PathState::Cold,

        keys: (0..5).map(|_| iroh::SecretKey::generate()).collect(),
        jobs: 1,
        records: true,
        overlap: Some(trace.clone()),
        delta_fault: None,
        common: 0,
        ..Scenario::default()
    })
    .await;
    assert!(trace.observed.load(Ordering::Acquire));
}
#[tokio::test]
#[ignore = "native 256-device/four-job transport coverage; run separately"]
async fn native_four_256_peer_jobs_obey_the_ceiling_and_cover_every_peer() {
    let result = run_scenario(Scenario {
        topology: Topology::Clique,
        loss: ParentLoss::None,
        paths: PathState::Cold,

        keys: (0..256).map(|_| iroh::SecretKey::generate()).collect(),
        jobs: 4,
        records: true,
        overlap: None,
        delta_fault: None,
        common: 0,
        ..Scenario::default()
    })
    .await;
    assert!(
        result["peak_connections"].as_u64().unwrap() <= 1028,
        "{result}"
    );
    if let Some(path) = std::env::var_os("LINN_SYNC_TEST_MEASUREMENTS") {
        std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_four_twenty_peer_jobs_keep_connections_beyond_the_old_floor() {
    let result = run_scenario(Scenario {
        topology: Topology::Star,
        loss: ParentLoss::None,
        paths: PathState::Cold,

        keys: (0..20).map(|_| iroh::SecretKey::generate()).collect(),
        jobs: 4,
        records: true,
        overlap: None,
        delta_fault: None,
        common: 0,
        ..Scenario::default()
    })
    .await;
    assert!(
        result["peak_connections"].as_u64().unwrap() > 32,
        "{result}"
    );
    assert!(
        result["peak_connections"].as_u64().unwrap() <= 84,
        "{result}"
    );
}

#[tokio::test]
#[ignore = "native diagnostic timing fixture; run without other native tests"]
async fn native_transfer_latency_matched_coverage() {
    let output = std::env::var_os("LINN_SYNC_TEST_MEASUREMENTS").expect("measurement output path");
    let mut measurements = Vec::new();
    for size in [5, 10, 20] {
        for paths in [PathState::Cold, PathState::Warm] {
            for bytes in [512, 16 * 1024] {
                for _ in 0..3 {
                    measurements.push(
                        run_scenario(Scenario {
                            keys: (0..size)
                                .map(|index| iroh::SecretKey::from_bytes(&[(index + 1) as u8; 32]))
                                .collect(),
                            paths,
                            bulk: Some(Bulk { count: 96, bytes }),
                            ..Scenario::default()
                        })
                        .await,
                    );
                }
            }
        }
    }
    std::fs::write(output, serde_json::to_vec_pretty(&measurements).unwrap()).unwrap();
}

use super::super::relay_fixture;

#[tokio::test]
#[ignore = "requires a local test relay; no IP transport is bound"]
async fn native_relay_transfer_latency_matched_coverage() {
    let relay = std::env::var("LINN_RELAY_TEST_URL").expect("local relay URL");
    let keys: Vec<_> = (0..5)
        .map(|index| iroh::SecretKey::from_bytes(&[(index + 1) as u8; 32]))
        .collect();
    relay_fixture::enroll(
        relay.trim_end_matches('/'),
        &keys.iter().collect::<Vec<_>>(),
    )
    .await
    .unwrap();
    let mut measurements = Vec::new();
    for paths in [PathState::Cold, PathState::Warm] {
        for _ in 0..3 {
            measurements.push(
                run_scenario(Scenario {
                    keys: keys.clone(),
                    paths,
                    network: Network::Relay(relay.clone()),
                    bulk: Some(Bulk {
                        count: 96,
                        bytes: 512,
                    }),
                    ..Scenario::default()
                })
                .await,
            );
        }
    }
    println!("MEASURE {}", serde_json::to_string(&measurements).unwrap());
}

#[tokio::test]
#[ignore = "native diagnostic timing fixture; run without other native tests"]
async fn native_delta_latency_matched_coverage() {
    let output = std::env::var_os("LINN_SYNC_TEST_MEASUREMENTS").expect("measurement output path");
    let mut measurements = Vec::new();
    for size in [5, 10, 20] {
        for warm in [false, true] {
            for records in [false, true] {
                for _ in 0..3 {
                    measurements.push(
                        run_scenario(Scenario {
                            topology: Topology::Gateway,
                            loss: ParentLoss::None,
                            paths: if warm {
                                PathState::Warm
                            } else {
                                PathState::Cold
                            },

                            keys: (0..size)
                                .map(|index| iroh::SecretKey::from_bytes(&[(index + 1) as u8; 32]))
                                .collect(),
                            jobs: 1,
                            records,
                            common: 1024,
                            overlap: None,
                            delta_fault: None,
                            ..Scenario::default()
                        })
                        .await,
                    );
                }
            }
        }
    }
    std::fs::write(output, serde_json::to_vec_pretty(&measurements).unwrap()).unwrap();
}
fn sized(item: &crate::storage::sync::Item) -> crate::storage::sync::InventoryEntry {
    crate::storage::sync::InventoryEntry {
        entry: crate::storage::sync::Entry {
            id: item.id.clone(),
            hash: item.envelope_hash.clone(),
        },
        frame_bytes: serde_json::to_vec(&Frame::Response(Response::Item {
            item: Some(item.clone()),
        }))
        .unwrap()
        .len() as u64
            + 4,
        envelope_bytes: item.envelope.as_ref().map_or(0, String::len) as u64,
    }
}

fn transport_sample(peer: &Peer) -> [u64; 3] {
    let metrics = &peer.endpoint.metrics().socket;
    [
        metrics.num_conns_opened.get(),
        metrics.send_ipv4.get() + metrics.send_ipv6.get() + metrics.send_relay.get(),
        metrics.recv_data_ipv4.get() + metrics.recv_data_ipv6.get() + metrics.recv_data_relay.get(),
    ]
}

async fn run_scenario(scenario: Scenario) -> serde_json::Value {
    let Scenario {
        mut keys,
        jobs,
        records,
        overlap,
        delta_fault,
        common,
        topology,
        loss,
        paths,
        bulk,
        pipeline,
        network,
        startup: startup_case,
        registration,
    } = scenario;
    let count = keys.len();
    let gateway = matches!(topology, Topology::Gateway);
    let fail_parent = !matches!(loss, ParentLoss::None);
    let fail_collection = matches!(loss, ParentLoss::Collection);
    let final_loss = matches!(loss, ParentLoss::Final);
    let warm = matches!(paths, PathState::Warm);
    let expected_fatal = delta_fault.as_ref().and_then(|fault| match fault.kind {
        DeltaFaultKind::Failure(kind) => Some(kind),
        _ => None,
    });
    keys.sort_by_key(|key| key.public().to_string());
    let failure_target = keys[1].public().to_string();
    let receiver_id = keys[2].public().to_string();
    let root_id = keys[0].public().to_string();
    let disconnect: Arc<Mutex<Option<Arc<Peer>>>> = Arc::default();
    let membership = Membership {
        head: "0".repeat(64),
        service: match &network {
            Network::Loopback => "http://127.0.0.1:1".into(),
            Network::Relay(url) => url.clone(),
        },
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let mut peers = Vec::new();
    let mut dispatchers = Vec::new();
    let mut listeners = Vec::new();
    let mut originals = Vec::new();
    for (index, key) in keys.into_iter().enumerate() {
        let item = crate::storage::sync::Item {
            id: format!("record-{index}"),
            envelope: Some(
                if index == 4
                    && delta_fault
                        .as_ref()
                        .is_some_and(|fault| fault.kind == DeltaFaultKind::Truncated)
                {
                    "large original\n".repeat(8192)
                } else {
                    format!("exact original bytes {index}\n")
                },
            ),
            envelope_hash: Some(format!("hash-{index}")),
            body_hash: Some(format!("body-{index}")),
        };
        let mut seeded = if records {
            BTreeMap::from([(item.id.clone(), item)])
        } else {
            BTreeMap::new()
        };
        for number in 0..common {
            let shared = crate::storage::sync::Item {
                id: format!("common-{number:04}"),
                envelope: Some(format!("exact shared original {number}\n")),
                envelope_hash: Some(format!("shared-hash-{number:04}")),
                body_hash: Some(format!("shared-body-{number:04}")),
            };
            seeded.insert(shared.id.clone(), shared);
        }
        if index == 0
            && let Some(bulk) = bulk
        {
            for number in 0..bulk.count {
                let item = crate::storage::sync::Item {
                    id: format!("bulk-{number:04}"),
                    envelope: Some("x".repeat(bulk.bytes)),
                    envelope_hash: Some(format!("bulk-hash-{number:04}")),
                    body_hash: Some("bulk-body".into()),
                };
                seeded.insert(item.id.clone(), item);
            }
        }
        let data = Arc::new(Mutex::new(seeded));
        originals.push(data.clone());
        let (store, mut requests) = store::channel();
        let state = membership.clone();
        let disconnect = disconnect.clone();
        let failure_target = failure_target.clone();
        let root_id = root_id.clone();
        let overlap = overlap.clone();
        let fault = delta_fault.clone();
        let probe = pipeline.clone();
        let registration = registration.clone();
        let changes = store.changes.clone();
        let buffers = store.buffers.clone();
        let receiver_id = receiver_id.clone();
        let mut ordinals: BTreeMap<_, _> = data
            .lock()
            .unwrap()
            .keys()
            .enumerate()
            .map(|(index, id)| (id.clone(), index as u64 + 1))
            .collect();
        let mut last_ordinal = ordinals.len() as u64;
        dispatchers.push(tokio::spawn(async move {
            let mut distributions = 0usize;
            let mut receiver_distributions = 0usize;
            while let Some(request) = requests.recv().await {
                if index == 0 && matches!(&request.step, Step::SourceSnapshot(remote) if remote == &receiver_id)
                    && let Some(reconnect) = &registration {
                    let call = reconnect.calls.fetch_add(1, Ordering::AcqRel) + 1;
                    if reconnect.final_round && call == 1 { changes.record(None, None); }
                    if call == if reconnect.final_round { 2 } else { 1 } {
                        let reconnect = reconnect.clone();
                        let cutoff = last_ordinal;
                        tokio::spawn(async move {
                            reconnect.reconnect().await;
                            let _ = request.reply.send(Ok(Value::SourceSnapshot(cutoff)));
                        });
                        continue;
                    }
                }
                let value = match request.step {
                    Step::Membership(_) => Value::Membership(Some(state.clone())),
                    Step::Finish(None) => {
                        if index == 2 && data.lock().unwrap().len() >= crate::storage::sync::PAGE
                            && let Some(probe) = &probe && probe.kind == PipelineProbeKind::SlowPublication
                            && !probe.used.swap(true, Ordering::AcqRel) {
                            tokio::time::sleep(Duration::from_secs(31)).await;
                        }
                        if index == 2 && data.lock().unwrap().contains_key("record-3")
                            && let Some(fault) = &fault { fault.prefix_published.store(true, Ordering::Release); }
                        Value::Done
                    },
                    Step::SourceSnapshot(remote) => {
                        let captured_cutoff = last_ordinal;
                        if index == 1 && remote == root_id && let Some(probe) = &probe
                            && probe.kind == PipelineProbeKind::SourceShift && !probe.used.swap(true, Ordering::AcqRel) {
                            data.lock().unwrap().insert("000-concurrent".into(), crate::storage::sync::Item {
                                id: "000-concurrent".into(), envelope: Some("later local original".into()), envelope_hash: Some("later-hash".into()), body_hash: Some("later-body".into()),
                            });
                            last_ordinal += 1;
                            ordinals.insert("000-concurrent".into(), last_ordinal);
                        }
                        if index == 0 && remote == receiver_id {
                            receiver_distributions += 1;
                            if let Some(fault) = &fault {
                                if receiver_distributions == 2 {
                                    fault.armed.store(true, Ordering::Release);
                                    if fault.kind == DeltaFaultKind::Untracked {
                                        changes.record(None, None); fault.injected.store(true, Ordering::Release);
                                    } else if matches!(fault.kind, DeltaFaultKind::CountOverflow | DeltaFaultKind::ByteOverflow) {
                                        let root = fault.root.lock().unwrap().upgrade().unwrap();
                                        let node = root.runs.lock().unwrap().values().find_map(|entry| entry.coordinated.clone()).unwrap();
                                        for number in 0..crate::mesh::delta::ITEMS + 1 {
                                            changes.record(Some(node.job.run.id.clone()), Some(crate::storage::sync::Entry {
                                                id: if fault.kind == DeltaFaultKind::ByteOverflow { format!("{number:04}{}", "x".repeat(1024)) } else { format!("overflow-{number:04}") },
                                                hash: Some("hash".into()),
                                            }));
                                            node.refresh_delta();
                                        }
                                        fault.injected.store(true, Ordering::Release);
                                    }
                                }
                                if receiver_distributions == 3 && matches!(fault.kind, DeltaFaultKind::Missing | DeltaFaultKind::Mismatch | DeltaFaultKind::Truncated) {
                                    assert!(fault.prefix_published.load(Ordering::Acquire), "the complete valid prefix publishes before full fallback starts");
                                }
                            }
                        }
                        if index == 0 && remote == failure_target {
                            distributions += 1;
                        }
                        let failed = if fail_parent
                            && ((!fail_collection
                                && index == 0
                                && remote == failure_target
                                && distributions == if final_loss { 2 } else { 1 })
                                || (fail_collection && index == 1 && remote == root_id))
                        {
                            disconnect.lock().unwrap().take()
                        } else {
                            None
                        };
                        if let Some(failed) = failed {
                            failed.shutdown().await.unwrap();
                        }
                        Value::SourceSnapshot(captured_cutoff)
                    }
                    Step::Authorize(_) => Value::Done,
                    Step::Needs { entries, .. } => {
                        if index == 2 && let Some(probe) = &probe && probe.kind == PipelineProbeKind::Prefetch
                            && entries.len() == crate::storage::sync::PAGE && !probe.used.swap(true, Ordering::AcqRel)
                            && !probe.prefetched.load(Ordering::Acquire) {
                            tokio::time::timeout(Duration::from_secs(5), probe.release.notified()).await.expect("next page must prefetch while current Needs is blocked");
                        }
                        let conflict = if index == 2 && entries.iter().any(|entry| entry.id == "record-4")
                            && let Some(fault) = &fault && fault.kind == DeltaFaultKind::Failure(Kind::Conflict)
                            && fault.armed.load(Ordering::Acquire) && !fault.injected.swap(true, Ordering::AcqRel) {
                            entries.iter().position(|entry| entry.id == "record-4")
                        } else { None };
                        if index == 2 && let Some(fault) = &fault
                            && fault.kind == DeltaFaultKind::ReceiverMutation
                            && fault.armed.load(Ordering::Acquire)
                            && !fault.injected.swap(true, Ordering::AcqRel) { changes.record(None, None); }
                        let data = data.lock().unwrap();
                        Value::Needs { prefix: crate::storage::sync::Prefix {
                                decisions: entries.iter().take(conflict.unwrap_or(entries.len())).map(|entry| !data.contains_key(&entry.id)).collect(),
                                error: conflict.map(|_| Kind::Conflict.error("injected immutable conflict")),
                            }, epoch: buffers.epoch() }
                    }
                    Step::Inventory { peer: requesting, tombstones, after, cutoff } => {
                        if index == 0 && requesting == receiver_id && !tombstones && after.is_some()
                            && let Some(probe) = &probe && probe.kind == PipelineProbeKind::Prefetch {
                            probe.prefetched.store(true, Ordering::Release);
                            probe.release.notify_one();
                        }
                        let mut entries: Vec<_> = data.lock().unwrap().values()
                            .filter(|item| item.envelope.is_none() == tombstones && after.as_ref().is_none_or(|after| item.id > *after) && ordinals[&item.id] <= cutoff)
                            .take(crate::storage::sync::PAGE + 1).map(sized).collect();
                        let done = entries.len() <= crate::storage::sync::PAGE;
                        entries.truncate(crate::storage::sync::PAGE);
                        Value::Inventory(crate::storage::sync::InventoryPage { entries, done })
                    }
                    Step::Sizes { entries, .. } => Value::Inventory(crate::storage::sync::InventoryPage {
                        entries: entries.iter().map(|entry| sized(&data.lock().unwrap()[&entry.id])).collect(), done: true,
                    }),
                    Step::Get { peer: requesting, id, .. } => {
                        if index == 0 && requesting == receiver_id && id.starts_with("bulk-")
                            && let Some(probe) = &probe {
                            let gets = probe.gets.fetch_add(1, Ordering::AcqRel) + 1;
                            probe.activity.notify_one();
                            if probe.kind == PipelineProbeKind::CommitOverlap && gets == 33 {
                                let mut blocked = probe.blocked.subscribe();
                                tokio::time::timeout(Duration::from_secs(5), blocked.wait_for(|blocked| *blocked)).await.unwrap().unwrap();
                            }
                            if probe.kind == PipelineProbeKind::SparseFlush && id == "bulk-0001" {
                                probe.used.store(true, Ordering::Release);
                                if !probe.sparse_committed.load(Ordering::Acquire) {
                                    tokio::time::timeout(Duration::from_secs(5), probe.release.notified()).await.expect("sparse group did not flush before its next response");
                                }
                            }
                        }
                        if index == 0 && requesting == receiver_id && id == "record-4"
                            && let Some(fault) = &fault
                            && fault.armed.load(Ordering::Acquire)
                            && matches!(fault.kind, DeltaFaultKind::Missing | DeltaFaultKind::Mismatch)
                            && !fault.injected.swap(true, Ordering::AcqRel) {
                            let mut item = data.lock().unwrap().get(&id).cloned();
                            if fault.kind == DeltaFaultKind::Missing { item = None; }
                            else { item.as_mut().unwrap().envelope_hash = Some("bad-delta-hash".into()); }
                            let _ = request.reply.send(Ok(Value::Item(item)));
                            continue;
                        }
                        if let Some(trace) = &overlap
                            && index == 2
                            && id == "record-2"
                            && !trace.used.swap(true, Ordering::AcqRel)
                        {
                            trace.active.store(true, Ordering::Release);
                            tokio::time::timeout(
                                Duration::from_secs(5),
                                trace.ready.notified(),
                            )
                            .await
                            .expect("island collection must overlap the slow L1 source");
                            trace.active.store(false, Ordering::Release);
                        }
                        Value::Item(data.lock().unwrap().get(&id).cloned())
                    }
                    Step::ApplyGroup { items, .. } => {
                        if index == 2 && let Some(probe) = &probe {
                            let _ = probe.first_group.compare_exchange(0, items.len(), Ordering::AcqRel, Ordering::Acquire);
                            if probe.kind == PipelineProbeKind::CommitOverlap && !probe.used.swap(true, Ordering::AcqRel) {
                                let receiver = probe.receiver.lock().unwrap().upgrade().unwrap();
                                let revision = receiver.runs.lock().unwrap().values().next().unwrap().progress.revision();
                                probe.progress_at_block.store(revision, Ordering::Release);
                                probe.blocked.send_replace(true);
                                probe.entered.notify_one();
                                tokio::time::timeout(Duration::from_secs(5), probe.release.notified()).await.expect("overlap acceptance did not release commit");
                            }
                        }
                        let mut decisions = Vec::new();
                        let mut error = None;
                        for body in items {
                            match body.take() {
                                Ok(item) => {
                                    if let Some(trace) = &overlap && index == 1 && matches!(item.id.as_str(), "record-3" | "record-4") && trace.active.load(Ordering::Acquire) {
                                        trace.observed.store(true, Ordering::Release); trace.ready.notify_one();
                                    }
                                    let id = item.id.clone();
                                    let changed = data.lock().unwrap().insert(id.clone(), item).is_none();
                                    if changed { last_ordinal += 1; ordinals.insert(id, last_ordinal); }
                                    decisions.push(changed);
                                }
                                Err(failure) => { error = Some(failure); break; }
                            }
                        }
                        if index == 2 && let Some(probe) = &probe && probe.kind == PipelineProbeKind::SparseFlush {
                            probe.sparse_committed.store(true, Ordering::Release);
                            probe.release.notify_one();
                        }
                        Value::Decisions(crate::storage::sync::Prefix { decisions, error })
                    }
                    Step::BodyChunk { peer, id, .. } => {
                        if index == 0 && peer == receiver_id && id == "record-4"
                            && let Some(fault) = &fault
                            && let DeltaFaultKind::Failure(kind @ (Kind::Protocol | Kind::Publication)) = fault.kind
                            && fault.armed.load(Ordering::Acquire) && !fault.injected.swap(true, Ordering::AcqRel) {
                            let _ = request.reply.send(Err(kind.error("injected typed source failure")));
                            continue;
                        }
                        if index == 0 && peer == receiver_id && id == "record-4"
                            && let Some(fault) = &fault
                            && fault.kind == DeltaFaultKind::Truncated && fault.armed.load(Ordering::Acquire)
                            && !fault.injected.load(Ordering::Acquire)
                            && fault.chunks.fetch_add(1, Ordering::AcqRel) == 1 {
                            fault.injected.store(true, Ordering::Release);
                            let _ = request.reply.send(Err(Kind::Unresolved.error("injected incomplete delta item")));
                            continue;
                        }
                        Value::Done
                    },
                    Step::Finish(Some(_)) => Value::Done,
                    _ => panic!("unexpected store operation"),
                };
                let _ = request.reply.send(Ok(value));
            }
        }));
        let identity = Identity {
            key,
            service: membership.service.clone(),
            receipt_key: Some(membership.receipt_key.clone()),
            head: Some(membership.head.clone()),
            enabled: true,
            pending_leave: false,
            conflict: false,
        };
        let peer = match &network {
            Network::Loopback => Peer::bind(store, identity, membership.clone())
                .await
                .unwrap(),
            Network::Relay(_) => {
                let relay: RelayUrl = membership.service.parse().unwrap();
                let lookup = MemoryLookup::new();
                let endpoint = Endpoint::builder(presets::Minimal)
                    .secret_key(identity.key.clone())
                    .clear_ip_transports()
                    .clear_address_lookup()
                    .address_lookup(lookup.clone())
                    .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
                        relay.clone(),
                        None,
                    )])))
                    .alpns(vec![wire::ALPN.to_vec()])
                    .net_report_config(NetReportConfig::minimal())
                    .bind()
                    .await
                    .unwrap();
                tokio::time::timeout(Duration::from_secs(10), endpoint.online())
                    .await
                    .expect("fixture relay registration");
                assert!(
                    endpoint.bound_sockets().is_empty(),
                    "delayed relay fixture cannot bypass the relay over IP"
                );
                Peer::with_endpoint(store, identity, membership.clone(), endpoint, lookup, relay)
            }
        };
        if index == 2
            && let Some(probe) = &pipeline
        {
            *probe.receiver.lock().unwrap() = Arc::downgrade(&peer);
        }
        if index == 0
            && let Some(fault) = &delta_fault
        {
            *fault.root.lock().unwrap() = Arc::downgrade(&peer);
        }
        if let Some(discovery) = &peer.discovery {
            discovery.stop();
        }
        let listening = peer.clone();
        listeners.push(tokio::spawn(async move { listening.listen().await }));
        peers.push(peer);
    }
    if fail_parent {
        *disconnect.lock().unwrap() = Some(peers[1].clone());
    }
    for (source_index, source) in peers.iter().enumerate() {
        if matches!(network, Network::Relay(_)) {
            break;
        }
        for (target_index, target) in peers.iter().enumerate() {
            // Delta faults require an L1 receiver that cannot independently
            // collect the late island originals before final redistribution.
            if source.local == target.local
                || (matches!(topology, Topology::Star) && source_index != 0 && target_index != 0)
                || (gateway && source_index == 0 && target_index > 2)
                || (delta_fault.is_some() && source_index == 2 && target_index > 2)
            {
                continue;
            }
            let address = target
                .endpoint
                .bound_sockets()
                .into_iter()
                .find(|address| address.is_ipv4())
                .unwrap();
            source.lookup.set_endpoint_info(
                EndpointAddr::new(target.endpoint.id())
                    .with_ip_addr((std::net::Ipv4Addr::LOCALHOST, address.port()).into()),
            );
        }
    }
    if let Some(reconnect) = &registration {
        let target = peers.last().unwrap();
        *reconnect.root.lock().unwrap() = Arc::downgrade(&peers[0]);
        *reconnect.target.lock().unwrap() = Arc::downgrade(target);
        target.endpoint.close().await;
        let Network::Relay(url) = &network else {
            panic!("registration fixture requires a relay")
        };
        let client = reqwest::Client::new();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let status: serde_json::Value = client
                    .get(format!("{}/_inspect", url.trim_end_matches('/')))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                if status["sockets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|socket| socket["endpoint"].as_str() != Some(&target.local))
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("target must be actually unregistered before discovery starts");
    }
    if warm {
        let saved: Vec<_> = originals
            .iter()
            .map(|data| std::mem::take(&mut *data.lock().unwrap()))
            .collect();
        let primed = tokio::time::timeout(
            Duration::from_secs(60),
            peers[0].clone().sync_started(
                super::super::super::membership::random_id(),
                Startup::default(),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(primed.complete, "{primed:?}");
        for (data, saved) in originals.iter().zip(saved) {
            *data.lock().unwrap() = saved;
        }
        assert!(
            peers
                .iter()
                .all(|peer| peer.runs.lock().unwrap().is_empty())
        );
    }
    wire::metrics::clear();
    for peer in &peers {
        for counter in peer.replica_rounds.iter().chain(&peer.delta_transfers) {
            counter.store(0, Ordering::Relaxed);
        }
    }
    for peer in &peers {
        peer.connections.reset_peak();
    }
    let mut startup = Startup::default();
    if !matches!(startup_case, StartupCase::None) {
        let hidden: Vec<_> = if matches!(startup_case, StartupCase::Offline(_)) {
            peers
                .iter()
                .skip(1)
                .filter_map(|target| peers[0].lookup.remove_endpoint_info(target.endpoint.id()))
                .collect()
        } else {
            Vec::new()
        };
        startup = Startup::start(peers[0].clone()).await.unwrap();
        let delay = match startup_case {
            StartupCase::Hold(delay)
            | StartupCase::Offline(delay)
            | StartupCase::Cancel(delay)
            | StartupCase::Closed(delay) => delay,
            StartupCase::None => unreachable!(),
        };
        tokio::time::sleep(delay).await;
        assert!(
            peers
                .iter()
                .all(|peer| peer.runs.lock().unwrap().is_empty()),
            "transport speculation must not register any job before verification"
        );
        for info in hidden {
            peers[0].lookup.set_endpoint_info(info);
        }
        if matches!(startup_case, StartupCase::Cancel(_)) {
            drop(startup);
            startup = Startup::default();
            // Aborted pending tasks must be polled before checking raw connections.
            tokio::task::yield_now().await;
        }
        startup
            .verified(&peers[0], tokio::time::Instant::now())
            .await
            .unwrap();
    }
    network.traffic(true).await;
    let transport_before: Vec<_> = peers.iter().map(|peer| transport_sample(peer)).collect();
    let started = std::time::Instant::now();
    let mut running = FuturesUnordered::new();
    for _ in 0..jobs {
        let peer = peers[0].clone();
        let startup = std::mem::take(&mut startup);
        running.push(async move {
            let mut coordinator =
                Coordinator::prepare(peer, super::super::super::membership::random_id(), startup)
                    .await?;
            if count == 256 {
                // Full-coverage data-plane stress is separate from production-cap partials.
                coordinator.scheduler.deadline = Duration::from_secs(300);
            }
            coordinator.run().await
        });
    }
    let reports = tokio::time::timeout(
        Duration::from_secs(if count == 256 { 300 } else { 60 }),
        async {
            let mut reports = Vec::new();
            while let Some(report) = running.next().await {
                reports.push(report.unwrap());
            }
            reports
        },
    )
    .await
    .unwrap();
    for report in &reports {
        assert_eq!(
            report.complete,
            !fail_parent && expected_fatal.is_none(),
            "{report:?}"
        );
        assert_eq!(
            report.collected.len(),
            if fail_collection { count - 1 } else { count },
            "{report:?}"
        );
        assert_eq!(
            report.distributed.len(),
            if expected_fatal.is_some() {
                count - 2
            } else if fail_parent {
                count - 1
            } else {
                count
            },
            "{report:?}"
        );
    }
    if let Some(probe) = &registration
        && let Some((replacement, listener)) = probe.replacement.lock().unwrap().take()
    {
        let last = peers.len() - 1;
        peers[last] = replacement;
        listeners.push(listener);
    }
    if let Some(probe) = &registration {
        assert!(
            probe.fired.load(Ordering::Acquire),
            "registration gate must run"
        );
    }
    let report = &reports[0];
    let connection_peaks: Vec<_> = peers.iter().map(|peer| peer.connections.peak()).collect();
    let peak_connections = connection_peaks.iter().copied().max().unwrap();
    assert!(
        connection_peaks
            .iter()
            .all(|peak| *peak <= crate::mesh::connections::capacity(count))
    );
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    let transport: Vec<[u64; 3]> = peers
        .iter()
        .zip(&transport_before)
        .map(|(peer, before)| {
            let after = transport_sample(peer);
            std::array::from_fn(|index| after[index].saturating_sub(before[index]))
        })
        .collect();
    let (reopens, source_reads) = wire::metrics::resources();
    let root_transport = transport[0];
    let root_source_reads = source_reads.get(&peers[0].local).copied().unwrap_or(0);
    let (bytes, frames) = wire::metrics::snapshot();
    let relay_traffic = network.traffic(false).await;
    let rounds: [usize; 2] = std::array::from_fn(|index| {
        peers
            .iter()
            .map(|peer| peer.replica_rounds[index].load(Ordering::Relaxed))
            .sum()
    });
    let deltas: [usize; 2] = std::array::from_fn(|index| {
        peers
            .iter()
            .map(|peer| peer.delta_transfers[index].load(Ordering::Relaxed))
            .sum()
    });
    if gateway && !fail_parent {
        if expected_fatal.is_some() {
            assert!(
                (1..=2).contains(&deltas[0]) && deltas[1] == 0,
                "real failures do not select delta fallback: {deltas:?}"
            );
            assert!(
                report
                    .failures
                    .iter()
                    .any(|failure| Some(failure.kind) == expected_fatal
                        && failure.peer == receiver_id),
                "{report:?}"
            );
        } else if delta_fault.is_some() {
            assert!(
                (1..=2).contains(&deltas[0])
                    && deltas[1] <= deltas[0]
                    && (deltas[1] > 0
                        || delta_fault
                            .as_ref()
                            .is_some_and(|fault| fault.kind == DeltaFaultKind::Truncated)),
                "uncertainty triggers bounded authoritative fallback: {deltas:?}; fault={:?}; peers={:?}",
                delta_fault.as_ref().map(|fault| fault.kind),
                peers
                    .iter()
                    .map(|peer| peer
                        .delta_transfers
                        .each_ref()
                        .map(|counter| counter.load(Ordering::Relaxed)))
                    .collect::<Vec<_>>()
            );
        } else {
            assert_eq!(
                deltas,
                [if records { 2 } else { 0 }, 0],
                "only uncovered stable L1 baselines use final deltas"
            );
        }
    }
    // Concurrent jobs can commit contributions after another job's L1 snapshot,
    // legitimately requiring final reconciliation even on a physical clique.
    if !gateway && !fail_parent && jobs == 1 && registration.is_none() {
        assert_eq!(
            rounds[1], 0,
            "an L1-only mesh must not start a redundant final round"
        );
    }
    assert_eq!(
        report.complete,
        !fail_parent && expected_fatal.is_none(),
        "{report:?}"
    );
    assert_eq!(
        report.collected.len(),
        if fail_collection { count - 1 } else { count },
        "{report:?}"
    );
    assert_eq!(
        report.distributed.len(),
        if expected_fatal.is_some() {
            count - 2
        } else if fail_parent {
            count - 1
        } else {
            count
        },
        "{report:?}"
    );
    if fail_parent {
        assert!(!report.distributed.contains(&failure_target));
        assert!(
            report
                .failures
                .iter()
                .all(|failure| failure.peer == failure_target),
            "{report:?}"
        );
    }
    if pipeline
        .as_ref()
        .is_some_and(|probe| probe.kind == PipelineProbeKind::SourceShift)
    {
        assert!(
            originals[0].lock().unwrap().contains_key("record-1"),
            "source count budget omitted a pre-existing item after an earlier-ID insertion; complete={}",
            report.complete
        );
    }
    let expected: Vec<_> = originals[0]
        .lock()
        .unwrap()
        .values()
        .map(|item| {
            (
                item.id.clone(),
                item.envelope.clone(),
                item.envelope_hash.clone(),
            )
        })
        .collect();
    assert_eq!(
        expected.len(),
        common
            + bulk.map_or(0, |bulk| bulk.count)
            + if !records {
                0
            } else if fail_collection {
                count - 1
            } else {
                count
            }
    );
    for (index, data) in originals.into_iter().enumerate() {
        if fail_parent && index == 1 {
            continue;
        }
        if expected_fatal.is_some() && index == 2 {
            assert!(data.lock().unwrap().contains_key("record-3"));
            assert!(!data.lock().unwrap().contains_key("record-4"));
            continue;
        }
        assert_eq!(
            data.lock()
                .unwrap()
                .values()
                .filter(|item| !(index == 1
                    && item.id == "000-concurrent"
                    && pipeline
                        .as_ref()
                        .is_some_and(|probe| probe.kind == PipelineProbeKind::SourceShift)))
                .map(|item| (
                    item.id.clone(),
                    item.envelope.clone(),
                    item.envelope_hash.clone()
                ))
                .collect::<Vec<_>>(),
            expected
        );
    }
    for peer in &peers {
        assert!(peer.runs.lock().unwrap().is_empty());
        assert_eq!(
            peer.store.buffers.live_count(),
            0,
            "completed jobs retain no transfer originals"
        );
    }
    let retained_roles: Vec<_> = peers
        .iter()
        .map(|peer| peer.runs.lock().unwrap().len())
        .collect();
    for peer in peers {
        peer.shutdown().await.unwrap();
    }
    for task in listeners {
        task.abort();
    }
    for task in dispatchers {
        task.abort();
    }
    serde_json::json!({"peers":count,"jobs":jobs,"records":records,"common":common,"bulk_items":bulk.map_or(0, |bulk| bulk.count),"bulk_item_bytes":bulk.map_or(0, |bulk| bulk.bytes),"peak_connections":peak_connections,"connection_peaks":connection_peaks,"discovery_budget_seconds":if count == 256 {300} else {crate::mesh::scheduler::DISCOVERY_CAP.as_secs()},"profile":2,"warm_iroh_paths":warm,"job_ms":elapsed_ms,"application_frame_bytes":bytes,"application_frames":frames,"relay_traffic":relay_traffic,"replica_rounds":rounds,"delta_transfers":deltas,"transport_handshakes_sent_received_bytes":transport,"root_transport_handshakes_sent_received_bytes":root_transport,"pressure_reopen_attempts":reopens,"source_get_requests":source_reads,"root_source_get_requests":root_source_reads,"retained_roles_after_completion":retained_roles,"job_discovery_milestones_ms":wire::metrics::milestones(),"collected":report.collected.len(),"distributed":report.distributed.len()})
}

#[tokio::test]
#[ignore = "requires native loopback UDP and an eleven-second membership gate"]
async fn native_successful_speculation_waits_for_verification_then_gets_full_admission() {
    run_scenario(Scenario {
        startup: StartupCase::Hold(Duration::from_secs(11)),
        ..Scenario::devices(3)
    })
    .await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP and speculative dial expiry"]
async fn native_failed_speculation_gets_one_fresh_verified_attempt() {
    run_scenario(Scenario {
        startup: StartupCase::Offline(Duration::from_secs(4)),
        ..Scenario::devices(3)
    })
    .await;
}
#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_cancelled_speculation_leaves_no_job_and_next_sync_starts_cold() {
    run_scenario(Scenario {
        startup: StartupCase::Cancel(Duration::from_millis(100)),
        ..Scenario::devices(3)
    })
    .await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_pending_speculation_is_adopted_and_completes_full_sync() {
    run_scenario(Scenario {
        startup: StartupCase::Hold(Duration::ZERO),
        ..Scenario::devices(3)
    })
    .await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP and the existing thirty-second idle expiry"]
async fn native_closed_successful_speculation_gets_one_fresh_verified_attempt() {
    run_scenario(Scenario {
        startup: StartupCase::Closed(Duration::from_secs(31)),
        ..Scenario::devices(3)
    })
    .await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_mixed_cohort_names_old_peer_and_fails_sync_and_checked_status() {
    let root_key = iroh::SecretKey::generate();
    let old_key = iroh::SecretKey::generate();
    let old_id = old_key.public().to_string();
    let membership = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: BTreeSet::from([root_key.public().to_string(), old_id.clone()]),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (store, dispatcher) = metadata_store(membership.clone());
    let root = Peer::bind(
        store,
        Identity {
            key: root_key,
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
    if let Some(discovery) = &root.discovery {
        discovery.stop();
    }
    let old = Endpoint::builder(presets::Minimal)
        .secret_key(old_key)
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .alpns(vec![b"linn/mesh/1".to_vec()])
        .bind()
        .await
        .unwrap();
    root.lookup
        .set_endpoint_info(EndpointAddr::new(old.id()).with_ip_addr(old.bound_sockets()[0]));
    let endpoint = old.clone();
    let accepting = tokio::spawn(async move {
        while let Some(incoming) = endpoint.accept().await {
            let _ = incoming.await;
        }
    });
    let error = root.check_transport(&old_id).await.unwrap_err();
    assert_eq!(Kind::of(&error), Kind::Version, "{error}");
    let report = root
        .clone()
        .sync_started(crate::mesh::membership::random_id(), Startup::default())
        .await
        .unwrap();
    assert!(!report.complete);
    assert!(!report.unavailable_only());
    assert!(
        report.failures.iter().any(|failure| failure.peer == old_id
            && failure.kind == Kind::Version
            && failure
                .message
                .contains(&format!("update linn on peer {old_id}, then retry"))),
        "{report:?}"
    );
    assert!(!report.collected.contains(&old_id));
    assert!(!report.distributed.contains(&old_id));
    root.shutdown().await.unwrap();
    old.close().await;
    accepting.await.unwrap();
    dispatcher.abort();
}

#[tokio::test]
#[ignore = "requires an explicitly configured local Worker relay"]
async fn native_registration_wakes_quiescent_and_final_reconciliation() {
    let url = std::env::var("LINN_RELAY_TEST_URL")
        .expect("local Worker relay required")
        .trim_end_matches('/')
        .to_owned();
    for final_round in [false, true] {
        let keys: Vec<_> = (0..5).map(|_| iroh::SecretKey::generate()).collect();
        relay_fixture::enroll(&url, &keys.iter().collect::<Vec<_>>())
            .await
            .unwrap();
        run_scenario(Scenario {
            keys,
            records: true,
            network: Network::Relay(url.clone()),
            registration: Some(RegistrationProbe::new(final_round)),
            ..Scenario::default()
        })
        .await;
    }
}

fn metadata_store(state: Membership) -> (Handle, tokio::task::JoinHandle<()>) {
    let (store, mut requests) = store::channel();
    let dispatcher = tokio::spawn(async move {
        while let Some(request) = requests.recv().await {
            let value = match request.step {
                Step::Membership(_) => Value::Membership(Some(state.clone())),
                Step::Finish(None) => Value::Done,
                Step::SourceSnapshot(_) => Value::SourceSnapshot(0),
                Step::Authorize(_) => Value::Done,
                _ => panic!("metadata-only fixture must never exchange records"),
            };
            let _ = request.reply.send(Ok(value));
        }
    });
    (store, dispatcher)
}

async fn metadata_peers(
    keys: &[iroh::SecretKey],
    state: &Membership,
) -> (Vec<Arc<Peer>>, Vec<tokio::task::JoinHandle<()>>) {
    let mut peers = Vec::new();
    let mut dispatchers = Vec::new();
    for key in keys {
        let (store, dispatcher) = metadata_store(state.clone());
        dispatchers.push(dispatcher);
        let peer = Peer::bind(
            store,
            Identity {
                key: key.clone(),
                service: state.service.clone(),
                receipt_key: Some(state.receipt_key.clone()),
                head: Some(state.head.clone()),
                enabled: true,
                pending_leave: false,
                conflict: false,
            },
            state.clone(),
        )
        .await
        .unwrap();
        if let Some(discovery) = &peer.discovery {
            discovery.stop();
        }
        peers.push(peer);
    }
    (peers, dispatchers)
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_admission_reports_more_than_eight_passive_cohort_hints() {
    let keys: Vec<_> = (0..11).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from([keys[0].public().to_string()]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (peers, dispatchers) = metadata_peers(&keys[..2], &state).await;
    let root = &peers[0];
    let target = &peers[1];
    root.lookup.set_endpoint_info(
        EndpointAddr::new(target.endpoint.id()).with_ip_addr(std::net::SocketAddr::from((
            std::net::Ipv4Addr::LOCALHOST,
            target
                .endpoint
                .bound_sockets()
                .iter()
                .find(|address| address.is_ipv4())
                .unwrap()
                .port(),
        ))),
    );
    for key in keys
        .iter()
        .filter(|key| key.public() != target.endpoint.id())
    {
        target
            .lookup
            .set_endpoint_info(EndpointAddr::new(key.public()).with_ip_addr(
                std::net::SocketAddr::from((
                    std::net::Ipv4Addr::LOCALHOST,
                    if key.public() == root.endpoint.id() {
                        root.endpoint
                            .bound_sockets()
                            .iter()
                            .find(|address| address.is_ipv4())
                            .unwrap()
                            .port()
                    } else {
                        9
                    },
                )),
            ));
    }
    let listening = target.clone();
    let listener = tokio::spawn(async move {
        listening.listen().await;
    });
    let link = root.connect(&target.local).await.unwrap();
    let (mut send, mut recv) = link.connection.open_bi().await.unwrap();
    root.handshake_client(&mut send, &mut recv, &target.local)
        .await
        .unwrap();
    let mut control = Control::new(send, recv);
    let job = SignedRun::new(&root.key, &state, crate::mesh::membership::random_id()).unwrap();
    control
        .send(Frame::Begin {
            job,
            replaces_parent: None,
            continuation: false,
            terminal_only: false,
        })
        .await
        .unwrap();
    let Frame::Accepted {
        hints,
        duplicate: false,
        ..
    } = control.receive().await.unwrap()
    else {
        panic!("fresh acceptance required")
    };
    assert_eq!(hints.len(), 9);
    assert!(
        hints
            .iter()
            .all(|hint| state.members.contains(hint) && hint != &target.local)
    );
    async fn expect_capacity(control: &mut Control, source: &str, expected: bool) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let Frame::Capacity(values) = control.receive().await.unwrap() else {
                    panic!("capacity update expected")
                };
                if values
                    .iter()
                    .any(|(peer, free)| peer == source && *free == expected)
                {
                    break;
                }
            }
        })
        .await
        .unwrap();
    }
    expect_capacity(&mut control, &target.local, true).await;
    let first = target.outgoing.acquire().await.unwrap();
    let second = target.outgoing.acquire().await.unwrap();
    expect_capacity(&mut control, &target.local, false).await;
    drop(first);
    expect_capacity(&mut control, &target.local, true).await;
    drop(second);
    assert_eq!(target.outgoing.available(), 2);
    assert!(
        tokio::time::timeout(Duration::from_millis(50), control.receive())
            .await
            .is_err(),
        "one-to-two free slots must not add a control frame"
    );
    control
        .send(Frame::Command(Envelope {
            id: 1,
            route: vec![],
            command: Command::Stop,
        }))
        .await
        .unwrap();
    let reply = loop {
        let frame = control.receive().await.unwrap();
        if !matches!(frame, Frame::Capacity(_)) {
            break frame;
        }
    };
    assert!(matches!(
        reply,
        Frame::Reply {
            result: Ok(Reply::Done { .. }),
            ..
        }
    ));
    drop(control);
    drop(link);
    root.shutdown().await.unwrap();
    target.shutdown().await.unwrap();
    listener.await.unwrap();
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP and eleven seconds of local capacity queueing"]
async fn native_adoption_keeps_capacity_queue_time_outside_total_admission() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from([keys[0].public().to_string()]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (peers, dispatchers) = metadata_peers(&keys, &state).await;
    let root = &peers[0];
    let target = &peers[1];
    let port = target
        .endpoint
        .bound_sockets()
        .iter()
        .find(|address| address.is_ipv4())
        .unwrap()
        .port();
    root.lookup
        .set_endpoint_info(EndpointAddr::new(target.endpoint.id()).with_ip_addr(
            std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port)),
        ));
    let serving = target.clone();
    let listener = tokio::spawn(async move {
        serving.listen().await;
    });
    let ticket = root.connections.queue().await;
    let held: Vec<_> = (0..crate::mesh::connections::capacity(2))
        .map(|_| root.connections.reserve(&ticket).unwrap())
        .collect();
    drop(ticket);
    let mut startup = Startup::start(root.clone()).await.unwrap();
    startup
        .verified(root, tokio::time::Instant::now())
        .await
        .unwrap();
    let (link, ()) = tokio::join!(
        startup.adopt(root, &target.local, Arc::new(|_, _| {})),
        async {
            tokio::time::sleep(Duration::from_secs(11)).await;
            drop(held);
        }
    );
    let link = link.unwrap().expect("the queued dial is adopted once");
    assert!(
        link.admission_deadline
            .saturating_duration_since(tokio::time::Instant::now())
            > Duration::from_secs(8),
        "capacity queue must not consume the admission clock"
    );
    drop(link);
    drop(startup);
    root.shutdown().await.unwrap();
    target.shutdown().await.unwrap();
    listener.await.unwrap();
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_control_actor_keeps_a_protocol_failure_without_pending_requests() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (peers, dispatchers) = metadata_peers(&keys, &state).await;
    let root = &peers[0];
    let target = &peers[1];
    root.lookup.set_endpoint_info(
        EndpointAddr::new(target.endpoint.id()).with_ip_addr(std::net::SocketAddr::from((
            std::net::Ipv4Addr::LOCALHOST,
            target
                .endpoint
                .bound_sockets()
                .iter()
                .find(|address| address.is_ipv4())
                .unwrap()
                .port(),
        ))),
    );
    let (outgoing, incoming) = tokio::join!(root.connect(&target.local), async {
        target.endpoint.accept().await.unwrap().await.unwrap()
    });
    let outgoing = outgoing.unwrap();
    let (send, recv) = outgoing.connection.open_bi().await.unwrap();
    let control = Control::new(send, recv);
    control.send(Frame::Ack).await.unwrap();
    let (mut send, mut recv) = incoming.accept_bi().await.unwrap();
    assert!(matches!(
        wire::receive_limit(&mut recv, 4096).await.unwrap(),
        Frame::Ack
    ));
    let session = Session::new(
        outgoing,
        control,
        Arc::default(),
        state.members.iter().cloned().collect(),
    );
    wire::send(
        &mut send,
        Frame::Capacity(vec![("outside the cohort".into(), true)]),
    )
    .await
    .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while !session.actor.is_finished() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let failure = session
        .request(
            Envelope {
                id: 1,
                route: vec![],
                command: Command::Stop,
            },
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(
        Kind::of(&failure),
        Kind::Protocol,
        "an unsolicited protocol error cannot become offline on a later request"
    );
    drop(session);
    drop((send, recv, incoming));
    for peer in peers {
        peer.shutdown().await.unwrap();
    }
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_discovery_cap_uses_verified_refresh_completion_and_names_unreached() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (peers, dispatchers) = metadata_peers(&keys[..1], &state).await;
    let root = peers[0].clone();
    let mut startup = Startup::default();
    let verified = tokio::time::Instant::now()
        - crate::mesh::scheduler::DISCOVERY_CAP
        - Duration::from_secs(1);
    startup.verified(&root, verified).await.unwrap();
    let coordinator =
        Coordinator::prepare(root.clone(), crate::mesh::membership::random_id(), startup)
            .await
            .unwrap();
    assert_eq!(coordinator.origin, verified);
    assert_eq!(
        coordinator.scheduler.deadline,
        crate::mesh::scheduler::DISCOVERY_CAP
    );
    let report = coordinator.run().await.unwrap();
    let target = keys[1].public().to_string();
    assert!(!report.complete && !report.unavailable_only());
    assert!(report.traversal.as_ref().unwrap().discovery_expired);
    assert_eq!(report.collected, vec![root.local.clone()]);
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.peer == target && failure.kind == Kind::Timeout)
    );
    root.shutdown().await.unwrap();
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}

#[tokio::test]
#[ignore = "native concurrent stream retirement stress; run without other native tests"]
async fn native_concurrent_replica_stream_retirement_does_not_stall() {
    for paths in [PathState::Cold, PathState::Warm] {
        for _ in 0..12 {
            run_scenario(Scenario {
                keys: (0..10)
                    .map(|index| iroh::SecretKey::from_bytes(&[(index + 1) as u8; 32]))
                    .collect(),
                jobs: 4,
                records: true,
                paths,
                ..Scenario::default()
            })
            .await;
        }
    }
}

#[tokio::test]
#[ignore = "native expansion/control regression; run without other native tests"]
async fn native_twenty_peer_expansion_reconciles_empty_and_populated_stores() {
    for _ in 0..4 {
        for paths in [PathState::Cold, PathState::Warm] {
            for records in [false, true] {
                run_scenario(Scenario {
                    topology: Topology::Gateway,
                    keys: (0..20)
                        .map(|index| iroh::SecretKey::from_bytes(&[(index + 1) as u8; 32]))
                        .collect(),
                    records,
                    paths,
                    ..Scenario::default()
                })
                .await;
            }
        }
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_admitted_peer_loss_mid_round_is_unresolved_and_fails_command() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (root_store, root_dispatcher) = metadata_store(state.clone());
    let (child_store, mut requests) = store::channel();
    let (entered, snapshot) = oneshot::channel();
    let release = Arc::new(tokio::sync::Notify::new());
    let released = release.clone();
    let membership = state.clone();
    let child_dispatcher = tokio::spawn(async move {
        let mut entered = Some(entered);
        while let Some(request) = requests.recv().await {
            let value = match request.step {
                Step::Membership(_) => Value::Membership(Some(membership.clone())),
                Step::Finish(None) => Value::Done,
                Step::Authorize(_) => Value::Done,
                Step::SourceSnapshot(_) => {
                    if let Some(entered) = entered.take() {
                        let _ = entered.send(());
                        released.notified().await;
                    }
                    Value::SourceSnapshot(0)
                }
                _ => panic!("source must lose its transport before sending inventory"),
            };
            let _ = request.reply.send(Ok(value));
        }
    });
    let mut peers = Vec::new();
    for (key, store) in keys.into_iter().zip([root_store, child_store]) {
        let peer = Peer::bind(
            store,
            Identity {
                key,
                service: state.service.clone(),
                receipt_key: Some(state.receipt_key.clone()),
                head: Some(state.head.clone()),
                enabled: true,
                pending_leave: false,
                conflict: false,
            },
            state.clone(),
        )
        .await
        .unwrap();
        if let Some(discovery) = &peer.discovery {
            discovery.stop();
        }
        peers.push(peer);
    }
    peers[0].lookup.set_endpoint_info(
        EndpointAddr::new(peers[1].endpoint.id()).with_ip_addr(
            (
                std::net::Ipv4Addr::LOCALHOST,
                peers[1]
                    .endpoint
                    .bound_sockets()
                    .iter()
                    .find(|addr| addr.is_ipv4())
                    .unwrap()
                    .port(),
            )
                .into(),
        ),
    );
    let listening = peers[1].clone();
    let listener = tokio::spawn(async move {
        listening.listen().await;
    });
    let root = peers[0].clone();
    let sync = tokio::spawn(async move {
        root.sync_started(crate::mesh::membership::random_id(), Startup::default())
            .await
    });
    tokio::time::timeout(Duration::from_secs(10), snapshot)
        .await
        .unwrap()
        .unwrap();
    // The source is inside an admitted replica round, without any absence observation.
    peers[1].endpoint.close().await;
    release.notify_one();
    let report = tokio::time::timeout(Duration::from_secs(15), sync)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!report.complete && !report.unavailable_only(), "{report:?}");
    assert!(
        report
            .failures
            .iter()
            .all(|failure| failure.kind != Kind::Offline),
        "{report:?}"
    );
    assert!(!crate::mesh::pairing::Output::Synced(report).successful());
    for peer in peers {
        peer.shutdown().await.unwrap();
    }
    root_dispatcher.abort();
    child_dispatcher.abort();
    listener.abort();
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn native_no_known_direct_route_returns_distinct_reply_without_dialing() {
    let keys: Vec<_> = (0..2).map(|_| iroh::SecretKey::generate()).collect();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from(["1".repeat(64)]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (peers, dispatchers) = metadata_peers(&keys[..1], &state).await;
    let root = peers[0].clone();
    let coordinator = Coordinator::prepare(
        root.clone(),
        crate::mesh::membership::random_id(),
        Startup::default(),
    )
    .await
    .unwrap();
    let before = transport_sample(&root)[0];
    let start = tokio::time::Instant::now();
    let reply = coordinator
        .network
        .node
        .clone()
        .execute(
            Envelope {
                id: 1,
                route: vec![],
                command: Command::Contact {
                    assignment: 1,
                    target: keys[1].public().to_string(),
                    routes: Routes {
                        direct: true,
                        relay: false,
                    },
                    replaces_parent: None,
                },
            },
            None,
        )
        .await
        .unwrap();
    assert!(matches!(reply, Reply::NoDirectRoute));
    assert!(start.elapsed() < Duration::from_millis(500));
    assert_eq!(transport_sample(&root)[0], before);
    drop(coordinator);
    root.shutdown().await.unwrap();
    for dispatcher in dispatchers {
        dispatcher.abort();
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP and passive discovery readiness"]
async fn native_readiness_learned_direct_version_failure_is_reported_as_attempted() {
    let keys = [iroh::SecretKey::generate(), iroh::SecretKey::generate()];
    let target = keys[1].public().to_string();
    let state = Membership {
        head: "0".repeat(64),
        service: "http://127.0.0.1:1".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::from([keys[0].public().to_string()]),
        members: keys.iter().map(|key| key.public().to_string()).collect(),
        revoked: Default::default(),
        invitations: Default::default(),
    };
    let (store, dispatcher) = metadata_store(state.clone());
    let root = Peer::bind(
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
    assert!(root.discovery.as_ref().is_some_and(LocalLookup::is_running));
    let old = Endpoint::builder(presets::Minimal)
        .secret_key(keys[1].clone())
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .alpns(vec![b"linn/mesh/1".to_vec()])
        .bind()
        .await
        .unwrap();
    let address = EndpointAddr::new(old.id()).with_ip_addr(old.bound_sockets()[0]);
    let accepting = old.clone();
    let listener = tokio::spawn(async move {
        let _ = accepting.accept().await.unwrap().await;
    });
    let coordinator = Coordinator::prepare(
        root.clone(),
        crate::mesh::membership::random_id(),
        Startup::default(),
    )
    .await
    .unwrap();
    assert!(!root.has_direct_route(&target));
    let (reply, ()) = tokio::join!(
        coordinator.network.node.contact_command(
            &root,
            1,
            Command::Contact {
                assignment: 1,
                target: target.clone(),
                routes: Routes {
                    direct: true,
                    relay: true
                },
                replaces_parent: None,
            },
            None
        ),
        async {
            tokio::time::timeout(Duration::from_secs(2), async {
                while root.connections.peak() == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert!(!root.has_direct_route(&target));
            root.lookup.set_endpoint_info(address);
        }
    );
    assert!(matches!(
        reply.unwrap(),
        Reply::ContactFailed {
            direct: true,
            kind: Kind::Version,
            ..
        }
    ));
    drop(coordinator);
    root.shutdown().await.unwrap();
    old.close().await;
    listener.await.unwrap();
    dispatcher.abort();
}
