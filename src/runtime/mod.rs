//! Automatic local background runtime.
//! @ref LLP 0007#automatic-detached-lifecycle
mod dispatcher;
mod mesh;
mod mesh_driver;
#[cfg(unix)]
mod platform;
mod protocol;
mod wire;
use crate::{
    application::{Application, Caller, Operation, Reply, Value},
    error::{Error, Result},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use interprocess::local_socket::{
    GenericFilePath, ListenerOptions, ToFsName,
    tokio::{Listener, Stream, prelude::*},
};
#[cfg(target_os = "linux")]
use interprocess::os::unix::local_socket::ListenerOptionsExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    sync::{Semaphore, mpsc, oneshot},
    time::{Instant, timeout},
};
use wire::{receive_file, receive_json, send_file, send_json};
const WAIT: Duration = Duration::from_secs(30);
use protocol::{COMPATIBILITY, Frame, Hello, Invocation, ReplyEnvelope, VERSION};
pub use protocol::{DaemonState, DaemonVersion};
#[derive(Clone, Serialize, Deserialize)]
struct Endpoint {
    identity: platform::Identity,
    #[serde(with = "crate::native_path")]
    path: PathBuf,
    // Legacy endpoint readers require this field, but it no longer identifies a build.
    #[serde(default)]
    build: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runtime: Option<DaemonVersion>,
}
#[derive(Serialize, Deserialize)]
struct Bootstrap {
    #[serde(default = "mesh_startup_default")]
    mesh_startup: bool,
    endpoint: Endpoint,
    #[serde(with = "crate::native_path")]
    root: PathBuf,
    #[serde(with = "crate::native_path")]
    store: PathBuf,
    #[serde(with = "crate::native_path")]
    ready: PathBuf,
    token: String,
}
fn mesh_startup_default() -> bool {
    true
}
#[derive(Clone)]
struct Location {
    path: PathBuf,
    id: platform::Identity,
    key: String,
    root: PathBuf,
}
impl Location {
    fn new(path: &Path) -> Result<Self> {
        let id = platform::identity(path)?;
        let path = id.path.clone();
        if id.path.to_str().is_none() {
            return Err(Error::Invalid(
                "store path must be valid UTF-8 (required by libSQL)".into(),
            ));
        }
        let key = platform::key(&id)?;
        let root = platform::owner_root(&path)?;
        Ok(Self {
            path,
            id,
            key,
            root,
        })
    }
    fn file(&self, suffix: &str) -> PathBuf {
        self.root.join(format!("{}.{}", self.key, suffix))
    }
    fn endpoint(&self) -> Result<Endpoint> {
        let path = self.file("endpoint");
        let file = platform::private_file(&path)?;
        let mut bytes = Vec::new();
        file.take(16385).read_to_end(&mut bytes)?;
        if bytes.len() > 16384 {
            return Err(Error::Invalid("oversized daemon endpoint metadata".into()));
        }
        let e: Endpoint = serde_json::from_slice(&bytes)?;
        if e.identity != self.id {
            return Err(Error::Invalid("daemon identity collision".into()));
        }
        Ok(e)
    }
}
async fn wait_lock(file: &File) -> Result<()> {
    let until = Instant::now() + WAIT;
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => {
                if Instant::now() >= until {
                    return Err(Error::Invalid(
                        "timed out acquiring daemon startup lock".into(),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(std::fs::TryLockError::Error(e)) => return Err(e.into()),
        }
    }
}
fn live_owner(location: &Location) -> Result<bool> {
    let f = platform::private_file(&location.file("owner.lock"))?;
    match f.try_lock() {
        Ok(()) => Ok(false),
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(std::fs::TryLockError::Error(e)) => Err(e.into()),
    }
}
fn peer(stream: &Stream) -> Result<()> {
    if stream.peer_creds()?.euid() != Some(unsafe { libc::geteuid() }) {
        return Err(Error::Invalid(
            "local peer belongs to another account".into(),
        ));
    }
    Ok(())
}
fn listener(path: &Path) -> Result<Listener> {
    platform::check_socket(path)?;
    let options = ListenerOptions::new()
        .name(path.to_fs_name::<GenericFilePath>()?)
        .reclaim_name(false)
        .try_overwrite(false);
    #[cfg(target_os = "linux")]
    let options = options.mode(0o600);
    let listener = options.create_tokio()?;
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(listener)
}

async fn connect(location: &Location, control: bool) -> Result<Stream> {
    connect_endpoint(location, location.endpoint()?, control).await
}
async fn connect_endpoint(location: &Location, ep: Endpoint, control: bool) -> Result<Stream> {
    let mut stream = timeout(
        WAIT,
        Stream::connect(ep.path.to_fs_name::<GenericFilePath>()?),
    )
    .await
    .map_err(|_| Error::Invalid("daemon connection timed out".into()))??;
    peer(&stream)?;
    wire::send(
        &mut stream,
        &Frame::Hello(Hello {
            version: VERSION,
            build: String::new(),
            compatibility: COMPATIBILITY,
            schema: crate::storage::SCHEMA_VERSION,
            identity: location.id.clone(),
            store: location.path.clone(),
            control,
        }),
    )
    .await?;
    match timeout(WAIT, wire::receive::<Frame>(&mut stream))
        .await
        .map_err(|_| Error::Invalid("daemon handshake timed out".into()))??
    {
        Frame::Accepted => Ok(stream),
        Frame::Rejected(e) => Err(Error::Invalid(e)),
        _ => Err(Error::Invalid("invalid daemon handshake".into())),
    }
}
struct ReadySocket {
    path: PathBuf,
    identity: (u64, u64),
}
impl Drop for ReadySocket {
    fn drop(&mut self) {
        if std::fs::symlink_metadata(&self.path)
            .ok()
            .as_ref()
            .map(file_key)
            == Some(self.identity)
        {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
async fn launch(location: &Location, mesh_startup: bool) -> Result<()> {
    let root = platform::socket_root()?;
    let socket = root.join(format!("{}.sock", location.key));
    platform::check_socket(&socket)?;
    let mut random = [0; 16];
    File::open("/dev/urandom")?.read_exact(&mut random)?;
    let token = random
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let ready = root.join(format!("r-{}.sock", &token[..16]));
    let accept = listener(&ready)?;
    let ready_socket = ReadySocket {
        path: ready.clone(),
        identity: file_key(&std::fs::symlink_metadata(&ready)?),
    };
    let bootstrap = Bootstrap {
        mesh_startup,
        endpoint: Endpoint {
            identity: location.id.clone(),
            path: socket,
            build: String::new(),
            runtime: Some(DaemonVersion::current()),
        },
        root: location.root.clone(),
        store: location.path.clone(),
        ready: ready.clone(),
        token: token.clone(),
    };
    let config = STANDARD.encode(serde_json::to_vec(&bootstrap)?);
    let mut child = std::process::Command::new(std::env::current_exe()?)
        .arg("--__linn-bootstrap")
        .arg(config)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let result = timeout(WAIT, async {
        let mut stream = accept.accept().await?;
        peer(&stream)?;
        match wire::receive::<Frame>(&mut stream).await? {
            Frame::Ready {
                token: received,
                error,
            } if received == token => match error {
                Some(e) => Err(Error::Invalid(e)),
                None => Ok(()),
            },
            _ => Err(Error::Invalid("invalid daemon readiness message".into())),
        }
    })
    .await
    .map_err(|_| {
        Error::Invalid("daemon startup timed out; inspect daemon:status before retrying".into())
    });
    drop(accept);
    drop(ready_socket);
    // The bootstrap's first parent exits promptly; reap it without waiting on the daemon.
    if child.try_wait()?.is_none() {
        tokio::task::spawn_blocking(move || child.wait());
    }
    result?
}
async fn session(location: &Location, mesh_startup: bool) -> Result<Stream> {
    if live_owner(location)?
        && let Ok(ep) = location.endpoint()
        && !DaemonVersion::current().supersedes(ep.runtime.as_ref())
        && let Ok(stream) = connect_endpoint(location, ep, false).await
    {
        return Ok(stream);
    }
    // An owner can hold its lifetime lock before publishing readiness metadata.
    // Serialize with the initiating client before deciding that connection failed.
    let start = platform::private_file(&location.file("start.lock"))?;
    wait_lock(&start).await?;
    if live_owner(location)? {
        // Metadata is only a hint. Authenticate and inspect the live owner again
        // under start.lock before issuing a graceful stop; never replay a request.
        let state = control_owner(location, false).await?;
        if state.running && DaemonVersion::current().supersedes(state.runtime.as_ref()) {
            let old = state
                .runtime
                .as_ref()
                .map(|v| v.release.as_str())
                .unwrap_or("legacy version");
            eprintln!(
                "linn: restarting older background process ({old} → {}).",
                env!("CARGO_PKG_VERSION")
            );
            control_owner(location, true).await?;
        }
    }
    if !live_owner(location)? {
        launch(location, mesh_startup).await?;
    }
    connect(location, false).await
}
pub async fn execute(
    path: &Path,
    operation: Operation,
    caller: Caller,
    input: impl Into<Option<File>>,
) -> Result<Reply> {
    execute_with_progress(path, operation, caller, input, |_| {}).await
}
pub async fn execute_with_progress(
    path: &Path,
    operation: Operation,
    caller: Caller,
    input: impl Into<Option<File>>,
    progress: impl FnMut(&crate::mesh::status::Progress),
) -> Result<Reply> {
    let input = input.into();
    let location = Location::new(path)?;
    let mesh_startup = !matches!(
        operation,
        Operation::Mesh(crate::mesh::pairing::Command::Reset)
    );
    let mut stream = session(&location, mesh_startup).await?;
    exchange_with_progress(
        &mut stream,
        Invocation { operation, caller },
        input,
        progress,
    )
    .await
}
#[cfg(test)]
async fn exchange(
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
    invocation: Invocation,
    input: Option<File>,
) -> Result<Reply> {
    exchange_with_progress(stream, invocation, input, |_| {}).await
}
async fn exchange_with_progress(
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
    invocation: Invocation,
    mut input: Option<File>,
    mut progress: impl FnMut(&crate::mesh::status::Progress),
) -> Result<Reply> {
    let invocation = std::sync::Arc::new(invocation);
    let mutating = invocation.operation.mutates();
    let mut admitted = None;
    let result = async {
        wire::send(stream, &Frame::Invoke { id: 1 }).await?;
        send_json(stream, invocation.clone()).await?;
        if let Some(input) = &mut input {
            send_file(stream, input).await?;
        } else {
            wire::send(stream, &Frame::End).await?;
        }
        loop {
            match wire::receive::<Frame>(stream).await? {
                Frame::Progress(event) => {
                    invocation.validate_progress(&event)?;
                    if let crate::mesh::status::Progress::Admitted(status) = &event {
                        if admitted.is_some() {
                            return Err(Error::Invalid("duplicate admission progress".into()));
                        }
                        admitted = Some(status.clone());
                    }
                    progress(&event);
                }
                Frame::Reply { id: 1 } => {
                    let envelope = receive_json(stream).await?;
                    invocation.validate_reply(&envelope)?;
                    let ReplyEnvelope {
                        value,
                        warnings,
                        output,
                    } = envelope;
                    let output = if output {
                        let mut file = tempfile::tempfile()?;
                        receive_file(stream, &mut file).await?;
                        file.seek(SeekFrom::Start(0))?;
                        Some(file)
                    } else {
                        None
                    };
                    return Ok(Reply {
                        value,
                        warnings,
                        output,
                    });
                }
                Frame::Failure(error) => return Err(error.into()),
                Frame::Rejected(message) => {
                    return Err(crate::error::ApplicationError::new(
                        Error::Invalid(message),
                        false,
                    )
                    .into());
                }
                _ => return Err(Error::Invalid("invalid daemon response".into())),
            }
        }
    }
    .await;
    let result = result.map_err(|error| match error {
        Error::Application(_) => error,
        error => {
            let mut error = crate::error::ApplicationError::new(error, mutating);
            error.message = format!(
                "daemon response incomplete or invalid; {}{}",
                if mutating {
                    "mutation may have committed; inspect the store before retrying: "
                } else {
                    ""
                },
                error.message
            );
            error.into()
        }
    });
    match (result, admitted) {
        (Err(error), Some(status)) => Ok(Reply::from(Value::Mesh(
            crate::mesh::pairing::Output::Changed {
                operation: crate::mesh::pairing::Change::Paired,
                status,
                initial_sync: None,
                initial_sync_error: Some(Box::new(crate::error::ApplicationError::new(
                    error, true,
                ))),
            },
        ))),
        (result, _) => result,
    }
}
pub async fn control(path: &Path, stop: bool) -> Result<DaemonState> {
    let location = Location::new(path)?;
    let start = platform::private_file(&location.file("start.lock"))?;
    wait_lock(&start).await?;
    if !live_owner(&location)? {
        return Ok(DaemonState::stopped());
    }
    control_owner(&location, stop).await
}
async fn control_owner(location: &Location, stop: bool) -> Result<DaemonState> {
    let result = async {
        let mut stream = connect(location, true).await?;
        wire::send(&mut stream, &if stop { Frame::Stop } else { Frame::Status }).await?;
        match wire::receive::<Frame>(&mut stream).await? {
            Frame::State(state) => {
                if stop {
                    let owner = platform::private_file(&location.file("owner.lock"))?;
                    wait_lock(&owner).await?;
                }
                Ok(state)
            }
            _ => Err(Error::Invalid("invalid daemon control response".into())),
        }
    }
    .await;
    match result {
        Err(Error::Io(error))
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::UnexpectedEof
            ) =>
        {
            // Retirement can remove the socket between owner inspection and
            // connect. Holding start.lock excludes a replacement owner.
            let owner = platform::private_file(&location.file("owner.lock"))?;
            wait_lock(&owner).await?;
            Ok(DaemonState::stopped())
        }
        result => result,
    }
}
struct Job {
    store: PathBuf,
    operation: Operation,
    caller: Caller,
    input: Option<File>,
    reply: oneshot::Sender<Result<Reply>>,
    delivered: oneshot::Receiver<()>,
    progress: mpsc::Sender<crate::mesh::status::Progress>,
}
async fn client(
    mut stream: Stream,
    id: platform::Identity,
    queue: mpsc::Sender<Job>,
    stop: mpsc::Sender<dispatcher::StopReply>,
    requests: std::sync::Arc<Semaphore>,
) -> Result<()> {
    peer(&stream)?;
    let Frame::Hello(hello) = timeout(WAIT, wire::receive(&mut stream))
        .await
        .map_err(|_| Error::Invalid("handshake timed out".into()))??
    else {
        return Err(Error::Invalid("missing hello".into()));
    };
    if hello.version != VERSION
        || hello.identity != id
        || (!hello.control
            && (hello.compatibility != COMPATIBILITY
                || hello.schema != crate::storage::SCHEMA_VERSION))
    {
        wire::send(&mut stream,&Frame::Rejected("incompatible daemon protocol/compatibility/schema/store; use a current Linn CLI, or run linn daemon:stop explicitly before switching versions".into())).await?;
        return Ok(());
    }
    // Reserve before receiving potentially large input. Control requests have a
    // separate connection budget and never wait behind application admission.
    let _request = if hello.control {
        None
    } else {
        match requests.try_acquire_owned() {
            Ok(permit) => Some(permit),
            Err(_) => {
                timeout(
                    WAIT,
                    wire::send(
                        &mut stream,
                        &Frame::Rejected("daemon busy; request not executed".into()),
                    ),
                )
                .await
                .map_err(|_| Error::Invalid("client handshake stalled".into()))??;
                return Ok(());
            }
        }
    };
    wire::send(&mut stream, &Frame::Accepted).await?;
    let frame = timeout(WAIT, wire::receive::<Frame>(&mut stream))
        .await
        .map_err(|_| Error::Invalid("request timed out".into()))??;
    if hello.control {
        return match frame {
            Frame::Status => {
                wire::send(
                    &mut stream,
                    &Frame::State(DaemonState {
                        running: true,
                        pid: Some(std::process::id()),
                        runtime: Some(DaemonVersion::current()),
                    }),
                )
                .await
            }
            Frame::Stop => {
                let (tx, rx) = oneshot::channel();
                let (done, delivered) = oneshot::channel();
                stop.send((tx, delivered))
                    .await
                    .map_err(|_| Error::Invalid("daemon stopping".into()))?;
                rx.await
                    .map_err(|_| Error::Invalid("daemon stopped".into()))?;
                let result = wire::send(&mut stream, &Frame::State(DaemonState::stopped())).await;
                let _ = done.send(());
                result
            }
            _ => Err(Error::Invalid("invalid control request".into())),
        };
    }
    let Frame::Invoke { id: 1 } = frame else {
        return Err(Error::Invalid("missing operation".into()));
    };
    let Invocation { operation, caller } = receive_json(&mut stream).await?;
    let mutating = operation.mutates();
    let input = if operation.takes_input() {
        let mut input = tempfile::tempfile()?;
        receive_file(&mut stream, &mut input).await?;
        input.seek(SeekFrom::Start(0))?;
        Some(input)
    } else {
        if !matches!(
            timeout(WAIT, wire::receive::<Frame>(&mut stream))
                .await
                .map_err(|_| Error::Invalid("request input stalled".into()))??,
            Frame::End
        ) {
            return Err(Error::Invalid("unexpected operation input".into()));
        }
        None
    };
    let (tx, mut rx) = oneshot::channel();
    let (progress, mut events) = mpsc::channel(2);
    let (done, delivered) = oneshot::channel();
    if queue
        .try_send(Job {
            store: hello.store,
            operation,
            caller,
            input,
            reply: tx,
            delivered,
            progress,
        })
        .is_err()
    {
        wire::send(
            &mut stream,
            &Frame::Rejected("daemon busy or stopping; request not executed".into()),
        )
        .await?;
        return Ok(());
    }
    let reply = loop {
        tokio::select! {
            reply = &mut rx => break reply.map_err(|_| Error::Invalid("daemon stopped before returning a reply".into()))?,
            Some(event) = events.recv() => {
                timeout(WAIT, wire::send(&mut stream, &Frame::Progress(event))).await.map_err(|_| Error::Invalid("client progress stalled".into()))??;
            },
            _ = stream.read_u8() => return Err(Error::Invalid("client disconnected or sent unexpected data".into())),
        }
    };
    let response = async {
        while let Ok(event) = events.try_recv() {
            wire::send(&mut stream, &Frame::Progress(event)).await?;
        }
        match reply {
            Ok(reply) => {
                wire::send(&mut stream, &Frame::Reply { id: 1 }).await?;
                send_json(
                    &mut stream,
                    ReplyEnvelope {
                        value: reply.value,
                        warnings: reply.warnings,
                        output: reply.output.is_some(),
                    },
                )
                .await?;
                if let Some(mut output) = reply.output {
                    send_file(&mut stream, &mut output).await?;
                }
                Ok(())
            }
            Err(e) => {
                wire::send(
                    &mut stream,
                    &Frame::Failure(crate::error::ApplicationError::new(e, mutating)),
                )
                .await
            }
        }
    };
    let result = timeout(WAIT, response)
        .await
        .map_err(|_| Error::Invalid("client response stalled".into()))?;
    let _ = done.send(());
    result
}
async fn daemon(config: Bootstrap) -> Result<()> {
    let location = Location::new(&config.store)?;
    if location.root != config.root || location.id != config.endpoint.identity {
        return Err(Error::Invalid("bootstrap store identity changed".into()));
    }
    let root_guard = File::open(&location.root)?;
    let root_identity = file_key(&root_guard.metadata()?);
    let owner = platform::private_file(&location.file("owner.lock"))?;
    owner
        .try_lock()
        .map_err(|e| Error::Invalid(format!("daemon already owned: {e}")))?;
    if let Ok(meta) = std::fs::symlink_metadata(&config.endpoint.path) {
        use std::os::unix::fs::FileTypeExt;
        if !meta.file_type().is_socket() {
            return Err(Error::Invalid(
                "refusing to replace a non-socket runtime endpoint".into(),
            ));
        }
        std::fs::remove_file(&config.endpoint.path)?;
    }
    let owned_file = platform::file_identity(&location.path)?;
    let accept = listener(&config.endpoint.path)?;
    let socket_identity = file_key(&std::fs::symlink_metadata(&config.endpoint.path)?);
    let lock_identity = file_key(&owner.metadata()?);
    let mut metadata = tempfile::NamedTempFile::new_in(&location.root)?;
    metadata.write_all(&serde_json::to_vec(&config.endpoint)?)?;
    metadata
        .persist(location.file("endpoint"))
        .map_err(|e| Error::Io(e.error))?;
    // Readiness acknowledges the initiator, but ownership does not depend on
    // that client remaining alive. Other clients can use the published endpoint.
    if let Ok(mut ready) = Stream::connect(config.ready.to_fs_name::<GenericFilePath>()?).await {
        peer(&ready)?;
        let _ = wire::send(
            &mut ready,
            &Frame::Ready {
                token: config.token,
                error: None,
            },
        )
        .await;
    }
    let (queue, mut incoming) = mpsc::channel::<Job>(64);
    let (writes, jobs) = mpsc::channel::<Job>(64);
    let (reads, read_jobs) = mpsc::channel::<Job>(64);
    // Route on the IPC runtime: synchronous indexing must not delay read admission.
    let router = tokio::task::spawn_local(async move {
        while let Some(job) = incoming.recv().await {
            let target = if matches!(
                job.operation,
                Operation::Search { .. }
                    | Operation::More { .. }
                    | Operation::Show { .. }
                    | Operation::Outline { .. }
                    | Operation::List(_)
                    | Operation::Export(_)
            ) {
                &reads
            } else {
                &writes
            };
            if let Err(error) = target.try_send(job) {
                let job = error.into_inner();
                let _ = job.reply.send(Err(crate::error::ApplicationError::new(
                    Error::Invalid("daemon busy or stopping; request not executed".into()),
                    false,
                )
                .into()));
            }
        }
    });
    let (stop, stopping) = mpsc::channel(1);
    let identity = location.id.clone();
    let socket = config.endpoint.path.clone();
    let requests = std::sync::Arc::new(Semaphore::new(64));
    let connections = std::sync::Arc::new(Semaphore::new(128));
    let accept_task = tokio::task::spawn_local(async move {
        while let Ok(stream) = accept.accept().await {
            let Ok(connection) = connections.clone().try_acquire_owned() else {
                // Do not allocate an unbounded task for clients that never hello.
                drop(stream);
                continue;
            };
            let requests = requests.clone();
            let queue = queue.clone();
            let stop = stop.clone();
            let id = identity.clone();
            tokio::task::spawn_local(async move {
                let _connection = connection;
                let _ = client(stream, id, queue, stop, requests).await;
            });
        }
    });
    let dispatch_location = location.clone();
    let listener = accept_task.abort_handle();
    let binding = dispatcher::Binding {
        root: root_identity,
        lock: lock_identity,
        socket: socket_identity,
        socket_path: socket.clone(),
        file: std::sync::Arc::new(std::sync::Mutex::new(owned_file)),
    };
    let read_barrier = crate::storage::read_barrier();
    let reader_location = location.clone();
    let reader_binding = binding.clone();
    let reader_barrier = read_barrier.clone();
    let (reader_failure, reader_failed) = mpsc::channel(1);
    let reader = tokio::task::spawn_blocking(move || -> Result<()> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(dispatcher::read_run(
            reader_location,
            reader_binding,
            read_jobs,
            reader_barrier,
            reader_failure,
        ));
        Ok(())
    });
    let network = tokio::runtime::Handle::current();
    let mesh_startup = config.mesh_startup;
    let dispatched = tokio::task::spawn_blocking(move || -> Result<_> {
        // libSQL's local async methods and native encoding execute synchronously.
        // Build and retain every Store on this thread; keep IPC independently polled.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(dispatcher::run(
            dispatcher::Owner {
                location: dispatch_location,
                binding,
                accept_task: listener,
            },
            jobs,
            stopping,
            network,
            mesh_startup,
            read_barrier,
            reader_failed,
        ))
    })
    .await;
    accept_task.abort();
    router.abort();
    let _ = router.await;
    reader
        .await
        .map_err(|error| Error::Invalid(format!("store reader failed: {error}")))??;
    let stop_reply = dispatched
        .map_err(|error| Error::Invalid(format!("store dispatcher failed: {error}")))??;
    if std::fs::symlink_metadata(&socket)
        .ok()
        .as_ref()
        .map(file_key)
        == Some(socket_identity)
    {
        let _ = std::fs::remove_file(&socket);
    }
    // Stale endpoint metadata is harmless without a live owner and is atomically
    // replaced on restart. Keeping it avoids unlinking a replacement after root loss.
    drop(owner);
    if let Some((reply, delivered)) = stop_reply {
        let _ = reply.send(());
        let _ = timeout(WAIT, delivered).await;
    }
    Ok(())
}
fn file_key(meta: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (meta.dev(), meta.ino())
}
/// Called before constructing an async runtime; the internal child is never a CLI command.
pub fn bootstrap() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--__linn-bootstrap")) {
        return None;
    }
    let result = (|| -> Result<()> {
        let value = args
            .next()
            .ok_or_else(|| Error::Invalid("missing bootstrap".into()))?;
        let config: Bootstrap = serde_json::from_slice(
            &STANDARD
                .decode(value.to_string_lossy().as_bytes())
                .map_err(|_| Error::Invalid("invalid bootstrap".into()))?,
        )?;
        if let Err(error) = platform::detach() {
            // Detachment can fail before Tokio exists. Return the actual setup
            // error through the same private readiness endpoint synchronously.
            use interprocess::local_socket::traits::Stream as _;
            if let Ok(name) = config.ready.to_fs_name::<GenericFilePath>()
                && let Ok(mut stream) = interprocess::local_socket::Stream::connect(name)
                && let Ok(bytes) = serde_json::to_vec(&Frame::Ready {
                    token: config.token.clone(),
                    error: Some(error.to_string()),
                })
            {
                let _ = stream
                    .write_all(&(bytes.len() as u32).to_be_bytes())
                    .and_then(|_| stream.write_all(&bytes))
                    .and_then(|_| stream.flush());
            }
            return Err(error);
        }
        // The initiating shell's temporary directory can disappear while this
        // process remains resident. Anonymous request spools use our owned root.
        unsafe {
            std::env::set_var("TMPDIR", &config.root);
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let local = tokio::task::LocalSet::new();
        rt.block_on(local.run_until(async {
            let error_ready = config.ready.clone();
            let token = config.token.clone();
            let result = daemon(config).await;
            if let Err(e) = &result
                && let Ok(mut s) =
                    Stream::connect(error_ready.to_fs_name::<GenericFilePath>()?).await
            {
                let _ = wire::send(
                    &mut s,
                    &Frame::Ready {
                        token,
                        error: Some(e.to_string()),
                    },
                )
                .await;
            }
            result
        }))
    })();
    Some(if result.is_ok() { 0 } else { 2 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    #[tokio::test]
    async fn framing_rejects_oversize_and_truncation() {
        let (mut tx, mut rx) = tokio::io::duplex(8);
        tx.write_u32((wire::MAX_FRAME + 1) as u32).await.unwrap();
        assert!(wire::receive::<Frame>(&mut rx).await.is_err());
        let (mut tx, mut rx) = tokio::io::duplex(16);
        tx.write_u32(12).await.unwrap();
        tx.write_all(b"{}").await.unwrap();
        drop(tx);
        assert!(wire::receive::<Frame>(&mut rx).await.is_err());
    }
    #[tokio::test]
    async fn streamed_values_have_no_single_frame_record_limit() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::Builder::new()
            .prefix("lw-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.path().join("s");
        let accept = listener(&path).unwrap();
        let target = path.clone();
        let client = async move {
            let mut stream = Stream::connect(target.to_fs_name::<GenericFilePath>().unwrap())
                .await
                .unwrap();
            for n in [
                0,
                wire::CHUNK_BYTES - 2,
                wire::CHUNK_BYTES - 1,
                wire::MAX_FRAME + 1,
            ] {
                let value: String = receive_json(&mut stream).await.unwrap();
                assert_eq!(value.len(), n);
                assert!(value.bytes().all(|b| b == b'x'));
            }
        };
        let server = async {
            let mut stream = accept.accept().await.unwrap();
            for n in [
                0,
                wire::CHUNK_BYTES - 2,
                wire::CHUNK_BYTES - 1,
                wire::MAX_FRAME + 1,
            ] {
                send_json(&mut stream, "x".repeat(n)).await.unwrap();
            }
        };
        tokio::join!(client, server);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn private_files_reject_symlinks_and_insecure_modes() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        std::fs::write(&target, b"").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
        let link = dir.path().join("link");
        symlink(&target, &link).unwrap();
        assert!(platform::private_file(&link).is_err());
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(platform::private_file(&target).is_err());
    }
    #[test]
    fn socket_keys_are_short_and_distinguish_routing_paths() {
        let a = platform::Identity {
            account: 1,
            path: "/a/store.db".into(),
        };
        let b = platform::Identity {
            path: "/b/store.db".into(),
            ..a.clone()
        };
        assert_eq!(platform::key(&a).unwrap(), "xjqkf45vfjzexen7zxbdvgodby");
        assert_ne!(platform::key(&a).unwrap(), platform::key(&b).unwrap());
        assert!(platform::check_socket(Path::new(&format!("/tmp/{}", "x".repeat(200)))).is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn published_owner_survives_missing_initiating_client() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::Builder::new()
            .prefix("la-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let store = dir.path().join("store.db");
        let location = Location::new(&store).unwrap();
        let config = Bootstrap {
            mesh_startup: true,
            endpoint: Endpoint {
                identity: location.id.clone(),
                path: dir.path().join("s"),
                build: String::new(),
                runtime: Some(DaemonVersion::current()),
            },
            root: location.root,
            store: location.path,
            ready: dir.path().join("initiator-already-gone"),
            token: "test".into(),
        };
        tokio::task::LocalSet::new()
            .run_until(async {
                let task = tokio::task::spawn_local(daemon(config));
                timeout(WAIT, async {
                    loop {
                        if let Ok(state) = control(&store, false).await
                            && state.running
                        {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert!(!control(&store, true).await.unwrap().running);
                task.await.unwrap().unwrap();
            })
            .await;
    }

    #[test]
    fn routing_missing_stores_and_dangling_aliases_never_creates_database() {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("nested/store.db");
        let alias = dir.path().join("alias.db");
        std::os::unix::fs::symlink("nested/store.db", &alias).unwrap();
        let direct = Location::new(&store).unwrap();
        let indirect = Location::new(&alias).unwrap();
        assert_eq!(direct.id, indirect.id);
        assert_eq!(direct.key, indirect.key);
        assert!(!store.exists());
        assert_eq!(platform::file_identity(&store).unwrap(), None);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn admission_is_bounded_before_upload_and_control_bypasses_request_slots() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::Builder::new()
            .prefix("lc-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = dir.path().join("s");
        let accept = listener(&socket).unwrap();
        let identity = platform::identity(&dir.path().join("store.db")).unwrap();
        let requests = std::sync::Arc::new(Semaphore::new(1));
        let (queue, _jobs) = mpsc::channel(1);
        let (stop, _stopping) = mpsc::channel(1);
        tokio::task::LocalSet::new().run_until(async {
            let server_id = identity.clone();
            let server = tokio::task::spawn_local(async move {
                let mut clients = Vec::new();
                for _ in 0..3 {
                    let stream = accept.accept().await.unwrap();
                    clients.push(tokio::task::spawn_local(client(stream, server_id.clone(),
                        queue.clone(), stop.clone(), requests.clone())));
                }
                for task in clients { let _ = task.await.unwrap(); }
            });
            let connect = |control| {
                let socket = &socket;
                let identity = &identity;
                async move {
                    let mut stream = Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap()).await.unwrap();
                    wire::send(&mut stream, &Frame::Hello(Hello { version: VERSION, build: String::new(), compatibility: COMPATIBILITY, schema: crate::storage::SCHEMA_VERSION,
                        identity: identity.clone(), store: identity.path.clone(), control })).await.unwrap();
                    stream
                }
            };
            let mut stalled = connect(false).await;
            assert!(matches!(wire::receive(&mut stalled).await.unwrap(), Frame::Accepted));
            let mut overflow = connect(false).await;
            assert!(matches!(wire::receive(&mut overflow).await.unwrap(), Frame::Rejected(message) if message.contains("not executed")));
            let mut control = connect(true).await;
            assert!(matches!(wire::receive(&mut control).await.unwrap(), Frame::Accepted));
            wire::send(&mut control, &Frame::Status).await.unwrap();
            assert!(matches!(wire::receive(&mut control).await.unwrap(), Frame::State(state) if state.running));
            drop(stalled);
            server.await.unwrap();
        }).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn response_delivery_barrier_remains_held_until_a_stalled_stream_is_closed() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::Builder::new()
            .prefix("ld-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = dir.path().join("s");
        let accept = listener(&socket).unwrap();
        let identity = platform::identity(&dir.path().join("store.db")).unwrap();
        let (queue, mut jobs) = mpsc::channel(1);
        let (stop, _stopping) = mpsc::channel(1);
        tokio::task::LocalSet::new()
            .run_until(async {
                let server_id = identity.clone();
                let server = tokio::task::spawn_local(async move {
                    let stream = accept.accept().await.unwrap();
                    client(
                        stream,
                        server_id,
                        queue,
                        stop,
                        std::sync::Arc::new(Semaphore::new(1)),
                    )
                    .await
                });
                let mut stream =
                    Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap())
                        .await
                        .unwrap();
                wire::send(
                    &mut stream,
                    &Frame::Hello(Hello {
                        version: VERSION,
                        build: String::new(),
                        compatibility: COMPATIBILITY,
                        schema: crate::storage::SCHEMA_VERSION,
                        identity: identity.clone(),
                        store: identity.path,
                        control: false,
                    }),
                )
                .await
                .unwrap();
                assert!(matches!(
                    wire::receive(&mut stream).await.unwrap(),
                    Frame::Accepted
                ));
                wire::send(&mut stream, &Frame::Invoke { id: 1 })
                    .await
                    .unwrap();
                send_json(
                    &mut stream,
                    Invocation {
                        operation: Operation::Check,
                        caller: Caller {
                            scope: None,
                            writer: None,
                            diagnostics: Default::default(),
                        },
                    },
                )
                .await
                .unwrap();
                wire::send(&mut stream, &Frame::End).await.unwrap();
                let job = jobs.recv().await.unwrap();
                let mut delivered = job.delivered;
                let output = tempfile::tempfile().unwrap();
                output.set_len(64 * 1024 * 1024).unwrap();
                job.reply
                    .send(Ok(Reply {
                        value: Value::Checked,
                        warnings: vec![],
                        output: Some(output),
                    }))
                    .ok()
                    .unwrap();
                assert!(matches!(
                    wire::receive(&mut stream).await.unwrap(),
                    Frame::Reply { id: 1 }
                ));
                let envelope: ReplyEnvelope = receive_json(&mut stream).await.unwrap();
                assert!(envelope.output);
                tokio::time::sleep(Duration::from_millis(20)).await;
                assert!(
                    matches!(
                        delivered.try_recv(),
                        Err(oneshot::error::TryRecvError::Empty)
                    ),
                    "the dispatcher must not pass the barrier while bytes remain queued"
                );
                drop(stream);
                assert!(server.await.unwrap().is_err());
                let _ = timeout(WAIT, delivered).await.unwrap();
            })
            .await;
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn native_descriptor_list_includes_sparse_high_numbers() {
        use std::os::fd::{AsRawFd, FromRawFd};
        let file = tempfile::tempfile().unwrap();
        let high = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 4096) };
        assert!(high >= 4096);
        let high = unsafe { std::os::fd::OwnedFd::from_raw_fd(high) };
        let descriptors = platform::open_descriptors().unwrap();
        assert!(descriptors.contains(&file.as_raw_fd()));
        assert!(descriptors.contains(&high.as_raw_fd()));
    }

    #[tokio::test]
    async fn rpc_rejects_invalid_replies_and_preserves_mutation_uncertainty() {
        use crate::error::{ApplicationError, ErrorCode};
        for case in 0..10 {
            let (mut caller, mut daemon) = tokio::io::duplex(1024);
            let request = Invocation {
                operation: if case >= 8 {
                    Operation::Export(crate::application::ExportOptions {
                        scope: None,
                        kind: None,
                        since: None,
                        id: vec![],
                        all_states: false,
                    })
                } else {
                    Operation::Rebuild
                },
                caller: Caller {
                    diagnostics: Default::default(),
                    scope: None,
                    writer: None,
                },
            };
            let client = exchange(&mut caller, request, None);
            let server = async {
                assert!(matches!(
                    wire::receive(&mut daemon).await.unwrap(),
                    Frame::Invoke { id: 1 }
                ));
                let _: Invocation = receive_json(&mut daemon).await.unwrap();
                receive_file(&mut daemon, &mut tempfile::tempfile().unwrap())
                    .await
                    .unwrap();
                match case {
                    2 => wire::send(&mut daemon, &Frame::Reply { id: 2 })
                        .await
                        .unwrap(),
                    3 => {
                        wire::send(&mut daemon, &Frame::Reply { id: 1 })
                            .await
                            .unwrap();
                        wire::send(&mut daemon, &Frame::Chunk(STANDARD.encode(b"{")))
                            .await
                            .unwrap();
                        wire::send(&mut daemon, &Frame::End).await.unwrap();
                    }
                    4 => {}
                    5 => wire::send(&mut daemon, &Frame::Rejected("busy; not executed".into()))
                        .await
                        .unwrap(),
                    6 => {
                        let mut error =
                            ApplicationError::new(Error::Invalid("invalid request".into()), false);
                        error.diagnostics.push("preserved diagnostic".into());
                        wire::send(&mut daemon, &Frame::Failure(error))
                            .await
                            .unwrap();
                    }
                    _ => {
                        wire::send(&mut daemon, &Frame::Reply { id: 1 })
                            .await
                            .unwrap();
                        send_json(
                            &mut daemon,
                            ReplyEnvelope {
                                value: match case {
                                    1 => Value::Checked,
                                    8 | 9 => Value::Exported,
                                    _ => Value::Rebuilt,
                                },
                                warnings: vec![],
                                output: case >= 7,
                            },
                        )
                        .await
                        .unwrap();
                        if case == 9 {
                            let mut file = tempfile::tempfile().unwrap();
                            file.write_all(b"original bytes").unwrap();
                            file.rewind().unwrap();
                            send_file(&mut daemon, &mut file).await.unwrap();
                        }
                    }
                }
                drop(daemon);
            };
            let (result, ()) = tokio::join!(client, server);
            if case == 0 || case == 9 {
                let reply = result.unwrap();
                if case == 9 {
                    let mut bytes = Vec::new();
                    reply.output.unwrap().read_to_end(&mut bytes).unwrap();
                    assert_eq!(bytes, b"original bytes");
                }
            } else {
                let Err(Error::Application(error)) = result else {
                    panic!("case {case} did not produce a typed error")
                };
                assert_eq!(
                    error.mutation_may_have_committed,
                    !matches!(case, 5 | 6 | 8),
                    "case {case}"
                );
                if case == 6 {
                    assert_eq!(error.diagnostics, ["preserved diagnostic"]);
                    assert!(matches!(error.code, ErrorCode::Invalid));
                }
                if case == 3 {
                    assert!(matches!(error.code, ErrorCode::Invalid));
                }
                if case == 4 || case == 8 {
                    assert!(matches!(error.code, ErrorCode::Io));
                }
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn routing_uses_native_spelling_on_case_insensitive_filesystems() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("MixedCase");
        std::fs::create_dir(&root).unwrap();
        let alias = dir.path().join("mixedcase");
        if !alias.exists() {
            return; // Case-sensitive volumes correctly treat these as different paths.
        }
        let missing = root.join("missing.db");
        assert_eq!(
            platform::resolve_store(&missing).unwrap(),
            platform::resolve_store(&alias.join("missing.db")).unwrap()
        );
        assert!(!missing.exists());
        let store = root.join("Store.db");
        std::fs::write(&store, b"").unwrap();
        let first = Location::new(&store).unwrap();
        let second = Location::new(&alias.join("store.DB")).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(first.key, second.key);
    }

    #[tokio::test]
    async fn handshake_checks_compatibility_and_schema_but_keeps_legacy_control() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::Builder::new()
            .prefix("lv-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = dir.path().join("s");
        let accept = listener(&socket).unwrap();
        let identity = platform::identity(&dir.path().join("store.db")).unwrap();
        let (queue, _jobs) = mpsc::channel(1);
        let (stop, _stopping) = mpsc::channel(1);
        let schema = crate::storage::SCHEMA_VERSION;
        for (version, compatibility, schema, build, control, accepted) in [
            (
                VERSION,
                Some(COMPATIBILITY),
                Some(schema),
                "different-build-a",
                false,
                true,
            ),
            (
                VERSION,
                Some(COMPATIBILITY),
                Some(schema),
                "different-build-b",
                false,
                true,
            ),
            (
                VERSION,
                Some(COMPATIBILITY),
                Some(schema + 1),
                "",
                false,
                false,
            ),
            (
                VERSION,
                Some(COMPATIBILITY + 1),
                Some(schema),
                "",
                false,
                false,
            ),
            (
                VERSION,
                Some(COMPATIBILITY - 1),
                Some(schema),
                "",
                false,
                false,
            ),
            (VERSION, None, Some(schema), "old-build", false, false),
            (VERSION, Some(COMPATIBILITY), None, "", false, false),
            (VERSION, None, None, "old-build", true, true),
            (
                VERSION,
                Some(COMPATIBILITY + 1),
                Some(schema + 1),
                "future-build",
                true,
                true,
            ),
            (
                VERSION + 1,
                Some(COMPATIBILITY),
                Some(schema),
                "",
                true,
                false,
            ),
        ] {
            let server = async {
                let stream = accept.accept().await.unwrap();
                let _ = client(
                    stream,
                    identity.clone(),
                    queue.clone(),
                    stop.clone(),
                    std::sync::Arc::new(Semaphore::new(1)),
                )
                .await;
            };
            let caller = async {
                let mut stream =
                    Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap())
                        .await
                        .unwrap();
                let mut hello = serde_json::json!({"version":version,"build":build,"identity":identity,"store":identity.path,"control":control});
                if let Some(schema) = schema {
                    hello["schema"] = schema.into();
                }
                if let Some(compatibility) = compatibility {
                    hello["compatibility"] = compatibility.into();
                }
                wire::send(&mut stream, &serde_json::json!({"Hello":hello}))
                    .await
                    .unwrap();
                let frame: Frame = wire::receive(&mut stream).await.unwrap();
                assert_eq!(matches!(frame, Frame::Accepted), accepted);
                if accepted && control {
                    wire::send(&mut stream, &Frame::Status).await.unwrap();
                    assert!(
                        matches!(wire::receive(&mut stream).await.unwrap(), Frame::State(state) if state.running)
                    );
                }
            };
            tokio::join!(server, caller);
        }
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn json_streaming_does_not_require_a_temporary_directory() {
        if std::env::var_os("LINN_TEST_RPC_NO_TMPDIR").is_none() {
            let dir = tempfile::tempdir().unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "runtime::tests::json_streaming_does_not_require_a_temporary_directory",
                ])
                .env("LINN_TEST_RPC_NO_TMPDIR", "1")
                .env("TMPDIR", dir.path().join("missing"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
            return;
        }
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            for n in [
                0,
                wire::CHUNK_BYTES - 2,
                wire::CHUNK_BYTES - 1,
                wire::MAX_FRAME + 1,
            ] {
                let (mut tx, mut rx) = tokio::io::duplex(1024);
                let (sent, received) = tokio::join!(
                    send_json(&mut tx, "x".repeat(n)),
                    receive_json::<String>(&mut rx)
                );
                sent.unwrap();
                assert_eq!(received.unwrap(), "x".repeat(n));
            }
            let (mut tx, mut rx) = tokio::io::duplex(1024);
            let sender =
                tokio::spawn(
                    async move { send_json(&mut tx, "x".repeat(wire::MAX_FRAME + 1)).await },
                );
            assert!(matches!(
                wire::receive::<Frame>(&mut rx).await.unwrap(),
                Frame::Chunk(_)
            ));
            drop(rx);
            assert!(
                timeout(Duration::from_secs(2), sender)
                    .await
                    .unwrap()
                    .unwrap()
                    .is_err()
            );
        });
    }

    #[test]
    fn native_paths_round_trip_across_startup_handshake_and_file_input() {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
        let path = PathBuf::from(OsStr::from_bytes(b"/native-\xff/path"));
        let identity = platform::Identity {
            account: 42,
            path: path.clone(),
        };
        let config = Bootstrap {
            mesh_startup: true,
            endpoint: Endpoint {
                identity: identity.clone(),
                path: path.clone(),
                build: String::new(),
                runtime: Some(DaemonVersion::current()),
            },
            root: path.clone(),
            store: path.clone(),
            ready: path.clone(),
            token: "test".into(),
        };
        let encoded = serde_json::to_vec(&config).unwrap();
        let decoded: Bootstrap = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.endpoint.identity, identity);
        for actual in [
            decoded.endpoint.path,
            decoded.root,
            decoded.store,
            decoded.ready,
        ] {
            assert_eq!(actual, path);
        }
        let hello = Hello {
            version: VERSION,
            compatibility: COMPATIBILITY,
            build: String::new(),
            schema: crate::storage::SCHEMA_VERSION,
            identity: identity.clone(),
            store: path.clone(),
            control: false,
        };
        let decoded: Hello = serde_json::from_slice(&serde_json::to_vec(&hello).unwrap()).unwrap();
        assert_eq!(decoded.identity, identity);
        assert_eq!(decoded.store, path);
        for repository in [None, Some(path.clone())] {
            let input = crate::application::FileInput {
                path: path.clone(),
                repository: repository.clone(),
            };
            let decoded: crate::application::FileInput =
                serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
            assert_eq!(decoded.path, path);
            assert_eq!(decoded.repository, repository);
        }
        let lossy = platform::Identity {
            path: path.to_string_lossy().into_owned().into(),
            ..identity.clone()
        };
        assert_ne!(
            platform::key(&identity).unwrap(),
            platform::key(&lossy).unwrap()
        );
        // UTF-8 routing must keep the key used by older owners, to avoid split ownership.
        let ascii = platform::Identity {
            account: 1,
            path: "/a/store.db".into(),
        };
        assert_eq!(
            serde_json::to_value(&ascii).unwrap(),
            serde_json::json!({"account":1,"path":"/a/store.db"})
        );
    }

    #[test]
    fn native_path_codec_rejects_foreign_encodings_and_retains_legacy_endpoint_shape() {
        assert!(
            serde_json::from_value::<crate::application::FileInput>(serde_json::json!({
                "path":{"Windows":[47,100]}, "repository":null
            }))
            .is_err()
        );
        #[derive(Deserialize)]
        struct LegacyEndpoint {
            identity: platform::Identity,
            path: PathBuf,
            build: String,
        }
        let endpoint = Endpoint {
            identity: platform::Identity {
                account: 1,
                path: "/store.db".into(),
            },
            path: "/runtime/s".into(),
            build: String::new(),
            runtime: Some(DaemonVersion::current()),
        };
        let old: LegacyEndpoint =
            serde_json::from_slice(&serde_json::to_vec(&endpoint).unwrap()).unwrap();
        assert_eq!(old.identity, endpoint.identity);
        assert_eq!(old.path, endpoint.path);
        assert!(old.build.is_empty());
    }

    #[tokio::test]
    async fn admission_progress_preserves_typed_success_when_the_sync_reply_is_lost() {
        use crate::mesh::{
            pairing::{Change, Command, Output, Status},
            status::Progress,
        };
        for admitted in [false, true] {
            let (mut client, mut server) = tokio::io::duplex(4096);
            let serving = async move {
                assert!(matches!(
                    wire::receive::<Frame>(&mut server).await.unwrap(),
                    Frame::Invoke { id: 1 }
                ));
                let _: Invocation = receive_json(&mut server).await.unwrap();
                assert!(matches!(
                    wire::receive::<Frame>(&mut server).await.unwrap(),
                    Frame::End
                ));
                if admitted {
                    wire::send(
                        &mut server,
                        &Frame::Progress(Progress::Admitted(Status {
                            reachability: None,
                            transport: Default::default(),
                            endpoint: Some("1".repeat(64)),
                            service: Some("https://mesh.example".into()),
                            head: Some("2".repeat(64)),
                            enabled: true,
                            pending_leave: false,
                            conflict: false,
                            members: vec!["1".repeat(64)],
                            revoked: vec![],
                        })),
                    )
                    .await
                    .unwrap();
                }
            };
            let invocation = Invocation {
                operation: Operation::Mesh(Command::Pair {
                    service: None,
                    invitation: Some("token".into()),
                    cancel: None,
                    initial_sync: true,
                }),
                caller: Caller {
                    scope: None,
                    writer: None,
                    diagnostics: Default::default(),
                },
            };
            let mut events = 0;
            let receiving = exchange_with_progress(&mut client, invocation, None, |_| events += 1);
            let (result, ()) = tokio::join!(receiving, serving);
            assert_eq!(events, usize::from(admitted));
            if admitted {
                let Value::Mesh(Output::Changed {
                    operation: Change::Paired,
                    status,
                    initial_sync: None,
                    initial_sync_error: Some(error),
                }) = result.unwrap().value
                else {
                    panic!("typed admission outcome expected");
                };
                assert!(status.enabled);
                assert!(error.mutation_may_have_committed);
            } else {
                assert!(
                    matches!(result, Err(Error::Application(error)) if error.mutation_may_have_committed)
                );
            }
        }
    }

    #[test]
    fn daemon_replacement_is_directional_and_independent_of_protocol_compatibility() {
        fn version(release: &str, compatibility: u32, schema: usize) -> DaemonVersion {
            DaemonVersion {
                release: release.into(),
                compatibility,
                schema,
            }
        }
        let local = version("0.10.0", 6, 10);
        assert!(local.supersedes(None));
        for (release, compatibility, schema, replace) in [
            ("0.9.0", 6, 10, true), // Compatible release upgrade still refreshes the owner.
            ("0.10.0-rc.1", 6, 10, true),
            ("0.10.0", 5, 10, true),
            ("0.10.0", 6, 9, true),
            ("0.10.0", 6, 10, false),
            ("0.10.0+other-build", 6, 10, false),
            ("0.11.0", 6, 10, false),
            ("0.11.0", 5, 9, false), // Release order cannot be overridden by older capabilities.
            ("0.9.0", 7, 10, false),
            ("0.9.0", 6, 11, false),
            ("not-a-version", 5, 9, false),
        ] {
            assert_eq!(
                local.supersedes(Some(&version(release, compatibility, schema))),
                replace,
                "{release}, {compatibility}, {schema}"
            );
        }
        let legacy: DaemonState = serde_json::from_str(r#"{"running":true,"pid":123}"#).unwrap();
        assert!(legacy.runtime.is_none());
        let state = DaemonState {
            running: true,
            pid: Some(123),
            runtime: Some(local),
        };
        #[derive(Deserialize)]
        struct LegacyState {
            running: bool,
            pid: Option<u32>,
        }
        let old: LegacyState =
            serde_json::from_value(serde_json::to_value(state).unwrap()).unwrap();
        assert!(old.running);
        assert_eq!(old.pid, Some(123));
    }
}
