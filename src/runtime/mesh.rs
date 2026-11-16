//! Network coordination stays outside the serialized store execution thread.
use super::*;

pub(super) async fn run(
    mut job: Job,
    store: crate::mesh::store::Handle,
    transport: mesh_driver::Driver,
) -> (Job, Result<Reply>) {
    let Operation::Mesh(command) = std::mem::replace(&mut job.operation, Operation::Check) else {
        return (job, Err(Error::Invalid("expected mesh operation".into())));
    };
    if let Err(error) = command.validate() {
        return (
            job,
            Err(crate::error::ApplicationError::new(error, false).into()),
        );
    }
    let mutating = command.mutates();
    let record_job = matches!(command, crate::mesh::pairing::Command::Sync);
    let first_sync = matches!(
        command,
        crate::mesh::pairing::Command::Pair {
            initial_sync: true,
            ..
        }
    );
    let suspending = matches!(
        command,
        crate::mesh::pairing::Command::Reset | crate::mesh::pairing::Command::Leave
    );
    let mut control = if mutating && !record_job {
        Some(transport.control().await)
    } else {
        None
    };
    if job.reply.is_closed() {
        return (
            job,
            Err(Error::Invalid(
                "mesh command cancelled before execution".into(),
            )),
        );
    }
    let mut cancellation = transport.cancellation();
    let _work = if !suspending && mutating {
        match transport.begin_work() {
            Ok(work) => Some(work),
            Err(error) => {
                return (
                    job,
                    Err(crate::error::ApplicationError::new(error, false).into()),
                );
            }
        }
    } else {
        None
    };
    let progress = job.progress.clone();
    let mut syncing = record_job;
    let work = async {
        let mut output = execute(&store, &transport, command, &progress).await?;
        if first_sync
            && let crate::mesh::pairing::Output::Changed {
                operation: crate::mesh::pairing::Change::Paired,
                status,
                initial_sync,
                initial_sync_error,
            } = &mut output
        {
            drop(control.take());
            syncing = true;
            let _ = progress.try_send(crate::mesh::status::Progress::Admitted(status.clone()));
            match synchronize(&store, &transport, &progress).await {
                Ok(report) => *initial_sync = Some(Box::new(report)),
                Err(error) => {
                    *initial_sync_error =
                        Some(Box::new(crate::error::ApplicationError::new(error, true)))
                }
            }
        }
        Ok(output)
    };
    let (result, cancelled) = tokio::select! {
        result = work => (result, false),
        _ = job.reply.closed(), if !suspending => (Err(Error::Invalid("mesh command cancelled; an issued membership change may already have committed".into())), true),
        _ = cancellation.changed(), if !suspending => (Err(Error::Invalid("mesh command cancelled for transport shutdown or identity change".into())), true),
    };
    let result = if cancelled && syncing {
        use crate::mesh::store::ControlStore;
        match store.step(crate::mesh::store::Step::Finish).await {
            Ok(_) => result,
            Err(error) => Err(error),
        }
    } else {
        result
    };
    if suspending {
        transport.release_gate();
    }
    let result = result
        .map(|output| Reply::from(Value::Mesh(output)))
        .map_err(|error| crate::error::ApplicationError::new(error, mutating).into());
    (job, result)
}

async fn execute(
    store: &crate::mesh::store::Handle,
    transport: &mesh_driver::Driver,
    mut command: crate::mesh::pairing::Command,
    progress: &mpsc::Sender<crate::mesh::status::Progress>,
) -> Result<crate::mesh::pairing::Output> {
    use crate::mesh::{
        pairing::{Command, Output},
        store::{ControlStore, Step},
    };
    let mutating = command.mutates();
    let pairing = matches!(command, Command::Pair { .. });
    let check_status = matches!(command, Command::CheckStatus);
    if matches!(command, Command::Reset | Command::Leave) {
        // Keep leave intent durable even if draining transport or publication fails.
        let intent = store
            .step(if matches!(command, Command::Leave) {
                Step::Leave
            } else {
                Step::Disable
            })
            .await;
        let suspended = transport.suspend().await;
        if let Err(error) = intent {
            return Err(Error::Invalid(format!(
                "could not persist transport shutdown intent: {error}; shutdown was requested for this daemon, but a restart may restore authorization"
            )));
        }
        suspended?;
    }
    if let Command::Pair { initial_sync, .. } = &mut command {
        *initial_sync = false;
    }
    let mut output = if matches!(command, Command::Sync) {
        Output::Synced(synchronize(store, transport, progress).await?)
    } else {
        let output = crate::mesh::pairing::run(store, command).await?;
        if mutating {
            if pairing {
                // Admission is complete before passive transport startup, which
                // must not turn a successful enrollment into a pairing failure.
                transport.resume();
            } else {
                let _ = transport.ensure().await?;
            }
        }
        output
    };
    match &mut output {
        Output::Status(status) | Output::Changed { status, .. } => {
            status.transport = transport.status()
        }
        _ => (),
    }
    if check_status && let Output::Status(status) = &mut output {
        let mut checks: std::collections::BTreeMap<_, _> = status
            .members
            .iter()
            .filter(|id| Some(*id) != status.endpoint.as_ref())
            .map(|id| {
                (
                    id.clone(),
                    "unknown (not checked before deadline)".to_owned(),
                )
            })
            .collect();
        if status.is_admitted() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(10), async {
                let Some(peer) = transport.ensure().await? else {
                    return Ok::<_, Error>(());
                };
                let slots = std::sync::Arc::new(tokio::sync::Semaphore::new(8));
                let mut pending = tokio::task::JoinSet::new();
                for id in checks.keys() {
                    let id = id.clone();
                    let peer = peer.clone();
                    let slots = slots.clone();
                    pending.spawn(async move {
                        let _slot = slots
                            .acquire_owned()
                            .await
                            .expect("probe slots remain open");
                        let result = peer.check_transport(&id).await;
                        (
                            id,
                            match result {
                                Ok(()) => "reachable (transport only)".into(),
                                Err(error) => {
                                    use crate::mesh::failure::Kind;
                                    if matches!(Kind::of(&error), Kind::Offline | Kind::Timeout) {
                                        format!("unreachable: {error}")
                                    } else {
                                        format!("unknown (check failed): {error}")
                                    }
                                }
                            },
                        )
                    });
                }
                while let Some(result) = pending.join_next().await {
                    if let Ok((id, result)) = result {
                        checks.insert(id, result);
                    }
                }
                Ok(())
            })
            .await;
        } else {
            for result in checks.values_mut() {
                *result = "unknown (local transport is not authorized)".into();
            }
        }
        status.transport = transport.status();
        status.reachability = Some(checks);
    }
    Ok(output)
}

