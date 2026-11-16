//! Detached CLI pairing; fixtures use independent stores under one test UID.
use std::{
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

struct Fixture {
    root: tempfile::TempDir,
    store: PathBuf,
}
impl Fixture {
    fn prompt_join(&self, token: &str) -> Output {
        self.prompt_command(
            &["sync:pair", "-y", "--join", "--no-sync", "--json"],
            b"Invitation: ",
            token,
        )
    }
    fn prompt_command(&self, args: &[&str], expected_prompt: &[u8], answer: &str) -> Output {
        use std::{
            fs::File,
            io::Read,
            os::fd::{AsRawFd, FromRawFd},
        };
        let (mut master, mut slave) = (-1, -1);
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            0
        );
        for fd in [master, slave] {
            assert_eq!(
                unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) },
                0
            );
        }
        let mut master = unsafe { File::from_raw_fd(master) };
        let slave = unsafe { File::from_raw_fd(slave) };
        let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
            .arg("--store")
            .arg(&self.store)
            .args(args)
            .env("XDG_RUNTIME_DIR", self.root.path())
            .env("HOME", self.root.path())
            .env("XDG_CONFIG_HOME", self.root.path().join("config"))
            .env("XDG_DATA_HOME", self.root.path().join("data"))
            .stdin(Stdio::from(slave))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let mut prompt = Vec::new();
        while !prompt.ends_with(expected_prompt) {
            let mut byte = [0];
            stderr.read_exact(&mut byte).unwrap();
            prompt.push(byte[0]);
        }
        master.write_all(format!("{answer}\n").as_bytes()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
        while child.try_wait().unwrap().is_none() {
            if std::time::Instant::now() > deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("prompt join timed out");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut output = child.wait_with_output().unwrap();
        let mut terminal = std::mem::MaybeUninit::<libc::termios>::uninit();
        assert_eq!(
            unsafe { libc::tcgetattr(master.as_raw_fd(), terminal.as_mut_ptr()) },
            0
        );
        let flags = unsafe { terminal.assume_init() }.c_lflag;
        assert_eq!(
            flags & (libc::ICANON | libc::ECHO | libc::ISIG),
            libc::ICANON | libc::ECHO | libc::ISIG
        );
        output.stderr.extend(prompt);
        stderr.read_to_end(&mut output.stderr).unwrap();
        output
    }
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("lp-")
            .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let store = root.path().join("store.db");
        Self { root, store }
    }
    fn run(&self, args: &[&str], input: Option<&str>) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
            .arg("--store")
            .arg(&self.store)
            .args(args)
            .env("XDG_RUNTIME_DIR", self.root.path())
            .env("HOME", self.root.path())
            .env("XDG_CONFIG_HOME", self.root.path().join("config"))
            .env("XDG_DATA_HOME", self.root.path().join("data"))
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(input) = input {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
        }
        child.wait_with_output().unwrap()
    }
    fn ok(&self, args: &[&str], input: Option<&str>) -> serde_json::Value {
        let output = self.run(args, input);
        assert!(
            output.status.success(),
            "{args:?}: {}\nstdout: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["daemon:stop"], None);
    }
}

#[test]
fn unpaired_status_and_invalid_invitation_do_not_create_identity_or_database() {
    let fixture = Fixture::new();
    let status = fixture.ok(&["sync:status", "--json"], None);
    assert!(status["endpoint"].is_null());
    assert!(!fixture.store.exists());
    let invalid = fixture.run(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some("not-a-valid-invitation"),
    );
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!fixture.store.exists());
    // A pasted invitation must not hang at the terminal's canonical line limit.
    for token in ["a".repeat(8192), "\x03".to_owned()] {
        let output = fixture.prompt_join(&token);
        assert_eq!(output.status.code(), Some(2));
        assert!(!fixture.store.exists());
    }
}

