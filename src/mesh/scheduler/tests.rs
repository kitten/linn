use super::*;
fn at(ms: u64) -> Duration {
    Duration::from_millis(ms)
}
fn scheduler(n: usize) -> Scheduler {
    Scheduler::new(
        "p000".into(),
        (0..n).map(|p| format!("p{p:03}")).collect(),
        "relay".into(),
    )
    .unwrap()
}
fn frontier() -> (Scheduler, Assignment) {
    let mut ledger = scheduler(4);
    let assignments = ledger.schedule(at(0));
    let source = assignments.iter().find(|a| a.target == "p001").unwrap();
    ledger.accept(source.id, false, at(1)).unwrap();
    let target = assignments
        .into_iter()
        .find(|a| a.target == "p002")
        .unwrap();
    (ledger, target)
}
#[test]
fn eight_contacts_two_sources_and_one_winner_with_no_data_round_barrier() {
    let mut large = scheduler(20);
    assert_eq!(large.schedule(at(0)).len(), 8);
    assert!(large.schedule(at(1000)).is_empty());
    let (mut ledger, first) = frontier();
    assert!(ledger.schedule(at(749)).is_empty());
    let second = ledger
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == first.target)
        .unwrap();
    assert_eq!(second.source, "p001");
    assert_eq!(second.level, 2);
    assert_eq!(
        ledger
            .pending
            .values()
            .filter(|a| a.target == first.target)
            .count(),
        2
    );
    assert_eq!(
        ledger.accept(second.id, false, at(751)),
        Some(vec![first.id])
    );
    assert!(ledger.accept(first.id, true, at(752)).is_none());
    assert_eq!(
        ledger.reached[&first.target].parent.as_deref(),
        Some("p001")
    );
    assert_eq!(
        ledger.relay_state("relay", &first.target),
        RelayState::Unknown
    );
    // Route failure permits expansion immediately, while the first direct
    // path remains pending and before any contribution has been collected.
    let (mut ledger, first) = frontier();
    assert!(ledger.relay_observation(first.id, 1, false, at(20)));
    let second = ledger
        .schedule(at(20))
        .into_iter()
        .find(|a| a.target == first.target)
        .unwrap();
    assert!(second.routes.direct);
    assert!(!second.routes.relay);
}
#[test]
fn expansion_does_not_wait_for_the_remaining_initial_contact_batches() {
    let mut ledger = scheduler(20);
    let initial = ledger.schedule(at(0));
    let first = initial.iter().find(|a| a.target == "p001").unwrap();
    ledger.accept(first.id, false, at(10)).unwrap();
    ledger.schedule(at(10));
    let failed = initial.iter().find(|a| a.target == "p002").unwrap();
    ledger.finished(failed.id);
    let expansion = ledger.schedule(at(20));
    assert_eq!(expansion.len(), 1);
    assert_eq!(expansion[0].source, "p001");
    assert_eq!(expansion[0].target, "p002");
    assert!(!ledger.tried.contains_key(&("p000".into(), "p010".into())));
}

#[test]
fn l1_seals_after_initial_outcomes_without_waiting_for_expansion() {
    let mut ledger = scheduler(4);
    let initial = ledger.schedule(at(0));
    assert_eq!(initial.len(), 3);
    assert!(
        !ledger.initial_complete(at(0)),
        "issuing the initial pass does not seal L1"
    );
    for assignment in initial {
        if assignment.target == "p001" {
            ledger.accept(assignment.id, false, at(1)).unwrap();
        } else {
            ledger.finished(assignment.id);
        }
    }
    assert!(ledger.initial_complete(at(2)));
    let expansion = ledger.schedule(at(2));
    assert!(!expansion.is_empty());
    assert!(expansion.iter().all(|assignment| assignment.level == 2));
    assert!(
        ledger.initial_complete(at(2)),
        "expansion does not reopen L1"
    );
}

