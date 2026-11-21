use super::*;
use std::{
    collections::BTreeSet,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
struct Network {
    edges: BTreeSet<(String, String)>,
    slow: bool,
    active: AtomicUsize,
    peak: AtomicUsize,
    calls: Mutex<Vec<Assignment>>,
}
struct Owned<'a>(&'a AtomicUsize);
impl Drop for Owned<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Dialer for Network {
    type Link = (String, String);
    async fn contact(
        &self,
        assignment: Assignment,
        updates: mpsc::Sender<Observation>,
    ) -> Attempt<Self::Link> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(active, Ordering::SeqCst);
        let _owned = Owned(&self.active);
        self.calls.lock().unwrap().push(assignment.clone());
        let observation = assignment.routes.relay.then_some(Observation {
            assignment: assignment.id,
            generation: 1,
            registered: false,
            connected: false,
        });
        if let Some(observation) = &observation {
            let _ = updates.try_send(observation.clone());
        }
        let edge = (assignment.source.clone(), assignment.target.clone());
        let delay = match (assignment.source.as_str(), assignment.target.as_str()) {
            ("a", "b") => 10,
            ("a", "c") => 80,
            ("a", "d") => 3000,
            ("b", "d") if self.slow => 2000,
            _ => 20,
        };
        tokio::time::sleep(Duration::from_millis(delay)).await;
        let result = if self.edges.contains(&edge) {
            Ok(Admission::Accepted {
                link: edge,
                via_relay: false,
                hints: vec![],
            })
        } else {
            Err(Kind::Timeout)
        };
        Attempt {
            direct: if assignment.routes.direct {
                Direct::Attempted
            } else {
                Direct::Unavailable
            },
            observation,
            result,
        }
    }
}
fn network(slow: bool) -> Network {
    Network {
        edges: [("a", "b"), ("a", "c"), ("b", "d")]
            .into_iter()
            .map(|(a, b)| (a.into(), b.into()))
            .collect(),
        slow,
        active: AtomicUsize::new(0),
        peak: AtomicUsize::new(0),
        calls: Mutex::default(),
    }
}
fn scheduler() -> Scheduler {
    Scheduler::new(
        "a".into(),
        ["a", "b", "c", "d"].map(String::from).into(),
        "relay".into(),
    )
    .unwrap()
}
#[tokio::test(start_paused = true)]
async fn expansion_and_admission_notifications_do_not_wait_for_l1_store_work() {
    let network = network(false);
    let origin = tokio::time::Instant::now();
    let (discovery, mut admissions) = start(scheduler(), &network, origin);
    let store_finished = Arc::new(AtomicUsize::new(0));
    let store = store_finished.clone();
    let data = async {
        let Event::Accepted(first) = admissions.recv().await.unwrap() else {
            panic!("admission expected");
        };
        assert_eq!(first.assignment.target, "b");
        tokio::time::sleep(Duration::from_secs(1)).await;
        store.store(1, Ordering::SeqCst);
    };
    let discovery = async {
        let result = discovery.await;
        assert!(origin.elapsed() < Duration::from_millis(100));
        assert_eq!(store_finished.load(Ordering::SeqCst), 0);
        assert_eq!(result.links.len(), 3);
        assert_eq!(result.links["d"], ("b".into(), "d".into()));
        assert!(result.scheduler.missing().next().is_none());
        assert!(result.failures.is_empty());
    };
    tokio::join!(discovery, data);
    assert_eq!(network.active.load(Ordering::SeqCst), 0);
    assert!(network.peak.load(Ordering::SeqCst) <= 8);
    let calls = network.calls.lock().unwrap();
    let expansion = calls
        .iter()
        .find(|a| a.source == "b" && a.target == "d")
        .unwrap();
    assert!(expansion.issued < Duration::from_millis(100));
    assert!(!expansion.routes.relay);
}
#[tokio::test(start_paused = true)]
async fn discovery_cap_cancels_contacts_but_not_admitted_data() {
    let network = network(true);
    let mut scheduler = scheduler();
    scheduler.deadline = Duration::from_millis(100);
    let origin = tokio::time::Instant::now();
    let (discovery, mut admissions) = start(scheduler, &network, origin);
    let data = async {
        let Event::Accepted(first) = admissions.recv().await.unwrap() else {
            panic!("admission expected");
        };
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(first.link, ("a".into(), "b".into()));
    };
    let discovery = async {
        let result = discovery.await;
        assert_eq!(origin.elapsed(), Duration::from_millis(100));
        assert!(result.links.contains_key("b") && result.links.contains_key("c"));
        assert_eq!(result.scheduler.missing().collect::<Vec<_>>(), ["d"]);
        assert_eq!(result.failures["d"], Kind::Timeout);
        assert_eq!(network.active.load(Ordering::SeqCst), 0);
    };
    tokio::join!(discovery, data);
    assert!(origin.elapsed() >= Duration::from_secs(2));
}
#[tokio::test(start_paused = true)]
async fn losing_the_data_consumer_cancels_owned_contacts_and_connections() {
    let network = network(false);
    let (discovery, admissions) = start(scheduler(), &network, tokio::time::Instant::now());
    drop(admissions);
    let result = discovery.await;
    assert!(result.links.is_empty());
    assert_eq!(network.active.load(Ordering::SeqCst), 0);
    assert!(!result.scheduler.discovery_open(Duration::from_secs(1)));
}