#[test]
#[ignore = "requires an explicitly configured Worker"]
fn failed_sync_reports_partial_durable_progress_and_retry_does_not_double_count() {
    let service = std::env::var("LINN_RELAY_TEST_URL").unwrap();
    let a = Fixture::new();
    let b = Fixture::new();
    fn bundle(records: &[(&str, &str)]) -> String {
        use linn::storage::{record::hash, sync::Item};
        let mut frames = String::new();
        for (id, body) in records {
            let envelope = serde_json::json!({"id":id,"body":body}).to_string();
            let item = Item {
                id: (*id).into(),
                envelope_hash: Some(hash(envelope.as_bytes())),
                body_hash: Some(hash(body.as_bytes())),
                envelope: Some(envelope),
            };
            frames.push_str(&serde_json::json!({"type":"item","item":item}).to_string());
            frames.push('\n');
        }
        format!(
            "{{\"format\":\"linn-export\",\"version\":1}}\n{frames}{}\n",
            serde_json::json!({"type":"end","count":records.len(),"sha256":hash(frames.as_bytes())})
        )
    }
    a.ok(
        &["import", "-", "--json"],
        Some(&bundle(&[
            ("a", "Durable partial progress."),
            ("z", "Original bytes."),
        ])),
    );
    b.ok(
        &["import", "-", "--json"],
        Some(&bundle(&[("z", "Conflicting bytes.")])),
    );
    let invite = a.ok(
        &[
            "sync:pair",
            "-y",
            "--service",
            service.trim_end_matches('/'),
            "--json",
        ],
        None,
    );
    b.ok(
        &[
            "sync:pair",
            "-y",
            "--join",
            invite["token"].as_str().unwrap(),
            "--no-sync",
            "--json",
        ],
        None,
    );
    for received in [1, 0] {
        let output = b.run(&["sync", "--json"], None);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["complete"], false);
        assert_eq!(report["received"], received);
        assert!(
            report["failures"]
                .as_array()
                .unwrap()
                .iter()
                .any(|failure| failure["kind"] == "conflict")
        );
        assert_eq!(
            b.ok(&["show", "a", "--all", "--json"], None)["text"],
            "Durable partial progress."
        );
        b.ok(&["index:check", "--json"], None);
    }
}

