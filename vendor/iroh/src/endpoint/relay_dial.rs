//! Attempt-scoped Linn relay observations. Unsolicited EndpointGone is advisory.
use iroh_base::{EndpointId, RelayUrl};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot;

/// Result of a negotiated relay check. Neither result proves peer admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayDialOutcome {
    /// Target had a registered relay socket at the observation point.
    Registered,
    /// Target had no registered relay socket at the observation point.
    NotRegistered,
}

/// A reply validated against a token, target, relay and observer connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayDialObservation {
    /// Current attempt's registered/not-registered result.
    pub outcome: RelayDialOutcome,
    /// Observer WebSocket generation, retained only as validation metadata.
    pub generation: u64,
}

/// A notification can only trigger bounded scheduler work, never admission or absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetWatchEvent {
    /// The subscription's target registered on the correlated observer connection.
    Registered {
        /// Correlated observer WebSocket generation.
        generation: u64,
    },
    /// Subscription unsupported or its observer connection ended; remain unresolved.
    Unknown,
}
/// One unresolved target's job-owned subscription. Drop retires the server token.
#[derive(Debug)]
pub struct TargetWatch {
    token: [u8; 16],
    registry: Arc<Registry>,
    result: tokio::sync::watch::Receiver<Option<TargetWatchEvent>>,
}
impl TargetWatch {
    /// Wait for a new notification; coalesce repeated registrations without polling.
    pub async fn event(&mut self) -> TargetWatchEvent {
        if self.result.changed().await.is_err() {
            return TargetWatchEvent::Unknown;
        }
        (*self.result.borrow_and_update()).unwrap_or(TargetWatchEvent::Unknown)
    }
}
impl Drop for TargetWatch {
    fn drop(&mut self) {
        let mut inner = self.registry.inner.lock().unwrap();
        let unsent = inner.unsent.remove(&self.token);
        if let Some(watch) = inner.watches.remove(&self.token)
            && !unsent
        {
            inner
                .retired
                .insert(self.token, (watch.relay, watch.target, watch.generation));
        }
        drop(inner);
        self.registry.changed.notify_one();
    }
}
#[derive(Debug)]
pub(crate) struct WatchCommand {
    pub(crate) target: EndpointId,
    pub(crate) token: [u8; 16],
    pub(crate) subscribe: bool,
}
#[derive(Debug)]
struct Watching {
    relay: RelayUrl,
    target: EndpointId,
    generation: u64,
    result: tokio::sync::watch::Sender<Option<TargetWatchEvent>>,
}

