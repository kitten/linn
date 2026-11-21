//! Account-private steps executed only on the existing store dispatcher.
use super::*;
use crate::mesh::{
    failure::Kind,
    store::{Step, Value as MeshValue},
};

impl Application {
    pub(super) async fn mesh_command(
        &mut self,
        command: crate::mesh::pairing::Command,
    ) -> Result<crate::mesh::pairing::Output> {
        // Network commands need the resident owner even for library callers;
        // raw store handles never create another endpoint for the same identity.
        let reply = crate::runtime::execute(
            &self.path,
            Operation::Mesh(command),
            Caller {
                scope: None,
                writer: None,
                diagnostics: Default::default(),
            },
            None,
        )
        .await?;
        if let Value::Mesh(output) = reply.value {
            Ok(output)
        } else {
            Err(Error::Invalid("unexpected resident mesh reply".into()))
        }
    }
    pub(crate) async fn mesh_step(&mut self, step: Step) -> Result<MeshValue> {
        let result = self.mesh_step_inner(step).await;
        if result.is_err() {
            self.replica = None;
            self.store = None;
        }
        result.map_err(|error| crate::error::ApplicationError::new(error, true).into())
    }
    async fn mesh_step_inner(&mut self, step: Step) -> Result<MeshValue> {
        if matches!(step, Step::Disable | Step::Reset | Step::Leave) && !self.path.exists() {
            return Ok(MeshValue::Done);
        }
        if matches!(step, Step::Disable | Step::Reset | Step::Leave) {
            crate::storage::mesh::private_store(&self.path)?;
        }
        if matches!(step, Step::Finish(_)) {
            self.store = None;
        }
        if matches!(step, Step::Identity) && !self.path.exists() {
            return Ok(MeshValue::Identity(None));
        }
        if matches!(step, Step::CreateIdentity(_)) {
            crate::storage::mesh::private_store(&self.path)?;
        }
        let buffers = self.mesh_buffers.clone();
        let replica = self.replica().await?;
        Ok(match step {
            Step::Identity => MeshValue::Identity(if replica.mesh_enrolled().await? {
                replica.mesh_identity().await?
            } else {
                None
            }),
            Step::CreateIdentity(service) => {
                MeshValue::Identity(Some(replica.mesh_create_identity(&service).await?))
            }
            Step::Pin(key) => {
                replica.mesh_pin_receipt_key(&key).await?;
                MeshValue::Done
            }
            Step::Membership(head) => {
                MeshValue::Membership(replica.mesh_membership(head.as_deref()).await?)
            }
            Step::Pending => MeshValue::Pending(replica.mesh_pending().await?),
            Step::Prepare(proposal) => {
                replica.mesh_prepare(&proposal).await?;
                MeshValue::Done
            }
            Step::Reject(proposal) => {
                replica.mesh_reject_pending(&proposal).await?;
                MeshValue::Done
            }
            Step::KnownEvent(hash) => MeshValue::KnownEvent(replica.mesh_known_event(&hash).await?),
            Step::Accept(accepted) => {
                MeshValue::Accepted(replica.mesh_accept_event(&accepted).await?)
            }
            Step::Enable(head) => {
                replica.mesh_enable(&head).await?;
                MeshValue::Done
            }
            Step::Leave => {
                replica.mesh_leave_local().await?;
                MeshValue::Done
            }
            Step::Disable => {
                replica.mesh_disable().await?;
                MeshValue::Done
            }
            Step::Reset => {
                replica.mesh_reset_identity().await?;
                MeshValue::Done
            }
            Step::History(after) => MeshValue::History(replica.mesh_history(after).await?),
            Step::SourceSnapshot(peer) => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::SourceSnapshot(replica.source_snapshot().await?)
            }
            Step::Authorize(peer) => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::Done
            }
            Step::Inventory {
                peer,
                tombstones,
                after,
                cutoff,
            } => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::Inventory(
                    replica
                        .inventory_page(tombstones, after.as_deref(), None, Some(cutoff))
                        .await?,
                )
            }
            Step::Needs { peer, entries } => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::Needs {
                    prefix: replica.needs_page(&entries).await?,
                    epoch: buffers.as_ref().map_or(0, |buffers| buffers.epoch()),
                }
            }
            Step::Sizes { peer, entries } => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::Inventory(crate::storage::sync::InventoryPage {
                    entries: replica.sizes(entries).await?,
                    done: true,
                })
            }
            Step::ApplyGroup { peer, job, items } => {
                replica.mesh_authorize(&peer).await?;
                if peer != job.run.initiator {
                    replica.mesh_authorize(&job.run.initiator).await?;
                }
                let mut originals = Vec::with_capacity(items.len());
                let mut error = None;
                for body in items {
                    match body.take() {
                        Ok(item) => originals.push(item),
                        Err(failure) => {
                            error = Some(failure);
                            break;
                        }
                    }
                }
                let mut prefix = replica.apply_group(&originals, None).await?;
                if let Some(buffers) = &buffers {
                    for item in &originals[..prefix.decisions.len()] {
                        if item.envelope.is_none() {
                            buffers.forget(&item.id);
                        }
                    }
                }
                if prefix.error.is_none() {
                    prefix.error = error;
                }
                MeshValue::Decisions(prefix)
            }
            Step::Get {
                peer,
                job,
                id,
                scope,
            } => {
                scope.ensure()?;
                replica.mesh_authorize(&peer).await?;
                if peer != job.run.initiator {
                    replica.mesh_authorize(&job.run.initiator).await?;
                }
                MeshValue::Item(replica.get(&id, None).await?)
            }
            Step::BodyChunk {
                scope,
                peer,
                job,
                id,
                hash,
            } => {
                scope.ensure()?;
                if peer != job.run.initiator {
                    replica.mesh_authorize(&job.run.initiator).await?;
                }
                replica.finish().await.map_err(Kind::publication)?;
                replica
                    .mesh_authorize_chunk(&peer, &id, hash.as_deref())
                    .await?;
                scope.ensure()?;
                MeshValue::Done
            }
            Step::Finish(peer) => {
                if let Some(peer) = peer {
                    replica.mesh_authorize(&peer).await?;
                }
                replica.finish().await.map_err(Kind::publication)?;
                MeshValue::Done
            }
        })
    }
}