#[test]
#[ignore = "requires an explicitly configured Worker"]
fn joining_supports_arguments_prompt_first_sync_failure_and_self_revocation() {
    let service = std::env::var("LINN_RELAY_TEST_URL").unwrap();
    let a = Fixture::new();
    let b = Fixture::new();
    let c = Fixture::new();
    let d = Fixture::new();
    let original = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Initial sync transfers this original."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let invite = a.ok(
        &[
            "sync:pair",
            "-y",
            "--service",
            service.trim_end_matches('/'),
            "--json",
        ],
        None,
    );
    let output = b.run(
        &[
            "sync:pair",
            "-y",
            "--join",
            invite["token"].as_str().unwrap(),
            "--sync",
            "--json",
        ],
        None,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let joined: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(joined["operation"], "paired");
    assert_eq!(joined["initial_sync"]["complete"], true);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Sync complete"));
    assert!(joined["initial_sync"]["complete"].is_boolean());
    assert_eq!(
        b.ok(&["show", &original, "--all", "--json"], None)["text"],
        "Initial sync transfers this original."
    );
    let endpoint = joined["status"]["endpoint"].as_str().unwrap();
    let revoked = b.ok(&["sync:revoke", endpoint, "--json"], None);
    assert_eq!(revoked["status"]["enabled"], false);
    assert_eq!(revoked["status"]["pending_leave"], false);
    let invite = a.ok(&["sync:pair", "-y", "--json"], None);
    a.ok(&["daemon:stop", "--json"], None);
    let output = c.run(
        &["sync:pair", "-y", "--join", "--sync", "--json"],
        Some(invite["token"].as_str().unwrap()),
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let joined: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(joined["operation"], "paired");
    assert_eq!(joined["status"]["enabled"], true);
    assert_eq!(joined["initial_sync"]["complete"], false);
    assert_eq!(c.ok(&["sync:status", "--json"], None)["enabled"], true);
    a.ok(&["sync:status", "--json"], None);
    assert_eq!(c.ok(&["sync", "--json"], None)["complete"], true);
    let invite = a.ok(&["sync:pair", "-y", "--json"], None);
    let output = d.prompt_join(invite["token"].as_str().unwrap());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invitation:"));
    let joined: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(joined["operation"], "paired");
    assert!(joined.get("initial_sync").is_none());
}

#[test]
#[ignore = "requires an explicitly configured Worker"]
fn daemon_pairing_and_explicit_sync_merge_three_independent_stores() {
    let service = std::env::var("LINN_RELAY_TEST_URL")
        .unwrap()
        .trim_end_matches('/')
        .to_owned();
    let a = Fixture::new();
    let b = Fixture::new();
    let c = Fixture::new();
    let original_a = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Original A remains private until an explicit sync."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let original_b = b.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Original B remains private until an explicit sync."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let invite = a.ok(&["sync:pair", "-y", "--service", &service, "--json"], None);
    let token = invite["token"].as_str().unwrap();
    let joined = b.ok(&["sync:pair", "-y", "--join", "-", "--json"], Some(token));
    assert_eq!(joined["status"]["members"].as_array().unwrap().len(), 2);
    assert_ne!(
        a.ok(&["sync:status", "--json"], None)["endpoint"],
        joined["status"]["endpoint"]
    );
    assert_eq!(
        b.run(&["show", &original_a, "--all", "--json"], None)
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        a.run(&["show", &original_b, "--all", "--json"], None)
            .status
            .code(),
        Some(2)
    );
    let synced = a.ok(&["sync", "--json"], None);
    assert_eq!(synced["complete"], true);
    assert_eq!(synced["collected"].as_array().unwrap().len(), 2);
    assert_eq!(
        b.ok(&["show", &original_a, "--all", "--json"], None)["text"],
        "Original A remains private until an explicit sync."
    );
    assert_eq!(
        a.ok(&["show", &original_b, "--all", "--json"], None)["text"],
        "Original B remains private until an explicit sync."
    );

    c.ok(&["sync:pair", "-y", "--service", &service, "--json"], None);
    let invite = a.ok(&["sync:pair", "-y", "--json"], None);
    let merged = c.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(invite["token"].as_str().unwrap()),
    );
    assert_eq!(merged["status"]["members"].as_array().unwrap().len(), 3);
    assert_eq!(
        c.run(&["show", &original_a, "--all", "--json"], None)
            .status
            .code(),
        Some(2)
    );
    let synced = c.ok(&["sync", "--json"], None);
    assert_eq!(synced["complete"], true);
    assert_eq!(synced["distributed"].as_array().unwrap().len(), 3);
    assert_eq!(
        c.ok(&["show", &original_a, "--all", "--json"], None)["text"],
        "Original A remains private until an explicit sync."
    );
    let b_peer = joined["status"]["endpoint"].as_str().unwrap();
    let revoked = a.ok(&["sync:revoke", b_peer, "--json"], None);
    assert_eq!(revoked["status"]["members"].as_array().unwrap().len(), 2);
    assert!(
        revoked["status"]["revoked"]
            .as_array()
            .unwrap()
            .iter()
            .any(|key| key == b_peer)
    );
    let refreshed = c.ok(&["sync:pair", "-y", "--json"], None);
    assert_eq!(refreshed["members"].as_array().unwrap().len(), 2);
    a.ok(&["daemon:stop", "--json"], None);
    let restored = a.ok(&["sync:status", "--json"], None);
    assert_eq!(restored["members"].as_array().unwrap().len(), 2);
    assert_eq!(
        a.ok(&["show", &original_a, "--all", "--json"], None)["text"],
        "Original A remains private until an explicit sync."
    );
    // Removing a store's identity keeps its own originals and does not revoke its sibling.
    b.ok(&["sync:reset-identity", "--json"], None);
    assert!(b.ok(&["sync:status", "--json"], None)["endpoint"].is_null());
    assert_eq!(
        b.ok(&["show", &original_b, "--all", "--json"], None)["text"],
        "Original B remains private until an explicit sync."
    );
    // Re-enrollment resumes a reset owner's transport before an immediate sync.
    let invitation = a.ok(&["sync:pair", "-y", "--json"], None);
    let rejoined = b.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(invitation["token"].as_str().unwrap()),
    );
    assert_ne!(rejoined["status"]["endpoint"], b_peer);
    assert_eq!(b.ok(&["sync", "--json"], None)["complete"], true);
    b.ok(&["sync:leave", "--json"], None);
    c.ok(&["sync:leave", "--json"], None);
    a.ok(&["sync:leave", "--json"], None);
}

