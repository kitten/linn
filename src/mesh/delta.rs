//! Bounded, job-local metadata for exact final redistribution.
//! @ref LLP 0008#manual-mesh-jobs
use crate::storage::sync::{Entry, PAGE};
use std::{collections::BTreeMap, sync::Mutex};
use tokio::sync::broadcast;

pub(super) const ITEMS: usize = 4096;
pub(super) const BYTES: usize = 1024 * 1024;
const EVENTS: usize = 256;
pub(super) const EVENT_BYTES: usize = BYTES / EVENTS;
const CHUNK: usize = 256 * 1024;

#[derive(Clone)]
struct Change {
    revision: u64,
    run: Option<String>,
    entry: Option<Entry>,
}
pub(crate) struct Changes(Mutex<(u64, broadcast::Sender<Change>)>);
impl Default for Changes {
    fn default() -> Self {
        Self(Mutex::new((0, broadcast::channel(EVENTS).0)))
    }
}
impl Changes {
    pub(crate) fn revision(&self) -> u64 {
        self.0.lock().unwrap().0
    }
    pub(crate) fn record(&self, run: Option<String>, entry: Option<Entry>) {
        let (run, entry) = if entry.as_ref().is_some_and(|entry| {
            entry.id.capacity() + entry.hash.as_ref().map_or(0, String::capacity) > EVENT_BYTES
        }) {
            (None, None)
        } else {
            (run, entry)
        };
        let mut state = self.0.lock().unwrap();
        state.0 += 1;
        let _ = state.1.send(Change {
            revision: state.0,
            run,
            entry,
        });
    }
}
pub(crate) struct Watch {
    run: String,
    receiver: broadcast::Receiver<Change>,
    revision: u64,
    valid: bool,
}
impl Watch {
    pub(crate) fn new(changes: &Changes, run: String) -> Self {
        let state = changes.0.lock().unwrap();
        Self {
            run,
            receiver: state.1.subscribe(),
            revision: state.0,
            valid: true,
        }
    }
    fn next(&mut self) -> Option<Change> {
        match self.receiver.try_recv() {
            Ok(change) => {
                if change.entry.is_none()
                    || change.revision != self.revision + 1
                    || change.run.as_ref() != Some(&self.run)
                {
                    self.valid = false;
                }
                self.revision = change.revision;
                Some(change)
            }
            Err(broadcast::error::TryRecvError::Empty) => None,
            Err(_) => {
                self.valid = false;
                None
            }
        }
    }
    pub(crate) fn valid(&mut self) -> bool {
        while self.next().is_some() {}
        self.valid
    }
}
pub(crate) struct Delta {
    watch: Watch,
    items: BTreeMap<String, (u64, Entry, usize)>,
    bytes: usize,
    baselines: BTreeMap<String, u64>,
}
impl Delta {
    pub(crate) fn new(changes: &Changes, run: String) -> Self {
        Self {
            watch: Watch::new(changes, run),
            items: BTreeMap::new(),
            bytes: 0,
            baselines: BTreeMap::new(),
        }
    }
    fn invalidate(&mut self) {
        self.watch.valid = false;
        self.items.clear();
        self.bytes = 0;
    }
    pub(crate) fn refresh(&mut self) {
        while let Some(change) = self.watch.next() {
            if !self.watch.valid {
                self.invalidate();
                continue;
            }
            let Some(entry) = change.entry else {
                self.invalidate();
                continue;
            };
            let bytes = entry.id.capacity()
                + entry.id.len()
                + entry.hash.as_ref().map_or(0, String::capacity);
            if let Some((_, _, previous)) = self.items.remove(&entry.id) {
                self.bytes -= previous;
            }
            self.bytes += bytes;
            self.items
                .insert(entry.id.clone(), (change.revision, entry, bytes));
            if self.items.len() > ITEMS || self.bytes > BYTES {
                self.invalidate();
            }
        }
        if !self.watch.valid {
            self.invalidate();
        }
    }
    pub(crate) fn start(&mut self) -> Option<u64> {
        self.refresh();
        self.watch.valid.then_some(self.watch.revision)
    }
    pub(crate) fn baseline(&mut self, peer: String, start: u64) -> bool {
        self.refresh();
        if self.watch.valid && self.watch.revision == start {
            self.baselines.insert(peer, start);
            true
        } else {
            self.baselines.remove(&peer);
            false
        }
    }
    pub(crate) fn eligible(&mut self, peer: &str) -> bool {
        self.refresh();
        self.watch.valid && self.baselines.contains_key(peer)
    }
    pub(crate) fn page(
        &mut self,
        peer: &str,
        tombstones: bool,
        after: Option<&str>,
    ) -> Option<(Vec<Entry>, bool)> {
        if !self.eligible(peer) {
            return None;
        }
        let baseline = self.baselines[peer];
        let mut entries = Vec::new();
        let mut bytes = 128; // Frame wrapper and final-page flag.
        for (id, (revision, entry, _)) in &self.items {
            if *revision <= baseline
                || entry.hash.is_none() != tombstones
                || after.is_some_and(|after| id.as_str() <= after)
            {
                continue;
            }
            // Two bounded size fields are appended from indexed metadata later.
            let size = serde_json::to_vec(entry).ok()?.len() + 64;
            if size > CHUNK {
                return None;
            }
            if entries.len() == PAGE || bytes + size > CHUNK {
                return Some((entries, false));
            }
            bytes += size;
            entries.push(entry.clone());
        }
        Some((entries, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(id: &str, tombstone: bool) -> Entry {
        Entry {
            id: id.into(),
            hash: (!tombstone).then(|| "hash".into()),
        }
    }
    fn baseline(delta: &mut Delta, peer: &str) {
        let start = delta.start().unwrap();
        assert!(delta.baseline(peer.into(), start));
    }
    #[test]
    fn baselines_are_per_peer_and_later_tombstones_replace_records() {
        let changes = Changes::default();
        let mut delta = Delta::new(&changes, "job".into());
        baseline(&mut delta, "b");
        changes.record(Some("job".into()), Some(entry("a", false)));
        baseline(&mut delta, "c");
        assert_eq!(delta.page("b", false, None).unwrap().0.len(), 1);
        assert!(delta.page("c", false, None).unwrap().0.is_empty());
        changes.record(Some("job".into()), Some(entry("a", true)));
        assert!(delta.page("b", false, None).unwrap().0.is_empty());
        assert_eq!(delta.page("c", true, None).unwrap().0[0].id, "a");
    }
    #[test]
    fn a_mutating_baseline_is_ineligible_but_an_earlier_stable_one_survives() {
        let changes = Changes::default();
        let mut delta = Delta::new(&changes, "job".into());
        baseline(&mut delta, "b");
        let start = delta.start().unwrap();
        changes.record(Some("job".into()), Some(entry("a", false)));
        assert!(!delta.baseline("c".into(), start));
        assert!(delta.eligible("b"));
        assert!(!delta.eligible("c"));
        changes.record(None, None);
        assert!(!delta.eligible("b"));
    }
    #[test]
    fn count_byte_and_event_overflow_disable_every_branch() {
        for bound in [0, 1, 2] {
            let changes = Changes::default();
            let mut delta = Delta::new(&changes, "job".into());
            baseline(&mut delta, "b");
            baseline(&mut delta, "c");
            if bound == 1 {
                for index in 0..ITEMS {
                    changes.record(
                        Some("job".into()),
                        Some(entry(&format!("{index:04}{}", "x".repeat(1024)), false)),
                    );
                    delta.refresh();
                    if !delta.watch.valid {
                        assert!(index + 1 < ITEMS);
                        break;
                    }
                }
            } else {
                for index in 0..if bound == 0 { ITEMS + 1 } else { EVENTS + 1 } {
                    changes.record(
                        Some("job".into()),
                        Some(entry(&format!("{index:04}"), false)),
                    );
                    if bound == 0 {
                        delta.refresh();
                    }
                }
            }
            assert!(!delta.eligible("b"));
            assert!(!delta.eligible("c"));
            assert!(delta.items.is_empty());
        }
    }
    #[test]
    fn receiver_watches_start_fresh_and_reject_untracked_or_other_job_writes() {
        let changes = Changes::default();
        changes.record(None, None);
        let mut watch = Watch::new(&changes, "job".into());
        changes.record(Some("job".into()), Some(entry("a", false)));
        assert!(watch.valid());
        changes.record(Some("another-job".into()), Some(entry("b", false)));
        assert!(!watch.valid());
        assert!(Watch::new(&changes, "job".into()).valid());
        changes.record(None, None);
        assert!(!watch.valid());
    }

    #[test]
    fn pages_are_chunked_and_restart_has_no_baselines() {
        let changes = Changes::default();
        let mut delta = Delta::new(&changes, "job".into());
        baseline(&mut delta, "b");
        for index in 0..PAGE + 1 {
            changes.record(
                Some("job".into()),
                Some(entry(&format!("{index:04}"), false)),
            );
            delta.refresh();
        }
        let (first, done) = delta.page("b", false, None).unwrap();
        assert_eq!(first.len(), PAGE);
        assert!(!done);
        let (last, done) = delta
            .page("b", false, Some(&first.last().unwrap().id))
            .unwrap();
        assert_eq!(last.len(), 1);
        assert!(done);
        assert!(!Delta::new(&changes, "job".into()).eligible("b"));
    }
}
