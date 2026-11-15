//! Serialized store execution on a dedicated daemon thread.
//! @ref LLP 0007#concurrency-and-scheduling
use super::*;
pub(super) struct Binding {
    pub root: (u64, u64),
    pub lock: (u64, u64),
    pub socket: (u64, u64),
    pub socket_path: PathBuf,
    pub file: Option<(u64, u64)>,
}
pub(super) type StopReply = (oneshot::Sender<()>, oneshot::Receiver<()>);
pub(super) async fn run(
    location: Location,
    mut jobs: mpsc::Receiver<Job>,
    mut stopping: mpsc::Receiver<StopReply>,
    accept_task: tokio::task::AbortHandle,
    binding: Binding,
    network: tokio::runtime::Handle,
    mesh_startup: bool,
) -> Result<Option<StopReply>> {
    let mut dispatcher = Dispatcher {
        app: Application::new(location.path.clone()),
        location,
        binding,
        retired: false,
    };
    let mut stop_reply = None;
    let mut draining = false;
    let mut workers = tokio::task::JoinSet::new();
    let mut deliveries = tokio::task::JoinSet::new();
    let (mesh_store, mut mesh_steps) = crate::mesh::store::channel();
    let (transport, mut transport_task) =
        mesh_driver::Driver::start(mesh_store.clone(), &network, mesh_startup);
    let mut transport_result = None;
    let mut transport_stop = None;
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    loop {
        while deliveries.try_join_next().is_some() {}
        if dispatcher.retired && !draining {
            draining = true;
            jobs.close();
            accept_task.abort();
        }
        if draining && transport_stop.is_none() {
            let transport = transport.clone();
            transport_stop = Some(network.spawn(async move { transport.quit().await }));
        }
        if draining
            && jobs.is_empty()
            && workers.is_empty()
            && deliveries.is_empty()
            && transport_result.is_some()
        {
            break;
        }
        tokio::select! {
            result = &mut transport_task, if transport_result.is_none() => {
                transport_result = Some(result);
                if !draining { draining = true; jobs.close(); accept_task.abort(); }
            }
            signal = stopping.recv(), if !draining => {
                stop_reply = signal;
                draining = true;
                jobs.close();
                accept_task.abort();
            }
            _ = tick.tick(), if !draining => {
                let file_changed = platform::file_identity(&dispatcher.location.path).ok()
                    .is_none_or(|current| dispatcher.binding.file.is_some() && current != dispatcher.binding.file)
                    || platform::identity(&dispatcher.location.path).ok().as_ref() != Some(&dispatcher.location.id);
                if file_changed || !dispatcher.intact() {
                    draining = true;
                    jobs.close();
                    accept_task.abort();
                }
            }
            Some(job) = jobs.recv(), if !draining || !jobs.is_empty() => {
                if !dispatcher.retired && matches!(job.operation, Operation::Mesh(_)) {
                    if job.reply.is_closed() { continue; }
                    use crate::mesh::pairing::Command;
                    let limit = if matches!(job.operation, Operation::Mesh(Command::Leave | Command::Reset | Command::Status | Command::Peers)) { 5 } else { 4 };
                    if workers.len() >= limit {
                        let _ = job.reply.send(Err(crate::error::ApplicationError::new(Error::Invalid("daemon busy; mesh command not executed".into()), false).into()));
                        continue;
                    }
                    workers.spawn_on(mesh::run(job, mesh_store.clone(), transport.clone()), &network);
                } else {
                    dispatcher.dispatch(job).await;
                }
            }
            Some(job) = mesh_steps.recv() => {
                if job.reply.is_closed() { continue; }
                let changed = matches!(job.step, crate::mesh::store::Step::CreateIdentity(_) | crate::mesh::store::Step::Accept(_) | crate::mesh::store::Step::Enable(_) | crate::mesh::store::Step::Leave | crate::mesh::store::Step::Disable | crate::mesh::store::Step::Reset);
                let result = dispatcher.mesh_step(job.step).await;
                let _ = job.reply.send(result);
                let _ = job.delivered.await;
                if changed { transport.changed(); }
            }
            Some(completed) = workers.join_next(), if !workers.is_empty() => {
                if let Ok((job, result)) = completed {
                    let _ = job.reply.send(result);
                    // Mesh outcomes contain metadata, not original record bytes.
                    // Drain socket delivery on shutdown without occupying the store lane.
                    deliveries.spawn_on(async move { let _ = job.delivered.await; }, &network);
                }
            }
            _ = deliveries.join_next(), if !deliveries.is_empty() => (),
        }
    }
    if let Some(stop) = transport_stop {
        stop.await
            .map_err(|_| Error::Invalid("peer transport shutdown failed".into()))??;
    }
    transport_result
        .unwrap()
        .map_err(|_| Error::Invalid("peer transport failed".into()))??;
    Ok(stop_reply)
}