#[test]
#[ignore = "requires an explicitly configured Worker"]
fn one_member_approval_admits_and_merges_without_sync_or_online_peers() {
    let service = std::env::var("LINN_RELAY_TEST_URL").unwrap();
    let a = Fixture::new();
    let b = Fixture::new();
    let c = Fixture::new();
    let d = Fixture::new();
    let original = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Admission does not require transferring this original."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let initialize = |peer: &Fixture| {
        peer.ok(
            &[
                "sync:pair",
                "-y",
                "--service",
                service.trim_end_matches('/'),
                "--json",
            ],
            None,
        )
    };
    let invitation = initialize(&a);
    a.ok(&["daemon:stop", "--json"], None);
    // The sole approver is already offline when its first invitation is redeemed.
    let admitted = b.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(invitation["token"].as_str().unwrap()),
    );
    assert_eq!(admitted["status"]["members"].as_array().unwrap().len(), 2);
    assert_eq!(admitted["status"]["enabled"], true);
    // A later member can approve another admission without the founder's participation.
    let invitation = b.ok(&["sync:pair", "-y", "--json"], None);
    b.ok(&["daemon:stop", "--json"], None);
    let other = initialize(&c);
    d.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(other["token"].as_str().unwrap()),
    );
    d.ok(&["daemon:stop", "--json"], None);
    let merged = c.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(invitation["token"].as_str().unwrap()),
    );
    assert_eq!(merged["operation"], "paired");
    assert_eq!(merged["status"]["enabled"], true);
    assert_eq!(merged["status"]["members"].as_array().unwrap().len(), 4);
    for peer in [&a, &b, &d] {
        assert_eq!(
            peer.ok(&["daemon:status", "--json"], None)["running"],
            false
        );
    }
    assert_eq!(
        c.run(&["show", &original, "--all", "--json"], None)
            .status
            .code(),
        Some(2)
    );
    // The accepted admission is durable without any explicit sync in this test.
    c.ok(&["daemon:stop", "--json"], None);
    let restored = c.ok(&["sync:status", "--json"], None);
    assert_eq!(restored["head"], merged["status"]["head"]);
    assert_eq!(restored["members"], merged["status"]["members"]);
    assert_eq!(restored["enabled"], true);
}