#[cfg(test)]
mod registration_tests {
    use super::*;
    fn gone() -> Scheduler {
        let mut scheduler =
            Scheduler::new("a".into(), vec!["a".into(), "b".into()], "relay".into()).unwrap();
        let assignment = scheduler.schedule(Duration::ZERO).remove(0);
        scheduler.relay_observation(assignment.id, 1, false, Duration::ZERO);
        scheduler.finished(assignment.id);
        scheduler
    }
    #[test]
    fn registration_only_triggers_one_relay_retry_and_never_admits_or_resets_direct() {
        let mut scheduler = gone();
        let mut failures = BTreeMap::from([("b".into(), Kind::Offline)]);
        registration(
            &mut scheduler,
            &mut failures,
            Registration {
                target: "b".into(),
                registered: true,
            },
            Duration::ZERO,
        );
        assert_eq!(scheduler.reached().len(), 1);
        assert_eq!(failures["b"], Kind::Timeout);
        let retry = scheduler.schedule(Duration::ZERO).remove(0);
        assert!(retry.refresh && retry.routes.relay && !retry.routes.direct);
        scheduler.finished(retry.id);
        registration(
            &mut scheduler,
            &mut failures,
            Registration {
                target: "b".into(),
                registered: true,
            },
            Duration::ZERO,
        );
        assert!(scheduler.schedule(Duration::ZERO).is_empty());
    }
    #[test]
    fn unsupported_notifications_and_expired_caps_stay_unresolved_without_retries() {
        for registered in [false, true] {
            let mut scheduler = gone();
            if registered {
                scheduler.deadline = Duration::ZERO;
            }
            let mut failures = BTreeMap::from([("b".into(), Kind::Offline)]);
            registration(
                &mut scheduler,
                &mut failures,
                Registration {
                    target: "b".into(),
                    registered,
                },
                Duration::ZERO,
            );
            assert_eq!(failures["b"], Kind::Timeout);
            assert_eq!(scheduler.reached().len(), 1);
            assert!(scheduler.schedule(Duration::ZERO).is_empty());
        }
    }
}

#[cfg(test)]
mod absence_tests {
    use super::*;
    use std::sync::Mutex;

