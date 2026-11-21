#![cfg(unix)]
use std::{
    fs::File,
    io::{Read, Write},
    os::fd::{AsRawFd, FromRawFd},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

fn command(db: &std::path::Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_linn"));
    command
        .arg("--store")
        .arg(db)
        .args(args)
        .stdin(Stdio::null());
    for name in [
        "NO_COLOR",
        "TERM",
        "CI",
        "CLICOLOR",
        "COLOR",
        "FORCE_COLOR",
        "CLICOLOR_FORCE",
    ] {
        command.env_remove(name);
    }
    command.env("TERM", "xterm-256color");
    command
}

// Only the selected stream gets a PTY: verify stdout/stderr policy independence.
fn terminal(mut command: Command, stdout: bool) -> Output {
    let (mut master, mut slave) = (-1, -1);
    let mut size = libc::winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &raw mut size,
            )
        },
        0
    );
    let mut master = unsafe { File::from_raw_fd(master) };
    let slave = unsafe { File::from_raw_fd(slave) };
    // Do not leak one test's PTY into another test's child or detached owner.
    for fd in [master.as_raw_fd(), slave.as_raw_fd()] {
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) },
            0
        );
    }
    if stdout {
        command.stdout(slave).stderr(Stdio::piped());
    } else {
        command.stderr(slave).stdout(Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    drop(command);
    let mut bytes = Vec::new();
    let started = Instant::now();
    loop {
        let mut poll = libc::pollfd {
            fd: master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll, 1, 100) };
        if ready > 0 {
            let mut buffer = [0; 4096];
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => bytes.extend_from_slice(&buffer[..n]),
                Err(e) if e.raw_os_error() == Some(libc::EIO) => break,
                Err(e) => panic!("PTY read: {e}"),
            }
        }
        if started.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            panic!("CLI PTY test timed out");
        }
    }
    let mut output = child.wait_with_output().unwrap();
    if stdout {
        output.stdout = bytes;
    } else {
        output.stderr = bytes;
    }
    output
}

#[test]
fn help_errors_and_status_obey_per_stream_disable_policy_in_a_pty() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("unused.db");
    let help = terminal(command(&db, &["--help"]), true);
    assert!(help.status.success());
    assert!(help.stdout.contains(&0x1b), "human help should be styled");
    for (key, value) in [
        ("NO_COLOR", "1"),
        ("NO_COLOR", "0"),
        ("TERM", "dumb"),
        ("TERM", "unknown"),
        ("CI", "1"),
        ("CI", "true"),
        ("CLICOLOR", "0"),
        ("COLOR", "0"),
        ("FORCE_COLOR", "0"),
    ] {
        let mut cmd = command(&db, &["--help"]);
        cmd.env(key, value).env("CLICOLOR_FORCE", "1");
        let output = terminal(cmd, true);
        assert!(!output.stdout.contains(&0x1b), "{key}={value}");
    }
    let json_help = terminal(command(&db, &["--json", "--help"]), true);
    assert!(!json_help.stdout.contains(&0x1b));
    let error = terminal(command(&db, &["--unknown-option"]), false);
    assert_eq!(error.status.code(), Some(2));
    assert!(error.stderr.contains(&0x1b));
    let status = terminal(command(&db, &["daemon:status"]), false);
    assert!(status.status.success());
    assert!(
        !status.stdout.contains(&0x1b),
        "piped stdout must stay plain when stderr is a terminal"
    );
    let mut piped = command(&db, &["--help"]);
    piped.env("FORCE_COLOR", "3").env("CLICOLOR_FORCE", "1");
    assert!(!piped.output().unwrap().stdout.contains(&0x1b));
    assert!(!db.exists(), "presentation checks must not create a store");
}