#[test]
#[ignore = "run through relay test:offline, which stops the service after pairing"]
fn offline_daemons_rediscover_changed_loopback_ports() {
    use std::{
        net::UdpSocket,
        time::{Duration, Instant},
    };
    let service = std::env::var("LINN_RELAY_TEST_URL").unwrap();
    let barrier = PathBuf::from(std::env::var_os("LINN_OFFLINE_BARRIER").unwrap());
    let a = Fixture::new();
    let b = Fixture::new();
    let invite = a.ok(
        &[
            "sync:pair",
            "-y",
            "--service",
            service.trim_end_matches('/'),
            "--json",
        ],
        None,
    );
    b.ok(
        &["sync:pair", "-y", "--join", "-", "--json"],
        Some(invite["token"].as_str().unwrap()),
    );
    let history = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Original history retained across an offline correction."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let forgotten = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("This original must be deleted on both offline peers."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(a.ok(&["sync", "--json"], None)["complete"], true);
    let before_a = a.ok(&["sync:status", "--json"], None);
    let before_b = b.ok(&["sync:status", "--json"], None);
    assert_ne!(before_a["endpoint"], before_b["endpoint"]);
    a.ok(&["daemon:stop", "--json"], None);
    b.ok(&["daemon:stop", "--json"], None);
    // Hold the old ports so success cannot depend on cached socket addresses.
    let held: Vec<_> = [&before_a, &before_b]
        .into_iter()
        .flat_map(|status| {
            status["transport"]["addresses"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|address| {
                    address
                        .as_str()
                        .unwrap()
                        .parse::<std::net::SocketAddr>()
                        .ok()
                })
                .filter(|address| address.is_ipv4())
                .map(|address| UdpSocket::bind(address).unwrap())
        })
        .collect();
    assert_eq!(held.len(), 2);
    std::fs::write(barrier.join("ready"), b"").unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    while !barrier.join("offline").exists() {
        assert!(Instant::now() < deadline, "service did not stop");
        std::thread::sleep(Duration::from_millis(20));
    }
    // Both owners start with an unavailable service, independently, with fresh ports.
    let after_a = a.ok(&["sync:status", "--json"], None);
    let after_b = b.ok(&["sync:status", "--json"], None);
    for (before, after) in [(&before_a, &after_a), (&before_b, &after_b)] {
        assert_eq!(before["endpoint"], after["endpoint"]);
        assert_ne!(
            before["transport"]["addresses"],
            after["transport"]["addresses"]
        );
        assert_eq!(after["transport"]["relay_connected"], false);
        assert_eq!(after["transport"]["local_discovery"], true);
    }
    let original = a.ok(
        &["add", "-", "--unscoped", "--json"],
        Some("Offline original survives a changed loopback port."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        b.run(&["show", &original, "--all", "--json"], None)
            .status
            .code(),
        Some(2)
    );
    let correction = a.ok(
        &["add", "-", "--unscoped", "--parent", &history, "--json"],
        Some("Offline correction preserves immutable parent bytes."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    b.ok(&["add:forget", &forgotten, "--json"], None);
    assert!(
        a.run(&["show", &forgotten, "--all", "--json"], None)
            .status
            .success()
    );
    let report = b.ok(&["sync", "--json"], None);
    assert_eq!(report["complete"], true, "{report}");
    assert_eq!(
        b.ok(&["show", &original, "--all", "--json"], None)["text"],
        "Offline original survives a changed loopback port."
    );
    let bundle = b.run(&["export", "--all-states"], None);
    assert!(
        bundle.status.success(),
        "{}",
        String::from_utf8_lossy(&bundle.stderr)
    );
    let items: Vec<serde_json::Value> = std::str::from_utf8(&bundle.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter_map(|frame| frame.get("item").cloned())
        .collect();
    let parent = items.iter().find(|item| item["id"] == history).unwrap();
    assert!(
        parent["envelope"]
            .as_str()
            .unwrap()
            .contains("Original history retained across an offline correction.")
    );
    let tombstone = items.iter().find(|item| item["id"] == forgotten).unwrap();
    assert!(tombstone["envelope"].is_null());
    assert_eq!(
        b.ok(&["show", &correction, "--all", "--json"], None)["text"],
        "Offline correction preserves immutable parent bytes."
    );
    for fixture in [&a, &b] {
        assert_eq!(
            fixture
                .run(&["show", &forgotten, "--all", "--json"], None)
                .status
                .code(),
            Some(2)
        );
    }
    assert_eq!(a.ok(&["sync", "--json"], None)["complete"], true);
    // First-time pairing still needs the service; it must not create a local mesh.
    let unpaired = Fixture::new();
    assert!(
        !unpaired
            .run(
                &[
                    "sync:pair",
                    "-y",
                    "--service",
                    service.trim_end_matches('/'),
                    "--json"
                ],
                None
            )
            .status
            .success()
    );
    assert!(unpaired.ok(&["sync:status", "--json"], None)["head"].is_null());
    drop(held);
}

#[test]
fn pairing_confirmation_is_explicit_before_membership_mutation() {
    let fixture = Fixture::new();
    for args in [
        vec!["sync:pair"],
        vec!["sync:pair", "--join", "-", "--json"],
    ] {
        let output = fixture.run(&args, None);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("pass -y"));
        assert!(output.stdout.is_empty());
        assert!(!fixture.store.exists());
    }
    for answer in ["", "n"] {
        let output =
            fixture.prompt_command(&["sync:pair"], b"Continue with pairing? [y/N] ", answer);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("pairing cancelled"));
        assert!(!fixture.store.exists());
    }
    let output = fixture.prompt_command(
        &["sync:pair", "--join", "invalid-token", "--no-sync"],
        b"Continue with pairing? [y/N] ",
        "y",
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("pairing cancelled"));
    assert!(!fixture.store.exists());
}

#[test]
fn no_op_sync_has_one_completion_and_outline_failure_is_json() {
    let fixture = Fixture::new();
    let output = fixture.run(&["sync"], None);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(text.matches("Sync complete").count(), 1);
    assert!(text.contains("No changes applied to this store"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Sync complete"));
    let record = fixture.ok(
        &["add", "-", "--json"],
        Some("Original note to close for an outline test."),
    )["record"]
        .as_str()
        .unwrap()
        .to_owned();
    fixture.ok(
        &["add:close", &record, "--reason", "correct", "--json"],
        None,
    );
    let output = fixture.run(&["show:outline", &record, "--json"], None);
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["status"], "not_current");
    let output = fixture.run(&["daemon:status"], None);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Store:"));
    assert!(text.contains("PID"));
}
