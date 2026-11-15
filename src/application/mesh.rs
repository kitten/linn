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
        if matches!(
            &step,
            Step::Peer {
                request: crate::sync::protocol::Request::Apply { .. }
                    | crate::sync::protocol::Request::Finish,
                ..
            } | Step::Finish
        ) {
            self.store = None;
        }
        if matches!(step, Step::Identity) && !self.path.exists() {
            return Ok(MeshValue::Identity(None));
        }
        if matches!(step, Step::CreateIdentity(_)) {
            crate::storage::mesh::private_store(&self.path)?;
        }
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
            Step::SourceBudget(peer) => {
                replica.mesh_authorize(&peer).await?;
                MeshValue::SourceBudget(replica.inventory_budget().await?)
            }
            Step::Peer { peer, request } => {
                replica.mesh_authorize(&peer).await?;
                match &request {
                    crate::sync::protocol::Request::Inventory { scope: Some(_), .. }
                    | crate::sync::protocol::Request::Get { scope: Some(_), .. }
                    | crate::sync::protocol::Request::Apply { scope: Some(_), .. } => {
                        return Err(Error::Invalid(
                            "mesh replication has no scope selector".into(),
                        ));
                    }
                    _ => (),
                }
                let publishing = matches!(request, crate::sync::protocol::Request::Finish);
                let result = crate::sync::protocol::handle(replica, request).await;
                MeshValue::Replica(if publishing {
                    result.map_err(Kind::publication)?
                } else {
                    result?
                })
            }
            Step::Chunk { peer, id, hash } => {
                replica.finish().await.map_err(Kind::publication)?;
                replica
                    .mesh_authorize_chunk(&peer, &id, hash.as_deref())
                    .await?;
                MeshValue::Done
            }
            Step::Finish => {
                replica.finish().await.map_err(Kind::publication)?;
                MeshValue::Done
            }
        })
    }
}
