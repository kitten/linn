//! Initiator-owned discovery ledger; transport and publication remain independent.
//! @ref LLP 0008#manual-mesh-jobs
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Duration,
};

pub(crate) const DISCOVERY_CAP: Duration = Duration::from_secs(20);
const CONTACTS: usize = 8;
const PER_TARGET: usize = 2;
const STAGGER: Duration = Duration::from_millis(750);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Routes {
    pub direct: bool,
    pub relay: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub id: u64,
    pub source: String,
    pub target: String,
    pub level: u16,
    pub routes: Routes,
    pub refresh: bool,
    pub issued: Duration,
    pub observer_generation: Option<u64>,
    epoch: u64,
    relay_observed: bool,
    route_failed: bool,
    presence_observed: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayState {
    #[default]
    Unknown,
    Gone,
    Reached,
}
#[derive(Clone, Debug, Default)]
struct RelayEntry {
    state: RelayState,
    epoch: u64,
    invalidated: bool,
    refresh_used: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub parent: Option<String>,
    pub level: u16,
}
/// One explicit job's state. Construct a new ledger for every job.
#[derive(Clone, Debug)]
pub struct Scheduler {
    relay: String,
    cohort: Vec<String>,
    root: String,
    reached: BTreeMap<String, Node>,
    relay_outcomes: BTreeMap<(String, String), RelayEntry>,
    tried: BTreeMap<(String, String), Routes>,
    settled: BTreeSet<String>,
    unresolved_direct: BTreeSet<String>,
    failed_direct: BTreeSet<String>,
    no_direct: BTreeSet<(String, String)>,
    pending: BTreeMap<u64, Assignment>,
    hints: BTreeSet<(String, String)>,
    registered: BTreeMap<String, u64>,
    recovering: BTreeSet<String>,
    expansion: VecDeque<String>,
    queued_expansion: BTreeSet<String>,
    expanded: BTreeSet<String>,
    expansion_turn: bool,
    next_id: u64,
    cursor: usize,
    source_after: String,
    free_transfers: BTreeMap<String, bool>,
    open: bool,
    pub(super) deadline: Duration,
}
impl Scheduler {
    pub fn new(root: String, cohort: Vec<String>, relay: String) -> Option<Self> {
        if cohort.is_empty()
            || cohort.len() > 256
            || !cohort.contains(&root)
            || cohort.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return None;
        }
        let reached = BTreeMap::from([(
            root.clone(),
            Node {
                parent: None,
                level: 0,
            },
        )]);
        Some(Self {
            relay,
            cohort,
            root,
            reached,
            relay_outcomes: BTreeMap::new(),
            tried: BTreeMap::new(),
            settled: BTreeSet::new(),
            unresolved_direct: BTreeSet::new(),
            failed_direct: BTreeSet::new(),
            no_direct: BTreeSet::new(),
            pending: BTreeMap::new(),
            hints: BTreeSet::new(),
            registered: BTreeMap::new(),
            recovering: BTreeSet::new(),
            expansion: VecDeque::new(),
            queued_expansion: BTreeSet::new(),
            expanded: BTreeSet::new(),
            expansion_turn: true,
            next_id: 0,
            cursor: 0,
            source_after: String::new(),
            free_transfers: BTreeMap::new(),
            open: true,
            deadline: DISCOVERY_CAP,
        })
    }
    pub fn discovery_open(&self, now: Duration) -> bool {
        self.open && now < self.deadline
    }
    pub fn open_until(&self) -> Option<Duration> {
        self.open.then_some(self.deadline)
    }
    pub fn initial_complete(&self, now: Duration) -> bool {
        !self.discovery_open(now)
            || (!self
                .pending
                .values()
                .any(|assignment| assignment.source == self.root && !assignment.refresh)
                && self.cohort.iter().all(|target| {
                    self.reached.contains_key(target)
                        || self
                            .tried
                            .contains_key(&(self.root.clone(), target.clone()))
                }))
    }
    pub fn close_discovery(&mut self) -> Vec<Assignment> {
        self.open = false;
        self.registered.clear();
        for assignment in self.pending.values() {
            self.unresolved_direct.insert(assignment.target.clone());
        }
        std::mem::take(&mut self.pending).into_values().collect()
    }
    /// Fence an actually failed subtree without resetting any tried routes or
    /// deadlines. Ordinary exhaustion is quiescence, not window closure.
    pub fn recover_subtree(
        &mut self,
        failed: &str,
        now: Duration,
    ) -> Option<BTreeMap<String, Node>> {
        if !self.discovery_open(now) || failed == self.root || !self.reached.contains_key(failed) {
            return None;
        }
        let mut affected = BTreeSet::from([failed.to_owned()]);
        loop {
            let old = affected.len();
            for (id, node) in &self.reached {
                if node
                    .parent
                    .as_ref()
                    .is_some_and(|parent| affected.contains(parent))
                {
                    affected.insert(id.clone());
                }
            }
            if affected.len() == old {
                break;
            }
        }
        // Recovery starts after outstanding discovery has settled. Never drop
        // assignment ownership behind an executor's back.
        if self
            .pending
            .values()
            .any(|a| affected.contains(&a.source) || affected.contains(&a.target))
        {
            return None;
        }
        let mut removed = BTreeMap::new();
        for id in affected {
            let node = self.reached.remove(&id).unwrap();
            self.registered.remove(&id);
            self.recovering.insert(id.clone());
            self.prioritize(id.clone());
            removed.insert(id, node);
        }
        Some(removed)
    }
    pub fn reached(&self) -> &BTreeMap<String, Node> {
        &self.reached
    }
    pub fn pending(&self) -> &BTreeMap<u64, Assignment> {
        &self.pending
    }
    pub fn relay_state(&self, relay: &str, target: &str) -> RelayState {
        self.relay_outcomes
            .get(&(relay.into(), target.into()))
            .map_or(RelayState::Unknown, |e| e.state)
    }
    pub fn unresolved_direct(&mut self, target: &str) {
        self.unresolved_direct.insert(target.into());
    }
    pub fn no_direct_route(&mut self, source: &str, target: &str) {
        self.no_direct.insert((source.into(), target.into()));
    }
    pub fn direct_failed(&mut self, target: &str) {
        self.failed_direct.insert(target.into());
        self.unresolved_direct(target);
    }
    pub fn failed_direct(&self, target: &str) -> bool {
        self.failed_direct.contains(target)
    }
    pub fn absence_complete(&self, target: &str) -> bool {
        !self.reached.contains_key(target)
            && self.relay_state(&self.relay, target) == RelayState::Gone
            && !self.unresolved_direct.contains(target)
            && !self
                .pending
                .values()
                .any(|assignment| assignment.target == target)
            && self.reached.keys().all(|source| {
                source == target
                    || self
                        .tried
                        .get(&(source.clone(), target.into()))
                        .is_some_and(|routes| routes.direct)
            })
    }
    pub fn relay(&self) -> &str {
        &self.relay
    }
    pub fn watch_targets(&self, now: Duration) -> Vec<String> {
        if !self.discovery_open(now) {
            return Vec::new();
        }
        self.missing()
            .filter(|target| {
                self.settled.contains(*target)
                    && !self.relay_entry(target).is_some_and(|e| e.refresh_used)
            })
            .map(str::to_owned)
            .collect()
    }
    pub fn registration_unknown(&mut self, target: &str) {
        if let Some(entry) = self
            .relay_outcomes
            .get_mut(&(self.relay.clone(), target.into()))
        {
            entry.epoch += 1;
            entry.state = RelayState::Unknown;
            entry.invalidated = false;
        }
    }
    pub fn snapshot(&self, now: Duration) -> super::status::Traversal {
        super::status::Traversal {
            round: 1,
            level: self.pending.values().map(|a| a.level).max().unwrap_or(1),
            direct_complete: self.initial_complete(now),
            discovery_expired: now >= self.deadline,
            reached: self.reached.len(),
            no_known_direct_routes: self.no_direct.iter().cloned().collect(),
            pending: self
                .pending
                .values()
                .map(|a| super::status::Contact {
                    id: a.id,
                    source: a.source.clone(),
                    target: a.target.clone(),
                    level: a.level,
                    routes: a.routes,
                    refresh: a.refresh,
                })
                .collect(),
            relay: self
                .relay_outcomes
                .iter()
                .map(|((relay, target), entry)| super::status::Relay {
                    relay: relay.clone(),
                    target: target.clone(),
                    outcome: entry.state,
                    refresh_used: entry.refresh_used,
                })
                .collect(),
        }
    }
    pub fn missing(&self) -> impl Iterator<Item = &str> {
        self.cohort
            .iter()
            .filter(|id| !self.reached.contains_key(*id))
            .map(String::as_str)
    }
    pub fn hint(&mut self, source: &str, target: &str) {
        if self.reached.contains_key(source)
            && self
                .cohort
                .binary_search_by(|p| p.as_str().cmp(target))
                .is_ok()
        {
            self.hints.insert((source.into(), target.into()));
        }
    }
    fn relay_entry(&self, target: &str) -> Option<&RelayEntry> {
        self.relay_outcomes
            .get(&(self.relay.clone(), target.into()))
    }
    fn routes(&self, source: &str, target: &str) -> (Routes, bool) {
        let tried = self
            .tried
            .get(&(source.into(), target.into()))
            .copied()
            .unwrap_or_default();
        let entry = self.relay_entry(target);
        let refresh = entry.is_some_and(|entry| entry.invalidated && !entry.refresh_used);
        let relay = entry.is_none_or(|entry| {
            (entry.state == RelayState::Unknown
                || (self.recovering.contains(target) && entry.state == RelayState::Reached))
                && (!entry.invalidated || !entry.refresh_used)
        }) && (!tried.relay || refresh);
        (
            Routes {
                direct: !tried.direct,
                relay,
            },
            refresh && relay,
        )
    }
    pub fn capacity(&mut self, values: BTreeMap<String, bool>) {
        self.free_transfers = values
            .into_iter()
            .filter(|(source, _)| self.reached.contains_key(source))
            .collect();
    }
    fn candidate(&self, target: &str, now: Duration) -> Option<(String, Routes, bool)> {
        if self.registered.contains_key(target) {
            return None;
        }
        let pending: Vec<_> = self
            .pending
            .values()
            .filter(|a| a.target == target)
            .collect();
        let gone = self
            .relay_entry(target)
            .is_some_and(|entry| entry.state == RelayState::Gone);
        if pending.len() >= if gone { CONTACTS } else { PER_TARGET }
            || (!gone
                && pending
                    .first()
                    .is_some_and(|a| !a.route_failed && now.saturating_sub(a.issued) < STAGGER))
        {
            return None;
        }
        let mut sources: Vec<_> = self
            .reached
            .keys()
            .filter(|source| {
                source.as_str() != target && !pending.iter().any(|a| &a.source == *source)
            })
            .collect();
        sources.sort_by_cached_key(|source| {
            let free = self.free_transfers.get(*source).copied().unwrap_or(false);
            (
                !free,
                !self.hints.contains(&((*source).clone(), target.into())),
                if free { self.reached[*source].level } else { 0 },
                source.as_str() <= self.source_after.as_str(),
                (*source).clone(),
            )
        });
        sources.into_iter().find_map(|source| {
            let (routes, refresh) = self.routes(source, target);
            (routes.direct || routes.relay).then(|| (source.clone(), routes, refresh))
        })
    }
    fn prioritize(&mut self, target: String) {
        if self.queued_expansion.insert(target.clone()) {
            let position = if !self.expanded.contains(&target) {
                self.expansion
                    .iter()
                    .position(|other| self.expanded.contains(other))
            } else {
                None
            };
            if let Some(position) = position {
                self.expansion.insert(position, target);
            } else {
                self.expansion.push_back(target);
            }
        }
    }
    fn expansion_candidate(&mut self, now: Duration) -> Option<(String, String, Routes, bool)> {
        let count = self.expansion.len();
        for _ in 0..count {
            let target = self.expansion.pop_front()?;
            self.queued_expansion.remove(&target);
            if !self.reached.contains_key(&target)
                && let Some((source, routes, refresh)) = self.candidate(&target, now)
            {
                return Some((target, source, routes, refresh));
            }
        }
        None
    }

    /// The only scheduler timer is the next eligible stagger or discovery cap.
    /// Idle work never causes a periodic discovery/relay poll.
    pub fn next_wakeup(&self, now: Duration) -> Option<Duration> {
        if !self.discovery_open(now) {
            return None;
        }
        let mut wake = Some(self.deadline);
        if self.pending.len() < CONTACTS {
            for assignment in self.pending.values() {
                let at = assignment.issued + STAGGER;
                if at > now
                    && !assignment.route_failed
                    && self.candidate(&assignment.target, at).is_some()
                {
                    wake = Some(wake.map_or(at, |old| old.min(at)));
                }
            }
        }
        wake
    }

    /// Issue at most eight owned contacts, fairly across still-missing targets.
    /// Queueing and transfer completion never change an issued attempt's clock.
    pub fn schedule(&mut self, now: Duration) -> Vec<Assignment> {
        if !self.discovery_open(now) {
            self.open = false;
            return vec![];
        }
        let ready: Vec<_> = self
            .pending
            .values()
            .filter(|a| self.candidate(&a.target, now).is_some())
            .map(|a| a.target.clone())
            .collect();
        for target in ready {
            self.prioritize(target);
        }
        let mut issued = Vec::new();
        let mut idle = 0;
        while self.pending.len() < CONTACTS && idle < self.cohort.len() {
            let urgent = self
                .expansion
                .iter()
                .any(|target| !self.expanded.contains(target));
            let preferred = if urgent || self.expansion_turn {
                self.expansion_candidate(now)
            } else {
                None
            };
            let (target, source, routes, refresh) = if let Some(assignment) = preferred {
                assignment
            } else {
                let target = self.cohort[self.cursor].clone();
                self.cursor = (self.cursor + 1) % self.cohort.len();
                if self.reached.contains_key(&target) {
                    idle += 1;
                    continue;
                }
                let Some((source, routes, refresh)) = self.candidate(&target, now) else {
                    idle += 1;
                    continue;
                };
                (target, source, routes, refresh)
            };
            if source != self.root {
                self.expanded.insert(target.clone());
            }
            self.expansion_turn = !self.expansion_turn;
            idle = 0;
            let level = self.reached[&source].level + 1;
            let key = (self.relay.clone(), target.clone());
            let entry = self.relay_outcomes.entry(key).or_default();
            if refresh {
                entry.refresh_used = true;
            }
            self.next_id += 1;
            self.source_after = source.clone();
            let assignment = Assignment {
                id: self.next_id,
                source: source.clone(),
                target: target.clone(),
                level,
                routes,
                refresh,
                issued: now,
                observer_generation: None,
                epoch: entry.epoch,
                relay_observed: false,
                route_failed: false,
                presence_observed: false,
            };
            let tried = self.tried.entry((source, target)).or_default();
            tried.direct |= routes.direct;
            tried.relay |= routes.relay;
            self.pending.insert(assignment.id, assignment.clone());
            issued.push(assignment);
        }
        issued
    }
    /// Consume only an adapter-validated, current-attempt correlated reply.
    /// Generation/token checks happen at the source adapter, not in this map.
    pub fn relay_observation(
        &mut self,
        id: u64,
        generation: u64,
        registered: bool,
        now: Duration,
    ) -> bool {
        let open = self.discovery_open(now);
        if generation == 0 || (!open && !registered) {
            return false;
        }
        let Some(assignment) = self.pending.get_mut(&id) else {
            return false;
        };
        if !assignment.routes.relay || assignment.relay_observed {
            return false;
        }
        assignment.relay_observed = true;
        assignment.observer_generation = Some(generation);
        let entry = self
            .relay_outcomes
            .entry((self.relay.clone(), assignment.target.clone()))
            .or_default();
        if assignment.epoch != entry.epoch {
            // Concurrent observers cannot order their relay snapshots. A late
            // positive makes a conflicting negative doubtful, not proof of a
            // new registration and not permission to retry a tried route.
            if registered && entry.state == RelayState::Gone {
                entry.state = RelayState::Unknown;
                entry.epoch += 1;
                return true;
            }
            return false;
        }
        entry.epoch += 1;
        if registered {
            entry.state = RelayState::Unknown;
        } else {
            entry.state = RelayState::Gone;
            assignment.route_failed = true;
        }
        true
    }
    /// A new authenticated transport connection is a target event, not an
    /// observer reconnect. Require an attempt issued after the negative so
    /// reordered observations from overlapping attempts cannot grant a retry.
    pub fn target_connected(&mut self, id: u64, now: Duration) -> bool {
        let Some(assignment) = self.pending.get_mut(&id) else {
            return false;
        };
        if assignment.presence_observed {
            return false;
        }
        assignment.presence_observed = true;
        let target = assignment.target.clone();
        let epoch = assignment.epoch;
        let Some(entry) = self
            .relay_outcomes
            .get_mut(&(self.relay.clone(), target.clone()))
        else {
            return false;
        };
        if entry.state != RelayState::Gone {
            return false;
        }
        if epoch != entry.epoch {
            entry.epoch += 1;
            entry.state = RelayState::Unknown;
            return false;
        }
        self.target_present(&self.relay.clone(), &target, now)
    }
    /// An independent fresh verified target-presence event can invalidate old
    /// conclusive evidence. It is never called for an observer-only reconnect.
    pub fn target_present(&mut self, relay: &str, target: &str, now: Duration) -> bool {
        if relay != self.relay || !self.cohort.iter().any(|p| p == target) {
            return false;
        }
        let entry = self
            .relay_outcomes
            .entry((relay.into(), target.into()))
            .or_default();
        entry.epoch += 1;
        entry.state = RelayState::Unknown;
        entry.invalidated = true;
        let retry = !entry.refresh_used;
        self.discovery_open(now) && retry
    }
    /// Finish an unresolved/failed contact. Absence is supplied only separately
    /// by a correlated relay observation; a timeout cannot write a negative.
    pub fn finished(&mut self, id: u64) -> Option<Assignment> {
        let result = self.pending.remove(&id);
        if let Some(assignment) = &result {
            self.settled.insert(assignment.target.clone());
            self.prioritize(assignment.target.clone());
        }
        self.registered.retain(|_, winner| *winner != id);
        result
    }
    /// A duplicate must name the already registered parent. Keep that parent's
    /// original attempt alive, without probing further parents or resetting it.
    pub fn registered_elsewhere(&mut self, id: u64, parent: &str) -> Option<u64> {
        let duplicate = self.pending.get(&id)?;
        let winner = self
            .pending
            .values()
            .find(|a| a.target == duplicate.target && a.source == parent && a.id != id)?
            .id;
        let target = duplicate.target.clone();
        self.pending.remove(&id);
        self.registered.insert(target, winner);
        Some(winner)
    }

    /// First valid acceptance wins atomically. Losing callbacks cannot replace
    /// its parent or cancel its connection. Returned IDs identify losers only.
    pub fn accept(&mut self, id: u64, via_relay: bool, now: Duration) -> Option<Vec<u64>> {
        if !self.discovery_open(now) {
            return None;
        }
        let assignment = self.pending.remove(&id)?;
        if self.reached.contains_key(&assignment.target) || (via_relay && !assignment.routes.relay)
        {
            return None;
        }
        self.registered.remove(&assignment.target);
        self.recovering.remove(&assignment.target);
        self.reached.insert(
            assignment.target.clone(),
            Node {
                parent: Some(assignment.source),
                level: assignment.level,
            },
        );
        if via_relay {
            let entry = self
                .relay_outcomes
                .entry((self.relay.clone(), assignment.target.clone()))
                .or_default();
            entry.state = RelayState::Reached;
            entry.epoch += 1;
        }
        let losers: Vec<_> = self
            .pending
            .values()
            .filter(|a| a.target == assignment.target)
            .map(|a| a.id)
            .collect();
        for loser in &losers {
            if self
                .pending
                .remove(loser)
                .is_some_and(|loser| loser.routes.direct)
            {
                // If this admitted target later needs recovery, a cancelled loser
                // is not a settled direct timeout and cannot certify its absence.
                self.unresolved_direct.insert(assignment.target.clone());
            }
        }
        let candidates: Vec<_> = self
            .missing()
            .filter(|target| {
                self.tried
                    .contains_key(&(self.root.clone(), (*target).into()))
            })
            .map(str::to_owned)
            .collect();
        for target in candidates {
            self.prioritize(target);
        }
        Some(losers)
    }
    pub fn exhausted(&self, now: Duration) -> bool {
        self.pending.is_empty()
            && self
                .missing()
                .all(|target| self.candidate(target, now).is_none())
    }
}

#[cfg(test)]
mod tests {
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
        assert!(
            fresh.pending.is_empty() && fresh.tried.is_empty() && fresh.relay_outcomes.is_empty()
        );
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
}