    struct Network {
        correlated: bool,
        direct: Direct,
        failure: Kind,
        calls: Mutex<Vec<Assignment>>,
    }
    impl Dialer for Network {
        type Link = ();
        async fn contact(
            &self,
            assignment: Assignment,
            updates: mpsc::Sender<Observation>,
        ) -> Attempt<()> {
            self.calls.lock().unwrap().push(assignment.clone());
            if assignment.target == "b" {
                return Attempt {
                    direct: Direct::Attempted,
                    observation: None,
                    result: Ok(Admission::Accepted {
                        link: (),
                        via_relay: false,
                        hints: vec![],
                    }),
                };
            }
            let observation = (self.correlated && assignment.routes.relay).then_some(Observation {
                assignment: assignment.id,
                generation: 1,
                registered: false,
                connected: false,
            });
            if let Some(observation) = &observation {
                updates.send(observation.clone()).await.unwrap();
            }
            // A real failure at b must survive the later timeout from a.
            tokio::time::sleep(Duration::from_millis(if assignment.source == "a" {
                3000
            } else {
                20
            }))
            .await;
            Attempt {
                direct: self.direct,
                observation,
                result: Err(if assignment.source == "a" {
                    Kind::Timeout
                } else {
                    self.failure
                }),
            }
        }
    }
    #[tokio::test(start_paused = true)]
    async fn current_correlated_absence_requires_all_direct_routes_and_preserves_real_failures() {
        for correlated in [false, true] {
            for failure in [
                Kind::Timeout,
                Kind::Version,
                Kind::Authentication,
                Kind::Protocol,
                Kind::Conflict,
                Kind::Publication,
                Kind::Capacity,
                Kind::HopLimit,
                Kind::Cancelled,
                Kind::Unresolved,
                Kind::Offline,
            ] {
                let network = Network {
                    correlated,
                    direct: Direct::Attempted,
                    failure,
                    calls: Mutex::default(),
                };
                let scheduler = Scheduler::new(
                    "a".into(),
                    vec!["a".into(), "b".into(), "c".into()],
                    "relay".into(),
                )
                .unwrap();
                let (discovery, _admissions) =
                    start(scheduler, &network, tokio::time::Instant::now());
                let result = discovery.await;
                assert_eq!(result.scheduler.reached().len(), 2);
                let expected = match failure {
                    Kind::Timeout if correlated => Kind::Offline,
                    Kind::Offline => Kind::Timeout,
                    other => other,
                };
                assert_eq!(
                    result.failures["c"], expected,
                    "correlated={correlated}, failure={failure:?}"
                );
                let calls = network.calls.lock().unwrap();
                for source in ["a", "b"] {
                    assert_eq!(
                        calls
                            .iter()
                            .filter(|a| a.source == source && a.target == "c" && a.routes.direct)
                            .count(),
                        1
                    );
                }
                assert!(result.scheduler.pending().is_empty());
            }
        }
    }
    #[tokio::test(start_paused = true)]
    async fn a_source_without_direct_transport_does_not_fabricate_a_direct_failure() {
        let network = Network {
            correlated: true,
            direct: Direct::Unavailable,
            failure: Kind::Offline,
            calls: Mutex::default(),
        };
        let scheduler = Scheduler::new(
            "a".into(),
            vec!["a".into(), "b".into(), "c".into()],
            "relay".into(),
        )
        .unwrap();
        let (discovery, _admissions) = start(scheduler, &network, tokio::time::Instant::now());
        assert_eq!(discovery.await.failures["c"], Kind::Offline);
    }
    #[test]
    fn invalidated_absence_and_cancelled_direct_attempts_never_certify_offline() {
        for cancel in [false, true] {
            let mut scheduler =
                Scheduler::new("a".into(), vec!["a".into(), "b".into()], "relay".into()).unwrap();
            let first = scheduler.schedule(Duration::ZERO).remove(0);
            assert!(scheduler.relay_observation(first.id, 1, false, Duration::ZERO));
            assert!(
                !scheduler.absence_complete("b"),
                "an active direct attempt has not settled"
            );
            if cancel {
                scheduler.close_discovery();
            } else {
                scheduler.finished(first.id);
                assert!(scheduler.absence_complete("b"));
                scheduler.registration_unknown("b");
            }
            assert!(!scheduler.absence_complete("b"));
        }
    }
    #[tokio::test(start_paused = true)]
    async fn source_outcome_unknown_never_means_no_address_or_certified_absence() {
        let network = Network {
            correlated: true,
            direct: Direct::Unknown,
            failure: Kind::Unresolved,
            calls: Mutex::default(),
        };
        let scheduler = Scheduler::new(
            "a".into(),
            vec!["a".into(), "b".into(), "c".into()],
            "relay".into(),
        )
        .unwrap();
        let (future, _admissions) = start(scheduler, &network, tokio::time::Instant::now());
        let result = future.await;
        assert_eq!(result.failures["c"], Kind::Unresolved);
        assert!(!result.scheduler.absence_complete("c"));
        assert!(!result.scheduler.failed_direct("c"));
        assert!(
            result
                .scheduler
                .snapshot(Duration::ZERO)
                .no_known_direct_routes
                .is_empty()
        );
    }
}

