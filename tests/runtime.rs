//! End-to-end detached-runtime behavior; each test owns its stores and socket root.
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};
struct Fixture {
    dir: tempfile::TempDir,
    db: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("lr-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let db = dir.path().join("store.db");
        Self { dir, db }
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_linn"));
        c.arg("--store")
            .arg(&self.db)
            .env("XDG_RUNTIME_DIR", self.dir.path());
        c
    }
    fn run(&self, args: &[&str], body: Option<&str>) -> Output {
        let mut c = self.command();
        c.args(args)
            .stdin(if body.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = c.spawn().unwrap();
        if let Some(body) = body {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(body.as_bytes())
                .unwrap();
        }
        child.wait_with_output().unwrap()
    }
    fn ok(&self, args: &[&str], body: Option<&str>) -> Output {
        let o = self.run(args, body);
        assert!(
            o.status.success(),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&o.stderr)
        );
        o
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["daemon:stop"], None);
    }
}
fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn concurrent_cold_start_and_cross_runtime_reconnect() {
    let f = Fixture::new();
    let mut children = Vec::new();
    for _ in 0..8 {
        children.push(
            f.command()
                .args(["list", "--json"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    for c in children {
        let o = c.wait_with_output().unwrap();
        assert_eq!(
            o.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert_eq!(json(&o)["records"], serde_json::json!([]));
    }
    let body = "Original bytes for detached ownership.\n";
    let added = f.ok(&["add", "-", "--json", "--unscoped"], Some(body));
    let id = json(&added)["record"].as_str().unwrap().to_owned();
    let other = tempfile::Builder::new()
        .prefix("lr-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(other.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let o = f
        .command()
        .env("XDG_RUNTIME_DIR", other.path())
        .args(["show", &id, "--all", "--json"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(json(&o)["text"], body);
    assert!(
        !other.path().join("linn").exists(),
        "existing owner must be found without creating another socket root"
    );
    f.ok(&["daemon:stop"], None);
    assert_eq!(
        json(&f.ok(&["daemon:status", "--json"], None))["running"],
        false
    );
    assert_eq!(
        json(&f.ok(&["show", &id, "--all", "--json"], None))["text"],
        body
    );
}
#[test]
fn parent_pipes_close_and_external_publication_reopens() {
    let f = Fixture::new();
    let o = f.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("A native detached routing decision.\n"),
    );
    let id = json(&o)["record"].as_str().unwrap().to_owned();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut store = linn::storage::Store::open(&f.db).await.unwrap();
        store.forget(&id).await.unwrap();
    });
    let out = f.run(&["show", &id, "--json"], None);
    assert!(
        !out.status.success(),
        "daemon must not return a cached forgotten body"
    );
    f.ok(&["index:check"], None);
    f.ok(&["daemon:stop"], None);
}
#[test]
fn invalid_runtime_starts_no_owner_and_invalid_import_creates_no_database() {
    let f = Fixture::new();
    let missing = f.dir.path().join("invalid-import.db");
    let mut c = Command::new(env!("CARGO_BIN_EXE_linn"));
    c.env("XDG_RUNTIME_DIR", f.dir.path());
    let mut child = c
        .arg("--store")
        .arg(&missing)
        .args(["import", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"{}\n").unwrap();
    assert!(!child.wait_with_output().unwrap().status.success());
    assert!(!missing.exists());
    let state = Command::new(env!("CARGO_BIN_EXE_linn"))
        .env("XDG_RUNTIME_DIR", f.dir.path())
        .arg("--store")
        .arg(&missing)
        .args(["daemon:status", "--json"])
        .output()
        .unwrap();
    assert_eq!(
        json(&state)["running"],
        true,
        "validation must run in the owner"
    );
    assert!(!missing.exists(), "status must not create a database");
    assert!(
        Command::new(env!("CARGO_BIN_EXE_linn"))
            .env("XDG_RUNTIME_DIR", f.dir.path())
            .arg("--store")
            .arg(&missing)
            .arg("daemon:stop")
            .output()
            .unwrap()
            .status
            .success()
    );
    let out = f
        .command()
        .env("XDG_RUNTIME_DIR", "relative")
        .args(["list"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("XDG_RUNTIME_DIR must be absolute"));
    assert_eq!(
        json(&f.ok(&["daemon:status", "--json"], None))["running"],
        false
    );
}
#[test]
fn alias_and_removed_endpoint_never_create_parallel_owners() {
    let f = Fixture::new();
    assert_eq!(f.run(&["list", "--json"], None).status.code(), Some(1));
    let alias = f.dir.path().join("alias.db");
    std::os::unix::fs::symlink(&f.db, &alias).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_linn"))
        .env("XDG_RUNTIME_DIR", f.dir.path())
        .arg("--store")
        .arg(&alias)
        .args(["daemon:status", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(json(&out)["running"], true);
    let socket = std::fs::read_dir(f.dir.path().join("linn"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "sock"))
        .unwrap();
    std::fs::remove_file(socket).unwrap();
    // Loss of a live endpoint may return a bounded error until the owner retires,
    // but it cannot authorize a second owner while that lock is held.
    let out = f.run(&["list", "--json"], None);
    assert!(matches!(out.status.code(), Some(1 | 2)));
}
#[tokio::test]
async fn application_and_ipc_search_pages_match() {
    use linn::application::{Application, Caller, Operation, Value};
    let f = Fixture::new();
    let added = f.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("# Native routing\n\nThe QZ_ROUTER_42 identifier selects native routing.\n"),
    );
    let id = json(&added)["record"].as_str().unwrap().to_owned();
    let mut app = Application::new(f.db.clone());
    let reply = app
        .execute(
            Operation::Search {
                query: "QZ_ROUTER_42 native routing".into(),
            },
            Caller {
                diagnostics: Default::default(),
                scope: linn::paths::scope(),
                writer: None,
            },
            tempfile::tempfile().unwrap(),
        )
        .await
        .unwrap();
    let Value::Search(page) = reply.value else {
        panic!("search")
    };
    let expected = page.call.unwrap();
    let output = f.run(&["search", "QZ_ROUTER_42 native routing", "--json"], None);
    assert!(
        matches!(output.status.code(), Some(0 | 1)),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let got = json(&output);
    let ids: Vec<_> = got["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["record"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        expected
            .cards
            .iter()
            .map(|c| c.record.as_str())
            .collect::<Vec<_>>()
    );
    for (a, b) in got["cards"].as_array().unwrap().iter().zip(expected.cards) {
        assert_eq!(a["heading"], b.heading);
        assert_eq!(a["excerpt"], b.text);
    }
    assert_eq!(
        json(&f.ok(&["show", &id, "--all", "--json"], None))["text"],
        "# Native routing\n\nThe QZ_ROUTER_42 identifier selects native routing.\n"
    );
}

#[test]
fn diagnostics_belong_to_the_request_and_compatibility_mismatch_is_rejected() {
    let f = Fixture::new();
    let first = f
        .command()
        .env("LINN_STARTUP_TIMINGS", "1")
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert_eq!(first.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&first.stderr).contains("linn application:"));
    let second = f
        .command()
        .env_remove("LINN_STARTUP_TIMINGS")
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    assert!(
        second.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    use std::io::Read;
    let socket = std::fs::read_dir(f.dir.path().join("linn"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "sock"))
        .unwrap();
    let mut stream = std::os::unix::net::UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let hello=serde_json::to_vec(&serde_json::json!({"Hello":{"version":1,"compatibility":999,"schema":linn::storage::SCHEMA_VERSION,"identity":{"account":unsafe{libc::geteuid()},"path":std::fs::canonicalize(&f.db).unwrap()},"store":f.db,"control":false}})).unwrap();
    stream
        .write_all(&(hello.len() as u32).to_be_bytes())
        .unwrap();
    stream.write_all(&hello).unwrap();
    let mut n = [0; 4];
    stream.read_exact(&mut n).unwrap();
    let mut value = vec![0; u32::from_be_bytes(n) as usize];
    stream.read_exact(&mut value).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&value).unwrap();
    assert!(
        value["Rejected"]
            .as_str()
            .unwrap()
            .contains("incompatible daemon")
    );
    // Incompatible clients do not replace the owner; the compatible client still works.
    assert_eq!(f.run(&["list", "--json"], None).status.code(), Some(1));
}

#[test]
fn crash_releases_ownership_and_stale_non_socket_is_preserved() {
    let f = Fixture::new();
    assert_eq!(f.run(&["list", "--json"], None).status.code(), Some(1));
    let status = json(&f.ok(&["daemon:status", "--json"], None));
    let pid = status["pid"].as_u64().unwrap() as i32;
    assert_eq!(unsafe { libc::kill(pid, libc::SIGKILL) }, 0);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let until = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Ok(state) = linn::runtime::control(&f.db, false).await
                && !state.running
            {
                break;
            }
            assert!(tokio::time::Instant::now() < until);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    });
    assert_eq!(f.run(&["list", "--json"], None).status.code(), Some(1));
    let status = json(&f.ok(&["daemon:status", "--json"], None));
    assert_ne!(status["pid"], pid);
    f.ok(&["daemon:stop"], None);
    let key = {
        let identity = serde_json::json!({"account":unsafe{libc::geteuid()},"path":std::fs::canonicalize(&f.db).unwrap()});
        // The retained metadata names exactly this fixture's authenticated endpoint.
        let owners = std::fs::read_dir(
            f.dir
                .path()
                .join(format!(".linn-runtime-{}", unsafe { libc::geteuid() })),
        )
        .unwrap();
        let metadata = owners
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|e| e == "endpoint"))
            .unwrap();
        let ep: serde_json::Value =
            serde_json::from_slice(&std::fs::read(metadata).unwrap()).unwrap();
        assert_eq!(ep["identity"], identity);
        PathBuf::from(ep["path"].as_str().unwrap())
    };
    std::fs::write(&key, b"preserve this file").unwrap();
    let failed = f.run(&["list", "--json"], None);
    assert_eq!(failed.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("non-socket"));
    assert_eq!(std::fs::read(key).unwrap(), b"preserve this file");
}

#[test]
fn simultaneous_peer_pulls_dispatch_local_steps_without_holding_network_jobs() {
    let a = Fixture::new();
    let b = Fixture::new();
    let aid = json(&a.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("A original bytes.\n"),
    ))["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let bid = json(&b.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("B original bytes.\n"),
    ))["record"]
        .as_str()
        .unwrap()
        .to_owned();
    use std::os::unix::fs::PermissionsExt;
    let ssh = a.dir.path().join("ssh");
    std::fs::write(
        &ssh,
        "#!/bin/sh\nfor arg do last=\"$arg\"; done\nexec /bin/sh -c \"$last\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let paths = std::env::join_paths(
        [
            a.dir.path().to_owned(),
            PathBuf::from(env!("CARGO_BIN_EXE_linn"))
                .parent()
                .unwrap()
                .to_owned(),
        ]
        .into_iter()
        .chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )),
    )
    .unwrap();
    for (local, peer) in [(&a, &b), (&b, &a)] {
        local.ok(
            &[
                "sync:remote",
                "pair",
                "peer",
                &format!("ssh://peer{}", peer.db.display()),
            ],
            None,
        );
    }
    std::thread::scope(|scope| {
        for local in [&a, &b] {
            let paths = &paths;
            scope.spawn(move || {
                let output = local
                    .command()
                    .env("PATH", paths)
                    .args(["sync", "--json"])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            });
        }
    });
    assert_eq!(
        json(&a.ok(&["show", &bid, "--all", "--json"], None))["text"],
        "B original bytes.\n"
    );
    assert_eq!(
        json(&b.ok(&["show", &aid, "--all", "--json"], None))["text"],
        "A original bytes.\n"
    );
}

