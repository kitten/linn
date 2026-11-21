//! In-process queue boundary. Endpoint secrets never enter serializable replies.
use super::delta::Changes;
use super::membership::{Acceptance, Membership, Proposal};
use crate::{error::Result, storage::mesh::Identity};
use std::sync::Arc;

pub enum Step {
    Identity,
    CreateIdentity(String),
    Pin(String),
    Membership(Option<String>),
    Pending,
    Prepare(Proposal),
    Reject(Proposal),
    Accept(Acceptance),
    KnownEvent(String),
    Enable(String),
    Leave,
    Disable,
    Reset,
    History(i64),
    SourceSnapshot(String),
    Authorize(String),
    Inventory {
        peer: String,
        tombstones: bool,
        after: Option<String>,
        cutoff: u64,
    },
    Needs {
        peer: String,
        entries: Vec<crate::storage::sync::Entry>,
    },
    Sizes {
        peer: String,
        entries: Vec<crate::storage::sync::Entry>,
    },
    ApplyGroup {
        peer: String,
        job: Arc<super::jobs::SignedRun>,
        items: Vec<super::buffers::Body>,
    },
    Get {
        scope: Arc<super::peer::source::Scope>,
        peer: String,
        job: Arc<super::jobs::SignedRun>,
        id: String,
    },
    BodyChunk {
        scope: Arc<super::peer::source::Scope>,
        peer: String,
        job: Arc<super::jobs::SignedRun>,
        id: String,
        hash: Option<String>,
    },
    Finish(Option<String>),
}
pub enum Value {
    Identity(Option<Identity>),
    Membership(Option<Membership>),
    Accepted(String),
    KnownEvent(bool),
    Pending(Option<Proposal>),
    History(Vec<(i64, Acceptance)>),
    SourceSnapshot(u64),
    Item(Option<crate::storage::sync::Item>),
    Inventory(crate::storage::sync::InventoryPage),
    Decisions(crate::storage::sync::Prefix<bool>),
    Body(super::buffers::Body),
    Needs {
        prefix: crate::storage::sync::Prefix<bool>,
        epoch: u64,
    },
    Done,
}
pub trait ControlStore {
    fn step(&self, step: Step) -> impl std::future::Future<Output = Result<Value>> + Send;
}