#[test]
fn terminal_layout_preserves_original_body_and_plain_layout() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("memory.db");
    let mut add = command(&db, &["add", "-", "--label", "Readable label", "--json"]);
    let mut child = add
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"First original line\nSecond original line")
        .unwrap();
    let added = child.wait_with_output().unwrap();
    assert!(added.status.success());
    let id = serde_json::from_slice::<serde_json::Value>(&added.stdout).unwrap()["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let shown = terminal(command(&db, &["show", &id, "--all"]), true);
    let shown = String::from_utf8(shown.stdout)
        .unwrap()
        .replace("\r\n", "\n");
    assert!(
        shown.contains("\x1b[1mReadable label\x1b[0m\nFirst original line\nSecond original line\n")
    );
    let plain = command(&db, &["show", &id, "--all"]).output().unwrap();
    assert_eq!(
        plain.stdout,
        b"Readable label\nFirst original line\nSecond original line\n"
    );
    let listed = terminal(command(&db, &["list"]), true);
    let listed = String::from_utf8(listed.stdout).unwrap();
    assert!(listed.starts_with("\x1b[1mReadable label\x1b[0m"));
    assert!(listed.contains(&id));
    let json = terminal(command(&db, &["show", &id, "--all", "--json"]), true);
    assert!(!json.stdout.contains(&0x1b));
    let json: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json["text"], "First original line\nSecond original line");
    assert!(
        command(&db, &["daemon:stop"])
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn agent_pty_keeps_search_navigation_and_show_instructions_plain() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("memory.db");
    let body = "# First\nneedle routing.\n# Second\nneedle routing.\n# Third\nneedle routing.\n";
    let mut child = command(&db, &["add", "-", "--label", "Sections", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let added = child.wait_with_output().unwrap();
    assert!(added.status.success());
    let id = serde_json::from_slice::<serde_json::Value>(&added.stdout).unwrap()["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let first = command(&db, &["search", "needle routing", "--json"])
        .output()
        .unwrap();
    let first: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    let cursor = first["cursor"]
        .as_str()
        .expect("fixture has another section page");
    let fetch = first["cards"][0]["fetch"].as_str().unwrap();
    let cases = [
        vec!["search", "needle routing"],
        vec!["search", "\"needle routing missingunique879\""],
        vec!["search:more", cursor],
        vec!["show", fetch],
        vec!["show", &id, "--all"],
        vec!["show:outline", &id],
        vec!["search", "--help"],
        vec!["search:more", "--help"],
        vec!["show", "--help"],
        vec!["show:outline", "--help"],
    ];
    fn plain_text(bytes: Vec<u8>, cursor: bool) -> String {
        let mut text = String::from_utf8(bytes).unwrap().replace("\r\n", "\n");
        assert!(
            !text.contains('\x1b'),
            "generated ANSI in agent output: {text:?}"
        );
        assert!(!text.contains('\r'), "progress rewrite in agent output");
        if cursor && let Some(start) = text.find("more: linn search:more ") {
            let start = start + "more: linn search:more ".len();
            let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
            text.replace_range(start..end, "<cursor>");
        }
        text
    }
    for args in cases {
        let baseline = command(&db, &args).output().unwrap();
        let expected = plain_text(baseline.stdout, args[0] == "search");
        if args == ["search", "needle routing"] {
            assert!(expected.contains("[1] Sections"));
            assert!(expected.contains("    fetch "));
            assert!(expected.contains("more: linn search:more <cursor>"));
        }
        if args == ["search", "\"needle routing missingunique879\""] {
            assert!(expected.contains("Near misses (not results):"));
            assert!(expected.contains("matched: needle, routing; missing: missingunique879"));
        }
        if args[0] == "show:outline" && args.len() == 2 && args[1] == id {
            assert!(expected.contains("[o1]"));
            assert!(expected.contains("    fetch "));
        }
        for (key, value) in [
            ("NO_COLOR", "1"),
            ("TERM", "dumb"),
            ("CI", "1"),
            ("FORCE_COLOR", "0"),
        ] {
            let mut cmd = command(&db, &args);
            cmd.env(key, value).env("CLICOLOR_FORCE", "1");
            let output = terminal(cmd, true);
            assert_eq!(output.status, baseline.status, "{args:?}");
            assert_eq!(
                plain_text(output.stdout, args[0] == "search"),
                expected,
                "{key}={value}: {args:?}"
            );
            assert_eq!(output.stderr, baseline.stderr, "{args:?}");
        }
    }
    assert!(
        command(&db, &["daemon:stop"])
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn show_separates_notices_without_rewriting_original_text() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("memory.db");
    for body in [
        "é".repeat(1506),
        format!("{}\ntail", "x".repeat(1499)),
        "already terminated\n\n".into(),
    ] {
        let mut child = command(&db, &["add", "-", "--label", "Boundary", "--json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(body.as_bytes())
            .unwrap();
        let added = child.wait_with_output().unwrap();
        assert!(added.status.success());
        let id = serde_json::from_slice::<serde_json::Value>(&added.stdout).unwrap()["record"]
            .as_str()
            .unwrap()
            .to_owned();
        let shown = command(&db, &["show", &id]).output().unwrap();
        let mut displayed: String = body.chars().take(1500).collect();
        if !displayed.ends_with('\n') {
            displayed.push('\n');
        }
        let mut expected = format!("Boundary\n{displayed}");
        if body.chars().count() > 1500 {
            expected.push_str(&format!(
                "… ({} more characters; --all to print the rest)\n",
                body.chars().count() - 1500
            ));
        }
        assert_eq!(shown.stdout, expected.as_bytes());
        let full = command(&db, &["show", &id, "--all"]).output().unwrap();
        let separator = if body.ends_with('\n') { "" } else { "\n" };
        assert_eq!(
            full.stdout,
            format!("Boundary\n{body}{separator}").as_bytes()
        );
        let json = command(&db, &["show", &id, "--all", "--json"])
            .output()
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
        assert_eq!(json["text"], body);
    }
    assert!(
        command(&db, &["daemon:stop"])
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn close_confirmation_and_filtered_empty_lists_are_unambiguous() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("memory.db");
    for mode in ["plain", "human", "json"] {
        let mut child = command(&db, &["add", "-", "--label", "Target", "--json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"Original closure fixture.")
            .unwrap();
        let added = child.wait_with_output().unwrap();
        assert!(added.status.success());
        let id = serde_json::from_slice::<serde_json::Value>(&added.stdout).unwrap()["record"]
            .as_str()
            .unwrap()
            .to_owned();
        // A filtered list can be empty even though a current note exists.
        let empty = command(&db, &["list", "--kind", "doc"]).output().unwrap();
        assert_eq!(empty.status.code(), Some(1));
        assert_eq!(empty.stdout, b"No matching records.\n");
        let empty_json = command(&db, &["list", "--kind", "doc", "--json"])
            .output()
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&empty_json.stdout).unwrap(),
            serde_json::json!({"records":[],"next":null})
        );
        let mut args = vec!["add:close", &id, "--reason", "correct"];
        if mode == "json" {
            args.push("--json");
        }
        let output = if mode == "plain" {
            command(&db, &args).output().unwrap()
        } else {
            terminal(command(&db, &args), true)
        };
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        match mode {
            "plain" => {
                assert!(text.starts_with("close-") && text.ends_with('\n'));
                assert_eq!(text.split_whitespace().count(), 1);
                assert!(!text.contains('\x1b'));
            }
            "human" => {
                assert!(text.starts_with("\x1b[32mClosed.\x1b[0m"));
                assert!(text.contains("Close record (") && text.contains("close-"));
                assert!(
                    !text.contains(&id),
                    "returned envelope ID must not be confused with target ID"
                );
            }
            _ => {
                assert!(!text.contains('\x1b'));
                let value: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(value.as_object().unwrap().len(), 1);
                assert!(value["record"].as_str().unwrap().starts_with("close-"));
            }
        }
    }
    let status_help = command(&db, &["sync:status", "--help"]).output().unwrap();
    let status_help = String::from_utf8(status_help.stdout).unwrap();
    assert!(status_help.contains("optionally check peer connections"));
    assert!(!status_help.contains("without contacting peers"));
    let add_help = command(&db, &["add", "--help"]).output().unwrap();
    assert!(String::from_utf8_lossy(&add_help.stdout).contains("UTC expiry timestamp"));
    for args in [
        vec!["show", "--help"],
        vec!["help", "show"],
        vec!["--json", "show", "--help"],
    ] {
        let help = command(&db, &args).output().unwrap();
        let help = String::from_utf8(help.stdout).unwrap();
        assert!(help.contains("Usage: linn show"), "{help}");
        assert!(help.contains("entire referenced span"));
    }
    let outline_help = command(&db, &["show:outline", "--help"]).output().unwrap();
    assert!(
        String::from_utf8_lossy(&outline_help.stdout).contains("Open with linn show <fetch-id>")
    );
    let close_help = command(&db, &["add:close", "--help"]).output().unwrap();
    assert!(
        String::from_utf8_lossy(&close_help.stdout)
            .contains("Existing replacement record reference")
    );
    assert!(
        command(&db, &["daemon:stop"])
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn remaining_command_outcomes_are_readable_without_decorating_bundle_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("store.db");
    let document = dir.path().join("example.md");
    std::fs::write(&document, "# Example\nA presentation fixture.").unwrap();
    let saved = terminal(command(&db, &["add", document.to_str().unwrap()]), true);
    assert!(saved.status.success());
    assert!(
        String::from_utf8(saved.stdout)
            .unwrap()
            .starts_with("\x1b[32mSaved\x1b[0m")
    );
    let list = terminal(command(&db, &["list"]), true);
    let list = String::from_utf8(list.stdout).unwrap();
    assert!(list.contains("Document · Created:") && list.contains("Record ("));
    let bundle = dir.path().join("backup.jsonl");
    let exported = terminal(
        command(&db, &["export", "--output", bundle.to_str().unwrap()]),
        false,
    );
    assert!(exported.status.success());
    assert!(exported.stdout.is_empty());
    assert!(
        String::from_utf8(exported.stderr)
            .unwrap()
            .contains("Export saved")
    );
    assert!(!std::fs::read(&bundle).unwrap().contains(&0x1b));
    let imported = terminal(command(&db, &["import", bundle.to_str().unwrap()]), true);
    assert!(imported.status.success());
    let imported = String::from_utf8(imported.stdout).unwrap();
    assert!(imported.contains("Import complete") && imported.contains("0 changes applied,"));
    assert!(imported.contains("already present."));
    let daemon = terminal(command(&db, &["daemon:status"]), true);
    assert!(
        String::from_utf8(daemon.stdout)
            .unwrap()
            .starts_with("\x1b[32mBackground process running")
    );
    let checked = terminal(command(&db, &["index:check"]), true);
    assert!(checked.status.success());
    assert!(
        String::from_utf8(checked.stdout)
            .unwrap()
            .contains("Integrity check passed.")
    );
    assert!(
        command(&db, &["daemon:stop"])
            .output()
            .unwrap()
            .status
            .success()
    );
}
