//! One recorded connection and operation authorize a bounded source stream.
//! @ref LLP 0008#bounded-forwarding-and-duplicate-control
use super::*;
use crate::mesh::buffers::{Body, WeakBody};

#[derive(Default)]
pub struct Scope {
    state: Mutex<ScopeState>,
    revoked: tokio::sync::watch::Sender<bool>,
}
#[derive(Default)]
struct ScopeState {
    revoked: bool,
    bodies: Vec<WeakBody>,
}
impl Scope {
    pub(crate) fn ensure(&self) -> Result<()> {
        if self.state.lock().unwrap().revoked {
            Err(Kind::Cancelled.error("source operation authority ended"))
        } else {
            Ok(())
        }
    }
    pub(crate) fn original(
        &self,
        buffers: &super::super::buffers::Buffers,
        item: crate::storage::sync::Item,
    ) -> Result<Body> {
        let mut state = self.state.lock().unwrap();
        if state.revoked {
            return Err(Kind::Cancelled.error("source operation authority ended"));
        }
        let body = buffers.original(item);
        state.bodies.retain(WeakBody::alive);
        state.bodies.push(body.downgrade());
        Ok(body)
    }
    fn revoke(&self) {
        let mut state = self.state.lock().unwrap();
        state.revoked = true;
        for body in state.bodies.drain(..) {
            body.discard();
        }
        self.revoked.send_replace(true);
    }
    pub(super) async fn ended(&self) {
        let _ = self.revoked.subscribe().wait_for(|ended| *ended).await;
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Delta,
    Full,
    Fallback,
    Ended,
}
struct Current {
    state: State,
    scope: Arc<Scope>,
}
pub(super) struct Grant {
    pub(super) round: u8,
    context: u64,
    connection: usize,
    current: Mutex<Current>,
}
impl Grant {
    pub(super) fn new(round: u8, context: u64, connection: usize) -> Arc<Self> {
        Arc::new(Self {
            round,
            context,
            connection,
            current: Mutex::new(Current {
                state: State::Ready,
                scope: Arc::default(),
            }),
        })
    }
    pub(super) fn claim(
        self: &Arc<Self>,
        round: u8,
        context: u64,
        connection: usize,
        delta: bool,
    ) -> Result<Stream> {
        let mut current = self.current.lock().unwrap();
        if (delta && round != 2)
            || self.round != round
            || self.context != context
            || self.connection != connection
            || !(current.state == State::Ready
                || (!delta && matches!(current.state, State::Delta | State::Fallback)))
        {
            return Err(Kind::Authentication
                .error("source stream is outside its recorded operation or connection"));
        }
        let fallback = current.state != State::Ready;
        if fallback {
            // A receiver detecting an invalid delta need not wait for its old
            // stream's FIN/reset to reach this source before requesting fallback.
            current.scope.revoke();
            current.scope = Arc::default();
        }
        current.scope.ensure()?;
        current.state = if delta { State::Delta } else { State::Full };
        Ok(Stream {
            grant: self.clone(),
            scope: current.scope.clone(),
            fallback,
            released: false,
        })
    }
    pub(super) fn context(&self) -> u64 {
        self.context
    }
    pub(super) fn active(&self) -> bool {
        matches!(
            self.current.lock().unwrap().state,
            State::Delta | State::Full
        )
    }
    pub(super) fn revoke(&self) {
        let mut current = self.current.lock().unwrap();
        current.state = State::Ended;
        current.scope.revoke();
    }
}
pub(super) struct Stream {
    grant: Arc<Grant>,
    pub(super) scope: Arc<Scope>,
    pub(super) fallback: bool,
    released: bool,
}
impl Stream {
    pub(super) fn fallback(&mut self) -> Result<()> {
        if self.fallback {
            return Err(protocol("a full inventory fallback cannot fall back again"));
        }
        let mut current = self.grant.current.lock().unwrap();
        if Arc::ptr_eq(&current.scope, &self.scope) && current.state == State::Delta {
            current.state = State::Fallback;
        }
        self.released = true;
        Ok(())
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        let mut current = self.grant.current.lock().unwrap();
        if Arc::ptr_eq(&current.scope, &self.scope) {
            // Delta reset/EOF permits the same single full fallback, never
            // another delta or a new operation on a different connection.
            current.state = if current.state == State::Delta {
                State::Fallback
            } else {
                State::Ended
            };
            current.scope.revoke();
        }
    }
}
pub(super) struct Receiving {
    peer: Weak<Peer>,
    run: String,
}
impl Receiving {
    pub(super) fn begin(peer: &Arc<Peer>, run: &str, round: u8) -> Result<Self> {
        let mut runs = peer.runs.lock().unwrap();
        let entry = runs.get_mut(run).ok_or_else(|| protocol("expired job"))?;
        if entry.terminal_only || entry.phase.is_terminal() {
            return Err(Kind::Cancelled.error("receiving role ended"));
        }
        entry.advance_round(round)?;
        entry.transfers += 1;
        Ok(Self {
            peer: Arc::downgrade(peer),
            run: run.into(),
        })
    }
}
impl Drop for Receiving {
    fn drop(&mut self) {
        if let Some(peer) = self.peer.upgrade()
            && let Some(entry) = peer.runs.lock().unwrap().get_mut(&self.run)
        {
            entry.transfers -= 1;
        }
    }
}
pub(super) struct Operation(pub(super) Arc<Grant>);
impl Drop for Operation {
    fn drop(&mut self) {
        self.0.revoke();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item() -> crate::storage::sync::Item {
        crate::storage::sync::Item {
            id: "secret".into(),
            envelope: Some("original".into()),
            envelope_hash: Some("hash".into()),
            body_hash: None,
        }
    }
    #[test]
    fn source_context_connection_and_round_cannot_replay_or_overlap() {
        let grant = Grant::new(1, 17, 9);
        assert!(grant.claim(2, 17, 9, true).is_err());
        assert!(grant.claim(1, 18, 9, false).is_err());
        assert!(grant.claim(1, 17, 10, false).is_err());
        let stream = grant.claim(1, 17, 9, false).unwrap();
        assert!(grant.claim(1, 17, 9, false).is_err());
        drop(stream);
        assert!(grant.claim(1, 17, 9, false).is_err());
    }
    #[test]
    fn one_full_fallback_keeps_the_same_context_and_cannot_repeat() {
        let grant = Grant::new(2, 17, 9);
        let mut delta = grant.claim(2, 17, 9, true).unwrap();
        delta.fallback().unwrap();
        drop(delta);
        let mut full = grant.claim(2, 17, 9, false).unwrap();
        assert!(full.fallback().is_err());
        drop(full);
        assert!(grant.claim(2, 17, 9, true).is_err());
    }
    #[test]
    fn fallback_revokes_a_still_active_delta_and_its_late_drop_cannot_revoke_full() {
        let buffers = super::super::super::buffers::Buffers::default();
        let grant = Grant::new(2, 17, 9);
        let delta = grant.claim(2, 17, 9, true).unwrap();
        let original = delta.scope.original(&buffers, item()).unwrap();
        let full = grant.claim(2, 17, 9, false).unwrap();
        assert!(original.take().is_err());
        assert!(delta.scope.ensure().is_err());
        drop(delta);
        assert!(full.scope.ensure().is_ok());
        assert!(grant.claim(2, 17, 9, false).is_err());
        drop(full);
        assert!(grant.claim(2, 17, 9, false).is_err());
    }
    #[tokio::test]
    async fn revocation_erases_registered_originals_and_fences_late_store_replies() {
        let buffers = super::super::super::buffers::Buffers::default();
        let grant = Grant::new(1, 17, 9);
        let stream = grant.claim(1, 17, 9, false).unwrap();
        let body = stream.scope.original(&buffers, item()).unwrap();
        grant.revoke();
        body.erased().await;
        assert!(body.take().is_err());
        assert!(stream.scope.original(&buffers, item()).is_err());
        assert!(stream.scope.ensure().is_err());
        assert_eq!(buffers.live_count(), 0);
    }
}