#[test]
fn resident_reads_work_when_cli_cannot_open_database() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let f = Fixture::new();
    let body = "Only the daemon owns the open store.\n";
    let added = f.ok(&["add", "-", "--json", "--unscoped"], Some(body));
    let id = json(&added)["record"].as_str().unwrap().to_owned();
    let permissions = std::fs::metadata(&f.db).unwrap().permissions();
    std::fs::set_permissions(&f.db, std::fs::Permissions::from_mode(0o000)).unwrap();
    assert!(std::fs::File::open(&f.db).is_err());
    let output = f.run(&["show", &id, "--all", "--json"], None);
    std::fs::set_permissions(&f.db, permissions).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(json(&output)["text"], body);
}

#[test]
fn stop_drains_daemon_sync_workers_and_their_internal_store_steps() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};
    let local = Fixture::new();
    let remote = Fixture::new();
    let local_added = local.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("Local original.\n"),
    );
    let remote_added = remote.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("Remote original.\n"),
    );
    let origin = format!("ssh://peer{}", remote.db.display());
    local.ok(&["sync:remote", "pair", "peer", &origin], None);
    let ssh = local.dir.path().join("ssh");
    std::fs::write(
        &ssh,
        r#"#!/bin/sh
: > "$SYNC_STARTED"
while [ ! -f "$SYNC_RELEASE" ]; do /bin/sleep 0.01; done
for arg do last="$arg"; done
exec /bin/sh -c "$last"
"#,
    )
    .unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let paths = std::env::join_paths(
        [
            local.dir.path().to_owned(),
            PathBuf::from(env!("CARGO_BIN_EXE_linn"))
                .parent()
                .unwrap()
                .to_owned(),
        ]
        .into_iter()
        .chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )),
    )
    .unwrap();
    let started = local.dir.path().join("started");
    let release = local.dir.path().join("release");
    let mut sync = local
        .command()
        .args(["sync", "--json"])
        .env("PATH", paths)
        .env("SYNC_STARTED", &started)
        .env("SYNC_RELEASE", &release)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !started.exists() {
        assert!(Instant::now() < deadline, "daemon did not start SSH");
        std::thread::sleep(Duration::from_millis(10));
    }
    // The store dispatcher remains responsive while its worker waits on SSH.
    local.ok(&["list", "--json"], None);
    let mut stop = local
        .command()
        .arg("daemon:stop")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wait for the authenticated owner to stop accepting new connections.
    let socket = std::fs::read_dir(local.dir.path().join("linn"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "sock"))
        .unwrap();
    while std::os::unix::net::UnixStream::connect(&socket).is_ok() {
        assert!(Instant::now() < deadline, "stop did not begin draining");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        stop.try_wait().unwrap().is_none(),
        "stop must wait for accepted sync"
    );
    std::fs::write(&release, b"").unwrap();
    for child in [&mut sync, &mut stop] {
        let deadline = Instant::now() + Duration::from_secs(15);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                panic!("sync and stop must drain without a dispatcher deadlock");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let output = sync.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stop.wait_with_output().unwrap().status.success());
    assert_eq!(
        json(&local.ok(&["daemon:status", "--json"], None))["running"],
        false
    );
    let remote_id = json(&remote_added)["record"].as_str().unwrap().to_owned();
    let local_id = json(&local_added)["record"].as_str().unwrap().to_owned();
    local.ok(&["show", &remote_id, "--all"], None);
    remote.ok(&["show", &local_id, "--all"], None);
}

