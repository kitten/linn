//! Meaningful job activity, independent of transport keepalives.
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Progress(Arc<tokio::sync::watch::Sender<u64>>);
impl Default for Progress {
    fn default() -> Self {
        Self(Arc::new(tokio::sync::watch::channel(0).0))
    }
}
impl Progress {
    pub(crate) fn advance(&self) {
        self.0
            .send_modify(|revision| *revision = revision.wrapping_add(1));
    }
    pub(crate) fn revision(&self) -> u64 {
        *self.0.borrow()
    }
    pub(crate) async fn changed_since(&self, revision: u64) {
        let mut receiver = self.0.subscribe();
        let _ = receiver.wait_for(|current| *current != revision).await;
    }
}
