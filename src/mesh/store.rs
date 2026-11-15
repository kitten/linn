//! In-process queue boundary. Endpoint secrets never enter serializable replies.
use super::membership::{Acceptance, Membership, Proposal};
use crate::{error::Result, storage::mesh::Identity};

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
    SourceBudget(String),
    Peer {
        peer: String,
        request: crate::sync::protocol::Request,
    },
    Chunk {
        peer: String,
        id: String,
        hash: Option<String>,
    },
    Finish,
}
pub enum Value {
    Identity(Option<Identity>),
    Membership(Option<Membership>),
    Accepted(String),
    KnownEvent(bool),
    Pending(Option<Proposal>),
    History(Vec<(i64, Acceptance)>),
    SourceBudget([u64; 2]),
    Replica(crate::sync::protocol::Response),
    Done,
}
pub trait ControlStore {
    fn step(&self, step: Step) -> impl std::future::Future<Output = Result<Value>> + Send;
}

pub(crate) struct Job {
    pub step: Step,
    pub reply: tokio::sync::oneshot::Sender<Result<Value>>,
    pub delivered: tokio::sync::oneshot::Receiver<()>,
}
#[derive(Clone)]
pub(crate) struct Handle {
    sender: tokio::sync::mpsc::Sender<Job>,
}
pub(crate) fn channel() -> (Handle, tokio::sync::mpsc::Receiver<Job>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(64);
    (Handle { sender }, receiver)
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
        let (reply, result) = tokio::sync::oneshot::channel();
        let (delivery, delivered) = tokio::sync::oneshot::channel();
        self.sender
            .send(Job {
                step,
                reply,
                delivered,
            })
            .await
            .map_err(|_| crate::error::Error::Invalid("mesh store dispatcher stopped".into()))?;
        let result = result
            .await
            .map_err(|_| crate::error::Error::Invalid("mesh store dispatcher stopped".into()))??;
        Ok((result, delivery))
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