#[test]
fn concurrent_writes_reads_and_queries_preserve_every_committed_record() {
    let f = Fixture::new();
    let seed = f.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("CONCURRENT_SEED routing evidence.\n"),
    );
    let seed_id = json(&seed)["record"].as_str().unwrap().to_owned();
    let mut writes = Vec::new();
    let mut reads = Vec::new();
    for index in 0..12 {
        let body = format!("Concurrent original {index}.\n");
        let mut child = f
            .command()
            .args(["add", "-", "--json", "--unscoped"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(body.as_bytes())
            .unwrap();
        writes.push((body, child));
        reads.push(
            f.command()
                .args(["show", &seed_id, "--all", "--json"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        reads.push(
            f.command()
                .args(["search", "CONCURRENT_SEED", "--json"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    let mut ids = std::collections::HashSet::new();
    for (body, child) in writes {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let id = json(&output)["record"].as_str().unwrap().to_owned();
        assert!(ids.insert(id.clone()));
        assert_eq!(
            json(&f.ok(&["show", &id, "--all", "--json"], None))["text"],
            body
        );
    }
    for (index, child) in reads.into_iter().enumerate() {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = json(&output);
        if index % 2 == 0 {
            assert_eq!(value["text"], "CONCURRENT_SEED routing evidence.\n");
        } else {
            assert!(
                value["cards"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|card| card["record"] == seed_id)
            );
        }
    }
    let listed = json(&f.ok(&["list", "--json"], None));
    assert_eq!(listed["records"].as_array().unwrap().len(), 13);
    f.ok(&["index:check"], None);
}

#[test]
fn control_remains_responsive_while_store_waits_on_an_external_sqlite_lock() {
    use std::time::{Duration, Instant};
    let f = Fixture::new();
    let added = f.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("Locked-store original.\n"),
    );
    let id = json(&added)["record"].as_str().unwrap().to_owned();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let database = runtime
        .block_on(libsql::Builder::new_local(&f.db).build())
        .unwrap();
    let connection = database.connect().unwrap();
    runtime
        .block_on(connection.execute_batch("BEGIN EXCLUSIVE"))
        .unwrap();
    let mut read = f
        .command()
        .args(["show", &id, "--all", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let blocked = read.try_wait().unwrap().is_none();
    let start = Instant::now();
    let status = f.run(&["daemon:status", "--json"], None);
    let elapsed = start.elapsed();
    let mut cancelled = f
        .command()
        .args(["add", "-", "--unscoped"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    cancelled
        .stdin
        .take()
        .unwrap()
        .write_all(b"Cancelled before store execution.\n")
        .unwrap();
    std::thread::sleep(Duration::from_millis(150));
    assert!(cancelled.try_wait().unwrap().is_none());
    cancelled.kill().unwrap();
    cancelled.wait().unwrap();
    // Let the IPC side observe EOF while the store thread is still blocked.
    std::thread::sleep(Duration::from_millis(50));
    runtime
        .block_on(connection.execute_batch("ROLLBACK"))
        .unwrap();
    let output = read.wait_with_output().unwrap();
    assert!(
        blocked,
        "the reader must actually encounter the external lock"
    );
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "IPC stalled for {elapsed:?}"
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(json(&output)["text"], "Locked-store original.\n");
    assert_eq!(
        json(&f.ok(&["list", "--json"], None))["records"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "disconnected queued mutations must not execute"
    );
}

#[test]
fn daemon_closes_inherited_high_descriptor_without_holding_caller_pipe_open() {
    use std::{
        io::Read,
        os::fd::{AsRawFd, FromRawFd},
    };
    let f = Fixture::new();
    let (mut reader, writer) = std::os::unix::net::UnixStream::pair().unwrap();
    let inherited = unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 4096) };
    assert!(inherited >= 4096);
    let inherited = unsafe { std::os::fd::OwnedFd::from_raw_fd(inherited) };
    let mut command = f.command();
    use std::os::unix::process::CommandExt;
    // The child inherits a descriptor above its reduced soft limit. A sweep
    // bounded by that limit would miss it; the native descriptor list must not.
    let inherited_fd = inherited.as_raw_fd();
    unsafe {
        command.pre_exec(move || {
            // Other tests' children must not inherit this test's caller pipe.
            if libc::fcntl(inherited_fd, libc::F_SETFD, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut limit = std::mem::MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, limit.as_mut_ptr()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut limit = limit.assume_init();
            limit.rlim_cur = limit.rlim_cur.min(128);
            if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = command
        .args(["list", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(writer);
    drop(inherited);
    reader
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let mut byte = [0];
    let eof = reader.read(&mut byte);
    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        matches!(eof, Ok(0)),
        "daemon retained the inherited caller pipe: {eof:?}"
    );
    assert_eq!(
        json(&f.ok(&["daemon:status", "--json"], None))["running"],
        true
    );
}

#[test]
#[cfg(target_os = "macos")]
fn differently_cased_missing_paths_cannot_keep_two_store_owners() {
    use linn::application::{Caller, Operation};
    let f = Fixture::new();
    let probe = f.dir.path().join("CaseProbe");
    std::fs::write(&probe, b"").unwrap();
    if !f.dir.path().join("caseprobe").exists() {
        return;
    }
    let alias = f.dir.path().join("STORE.DB");
    let start = |path: &std::path::Path| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
            .arg("--store")
            .arg(path)
            .args(["sync:serve", "--stdio"])
            .env("XDG_RUNTIME_DIR", f.dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"{\"op\":\"hello\",\"version\":1}\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    // Start both routing identities before either creates the database.
    start(&f.db);
    start(&alias);
    assert!(!f.db.exists());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let invoke = |path, operation| {
        linn::runtime::execute(
            path,
            operation,
            Caller {
                diagnostics: Default::default(),
                scope: None,
                writer: None,
            },
            tempfile::tempfile().unwrap(),
        )
    };
    rt.block_on(async {
        let root = f
            .dir
            .path()
            .join(format!(".linn-runtime-{}", unsafe { libc::geteuid() }));
        let owner_locks = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.to_string_lossy().ends_with(".owner.lock"))
            .collect::<Vec<_>>();
        assert_eq!(owner_locks.len(), 2);
        let live = || {
            owner_locks
                .iter()
                .filter(|p| {
                    let lock = std::fs::File::open(p).unwrap();
                    matches!(lock.try_lock(), Err(std::fs::TryLockError::WouldBlock))
                })
                .count()
        };
        assert_eq!(live(), 2);
        let create = std::fs::File::open(root.join("create.lock")).unwrap();
        create.lock().unwrap();
        let (first, second, ()) = tokio::join!(
            invoke(&f.db, Operation::Check),
            invoke(&alias, Operation::Check),
            async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                create.unlock().unwrap();
            },
        );
        assert!(first.is_ok() || second.is_ok());
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while live() != 1 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert!(linn::runtime::control(&alias, false).await.unwrap().running);
    });
}

#[test]
fn newer_schema_fences_and_retires_the_resident_daemon() {
    let f = Fixture::new();
    f.ok(
        &["add", "-", "--json", "--unscoped"],
        Some("Original record survives migration."),
    );
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let database = libsql::Builder::new_local(&f.db).build().await.unwrap();
        let db = database.connect().unwrap();
        db.execute_batch(&format!("BEGIN IMMEDIATE; CREATE TABLE future_only(value TEXT); INSERT INTO future_only VALUES('untouched'); PRAGMA user_version={}; COMMIT;", linn::storage::SCHEMA_VERSION + 1)).await.unwrap();
    });
    let blocked = f.run(&["add", "-", "--json"], Some("Must never commit."));
    assert_eq!(blocked.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("store schema"));
    rt.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while linn::runtime::control(&f.db, false).await.unwrap().running {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let database = libsql::Builder::new_local(&f.db).build().await.unwrap();
        let db = database.connect().unwrap();
        let mut rows = db
            .query(
                "SELECT (SELECT count(*) FROM records),(SELECT value FROM future_only)",
                (),
            )
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<i64>(0).unwrap(), 1);
        assert_eq!(row.get::<String>(1).unwrap(), "untouched");
    });
}

#[test]
fn ordinary_queries_do_not_require_client_temporary_files() {
    let f = Fixture::new();
    f.ok(
        &["add", "-", "--json"],
        Some("NO_TMPDIR_ROUTING original bytes.\n"),
    );
    for args in [
        vec!["search", "NO_TMPDIR_ROUTING", "--json"],
        vec!["list", "--json"],
        vec!["daemon:status", "--json"],
    ] {
        let output = f
            .command()
            .env("TMPDIR", f.dir.path().join("missing"))
            .args(args)
            .output()
            .unwrap();
        assert!(
            matches!(output.status.code(), Some(0 | 1)),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn sync_delivery_orders_forget_but_releases_before_peer_acknowledgement() {
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    // Always unblock the fake peer before fixture cleanup, including on failure.
    struct Release(PathBuf);
    impl Drop for Release {
        fn drop(&mut self) {
            let _ = std::fs::write(self.0.join("release"), b"");
            let _ = std::fs::write(self.0.join("ack"), b"");
        }
    }
    for cancel in [false, true] {
        let f = Fixture::new();
        let body = format!(
            "Original sync delivery boundary.\n{}\n",
            "x".repeat(2 * 1024 * 1024)
        );
        let id = json(&f.ok(&["add", "-", "--json", "--unscoped"], Some(&body)))["record"]
            .as_str()
            .unwrap()
            .to_owned();
        f.ok(&["sync:remote", "pair", "peer", "ssh://peer"], None);
        let script = f.dir.path().join("ssh");
        std::fs::write(
            &script,
            r#"#!/bin/sh
set -eu
IFS= read -r hello
printf '%s\n' '{"result":"hello","version":1}'
if [ ! -f "$SYNC_ROOT/pulled" ]; then
    while IFS= read -r request; do
        printf '%s\n' '{"result":"entries","entries":[]}'
    done
    : > "$SYNC_ROOT/pulled"
else
    IFS= read -r needs
    printf '{"result":"needed","ids":["%s"]}\n' "$SYNC_ID"
    dd bs=1 count=1 of="$SYNC_ROOT/first" 2>/dev/null
    : > "$SYNC_ROOT/writing"
    while [ ! -f "$SYNC_ROOT/release" ]; do sleep 0.01; done
    if [ "$SYNC_CANCEL" = 1 ]; then exit 1; fi
    IFS= read -r rest
    printf '%s' "$rest" > "$SYNC_ROOT/delivered"
    while [ ! -f "$SYNC_ROOT/ack" ]; do sleep 0.01; done
    printf '%s\n' '{"result":"applied","changed":true}'
    IFS= read -r finish
    printf '%s\n' '{"result":"done"}'
fi
"#,
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let paths = std::env::join_paths(std::iter::once(f.dir.path().to_owned()).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        let mut sync = f
            .command()
            .arg("sync")
            .env("PATH", paths)
            .env("SYNC_ROOT", f.dir.path())
            .env("SYNC_ID", &id)
            .env("SYNC_CANCEL", if cancel { "1" } else { "0" })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let _release = Release(f.dir.path().to_owned());
        let wait = |file: &str| {
            let deadline = Instant::now() + Duration::from_secs(15);
            while !f.dir.path().join(file).exists() {
                assert!(Instant::now() < deadline, "peer did not reach {file}");
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        wait("writing");
        let mut forget = f
            .command()
            .args(["add:forget", &id, "--json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(200));
        let premature = forget.try_wait().unwrap();
        // Controls stay available while the original-data delivery blocks work.
        f.ok(&["daemon:status", "--json"], None);
        std::fs::write(f.dir.path().join("release"), b"").unwrap();
        if !cancel {
            wait("delivered");
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        while forget.try_wait().unwrap().is_none() {
            assert!(
                Instant::now() < deadline,
                "forget waited for the peer acknowledgement"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let forgotten = forget.wait_with_output().unwrap();
        let waiting_for_ack = sync.try_wait().unwrap().is_none();
        std::fs::write(f.dir.path().join("ack"), b"").unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        while sync.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = sync.kill();
                panic!("sync did not finish");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let synced = sync.wait_with_output().unwrap();
        assert!(
            premature.is_none(),
            "forget completed before original-data delivery"
        );
        assert!(
            forgotten.status.success(),
            "{}",
            String::from_utf8_lossy(&forgotten.stderr)
        );
        assert_eq!(
            synced.status.success(),
            !cancel,
            "{}",
            String::from_utf8_lossy(&synced.stderr)
        );
        if !cancel {
            assert!(
                waiting_for_ack,
                "peer must still be waiting for acknowledgement release"
            );
            let bytes = [
                std::fs::read(f.dir.path().join("first")).unwrap(),
                std::fs::read(f.dir.path().join("delivered")).unwrap(),
            ]
            .concat();
            let request: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let envelope: serde_json::Value =
                serde_json::from_str(request["item"]["envelope"].as_str().unwrap()).unwrap();
            assert_eq!(envelope["body"], body);
        }
    }
}

#[test]
fn non_utf8_store_path_returns_a_regular_error_without_starting_an_owner() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let f = Fixture::new();
    let path = f.dir.path().join(OsStr::from_bytes(b"store-\xff.db"));
    let output = Command::new(env!("CARGO_BIN_EXE_linn"))
        .arg("--store")
        .arg(&path)
        .env("XDG_RUNTIME_DIR", f.dir.path())
        .args(["list", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("store path must be valid UTF-8"));
    assert!(!path.exists());
    assert!(
        !f.dir
            .path()
            .join(format!(".linn-runtime-{}", unsafe { libc::geteuid() }))
            .exists()
    );
}

#[test]
#[cfg(target_os = "linux")]
fn non_utf8_runtime_source_and_repository_paths_work_end_to_end() {
    use std::{
        ffi::OsStr,
        os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    };
    let f = Fixture::new();
    let runtime = f.dir.path().join(OsStr::from_bytes(b"r-\xff"));
    std::fs::create_dir(&runtime).unwrap();
    std::fs::set_permissions(&runtime, std::fs::Permissions::from_mode(0o700)).unwrap();
    let repository = f.dir.path().join(OsStr::from_bytes(b"repo-\xff"));
    git2::Repository::init(&repository).unwrap();
    let source = repository.join(OsStr::from_bytes(b"source-\xfe.md"));
    let body = "Original bytes through native-path IPC.\n";
    std::fs::write(&source, body).unwrap();
    let added = f
        .command()
        .current_dir(&repository)
        .env("XDG_RUNTIME_DIR", &runtime)
        .arg("add")
        .arg(&source)
        .args(["--json", "--unscoped"])
        .output()
        .unwrap();
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let id = json(&added)["record"].as_str().unwrap().to_owned();
    // Reconnect using endpoint metadata from another runtime root.
    assert_eq!(
        json(&f.ok(&["show", &id, "--all", "--json"], None))["text"],
        body
    );
    f.ok(&["daemon:stop"], None);
}
