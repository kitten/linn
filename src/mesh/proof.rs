//! @ref LLP 0008#derived-membership-storage-and-history
//! Unfamiliar component ancestry stays transient until a verified merge connects
//! it to our mesh. Unrelated signed histories never become local durable state.
use super::{
    failure::Kind,
    membership::{Acceptance, Membership, canonical},
    store::{self, ControlStore, Step, Value},
};
use crate::error::Result;
use std::collections::BTreeMap;

const MAX_STAGED: usize = 4 * 1024 * 1024;
struct Node {
    event: Acceptance,
    state: Option<Membership>,
    bytes: usize,
    state_bytes: usize,
}
pub(crate) struct Staging {
    trusted: Membership,
    nodes: BTreeMap<String, Node>,
    bytes: usize,
}
impl Staging {
    pub(crate) async fn new(store: &impl ControlStore) -> Result<Self> {
        let trusted = store::membership(store, None)
            .await?
            .ok_or_else(|| Kind::Authentication.error("no trusted mesh membership"))?;
        Ok(Self {
            trusted,
            nodes: BTreeMap::new(),
            bytes: 0,
        })
    }
    pub(crate) async fn accept(
        &mut self,
        store: &impl ControlStore,
        event: Acceptance,
    ) -> Result<()> {
        let hash = event.proposal.hash()?;
        let Value::KnownEvent(known) = store.step(Step::KnownEvent(hash.clone())).await? else {
            return Err(Kind::Protocol.error("event lookup expected"));
        };
        if known || self.nodes.contains_key(&hash) {
            return Ok(());
        }
        let mut parents = Vec::new();
        for head in &event.proposal.event.heads {
            parents.push(if let Some(node) = self.nodes.get(head) {
                node.state.clone().ok_or_else(|| {
                    Kind::Conflict.error("unfamiliar membership history forks a consumed head")
                })?
            } else {
                store::membership(store, Some(head.clone()))
                    .await?
                    .ok_or_else(|| {
                        Kind::Authentication.error("membership proof is missing a predecessor")
                    })?
            });
        }
        let state = event.evaluate(&parents.iter().collect::<Vec<_>>())?;
        if state.service != self.trusted.service || state.receipt_key != self.trusted.receipt_key {
            return Err(
                Kind::Authentication.error("membership proof uses a different service identity")
            );
        }
        let connected = state
            .roots
            .iter()
            .any(|root| self.trusted.roots.contains(root));
        if connected {
            // Commit only this connected event's ancestors, never other staged components.
            let mut work: Vec<_> = event
                .proposal
                .event
                .heads
                .iter()
                .rev()
                .cloned()
                .map(|h| (h, false))
                .collect();
            while let Some((head, ready)) = work.pop() {
                let Some(node) = self.nodes.get(&head) else {
                    continue;
                };
                if !ready {
                    work.push((head, true));
                    work.extend(
                        node.event
                            .proposal
                            .event
                            .heads
                            .iter()
                            .rev()
                            .cloned()
                            .map(|h| (h, false)),
                    );
                } else {
                    let node = self.nodes.remove(&head).unwrap();
                    store.step(Step::Accept(node.event)).await?;
                    self.bytes -= node.bytes;
                }
            }
            store.step(Step::Accept(event)).await?;
        } else {
            // Retain envelopes for replay, but only leaf aggregates. Keeping every
            // ancestor state would make a linear unfamiliar chain quadratic.
            let state_bytes = canonical(&state)?.len();
            let bytes = canonical(&event)?.len() + state_bytes + 512;
            let reclaimed: usize = event
                .proposal
                .event
                .heads
                .iter()
                .filter_map(|head| self.nodes.get(head).map(|node| node.state_bytes))
                .sum();
            if (self.bytes - reclaimed).saturating_add(bytes) > MAX_STAGED {
                return Err(
                    Kind::Capacity.error("unfamiliar membership ancestry exceeds staging capacity")
                );
            }
            for head in &event.proposal.event.heads {
                if let Some(node) = self.nodes.get_mut(head) {
                    node.state = None;
                    node.bytes -= node.state_bytes;
                    node.state_bytes = 0;
                }
            }
            self.bytes = self.bytes - reclaimed + bytes;
            self.nodes.insert(
                hash,
                Node {
                    event,
                    state: Some(state),
                    bytes,
                    state_bytes,
                },
            );
        }
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<()> {
        if self.nodes.is_empty() {
            Ok(())
        } else {
            Err(Kind::Authentication.error("membership proof contains unrelated history"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Store {
        current: String,
        states: Mutex<BTreeMap<String, Membership>>,
        accepted: Mutex<Vec<String>>,
    }
    impl ControlStore for Store {
        async fn step(&self, step: Step) -> Result<Value> {
            Ok(match step {
                Step::KnownEvent(hash) => {
                    Value::KnownEvent(self.states.lock().unwrap().contains_key(&hash))
                }
                Step::Membership(head) => Value::Membership(
                    self.states
                        .lock()
                        .unwrap()
                        .get(head.as_ref().unwrap_or(&self.current))
                        .cloned(),
                ),
                Step::Accept(event) => {
                    let mut states = self.states.lock().unwrap();
                    let parents: Vec<_> = event
                        .proposal
                        .event
                        .heads
                        .iter()
                        .map(|head| &states[head])
                        .collect();
                    let state = event.evaluate(&parents)?;
                    let head = state.head.clone();
                    states.insert(head.clone(), state);
                    self.accepted.lock().unwrap().push(head.clone());
                    Value::Accepted(head)
                }
                _ => panic!("unexpected proof store operation"),
            })
        }
    }
    fn fixture() -> (Store, Vec<Acceptance>) {
        let values: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../tests/fixtures/mesh-membership-v1.json"))
                .unwrap();
        let initial: Membership = serde_json::from_value(values[0]["state"].clone()).unwrap();
        let store = Store {
            current: initial.head.clone(),
            states: Mutex::new(BTreeMap::from([(initial.head.clone(), initial)])),
            accepted: Mutex::default(),
        };
        let events = values
            .into_iter()
            .map(|v| serde_json::from_value(v["accepted"].clone()).unwrap())
            .collect();
        (store, events)
    }
    #[tokio::test]
    async fn unrelated_signed_history_never_becomes_durable() {
        let (store, events) = fixture();
        let mut proof = Staging::new(&store).await.unwrap();
        for event in &events[3..5] {
            proof.accept(&store, event.clone()).await.unwrap();
        }
        assert!(store.accepted.lock().unwrap().is_empty());
        assert_eq!(
            Kind::of(&proof.finish().err().unwrap()),
            Kind::Authentication
        );
        assert_eq!(store.states.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn verified_merge_imports_its_foreign_ancestry_and_revocation() {
        let (store, events) = fixture();
        let mut proof = Staging::new(&store).await.unwrap();
        for event in &events[..5] {
            proof.accept(&store, event.clone()).await.unwrap();
        }
        assert_eq!(store.accepted.lock().unwrap().len(), 2);
        for event in &events[5..] {
            proof.accept(&store, event.clone()).await.unwrap();
        }
        proof.finish().unwrap();
        assert_eq!(store.accepted.lock().unwrap().len(), 6);
        // Repeated full histories use only indexed existence checks.
        let mut proof = Staging::new(&store).await.unwrap();
        for event in events {
            proof.accept(&store, event).await.unwrap();
        }
        proof.finish().unwrap();
        assert_eq!(store.accepted.lock().unwrap().len(), 6);
    }
    #[tokio::test]
    async fn staging_capacity_fails_without_importing_a_foreign_component() {
        let (store, events) = fixture();
        let mut proof = Staging::new(&store).await.unwrap();
        proof.bytes = MAX_STAGED;
        assert_eq!(
            Kind::of(&proof.accept(&store, events[3].clone()).await.err().unwrap()),
            Kind::Capacity
        );
        assert!(store.accepted.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn long_unfamiliar_chains_keep_linear_aggregate_storage() {
        use crate::mesh::membership::{Action, Event, Proposal, endpoint, random_id};
        use iroh::SecretKey;
        let (store, _) = fixture();
        let owner = SecretKey::from_bytes(&[250; 32]);
        let receipt = SecretKey::from_bytes(&[9; 32]);
        let mut proof = Staging::new(&store).await.unwrap();
        let mut heads = vec![];
        for index in 0..=200u16 {
            let action = if index == 0 {
                Action::Genesis {
                    actor: endpoint(&owner),
                    root: random_id(),
                    receipt_key: endpoint(&receipt),
                }
            } else {
                let delegated = SecretKey::from_bytes(&[(index - 1) as u8; 32]);
                Action::Invite {
                    actor: endpoint(&owner),
                    key: endpoint(&delegated),
                    receipt_key: endpoint(&receipt),
                    not_before: 1000,
                    expires: 1600,
                }
            };
            let proposal = Proposal::sign(
                Event {
                    version: 1,
                    service: proof.trusted.service.clone(),
                    id: random_id(),
                    heads,
                    action,
                },
                &[&owner],
            )
            .unwrap();
            heads = vec![proposal.hash().unwrap()];
            proof
                .accept(&store, Acceptance::sign(proposal, 1001, &receipt).unwrap())
                .await
                .unwrap();
        }
        assert!(proof.bytes < MAX_STAGED / 4);
        assert!(store.accepted.lock().unwrap().is_empty());
        assert!(proof.finish().is_err());
    }
}
