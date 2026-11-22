//! Live transfer capacity is scheduling advice, never reachability or authority.
//! @ref LLP 0008#assignment-ownership-and-amplification-bounds
use super::protocol;
use crate::error::Result;
use std::{collections::BTreeMap, sync::Mutex};
use tokio::sync::{Semaphore, SemaphorePermit, watch};

pub(super) struct Transfers {
    slots: Semaphore,
    readiness: watch::Sender<bool>,
}
type PendingSlot<'a> = std::pin::Pin<
    Box<
        dyn std::future::Future<
                Output = std::result::Result<SemaphorePermit<'a>, tokio::sync::AcquireError>,
            > + Send
            + 'a,
    >,
>;
// Pending cancellation must return any assigned semaphore permit before publishing.
struct Acquisition<'a> {
    pending: Option<PendingSlot<'a>>,
    owner: &'a Transfers,
}
impl Drop for Acquisition<'_> {
    fn drop(&mut self) {
        drop(self.pending.take());
        self.owner.notify();
    }
}
pub(super) struct Permit<'a> {
    slot: Option<SemaphorePermit<'a>>,
    owner: &'a Transfers,
}
impl Transfers {
    pub(super) fn new() -> Self {
        Self {
            slots: Semaphore::new(2),
            readiness: watch::channel(true).0,
        }
    }
    pub(super) async fn acquire(
        &self,
    ) -> std::result::Result<Permit<'_>, tokio::sync::AcquireError> {
        let mut acquisition = Acquisition {
            pending: Some(Box::pin(self.slots.acquire())),
            owner: self,
        };
        let slot = acquisition.pending.as_mut().unwrap().await?;
        drop(acquisition);
        Ok(Permit {
            slot: Some(slot),
            owner: self,
        })
    }
    #[cfg(test)]
    pub(super) fn available(&self) -> u8 {
        self.slots.available_permits() as u8
    }
    fn notify(&self) {
        self.readiness.send_if_modified(|previous| {
            let ready = self.slots.available_permits() > 0;
            let changed = *previous != ready;
            *previous = ready;
            changed
        });
    }
    pub(super) fn has_free(&self) -> bool {
        *self.readiness.borrow()
    }
    pub(super) fn subscribe(&self) -> watch::Receiver<bool> {
        self.readiness.subscribe()
    }
    pub(super) async fn changed_after(&self, readiness: bool) {
        let _ = self
            .subscribe()
            .wait_for(|current| *current != readiness)
            .await;
    }
}
impl Drop for Permit<'_> {
    fn drop(&mut self) {
        drop(self.slot.take());
        self.owner.notify();
    }
}

#[derive(Default)]
pub(super) struct Sources {
    values: Mutex<BTreeMap<String, bool>>,
    revision: watch::Sender<u64>,
}
impl Sources {
    pub(super) fn record(&self, source: &str, free: bool) {
        let mut values = self.values.lock().unwrap();
        if values.get(source) != Some(&free) {
            values.insert(source.into(), free);
            self.revision.send_modify(|revision| *revision += 1);
        }
    }
    pub(super) fn accept(&self, values: Vec<(String, bool)>, cohort: &[String]) -> Result<()> {
        if values.len() > cohort.len()
            || values
                .iter()
                .any(|(source, _)| cohort.binary_search(source).is_err())
        {
            return Err(protocol(
                "source capacity is outside the frozen cohort or transfer limit",
            ));
        }
        for (source, free) in values {
            self.record(&source, free);
        }
        Ok(())
    }
    pub(super) fn forget(&self, source: &str) {
        if self.values.lock().unwrap().remove(source).is_some() {
            self.revision.send_modify(|revision| *revision += 1);
        }
    }
    pub(super) fn snapshot(&self) -> (u64, BTreeMap<String, bool>) {
        let values = self.values.lock().unwrap();
        (*self.revision.borrow(), values.clone())
    }
    pub(super) fn subscribe(&self) -> watch::Receiver<u64> {
        self.revision.subscribe()
    }
    pub(super) async fn changed_after(&self, revision: u64) {
        let _ = self
            .subscribe()
            .wait_for(|current| *current != revision)
            .await;
    }
    pub(super) fn delta(&self, sent: &mut BTreeMap<String, bool>) -> Vec<(String, bool)> {
        let (_, values) = self.snapshot();
        let delta = values
            .iter()
            .filter(|(source, free)| sent.get(*source) != Some(*free))
            .map(|(source, free)| (source.clone(), *free))
            .collect();
        *sent = values;
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn live_slot_updates_cannot_be_lost_between_snapshot_and_wait() {
        let slots = Transfers::new();
        let before = slots.has_free();
        let first = slots.acquire().await.unwrap();
        let second = slots.acquire().await.unwrap();
        assert_eq!(slots.available(), 0);
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            slots.changed_after(before),
        )
        .await
        .unwrap();
        let before = slots.has_free();
        drop(first);
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            slots.changed_after(before),
        )
        .await
        .unwrap();
        assert_eq!(slots.available(), 1);
        drop(second);
        assert_eq!(slots.available(), 2);
    }
    #[tokio::test]
    async fn job_capacity_deltas_are_bounded_coalesced_and_not_reused_by_new_jobs() {
        let sources = Sources::default();
        let cohort = vec!["a".into(), "b".into()];
        assert!(sources.accept(vec![("x".into(), true)], &cohort).is_err());
        assert!(
            sources
                .accept(vec![("a".into(), true); 3], &cohort)
                .is_err()
        );
        assert!(sources.snapshot().1.is_empty());
        let (before, _) = sources.snapshot();
        sources
            .accept(vec![("a".into(), true), ("b".into(), true)], &cohort)
            .unwrap();
        sources.record("a", false);
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            sources.changed_after(before),
        )
        .await
        .unwrap();
        let mut sent = BTreeMap::new();
        assert_eq!(
            sources.delta(&mut sent),
            vec![("a".into(), false), ("b".into(), true)]
        );
        assert!(sources.delta(&mut sent).is_empty());
        let (before, _) = sources.snapshot();
        sources.record("a", false);
        assert_eq!(
            sources.snapshot().0,
            before,
            "duplicate advertisements do not generate wakeups"
        );
        sources.forget("a");
        assert!(!sources.snapshot().1.contains_key("a"));
        assert!(Sources::default().snapshot().1.is_empty());
    }
}

#[cfg(test)]
mod cancellation_regression {
    use super::*;
    use futures_util::FutureExt;
    #[tokio::test]
    async fn cancelling_an_assigned_unpolled_waiter_restores_readiness() {
        let slots = Transfers::new();
        let first = slots.acquire().await.unwrap();
        let second = slots.acquire().await.unwrap();
        let mut waiting = Box::pin(slots.acquire());
        assert!(waiting.as_mut().now_or_never().is_none());
        drop(first);
        assert!(!slots.has_free());
        drop(waiting);
        assert_eq!(slots.available(), 1);
        assert!(
            slots.has_free(),
            "cancelled assigned permit must advertise free capacity"
        );
        drop(second);
    }
}