struct RecoveredVersion;
impl Dialer for RecoveredVersion {
    type Link = ();
    async fn contact(&self, assignment: Assignment, _: mpsc::Sender<Observation>) -> Attempt<()> {
        Attempt {
            direct: Direct::Attempted,
            observation: None,
            result: if assignment.source == "a" && assignment.target == "c" {
                Err(Kind::Version)
            } else {
                Ok(Admission::Accepted {
                    link: (),
                    via_relay: false,
                    hints: vec![],
                })
            },
        }
    }
}
#[tokio::test(start_paused = true)]
async fn actual_non_timeout_direct_failure_survives_later_successful_admission() {
    let scheduler = Scheduler::new(
        "a".into(),
        vec!["a".into(), "b".into(), "c".into()],
        "relay".into(),
    )
    .unwrap();
    let (future, _admissions) = start(scheduler, &RecoveredVersion, tokio::time::Instant::now());
    let result = future.await;
    assert_eq!(result.scheduler.reached().len(), 3);
    assert_eq!(result.failures["c"], Kind::Version);
    assert!(result.scheduler.failed_direct("c"));
}

struct NoAddresses;
impl Dialer for NoAddresses {
    type Link = ();
    async fn contact(&self, assignment: Assignment, _: mpsc::Sender<Observation>) -> Attempt<()> {
        if assignment.target != "d" {
            return Attempt {
                direct: Direct::Attempted,
                observation: None,
                result: Ok(Admission::Accepted {
                    link: (),
                    via_relay: false,
                    hints: vec![],
                }),
            };
        }
        Attempt {
            direct: Direct::Unavailable,
            observation: assignment.routes.relay.then_some(Observation {
                assignment: assignment.id,
                generation: 1,
                registered: false,
                connected: false,
            }),
            result: if assignment.routes.relay {
                Err(Kind::Timeout)
            } else {
                Ok(Admission::NoDirectRoute)
            },
        }
    }
}
#[tokio::test(start_paused = true)]
async fn no_known_direct_routes_are_reported_without_synthetic_timeouts() {
    let scheduler = Scheduler::new(
        "a".into(),
        ["a", "b", "c", "d"].map(String::from).to_vec(),
        "relay".into(),
    )
    .unwrap();
    let origin = tokio::time::Instant::now();
    let (future, _admissions) = start(scheduler, &NoAddresses, origin);
    let result = future.await;
    assert_eq!(origin.elapsed(), Duration::ZERO);
    assert_eq!(result.failures["d"], Kind::Offline);
    assert_eq!(
        result
            .scheduler
            .snapshot(Duration::ZERO)
            .no_known_direct_routes,
        vec![
            ("a".into(), "d".into()),
            ("b".into(), "d".into()),
            ("c".into(), "d".into())
        ]
    );
}