async fn synchronize(
    store: &crate::mesh::store::Handle,
    transport: &mesh_driver::Driver,
    progress: &mpsc::Sender<crate::mesh::status::Progress>,
) -> Result<crate::mesh::jobs::Report> {
    use crate::mesh::{
        failure::Kind,
        status::Progress,
        store::{self as control, ControlStore, Step},
    };
    let result = async {
        store.step(Step::Finish).await?;
        let Some(identity) = control::identity(store).await? else {
            return Ok(crate::mesh::jobs::Report {
                complete: true,
                ..Default::default()
            });
        };
        if identity.conflict {
            return Err(Kind::Conflict.error("mesh membership is conflicted"));
        }
        if !identity.enabled || identity.pending_leave {
            return Err(Kind::Authentication.error("mesh transport is disabled"));
        }
        let client = crate::mesh::client::ControlClient::new(&identity.service)?;
        match crate::mesh::pairing::refresh(store, &client, &identity.key, None).await {
            Ok(_) | Err(Error::MeshControl(crate::mesh::client::ControlError::Unavailable)) => (),
            Err(error) => return Err(error),
        }
        let peer = transport
            .ensure()
            .await?
            .ok_or_else(|| Error::Invalid("peer transport is not running".into()))?;
        let run = crate::mesh::membership::random_id();
        let sync = peer.clone().sync(run.clone());
        tokio::pin!(sync);
        let mut tick = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                result = &mut sync => return result,
                _ = tick.tick() => {
                    if let Some(job) = peer.status().jobs.into_iter().find(|job| job.id == run) {
                        let _ = progress.try_send(Progress::Sync(job));
                    }
                }
            }
        }
    }
    .await;
    store.step(Step::Finish).await?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{
        pairing::Command,
        store::{self, Step},
    };
    #[tokio::test]
    async fn failed_transport_suspension_preserves_pending_self_revocation() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = root.path().join("store.db");
        let mut app = Application::new(path.clone());
        app.mesh_step(Step::CreateIdentity("https://mesh.example.test".into()))
            .await
            .unwrap();
        let (store, mut jobs) = store::channel();
        let (driver, task) =
            mesh_driver::Driver::start(store.clone(), &tokio::runtime::Handle::current(), false);
        // A stopped driver makes suspension fail before any service request.
        task.abort();
        let _ = task.await;
        let progress = mpsc::channel(2).0;
        let (result, ()) =
            tokio::join!(execute(&store, &driver, Command::Leave, &progress), async {
                let job = jobs.recv().await.unwrap();
                assert!(matches!(job.step, Step::Leave));
                let _ = job.reply.send(app.mesh_step(job.step).await);
                let _ = job.delivered.await;
            });
        assert!(result.is_err());
        drop(app);
        let replica = crate::storage::sync::Replica::open_private_mesh(&path)
            .await
            .unwrap();
        let identity = replica.mesh_identity().await.unwrap().unwrap();
        assert!(!identity.enabled);
        assert!(identity.pending_leave);
    }
    #[tokio::test]
    async fn failed_shutdown_intent_still_requests_transport_suspension() {
        let (store, mut jobs) = store::channel();
        let (driver, task) =
            mesh_driver::Driver::start(store.clone(), &tokio::runtime::Handle::current(), false);
        let cancellation = driver.cancellation();
        let progress = mpsc::channel(2).0;
        let (result, ()) =
            tokio::join!(execute(&store, &driver, Command::Leave, &progress), async {
                let job = jobs.recv().await.unwrap();
                assert!(matches!(job.step, Step::Leave));
                let _ = job
                    .reply
                    .send(Err(Error::Invalid("simulated storage exhaustion".into())));
            });
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("restart may restore authorization")
        );
        assert!(cancellation.has_changed().unwrap());
        task.abort();
        let _ = task.await;
    }
}
