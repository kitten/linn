//! Store-wide transport reservations from current verified membership.
//! @ref LLP 0008#connectivity-and-propagation
use std::sync::{Arc, Mutex as StateMutex};
use tokio::sync::{Mutex, MutexGuard, Notify};

pub(crate) const FLOOR: usize = 32;
pub(crate) const CEILING: usize = 1028;
pub(crate) fn capacity(members: usize) -> usize {
    assert!(
        (1..=256).contains(&members),
        "connection capacity requires verified membership of 1–256 peers"
    );
    (4 * (members - 1) + 8).clamp(FLOOR, CEILING)
}
pub(crate) struct Connections {
    state: StateMutex<State>,
    queue: Mutex<()>,
    changed: Notify,
    #[cfg(test)]
    peak: std::sync::atomic::AtomicUsize,
}
struct State {
    target: usize,
    used: usize,
}
pub(crate) struct Permit(Arc<Connections>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.state.lock().unwrap().used -= 1;
        self.0.wake();
    }
}
impl Connections {
    pub(crate) fn new(members: usize) -> Arc<Self> {
        Arc::new(Self {
            state: StateMutex::new(State {
                target: capacity(members),
                used: 0,
            }),
            queue: Mutex::new(()),
            changed: Notify::new(),
            #[cfg(test)]
            peak: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    #[cfg(test)]
    pub(crate) fn peak(&self) -> usize {
        self.peak.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(test)]
    pub(crate) fn reset_peak(&self) {
        let state = self.state.lock().unwrap();
        self.peak
            .store(state.used, std::sync::atomic::Ordering::Relaxed);
    }
    pub(crate) fn membership(&self, members: usize) {
        self.state.lock().unwrap().target = capacity(members);
        self.wake();
    }
    pub(crate) fn excess(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.used > state.target
    }
    pub(crate) fn wake(&self) {
        self.changed.notify_waiters();
    }
    pub(crate) fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.changed.notified()
    }
    pub(crate) async fn queue(&self) -> MutexGuard<'_, ()> {
        self.queue.lock().await
    }
    pub(crate) fn reserve(self: &Arc<Self>, _ticket: &MutexGuard<'_, ()>) -> Option<Permit> {
        let mut state = self.state.lock().unwrap();
        if state.used >= state.target {
            return None;
        }
        state.used += 1;
        #[cfg(test)]
        {
            self.peak
                .fetch_max(state.used, std::sync::atomic::Ordering::Relaxed);
        }
        Some(Permit(self.clone()))
    }
    pub(crate) fn try_acquire_owned(self: Arc<Self>) -> Result<Permit, ()> {
        let ticket = self.queue.try_lock().map_err(|_| ())?;
        self.reserve(&ticket).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn four_tree_budgets_and_headroom_have_a_floor_and_hard_ceiling() {
        for (n, want) in [(1, 32), (5, 32), (10, 44), (20, 84), (256, 1028)] {
            assert_eq!(capacity(n), want);
        }
    }
    #[test]
    fn invalid_membership_counts_fail_instead_of_hiding_authority_errors() {
        for members in [0, 257, usize::MAX] {
            assert!(std::panic::catch_unwind(|| capacity(members)).is_err());
        }
    }
    #[tokio::test]
    async fn growth_and_shrink_preserve_live_reservations_without_over_admission() {
        let limit = Connections::new(5);
        let mut held: Vec<_> = (0..32)
            .map(|_| limit.clone().try_acquire_owned().unwrap())
            .collect();
        assert!(limit.clone().try_acquire_owned().is_err());
        limit.membership(20);
        for _ in 32..84 {
            held.push(limit.clone().try_acquire_owned().unwrap());
        }
        assert!(limit.clone().try_acquire_owned().is_err());
        limit.membership(5);
        assert!(limit.excess());
        while held.len() > 32 {
            held.pop();
            assert!(limit.clone().try_acquire_owned().is_err());
        }
        assert!(!limit.excess());
        held.pop();
        let replacement = limit.clone().try_acquire_owned().unwrap();
        assert!(limit.clone().try_acquire_owned().is_err());
        drop(replacement);
        drop(held);
        assert_eq!(limit.state.lock().unwrap().used, 0);
    }
    #[tokio::test]
    async fn queued_admission_is_fifo_and_cancelling_a_waiter_releases_its_place() {
        let limit = Connections::new(5);
        let held: Vec<_> = (0..32)
            .map(|_| limit.clone().try_acquire_owned().unwrap())
            .collect();
        let ticket = limit.queue().await;
        let (started, mut queued) = tokio::sync::mpsc::channel(3);
        let (completed, mut finished) = tokio::sync::mpsc::channel(3);
        let mut tasks = Vec::new();
        for id in 0..3 {
            let limit = limit.clone();
            let started = started.clone();
            let completed = completed.clone();
            tasks.push(tokio::spawn(async move {
                started.send(id).await.unwrap();
                let ticket = limit.queue().await;
                let _hold = limit.reserve(&ticket).unwrap();
                completed.send(id).await.unwrap();
            }));
            assert_eq!(queued.recv().await, Some(id));
        }
        tasks[1].abort();
        limit.membership(20);
        drop(ticket);
        assert_eq!(finished.recv().await, Some(0));
        assert_eq!(finished.recv().await, Some(2));
        for task in tasks {
            let _ = task.await;
        }
        drop(held);
        assert_eq!(limit.state.lock().unwrap().used, 0);
    }
    #[tokio::test]
    async fn concurrent_directions_share_one_counter_and_cancelled_reservations_release_it() {
        let limit = Connections::new(256);
        let release = Arc::new(Notify::new());
        let (started, mut entered) = tokio::sync::mpsc::channel(1100);
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..1100 {
            let limit = limit.clone();
            let release = release.clone();
            let started = started.clone();
            tasks.spawn(async move {
                let hold = limit.clone().try_acquire_owned();
                started.send(hold.is_ok()).await.unwrap();
                if hold.is_ok() {
                    release.notified().await;
                }
            });
        }
        let mut admitted = 0;
        for _ in 0..1100 {
            admitted += usize::from(entered.recv().await.unwrap());
        }
        assert_eq!(admitted, CEILING);
        assert_eq!(limit.state.lock().unwrap().used, CEILING);
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        assert_eq!(limit.state.lock().unwrap().used, 0);
    }
}
