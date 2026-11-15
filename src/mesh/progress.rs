//! Meaningful job activity, independent of transport keepalives.
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Default)]
pub(crate) struct Progress(Arc<AtomicU64>);
impl Progress {
    pub(crate) fn advance(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn revision(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}