pub(crate) struct Job {
    pub step: Step,
    pub reply: Reply,
    pub delivered: tokio::sync::oneshot::Receiver<()>,
}
enum Mutation {
    Group {
        run: String,
        entries: Vec<crate::storage::sync::Entry>,
    },
    Invalidate,
    Publication,
}
pub(crate) struct Reply {
    sender: tokio::sync::oneshot::Sender<Result<Value>>,
    changes: Arc<Changes>,
    mutation: Option<Mutation>,
    buffers: Arc<super::buffers::Buffers>,
    source: Option<Arc<super::peer::source::Scope>>,
    pins_delivery: bool,
}
impl Reply {
    pub(crate) async fn deliver(
        self,
        result: Result<Value>,
        delivered: tokio::sync::oneshot::Receiver<()>,
        barrier: &crate::storage::ReadBarrier,
    ) {
        let _readers = barrier.read().await;
        let pins_delivery = self.pins_delivery;
        let _ = self.send(result);
        // Metadata consumers may be unpolled while their control owner awaits
        // another store step. Only original-byte delivery can pin this lane.
        if pins_delivery {
            let _ = delivered.await;
        }
    }
    pub(crate) fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
    pub(crate) fn send(self, result: Result<Value>) -> std::result::Result<(), ()> {
        let result = match result {
            Ok(Value::Item(Some(item))) if self.source.is_some() => self
                .source
                .as_ref()
                .unwrap()
                .original(&self.buffers, item)
                .map(Value::Body),
            other => other,
        };
        match (&self.mutation, &result) {
            (Some(Mutation::Group { run, entries }), Ok(Value::Decisions(prefix))) => {
                for (entry, changed) in entries.iter().zip(&prefix.decisions) {
                    if *changed {
                        self.changes.record(Some(run.clone()), Some(entry.clone()));
                    }
                }
                if prefix.error.is_some() {
                    self.changes.record(None, None);
                }
            }
            (Some(Mutation::Invalidate), _) | (Some(_), Err(_)) => self.changes.record(None, None),
            _ => (),
        }
        self.sender.send(result).map_err(|_| ())
    }
}
#[derive(Clone)]
pub(crate) struct Handle {
    sender: tokio::sync::mpsc::Sender<Job>,
    pub(crate) changes: Arc<Changes>,
    pub(crate) buffers: Arc<super::buffers::Buffers>,
}
pub(crate) fn channel() -> (Handle, tokio::sync::mpsc::Receiver<Job>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(64);
    (
        Handle {
            sender,
            changes: Arc::default(),
            buffers: Arc::default(),
        },
        receiver,
    )
}
impl ControlStore for Handle {
    async fn step(&self, step: Step) -> Result<Value> {
        let (result, delivery) = self.lease(step).await?;
        drop(delivery);
        Ok(result)
    }
}
impl Handle {
    pub(crate) async fn lease(
        &self,
        step: Step,
    ) -> Result<(Value, tokio::sync::oneshot::Sender<()>)> {
        let (message, result, delivery) = self.message(step);
        self.sender.send(message).await.map_err(|_| stopped())?;
        let result = result.await.map_err(|_| stopped())??;
        Ok((result, delivery))
    }
    fn message(
        &self,
        step: Step,
    ) -> (
        Job,
        tokio::sync::oneshot::Receiver<Result<Value>>,
        tokio::sync::oneshot::Sender<()>,
    ) {
        let (reply, result) = tokio::sync::oneshot::channel();
        let pins_delivery = matches!(&step, Step::Get { .. } | Step::BodyChunk { .. });
        let mutation = match &step {
            Step::ApplyGroup { job, items, .. } => {
                let entries: Vec<_> = items.iter().map(super::buffers::Body::entry).collect();
                if entries.iter().any(|entry| {
                    entry.id.len() + entry.hash.as_ref().map_or(0, String::len)
                        > super::delta::EVENT_BYTES
                }) {
                    Some(Mutation::Invalidate)
                } else {
                    Some(Mutation::Group {
                        run: job.run.id.clone(),
                        entries,
                    })
                }
            }
            Step::Finish(_) | Step::BodyChunk { .. } => Some(Mutation::Publication),
            Step::Leave | Step::Reset | Step::Disable | Step::Accept(_) | Step::Enable(_) => {
                Some(Mutation::Invalidate)
            }
            _ => None,
        };
        let (delivery, delivered) = tokio::sync::oneshot::channel();
        let source = match &step {
            Step::Get { scope, .. } => Some(scope.clone()),
            _ => None,
        };
        (
            Job {
                step,
                reply: Reply {
                    sender: reply,
                    changes: self.changes.clone(),
                    mutation,
                    buffers: self.buffers.clone(),
                    source,
                    pins_delivery,
                },
                delivered,
            },
            result,
            delivery,
        )
    }
    pub(crate) async fn prepare(&self, step: Step) -> Result<Prepared> {
        let permit = self
            .sender
            .clone()
            .reserve_owned()
            .await
            .map_err(|_| stopped())?;
        Ok(Prepared {
            handle: self.clone(),
            permit: Some(permit),
            step: Some(step),
        })
    }
}
fn stopped() -> crate::error::Error {
    crate::error::Error::Invalid("mesh store dispatcher stopped".into())
}
/// A reserved writer message makes cancellation cleanup synchronous to enqueue.
/// The caller reserves Finish before any group, and drops groups before Finish.
pub(crate) struct Prepared {
    handle: Handle,
    permit: Option<tokio::sync::mpsc::OwnedPermit<Job>>,
    step: Option<Step>,
}
impl Prepared {
    pub(crate) fn remove_missing(&mut self, mut skip: impl FnMut(usize)) {
        if let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() {
            let mut retained = 0;
            items.retain(|body| {
                if body.missing() {
                    skip(retained);
                    false
                } else {
                    retained += 1;
                    true
                }
            });
        }
    }
    pub(crate) fn complete_prefix(&self) -> usize {
        match &self.step {
            Some(Step::ApplyGroup { items, .. }) => {
                items.iter().take_while(|body| body.complete()).count()
            }
            _ => 0,
        }
    }
    pub(crate) fn move_prefix(&mut self, destination: &mut Self, count: usize) {
        let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() else {
            unreachable!("Apply group expected");
        };
        let Some(Step::ApplyGroup { items: target, .. }) = destination.step.as_mut() else {
            unreachable!("Apply group expected");
        };
        target.extend(items.drain(..count));
    }
    pub(crate) fn discard(&mut self) {
        if let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() {
            items.clear();
        }
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        if let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() {
            items.truncate(len);
        }
    }
    pub(crate) fn remove(&mut self, index: usize) {
        if let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() {
            items.remove(index);
        }
    }
    pub(crate) fn push(&mut self, body: super::buffers::Body) {
        let Some(Step::ApplyGroup { items, .. }) = self.step.as_mut() else {
            unreachable!("Apply group expected");
        };
        items.push(body);
    }
    pub(crate) async fn submit(mut self) -> Result<Value> {
        self.start().await.map_err(|_| stopped())?
    }
    pub(crate) fn start(&mut self) -> tokio::sync::oneshot::Receiver<Result<Value>> {
        let step = self.step.take().expect("prepared step is submitted once");
        let (message, reply, delivery) = self.handle.message(step);
        drop(delivery);
        self.permit.take().unwrap().send(message);
        reply
    }
}
impl Drop for Prepared {
    fn drop(&mut self) {
        self.remove_missing(|_| {});
        let Some(step) = self.step.take() else {
            return;
        };
        if matches!(&step, Step::ApplyGroup { items, .. } if items.is_empty()) {
            return;
        }
        let (message, _, delivery) = self.handle.message(step);
        drop(delivery);
        self.permit.take().unwrap().send(message);
    }
}
pub async fn identity(store: &impl ControlStore) -> Result<Option<Identity>> {
    if let Value::Identity(identity) = store.step(Step::Identity).await? {
        Ok(identity)
    } else {
        Err(crate::error::Error::Invalid(
            "unexpected mesh identity reply".into(),
        ))
    }
}
pub async fn membership(
    store: &impl ControlStore,
    head: Option<String>,
) -> Result<Option<Membership>> {
    if let Value::Membership(state) = store.step(Step::Membership(head)).await? {
        Ok(state)
    } else {
        Err(crate::error::Error::Invalid(
            "unexpected mesh membership reply".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mesh::jobs::{Run, SignedRun},
        storage::sync::{Entry, Item},
    };

    #[tokio::test]
    async fn only_original_reads_and_chunk_writes_pin_following_publication() {
        let (handle, _) = channel();
        let job = Arc::new(SignedRun {
            run: Run {
                version: 2,
                id: "run".into(),
                initiator: "peer".into(),
                head: String::new(),
                cohort: vec!["peer".into()],
                max_hops: 0,
            },
            signature: String::new(),
            path: vec!["peer".into()],
        });
        for step in [
            Step::KnownEvent("head".into()),
            Step::Get {
                peer: "peer".into(),
                job: job.clone(),
                scope: Arc::default(),
                id: "record".into(),
            },
            Step::BodyChunk {
                peer: "peer".into(),
                job,
                scope: Arc::default(),
                id: "record".into(),
                hash: None,
            },
        ] {
            let pins = !matches!(step, Step::KnownEvent(_));
            let (original, result, delivery) = handle.message(step);
            let (finish, finished, finish_delivery) = handle.message(Step::Finish(None));
            let barrier = crate::storage::read_barrier();
            let writer = tokio::spawn(async move {
                original
                    .reply
                    .deliver(Ok(Value::Done), original.delivered, &barrier)
                    .await;
                finish
                    .reply
                    .deliver(Ok(Value::Done), finish.delivered, &barrier)
                    .await;
            });
            let mut delivery = Some(delivery);
            let mut finished = finished;
            if pins {
                result.await.unwrap().unwrap();
                assert!(
                    tokio::time::timeout(std::time::Duration::from_millis(10), &mut finished)
                        .await
                        .is_err()
                );
                drop(delivery.take());
            }
            // Metadata must let publication finish while its reply is unpolled
            // and the consumer still holds its delivery sender.
            tokio::time::timeout(std::time::Duration::from_secs(1), finished)
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            writer.await.unwrap();
            drop((delivery, finish_delivery));
        }
    }

    #[tokio::test]
    async fn cancelled_groups_and_finish_keep_reserved_queue_order_under_pressure() {
        let (handle, mut jobs) = channel();
        let finish = handle.prepare(Step::Finish(None)).await.unwrap();
        let job = Arc::new(SignedRun {
            run: Run {
                version: 2,
                id: "run".into(),
                initiator: "peer".into(),
                head: String::new(),
                cohort: vec!["peer".into()],
                max_hops: 0,
            },
            signature: String::new(),
            path: vec!["peer".into()],
        });
        let step = || Step::ApplyGroup {
            peer: "peer".into(),
            job: job.clone(),
            items: vec![],
        };
        let mut filling = handle.prepare(step()).await.unwrap();
        let spare = handle.prepare(step()).await.unwrap();
        // The reader can decode a full-inventory miss and a later valid item
        // before the writer consumes either response notification.
        let missing = handle
            .buffers
            .receive(
                handle.buffers.epoch(),
                Entry {
                    id: "missing".into(),
                    hash: Some("hash".into()),
                },
            )
            .unwrap();
        missing
            .with(|payload| {
                *payload = super::super::buffers::Payload::Missing;
                Ok(())
            })
            .unwrap();
        filling.push(missing);
        let complete = handle.buffers.original(Item {
            id: "valid".into(),
            envelope: Some("original".into()),
            envelope_hash: Some("hash".into()),
            body_hash: None,
        });
        filling.push(complete);
        filling.push(
            handle
                .buffers
                .receive(
                    handle.buffers.epoch(),
                    Entry {
                        id: "unfinished".into(),
                        hash: Some("hash".into()),
                    },
                )
                .unwrap(),
        );
        for _ in 0..61 {
            let (message, _, _) = handle.message(Step::Identity);
            assert!(handle.sender.try_send(message).is_ok());
        }
        assert_eq!(handle.sender.capacity(), 0);
        drop(filling);
        drop(spare);
        drop(finish);
        for _ in 0..61 {
            assert!(matches!(jobs.recv().await.unwrap().step, Step::Identity));
        }
        let group = jobs.recv().await.unwrap();
        assert!(group.reply.is_closed());
        let Step::ApplyGroup { items, .. } = group.step else {
            panic!("group must precede publication")
        };
        assert_eq!(items[0].take().unwrap().id, "valid");
        assert!(items[1].take().is_err());
        let publication = jobs.recv().await.unwrap();
        assert!(matches!(publication.step, Step::Finish(None)));
        assert!(publication.reply.is_closed());
        assert!(jobs.try_recv().is_err());
    }
}
