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
mod tests;