/// A single dial's observation subscription. Dropping it retires its token.
#[derive(Debug)]
pub struct RelayDial {
    relay: RelayUrl,
    observed: Option<RelayDialObservation>,
    token: [u8; 16],
    registry: Arc<Registry>,
    result: oneshot::Receiver<RelayDialObservation>,
}
impl RelayDial {
    /// Wait for a correlated observation. None means unknown, never absence.
    pub async fn outcome(&mut self) -> Option<RelayDialOutcome> {
        self.observation().await.map(|reply| reply.outcome)
    }
    /// Wait for the validated reply and its observer generation.
    pub async fn observation(&mut self) -> Option<RelayDialObservation> {
        self.observed = (&mut self.result).await.ok();
        self.current_observation()
    }
    /// Retain validated evidence only while its observer generation is current.
    /// The token and target were checked before this dial received the reply.
    pub fn current_observation(&self) -> Option<RelayDialObservation> {
        self.observed.filter(|reply| {
            self.registry.inner.lock().unwrap().active.get(&self.relay) == Some(&reply.generation)
        })
    }
}
impl Drop for RelayDial {
    fn drop(&mut self) {
        self.registry
            .inner
            .lock()
            .unwrap()
            .pending
            .remove(&self.token);
    }
}
#[derive(Debug)]
struct Pending {
    relay: RelayUrl,
    target: EndpointId,
    generation: Option<u64>,
    result: oneshot::Sender<RelayDialObservation>,
}
#[derive(Debug, Default)]
struct Inner {
    generation: u64,
    active: BTreeMap<RelayUrl, u64>,
    pending: BTreeMap<[u8; 16], Pending>,
    watches: BTreeMap<[u8; 16], Watching>,
    unsent: BTreeSet<[u8; 16]>,
    retired: BTreeMap<[u8; 16], (RelayUrl, EndpointId, u64)>,
}
#[derive(Debug, Default)]
pub(crate) struct Registry {
    inner: Mutex<Inner>,
    pub(crate) changed: tokio::sync::Notify,
}
impl Registry {
    pub(crate) fn watch(
        self: &Arc<Self>,
        relay: RelayUrl,
        target: EndpointId,
    ) -> Option<TargetWatch> {
        let mut inner = self.inner.lock().unwrap();
        if inner.watches.len() + inner.retired.len() >= 1024 {
            return None;
        }
        let mut token = rand::random();
        while inner.watches.contains_key(&token) || inner.retired.contains_key(&token) {
            token = rand::random();
        }
        let (result, receiver) = tokio::sync::watch::channel(None);
        let Some(generation) = inner.active.get(&relay).copied() else {
            result.send_replace(Some(TargetWatchEvent::Unknown));
            return Some(TargetWatch {
                token,
                registry: self.clone(),
                result: receiver,
            });
        };
        inner.watches.insert(
            token,
            Watching {
                relay,
                target,
                generation,
                result,
            },
        );
        inner.unsent.insert(token);
        drop(inner);
        self.changed.notify_one();
        Some(TargetWatch {
            token,
            registry: self.clone(),
            result: receiver,
        })
    }
    pub(crate) fn watch_commands(
        &self,
        relay: &RelayUrl,
        generation: u64,
        supported: bool,
    ) -> Vec<WatchCommand> {
        let mut inner = self.inner.lock().unwrap();
        let mut commands = Vec::new();
        inner.retired.retain(|token, (url, target, old)| {
            if url != relay {
                return true;
            }
            if *old == generation && supported {
                commands.push(WatchCommand {
                    target: *target,
                    token: *token,
                    subscribe: false,
                });
            }
            false
        });
        let tokens: Vec<_> = inner
            .unsent
            .iter()
            .filter(|token| {
                inner
                    .watches
                    .get(*token)
                    .is_some_and(|watch| &watch.relay == relay && watch.generation == generation)
            })
            .copied()
            .collect();
        for token in tokens {
            inner.unsent.remove(&token);
            if supported {
                commands.push(WatchCommand {
                    target: inner.watches[&token].target,
                    token,
                    subscribe: true,
                });
            } else {
                inner.watches.remove(&token);
            }
        }
        commands
    }
    pub(crate) fn registered(
        &self,
        relay: &RelayUrl,
        generation: u64,
        target: EndpointId,
        token: [u8; 16],
    ) {
        let inner = self.inner.lock().unwrap();
        if inner.watches.get(&token).is_some_and(|watch| {
            &watch.relay == relay && watch.target == target && watch.generation == generation
        }) {
            inner.watches[&token]
                .result
                .send_replace(Some(TargetWatchEvent::Registered { generation }));
        }
    }
    pub(crate) fn register(
        self: &Arc<Self>,
        relay: RelayUrl,
        target: EndpointId,
    ) -> Option<RelayDial> {
        let mut inner = self.inner.lock().unwrap();
        if inner.pending.len() >= 256 {
            return None;
        }
        let mut token = rand::random();
        while inner.pending.contains_key(&token) {
            token = rand::random();
        }
        let (tx, result) = oneshot::channel();
        inner.pending.insert(
            token,
            Pending {
                relay: relay.clone(),
                target,
                generation: None,
                result: tx,
            },
        );
        Some(RelayDial {
            relay,
            observed: None,
            token,
            registry: self.clone(),
            result,
        })
    }
    pub(crate) fn connected(self: &Arc<Self>, relay: RelayUrl) -> Generation {
        let mut inner = self.inner.lock().unwrap();
        inner.generation += 1;
        let number = inner.generation;
        inner.active.insert(relay.clone(), number);
        Generation {
            relay,
            number: inner.generation,
            registry: self.clone(),
        }
    }
    pub(crate) fn requests(
        &self,
        relay: &RelayUrl,
        generation: u64,
    ) -> Vec<(EndpointId, [u8; 16])> {
        let mut inner = self.inner.lock().unwrap();
        let mut requests = Vec::new();
        for (token, pending) in &mut inner.pending {
            if &pending.relay == relay && pending.generation.is_none() {
                pending.generation = Some(generation);
                requests.push((pending.target, *token));
            }
        }
        requests
    }
    pub(crate) fn reply(
        &self,
        relay: &RelayUrl,
        generation: u64,
        target: EndpointId,
        token: [u8; 16],
        registered: bool,
    ) {
        let mut inner = self.inner.lock().unwrap();
        if inner.active.get(relay) == Some(&generation)
            && inner.pending.get(&token).is_some_and(|p| {
                &p.relay == relay && p.target == target && p.generation == Some(generation)
            })
        {
            let pending = inner.pending.remove(&token).unwrap();
            let _ = pending.result.send(RelayDialObservation {
                generation,
                outcome: if registered {
                    RelayDialOutcome::Registered
                } else {
                    RelayDialOutcome::NotRegistered
                },
            });
        }
    }
}
#[derive(Debug)]
pub(crate) struct Generation {
    pub(crate) number: u64,
    relay: RelayUrl,
    registry: Arc<Registry>,
}
impl Drop for Generation {
    fn drop(&mut self) {
        let mut inner = self.registry.inner.lock().unwrap();
        if inner.active.get(&self.relay) == Some(&self.number) {
            inner.active.remove(&self.relay);
        }
        inner
            .pending
            .retain(|_, p| !(p.relay == self.relay && p.generation == Some(self.number)));
        inner.watches.retain(|_, p| {
            if p.relay == self.relay && p.generation == self.number {
                p.result.send_replace(Some(TargetWatchEvent::Unknown));
                false
            } else {
                true
            }
        });
        let retained: BTreeSet<_> = inner.watches.keys().copied().collect();
        inner.unsent.retain(|token| retained.contains(token));
        inner
            .retired
            .retain(|_, (relay, _, generation)| relay != &self.relay || *generation != self.number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Arc<Registry>, RelayUrl, EndpointId) {
        (
            Arc::default(),
            "https://relay.example".parse().unwrap(),
            iroh_base::SecretKey::generate().public(),
        )
    }
    #[tokio::test]
    async fn only_current_token_target_relay_and_generation_can_confirm_absence() {
        let (registry, relay, target) = setup();
        let mut dial = registry.register(relay.clone(), target).unwrap();
        let generation = registry.connected(relay.clone());
        let (_, token) = registry.requests(&relay, generation.number)[0];
        registry.reply(&relay, generation.number + 1, target, token, false);
        registry.reply(&relay, generation.number, target, [0; 16], false);
        registry.reply(
            &"https://other.example".parse().unwrap(),
            generation.number,
            target,
            token,
            false,
        );
        registry.reply(
            &relay,
            generation.number,
            iroh_base::SecretKey::generate().public(),
            token,
            false,
        );
        assert!(matches!(
            dial.result.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        registry.reply(&relay, generation.number, target, token, false);
        assert_eq!(dial.outcome().await, Some(RelayDialOutcome::NotRegistered));
    }
    #[tokio::test]
    async fn reconnect_retires_old_attempt_and_delayed_reply_cannot_poison_new_dial() {
        let (registry, relay, target) = setup();
        let mut old = registry.register(relay.clone(), target).unwrap();
        let generation = registry.connected(relay.clone());
        let old_number = generation.number;
        let (_, old_token) = registry.requests(&relay, old_number)[0];
        drop(generation);
        assert_eq!(old.outcome().await, None);
        let mut new = registry.register(relay.clone(), target).unwrap();
        let generation = registry.connected(relay.clone());
        let (_, token) = registry.requests(&relay, generation.number)[0];
        registry.reply(&relay, generation.number, target, old_token, false);
        assert!(matches!(
            new.result.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        registry.reply(&relay, generation.number, target, token, true);
        assert_eq!(new.outcome().await, Some(RelayDialOutcome::Registered));
    }
    #[tokio::test]
    async fn retained_or_buffered_evidence_cannot_cross_a_relay_generation() {
        for consumed in [false, true] {
            let (registry, relay, target) = setup();
            let mut dial = registry.register(relay.clone(), target).unwrap();
            let mut late = registry.register(relay.clone(), target).unwrap();
            let generation = registry.connected(relay.clone());
            let requests = registry.requests(&relay, generation.number);
            registry.reply(&relay, generation.number, target, dial.token, false);
            if consumed {
                assert!(dial.observation().await.is_some());
                assert!(dial.current_observation().is_some());
            }
            let replacement = registry.connected(relay.clone());
            assert_ne!(generation.number, replacement.number);
            if !consumed {
                assert!(dial.observation().await.is_none());
            }
            assert!(dial.current_observation().is_none());
            // The old transport may not have dropped its Generation guard yet.
            registry.reply(&relay, generation.number, target, late.token, false);
            assert!(matches!(
                late.result.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
            assert_eq!(requests.len(), 2);
            drop(generation);
            assert!(late.observation().await.is_none());
        }
    }

    #[test]
    fn old_relay_without_negotiation_never_starts_a_check_and_cancel_releases_capacity() {
        let (registry, relay, target) = setup();
        let mut dial = registry.register(relay.clone(), target).unwrap();
        let _generation = registry.connected(relay);
        assert!(matches!(
            dial.result.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        drop(dial);
        assert!(registry.inner.lock().unwrap().pending.is_empty());
    }

    #[test]
    fn congested_checks_are_bounded_and_missing_replies_remain_unknown() {
        let (registry, relay, target) = setup();
        let mut dials: Vec<_> = (0..256)
            .map(|_| registry.register(relay.clone(), target).unwrap())
            .collect();
        assert!(registry.register(relay.clone(), target).is_none());
        let generation = registry.connected(relay.clone());
        assert_eq!(registry.requests(&relay, generation.number).len(), 256);
        assert!(registry.requests(&relay, generation.number).is_empty());
        for dial in &mut dials {
            assert!(matches!(
                dial.result.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
        }
        drop(dials);
        assert!(registry.inner.lock().unwrap().pending.is_empty());
    }
}

#[cfg(test)]
mod target_watch_tests {
    use super::*;
    #[tokio::test]
    async fn registration_is_correlated_and_retained_until_drop_without_accepting_old_generations()
    {
        let registry = Arc::new(Registry::default());
        let relay: RelayUrl = "https://relay.example".parse().unwrap();
        let target = iroh_base::SecretKey::generate().public();
        let generation = registry.connected(relay.clone());
        let mut watch = registry.watch(relay.clone(), target).unwrap();
        let commands = registry.watch_commands(&relay, generation.number, true);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].subscribe);
        assert_eq!(commands[0].target, target);
        registry.registered(&relay, generation.number + 1, target, commands[0].token);
        registry.registered(&relay, generation.number, target, [0; 16]);
        registry.registered(
            &"https://other.example".parse().unwrap(),
            generation.number,
            target,
            commands[0].token,
        );
        registry.registered(
            &relay,
            generation.number,
            iroh_base::SecretKey::generate().public(),
            commands[0].token,
        );
        assert!(!watch.result.has_changed().unwrap());
        registry.registered(&relay, generation.number, target, commands[0].token);
        assert_eq!(
            watch.event().await,
            TargetWatchEvent::Registered {
                generation: generation.number
            }
        );
        assert_eq!(registry.inner.lock().unwrap().watches.len(), 1);
        registry.registered(&relay, generation.number, target, commands[0].token);
        assert_eq!(
            watch.event().await,
            TargetWatchEvent::Registered {
                generation: generation.number
            }
        );
        drop(watch);
        assert!(registry.inner.lock().unwrap().watches.is_empty());
    }
    #[tokio::test]
    async fn unsupported_and_observer_reconnect_retire_to_unknown_without_resubscription() {
        let registry = Arc::new(Registry::default());
        let relay: RelayUrl = "https://relay.example".parse().unwrap();
        let target = iroh_base::SecretKey::generate().public();
        let generation = registry.connected(relay.clone());
        let mut unsupported = registry.watch(relay.clone(), target).unwrap();
        assert!(
            registry
                .watch_commands(&relay, generation.number, false)
                .is_empty()
        );
        assert_eq!(unsupported.event().await, TargetWatchEvent::Unknown);
        let mut reconnect = registry.watch(relay.clone(), target).unwrap();
        let commands = registry.watch_commands(&relay, generation.number, true);
        registry.registered(&relay, generation.number, target, commands[0].token);
        drop(generation);
        assert_eq!(reconnect.event().await, TargetWatchEvent::Unknown);
        let new = registry.connected(relay.clone());
        registry.registered(&relay, new.number, target, commands[0].token);
        assert!(registry.watch_commands(&relay, new.number, true).is_empty());
    }
    #[tokio::test]
    async fn unsent_watches_cannot_cross_an_observer_reconnect() {
        let registry = Arc::new(Registry::default());
        let relay: RelayUrl = "https://relay.example".parse().unwrap();
        let target = iroh_base::SecretKey::generate().public();
        let generation = registry.connected(relay.clone());
        let mut unsent = registry.watch(relay.clone(), target).unwrap();
        drop(generation);
        assert_eq!(unsent.event().await, TargetWatchEvent::Unknown);
        let mut disconnected = registry.watch(relay.clone(), target).unwrap();
        assert_eq!(disconnected.event().await, TargetWatchEvent::Unknown);
        let new = registry.connected(relay.clone());
        assert!(registry.watch_commands(&relay, new.number, true).is_empty());
        let cancelled = registry.watch(relay.clone(), target).unwrap();
        drop(cancelled);
        assert!(registry.watch_commands(&relay, new.number, true).is_empty());
    }
    #[tokio::test]
    async fn watch_capacity_includes_retirements_and_drop_queues_unsubscribe() {
        let registry = Arc::new(Registry::default());
        let relay: RelayUrl = "https://relay.example".parse().unwrap();
        let target = iroh_base::SecretKey::generate().public();
        let generation = registry.connected(relay.clone());
        let watches: Vec<_> = (0..1024)
            .map(|_| registry.watch(relay.clone(), target).unwrap())
            .collect();
        assert!(registry.watch(relay.clone(), target).is_none());
        assert_eq!(
            registry
                .watch_commands(&relay, generation.number, true)
                .len(),
            1024
        );
        drop(watches);
        assert!(registry.watch(relay.clone(), target).is_none());
        let retired = registry.watch_commands(&relay, generation.number, true);
        assert_eq!(retired.len(), 1024);
        assert!(retired.iter().all(|command| !command.subscribe));
        assert!(registry.watch(relay, target).is_some());
    }
}