#[test]
fn duplicate_registration_waits_for_its_actual_parent_without_new_probes() {
    let (mut ledger, first) = frontier();
    let second = ledger
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == first.target)
        .unwrap();
    assert_eq!(
        ledger.registered_elsewhere(second.id, &first.source),
        Some(first.id)
    );
    let third = ledger
        .pending
        .values()
        .find(|a| a.target == "p003")
        .unwrap()
        .id;
    ledger.accept(third, false, at(751)).unwrap();
    assert!(ledger.schedule(at(1000)).is_empty());
    assert_eq!(ledger.pending[&first.id].issued, at(0));
    ledger.finished(first.id);
    let replacement = ledger.schedule(at(1001));
    assert_eq!(replacement.len(), 1);
    assert_eq!(replacement[0].source, "p003");
    assert_eq!(replacement[0].target, first.target);
}

fn hinted_sources() -> Scheduler {
    let mut ledger = scheduler(8);
    for attempt in ledger.schedule(at(0)) {
        if attempt.target.as_str() < "p004" {
            ledger.accept(attempt.id, false, at(1)).unwrap();
        } else {
            ledger.finished(attempt.id);
        }
    }
    ledger.reached.get_mut("p003").unwrap().level = 2;
    ledger.reached.get_mut("p003").unwrap().parent = Some("p001".into());
    for source in ["p001", "p002", "p003"] {
        for target in 4..8 {
            ledger.hint(source, &format!("p{target:03}"));
        }
    }
    ledger
}
#[test]
fn idle_equally_hinted_sources_prefer_shallow_depth_and_rotate_fairly() {
    let mut ledger = hinted_sources();
    ledger.capacity(BTreeMap::from([
        ("p001".into(), true),
        ("p002".into(), true),
        ("p003".into(), true),
    ]));
    let assignments = ledger.schedule(at(2));
    assert_eq!(
        assignments
            .iter()
            .map(|attempt| attempt.source.as_str())
            .collect::<Vec<_>>(),
        ["p001", "p002", "p001", "p002"]
    );
    for attempt in assignments {
        ledger.accept(attempt.id, false, at(3)).unwrap();
    }
    assert_eq!(ledger.reached().len(), 8);
}
#[test]
fn busy_or_unknown_shallower_sources_never_delay_an_idle_deeper_source() {
    for capacities in [
        vec![
            ("p001".into(), false),
            ("p002".into(), false),
            ("p003".into(), true),
        ],
        vec![("p003".into(), true)],
    ] {
        let mut ledger = hinted_sources();
        let tried = ledger.tried.clone();
        ledger.capacity(capacities.into_iter().collect());
        assert_eq!(
            ledger.tried, tried,
            "capacity updates cannot reset route attempts"
        );
        assert!(
            ledger
                .schedule(at(2))
                .iter()
                .all(|attempt| attempt.source == "p003")
        );
    }
    let mut ledger = hinted_sources();
    ledger.reached.get_mut("p001").unwrap().level = 3;
    ledger.capacity(BTreeMap::from([
        ("p001".into(), false),
        ("p002".into(), false),
        ("p003".into(), false),
    ]));
    assert_eq!(
        ledger.schedule(at(2))[0].source,
        "p001",
        "busy sources get no depth preference and remain eligible"
    );
}
#[test]
fn hints_change_order_but_an_older_fourth_gateway_remains_eligible() {
    let mut ledger = scheduler(6);
    let assignments = ledger.schedule(at(0));
    for a in &assignments {
        if a.target != "p005" {
            ledger.accept(a.id, false, at(1)).unwrap();
        }
    }
    let missing = assignments.iter().find(|a| a.target == "p005").unwrap();
    ledger.finished(missing.id).unwrap();
    ledger.hint("p004", "p005");
    let mut sources = vec!["p000".to_owned()];
    loop {
        let assignments = ledger.schedule(at(2));
        if assignments.is_empty() {
            break;
        }
        for a in assignments {
            assert!(a.routes.direct && a.routes.relay);
            sources.push(a.source.clone());
            ledger.finished(a.id);
        }
    }
    assert_eq!(sources, ["p000", "p004", "p001", "p002", "p003"]);
    assert!(ledger.exhausted(at(2)));
    assert_eq!(ledger.relay_state("relay", "p005"), RelayState::Unknown);
    assert!(ledger.discovery_open(at(2)));
}
#[test]
fn only_a_connection_from_a_causally_new_attempt_grants_presence_refresh() {
    let (mut ledger, old) = frontier();
    ledger.relay_observation(old.id, 1, false, at(2));
    ledger.finished(old.id);
    let direct = ledger
        .schedule(at(3))
        .into_iter()
        .find(|a| a.target == old.target)
        .unwrap();
    assert!(direct.routes.direct && !direct.routes.relay);
    assert!(ledger.target_connected(direct.id, at(4)));
    assert!(!ledger.target_connected(direct.id, at(5)));
    assert!(
        ledger
            .schedule(at(4))
            .iter()
            .all(|a| a.target != old.target)
    );
    let retry = ledger
        .schedule(at(753))
        .into_iter()
        .find(|a| a.target == old.target)
        .unwrap();
    assert!(retry.refresh && retry.routes.relay && !retry.routes.direct);

    let (mut overlapping, old) = frontier();
    let second = overlapping
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == old.target)
        .unwrap();
    overlapping.relay_observation(old.id, 1, false, at(751));
    assert!(!overlapping.target_connected(second.id, at(752)));
    assert_eq!(
        overlapping.relay_state("relay", &old.target),
        RelayState::Unknown
    );
    assert!(!overlapping.relay_entry(&old.target).unwrap().invalidated);
}
#[test]
fn one_refresh_can_reuse_a_relay_route_but_never_a_direct_route() {
    let (mut ledger, old) = frontier();
    ledger.relay_observation(old.id, 1, false, at(2));
    ledger.finished(old.id);
    assert_eq!(ledger.relay_state("relay", &old.target), RelayState::Gone);
    assert!(ledger.target_present("relay", &old.target, at(3)));
    ledger.hint(&old.source, &old.target);
    let refresh = ledger
        .schedule(at(3))
        .into_iter()
        .find(|a| a.target == old.target)
        .unwrap();
    assert_eq!(refresh.source, old.source);
    assert!(refresh.refresh && refresh.routes.relay && !refresh.routes.direct);
    ledger.finished(refresh.id); // A timeout stays unknown.
    assert_eq!(
        ledger.relay_state("relay", &old.target),
        RelayState::Unknown
    );
    ledger.target_present("relay", &old.target, at(4));
    let attempts = ledger.schedule(at(4));
    assert!(
        attempts
            .iter()
            .filter(|a| a.target == old.target)
            .all(|a| a.routes.direct && !a.routes.relay)
    );
    assert!(
        ledger
            .pending
            .values()
            .filter(|a| a.target == old.target)
            .all(|a| !a.refresh)
    );
    assert!(!ledger.target_present("other-relay", &old.target, at(5)));
}
#[test]
fn current_registered_reply_fences_a_delayed_negative_from_another_attempt() {
    let (mut ledger, first) = frontier();
    let second = ledger
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == first.target)
        .unwrap();
    assert!(ledger.relay_observation(second.id, 1, true, at(751)));
    assert!(!ledger.relay_observation(first.id, 1, false, at(752)));
    assert_eq!(
        ledger.relay_state("relay", &first.target),
        RelayState::Unknown
    );
    assert!(!ledger.relay_observation(second.id, 1, false, at(753)));
    ledger.finished(first.id);
    assert!(!ledger.relay_observation(first.id, 1, false, at(754)));
}
#[test]
fn overlapping_positive_after_negative_is_unknown_without_a_refresh_allowance() {
    let (mut ledger, first) = frontier();
    let second = ledger
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == first.target)
        .unwrap();
    assert!(ledger.relay_observation(first.id, 1, false, at(751)));
    assert!(ledger.relay_observation(second.id, 1, true, at(752)));
    assert_eq!(
        ledger.relay_state("relay", &first.target),
        RelayState::Unknown
    );
    assert!(!ledger.relay_entry(&first.target).unwrap().invalidated);
    assert!(!ledger.relay_entry(&first.target).unwrap().refresh_used);
    assert!(!ledger.pending[&second.id].refresh);
    ledger.deadline = at(753);
    assert!(!ledger.discovery_open(at(753)));
}
#[test]
fn quiescent_discovery_recovers_only_untried_routes_until_finalization() {
    let mut ledger = scheduler(4);
    let initial = ledger.schedule(at(0));
    for a in initial {
        ledger.accept(a.id, false, at(1)).unwrap();
    }
    assert!(ledger.exhausted(at(2)) && ledger.discovery_open(at(2)));
    let lost = ledger.recover_subtree("p002", at(3)).unwrap();
    assert_eq!(lost.len(), 1);
    let replacement = ledger
        .schedule(at(3))
        .into_iter()
        .find(|a| a.target == "p002")
        .unwrap();
    assert_ne!(
        replacement.source, "p000",
        "the previous successful direct pair is already tried"
    );
    ledger.accept(replacement.id, false, at(4)).unwrap();
    assert!(ledger.schedule(at(5)).is_empty());
    ledger.close_discovery();
    assert!(ledger.recover_subtree("p002", at(6)).is_none());
    assert!(!ledger.target_present("relay", "p002", at(6)));
}
#[test]
fn closed_or_expired_discovery_never_reopens_and_admitted_nodes_survive() {
    for cap in [false, true] {
        let (mut ledger, first) = frontier();
        ledger.relay_observation(first.id, 1, false, at(2));
        if cap {
            ledger.deadline = at(10);
        } else {
            ledger.close_discovery();
        }
        assert!(!ledger.target_present("relay", &first.target, at(10)));
        assert!(ledger.schedule(at(10)).is_empty());
        assert_eq!(
            ledger.relay_state("relay", &first.target),
            RelayState::Unknown
        );
        assert!(ledger.accept(first.id, true, at(10)).is_none());
        assert_eq!(ledger.reached["p001"].parent.as_deref(), Some("p000"));
    }
    let mut exhausted = scheduler(2);
    let first = exhausted.schedule(at(0)).pop().unwrap();
    exhausted.relay_observation(first.id, 1, false, at(1));
    exhausted.finished(first.id);
    assert!(exhausted.target_present("relay", &first.target, at(2)));
    let retry = exhausted.schedule(at(2));
    assert_eq!(retry.len(), 1);
    assert!(retry[0].refresh && retry[0].routes.relay && !retry[0].routes.direct);
    exhausted.finished(retry[0].id);
    exhausted.target_present("relay", &first.target, at(3));
    assert!(exhausted.schedule(at(3)).is_empty());
    assert_eq!(
        exhausted.relay_state("relay", &first.target),
        RelayState::Unknown
    );
}
#[test]
fn a_new_job_has_no_relay_outcomes_assignments_or_tried_routes() {
    let mut old = scheduler(3);
    let first = old.schedule(at(0)).pop().unwrap();
    old.relay_observation(first.id, 1, false, at(1));
    let mut fresh = scheduler(3);
    assert!(fresh.pending.is_empty() && fresh.tried.is_empty() && fresh.relay_outcomes.is_empty());
    assert!(
        fresh
            .schedule(at(0))
            .iter()
            .all(|a| a.routes.direct && a.routes.relay && !a.refresh)
    );
}
#[test]
fn full_256_node_direct_chain_survives_shared_relay_absence_past_16_edges() {
    let mut ledger = scheduler(256);
    let mut direct = BTreeSet::new();
    let mut relay_checks = 0;
    let mut time = 0;
    loop {
        let assignments = ledger.schedule(at(time));
        assert!(ledger.pending.len() <= CONTACTS);
        if assignments.is_empty() {
            break;
        }
        for a in assignments {
            if !ledger.pending.contains_key(&a.id) {
                continue;
            }
            assert!(a.routes.direct);
            assert!(direct.insert((a.source.clone(), a.target.clone())));
            if a.routes.relay {
                relay_checks += 1;
                ledger.relay_observation(a.id, 1, false, at(time));
            }
            let source: usize = a.source[1..].parse().unwrap();
            let target: usize = a.target[1..].parse().unwrap();
            if target == source + 1 {
                ledger.accept(a.id, false, at(time)).unwrap();
            } else {
                ledger.finished(a.id);
            }
        }
        time += 1;
    }
    assert!(ledger.missing().next().is_none());
    assert_eq!(ledger.reached["p255"].level, 255);
    assert_eq!(relay_checks, 255);
    assert!(direct.len() <= 256 * 255);
    assert!(ledger.pending.is_empty());
}