struct Dispatcher {
    location: Location,
    app: Application,
    binding: Binding,
    retired: bool,
}
impl Dispatcher {
    async fn mesh_step(
        &mut self,
        step: crate::mesh::store::Step,
    ) -> Result<crate::mesh::store::Value> {
        if self.retired
            || !self.intact()
            || platform::identity(&self.location.path)? != self.location.id
            || (self.binding.file.is_some()
                && platform::file_identity(&self.location.path)? != self.binding.file)
        {
            return Err(Error::Invalid(
                "store ownership changed; mesh step not executed".into(),
            ));
        }
        let result = self.app.mesh_step(step).await;
        self.retired = matches!(&result, Err(Error::Application(error)) if matches!(error.code, crate::error::ErrorCode::Schema));
        let after = platform::file_identity(&self.location.path)?;
        if self.binding.file.is_some() && after != self.binding.file {
            self.retired = true;
            return Err(Error::Invalid(
                "store replaced during mesh step; owner is retiring".into(),
            ));
        }
        self.binding.file = after;
        result
    }
    fn intact(&self) -> bool {
        let identity = |path: &Path| std::fs::symlink_metadata(path).ok().as_ref().map(file_key);
        identity(&self.location.root) == Some(self.binding.root)
            && identity(&self.location.file("owner.lock")) == Some(self.binding.lock)
            && identity(&self.binding.socket_path) == Some(self.binding.socket)
    }
    async fn dispatch(&mut self, job: Job) {
        if job.reply.is_closed() {
            return;
        }
        let result = self
            .perform(Some(&job.store), job.operation, job.caller, job.input)
            .await;
        let _ = job.reply.send(result);
        // Only transport completion or cancellation releases the delivery barrier.
        let _ = job.delivered.await;
    }
    async fn perform(
        &mut self,
        requested_store: Option<&Path>,
        operation: Operation,
        caller: Caller,
        input: Option<File>,
    ) -> Result<Reply> {
        if self.retired {
            return Err(crate::error::ApplicationError::new(
                Error::Invalid(
                    "daemon retiring after incompatible store schema; request not executed".into(),
                ),
                false,
            )
            .into());
        }
        // Until creation, differently cased basenames can name the same future
        // file on macOS. Serialize that first operation across this parent and
        // recheck native routing after the preceding creator completes.
        let _creation = if self.binding.file.is_none() {
            let lock = platform::private_file(&self.location.root.join("create.lock"))?;
            lock.lock()?;
            Some(lock)
        } else {
            None
        };
        if !self.intact() {
            return Err(Error::Invalid(
                "daemon ownership or socket directory changed; stop and retry".into(),
            ));
        }
        let requested_store = requested_store.unwrap_or(&self.location.path);
        if platform::identity(requested_store)? != self.location.id {
            return Err(Error::Invalid(
                "store path changed; stop the daemon and retry".into(),
            ));
        }
        let current = platform::file_identity(&self.location.path)?;
        if self.binding.file.is_some() && current != self.binding.file {
            return Err(Error::Invalid(
                "store replaced or removed; stop the daemon and retry".into(),
            ));
        }
        if self.binding.file.is_none() {
            self.binding.file = current;
        }
        let result = self.app.execute(operation, caller, input).await;
        self.retired = matches!(&result, Err(Error::Application(error))
            if matches!(error.code, crate::error::ErrorCode::Schema));
        let after = platform::file_identity(&self.location.path)?;
        if self.binding.file.is_some() && after != self.binding.file {
            return Err(Error::Invalid(
                "store replaced during operation; stop the daemon and retry".into(),
            ));
        }
        self.binding.file = after;
        result
    }
}
