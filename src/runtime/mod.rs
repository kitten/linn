//! Automatic local background runtime.
//! @ref LLP 0007#automatic-detached-lifecycle
mod coordinator;
mod dispatcher;
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
pub use protocol::DaemonState;
use protocol::{COMPATIBILITY, Frame, Hello, Invocation, ReplyEnvelope, VERSION};
#[derive(Clone, Serialize, Deserialize)]
struct Endpoint {
    identity: platform::Identity,
    #[serde(with = "crate::native_path")]
    path: PathBuf,
    // Legacy endpoint readers require this field, but it no longer identifies a build.
    #[serde(default)]
    build: String,
}
#[derive(Serialize, Deserialize)]
struct Bootstrap {
    endpoint: Endpoint,
    #[serde(with = "crate::native_path")]
    root: PathBuf,
    #[serde(with = "crate::native_path")]
    store: PathBuf,
    #[serde(with = "crate::native_path")]
    ready: PathBuf,
    token: String,
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
    let ep = location.endpoint()?;
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
async fn launch(location: &Location) -> Result<()> {
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
        endpoint: Endpoint {
            identity: location.id.clone(),
            path: socket,
            build: String::new(),
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
async fn session(location: &Location) -> Result<Stream> {
    if live_owner(location)?
        && let Ok(stream) = connect(location, false).await
    {
        return Ok(stream);
    }
    // An owner can hold its lifetime lock before publishing readiness metadata.
    // Serialize with the initiating client before deciding that connection failed.
    let start = platform::private_file(&location.file("start.lock"))?;
    wait_lock(&start).await?;
    if !live_owner(location)? {
        launch(location).await?;
    }
    connect(location, false).await
}
pub async fn execute(
    path: &Path,
    operation: Operation,
    caller: Caller,
    input: impl Into<Option<File>>,
) -> Result<Reply> {
    let input = input.into();
    let location = Location::new(path)?;
    let mut stream = session(&location).await?;
    exchange(&mut stream, Invocation { operation, caller }, input).await
}
async fn exchange(
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
    invocation: Invocation,
    mut input: Option<File>,
) -> Result<Reply> {
    let invocation = std::sync::Arc::new(invocation);
    let mutating = invocation.operation.mutates();
    let result = async {
        wire::send(stream, &Frame::Invoke { id: 1 }).await?;
        send_json(stream, invocation.clone()).await?;
        if let Some(input) = &mut input {
            send_file(stream, input).await?;
        } else {
            wire::send(stream, &Frame::End).await?;
        }
        match wire::receive::<Frame>(stream).await? {
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
                Ok(Reply {
                    value,
                    warnings,
                    output,
                })
            }
            Frame::Failure(error) => Err(error.into()),
            Frame::Rejected(message) => {
                Err(crate::error::ApplicationError::new(Error::Invalid(message), false).into())
            }
            _ => Err(Error::Invalid("invalid daemon response".into())),
        }
    }
    .await;
    result.map_err(|error| match error {
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
    })
}
pub async fn control(path: &Path, stop: bool) -> Result<DaemonState> {
    let location = Location::new(path)?;
    let start = platform::private_file(&location.file("start.lock"))?;
    wait_lock(&start).await?;
    if !live_owner(&location)? {
        return Ok(DaemonState::stopped());
    }
    let mut stream = connect(&location, true).await?;
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
struct Job {
    store: PathBuf,
    operation: Operation,
    caller: Caller,
    input: Option<File>,
    reply: oneshot::Sender<Result<Reply>>,
    delivered: oneshot::Receiver<()>,
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
        wire::send(&mut stream,&Frame::Rejected("incompatible daemon protocol/compatibility/schema/store; run linn daemon:stop, then retry with this executable".into())).await?;
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
    let (tx, rx) = oneshot::channel();
    let (done, delivered) = oneshot::channel();
    if queue
        .try_send(Job {
            store: hello.store,
            operation,
            caller,
            input,
            reply: tx,
            delivered,
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
    let reply = tokio::select! {
    reply=rx=>reply.map_err(|_|Error::Invalid("daemon stopped before returning a reply".into()))?,
    _=stream.read_u8()=>return Err(Error::Invalid("client disconnected or sent unexpected data".into())),
    };
    let response = async {
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
    let (queue, jobs) = mpsc::channel::<Job>(64);
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
        file: owned_file,
    };
    let dispatched = tokio::task::spawn_blocking(move || -> Result<_> {
        // libSQL's local async methods and native encoding execute synchronously.
        // Build and retain every Store on this thread; keep IPC independently polled.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(dispatcher::run(
            dispatch_location,
            jobs,
            stopping,
            listener,
            binding,
        ))
    })
    .await;
    accept_task.abort();
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
mod tests;