#[cfg(test)]
mod hint_priority_tests {
    use super::*;
    #[test]
    fn a_hinted_source_precedes_the_root_for_a_registration_refresh() {
        let mut scheduler = Scheduler::new(
            "a".into(),
            vec!["a".into(), "b".into(), "c".into()],
            "relay".into(),
        )
        .unwrap();
        let issued = scheduler.schedule(Duration::ZERO);
        let source = issued.iter().find(|a| a.target == "b").unwrap();
        scheduler.accept(source.id, false, Duration::ZERO).unwrap();
        let target = issued.iter().find(|a| a.target == "c").unwrap();
        scheduler.relay_observation(target.id, 1, false, Duration::ZERO);
        scheduler.finished(target.id);
        scheduler.hint("b", "c");
        assert!(scheduler.target_present("relay", "c", Duration::ZERO));
        let refresh = scheduler
            .schedule(Duration::ZERO)
            .into_iter()
            .find(|a| a.target == "c")
            .unwrap();
        assert_eq!(refresh.source, "b");
        assert!(refresh.refresh && refresh.routes.direct && refresh.routes.relay);
    }
}

#[test]
fn twenty_sources_exhaust_gone_target_direct_routes_in_three_batches_without_stagger() {
    let cohort: Vec<_> = (0..=20).map(|n| format!("{n:02}")).collect();
    let mut scheduler = Scheduler::new("00".into(), cohort.clone(), "relay".into()).unwrap();
    for source in &cohort[1..20] {
        scheduler.reached.insert(
            source.clone(),
            Node {
                parent: Some("00".into()),
                level: 1,
            },
        );
    }
    let mut now = Duration::ZERO;
    let first = scheduler.schedule(now);
    assert_eq!(first.len(), 1);
    scheduler.relay_observation(first[0].id, 1, false, now);
    let more = scheduler.schedule(now);
    assert_eq!(
        more.len(),
        7,
        "correlated Gone removes per-target staggering"
    );
    assert!(more.iter().all(|a| a.routes.direct && !a.routes.relay));
    let mut pairs = BTreeSet::new();
    let mut count = 0;
    while !scheduler.pending.is_empty() {
        assert!(scheduler.pending.len() <= CONTACTS);
        now += Duration::from_secs(3);
        let settled: Vec<_> = scheduler.pending.values().cloned().collect();
        for assignment in settled {
            assert!(pairs.insert((assignment.source, assignment.target)));
            scheduler.finished(assignment.id);
            count += 1;
        }
        scheduler.schedule(now);
    }
    assert_eq!(count, 20);
    assert_eq!(now, Duration::from_secs(9));
    assert!(scheduler.absence_complete("20"));
    assert!(scheduler.discovery_open(now));
}

#[test]
fn cancelled_losing_direct_contact_never_becomes_an_absence_proof_after_recovery() {
    let (mut scheduler, first) = frontier();
    let target = first.target.clone();
    let other = scheduler
        .schedule(at(750))
        .into_iter()
        .find(|a| a.target == target)
        .unwrap();
    let losers = scheduler.accept(other.id, false, at(751)).unwrap();
    assert!(losers.contains(&first.id));
    assert!(scheduler.unresolved_direct.contains(&target));
    assert!(
        !scheduler.failed_direct(&target),
        "cancelled work is unknown, not an observed failure"
    );
}
