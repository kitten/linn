//! Daemon-owned network coordination; local store steps return to the dispatcher.
use super::*;
use crate::{
    storage::sync::{Entry, Item, Remote},
    sync::{
        LocalReplica,
        protocol::{Request, Response},
    },
};
pub(super) struct InternalJob {
    pub operation: Operation,
    pub diagnostics: crate::diagnostics::Diagnostics,
    pub reply: oneshot::Sender<Result<Reply>>,
    pub delivered: oneshot::Receiver<()>,
}
struct DispatcherReplica {
    queue: mpsc::Sender<InternalJob>,
    diagnostics: crate::diagnostics::Diagnostics,
}
impl DispatcherReplica {
    async fn request(&self, operation: Operation) -> Result<(Reply, oneshot::Sender<()>)> {
        let (tx, rx) = oneshot::channel();
        let (delivery, delivered) = oneshot::channel();
        self.queue
            .send(InternalJob {
                operation,
                diagnostics: self.diagnostics.clone(),
                reply: tx,
                delivered,
            })
            .await
            .map_err(|_| Error::Invalid("store dispatcher stopped during sync".into()))?;
        let mut reply = rx
            .await
            .map_err(|_| Error::Invalid("store dispatcher stopped during sync".into()))??;
        for warning in reply.warnings.drain(..) {
            crate::diagnostics::emit(warning);
        }
        Ok((reply, delivery))
    }
    async fn frame(&self, request: Request) -> Result<Response> {
        if let Value::SyncFrame(response) =
            self.request(Operation::SyncFrame(request)).await?.0.value
        {
            Ok(response)
        } else {
            Err(Error::Invalid("unexpected local replica reply".into()))
        }
    }
}
impl LocalReplica for DispatcherReplica {
    type Delivery = oneshot::Sender<()>;
    async fn remotes(&self) -> Result<Vec<Remote>> {
        if let Value::Remotes(remotes) = self.request(Operation::Remotes).await?.0.value {
            Ok(remotes)
        } else {
            Err(Error::Invalid(
                "unexpected remote configuration reply".into(),
            ))
        }
    }
    async fn inventory(
        &self,
        tombstones: bool,
        after: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Vec<Entry>> {
        if let Response::Entries { entries } = self
            .frame(Request::Inventory {
                tombstones,
                after: after.map(str::to_owned),
                scope: scope.map(str::to_owned),
            })
            .await?
        {
            Ok(entries)
        } else {
            Err(Error::Invalid("unexpected inventory reply".into()))
        }
    }
    async fn get(&self, id: &str, scope: Option<&str>) -> Result<Option<(Item, Self::Delivery)>> {
        let (reply, delivery) = self
            .request(Operation::SyncFrame(Request::Get {
                id: id.into(),
                scope: scope.map(str::to_owned),
            }))
            .await?;
        if let Value::SyncFrame(Response::Item { item }) = reply.value {
            Ok(item.map(|item| (item, delivery)))
        } else {
            Err(Error::Invalid("unexpected item reply".into()))
        }
    }
    async fn needs(&self, entry: &Entry) -> Result<bool> {
        if let Response::Needed { ids } = self
            .frame(Request::Needs {
                entries: vec![entry.clone()],
            })
            .await?
        {
            Ok(!ids.is_empty())
        } else {
            Err(Error::Invalid("unexpected needs reply".into()))
        }
    }
    async fn apply(&self, item: Item, scope: Option<&str>) -> Result<bool> {
        if let Response::Applied { changed } = self
            .frame(Request::Apply {
                item,
                scope: scope.map(str::to_owned),
            })
            .await?
        {
            Ok(changed)
        } else {
            Err(Error::Invalid("unexpected merge reply".into()))
        }
    }
    async fn finish(&self) -> Result<()> {
        if matches!(self.frame(Request::Finish).await?, Response::Done) {
            Ok(())
        } else {
            Err(Error::Invalid("unexpected publication reply".into()))
        }
    }
}
pub(super) fn run(job: Job, queue: mpsc::Sender<InternalJob>) -> (Job, Result<Reply>) {
    let result = (|| -> Result<Reply> {
        let Operation::Sync { environment } = &job.operation else {
            return Err(Error::Invalid("expected synchronization request".into()));
        };
        let replica = DispatcherReplica {
            queue,
            diagnostics: job.caller.diagnostics.clone(),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let (result, diagnostics) = rt.block_on(crate::diagnostics::capture(
            job.caller.diagnostics.clone(),
            crate::application::synchronize(&replica, Some(environment)),
        ));
        match result {
            Ok(counts) => Ok(Reply {
                value: Value::Synced(counts),
                warnings: diagnostics,
                output: None,
            }),
            Err(error) => {
                let mut error = crate::error::ApplicationError::new(error, true);
                error.diagnostics.extend(diagnostics);
                Err(error.into())
            }
        }
    })();
    (job, result)
}
