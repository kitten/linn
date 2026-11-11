use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn cli(db: &std::path::Path, args: &[&str], body: Option<&str>) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
        .arg("--store")
        .arg(db)
        .args(args)
        .stdin(if body.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
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
#[test]
fn stdin_versions_original_bytes_and_secure_forget() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("memory.db");
    let body = "A precise routing decision.\nUse the native menu.\n";
    let added = cli(
        &db,
        &[
            "add",
            "-",
            "--unscoped",
            "--source-path",
            "guide.md",
            "--json",
        ],
        Some(body),
    );
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let id = serde_json::from_slice::<Value>(&added.stdout).unwrap()["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let searched = cli(&db, &["search", "\"guide.md\"", "--json"], None);
    assert!(searched.status.success());
    let result: Value = serde_json::from_slice(&searched.stdout).unwrap();
    let fetch = result["cards"][0]["fetch"].as_str().unwrap();
    let shown = cli(&db, &["show", fetch, "--all", "--json"], None);
    assert!(shown.status.success());
    let result: Value = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(result["text"], body);
    let child = cli(
        &db,
        &["add", "-", "--unscoped", "--parent", &id, "--json"],
        Some("A replacement routing decision."),
    );
    assert!(child.status.success());
    let old = cli(&db, &["show", &id], None);
    assert_eq!(old.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&old.stdout).contains("superseded by"));
    let check = cli(&db, &["index:check"], None);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(cli(&db, &["add:forget", &id], None).status.success());
    assert!(cli(&db, &["index:check"], None).status.success());
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(db).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
#[test]
fn relative_xdg_is_an_error_and_help_needs_no_store() {
    let tmp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_linn"))
        .env("XDG_DATA_HOME", "relative")
        .args(["search", "anything"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(std::fs::read_dir(tmp.path()).unwrap().next().is_none());
    let output = Command::new(env!("CARGO_BIN_EXE_linn"))
        .env("XDG_DATA_HOME", "relative")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn damaged_index_has_maintenance_exit_code() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("broken.db");
    std::fs::write(&path, b"not a sqlite database").unwrap();
    let result = cli(&path, &["index:check"], None);
    assert_eq!(result.status.code(), Some(3));
    assert!(result.stdout.is_empty());
}

#[test]
fn cursor_replays_without_repeating_the_first_card() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("memory.db");
    let added = cli(
        &db,
        &["add", "-", "--unscoped"],
        Some("# First\nneedle routing.\n# Second\nneedle routing.\n# Third\nneedle routing.\n"),
    );
    assert!(added.status.success());
    let first = cli(&db, &["search", "needle routing", "--json"], None);
    assert!(first.status.success());
    let first: Value = serde_json::from_slice(&first.stdout).unwrap();
    let cursor = first["cursor"].as_str().expect("additional section cursor");
    let next = cli(&db, &["search:more", cursor, "--json"], None);
    assert!(next.status.success());
    let repeated = cli(&db, &["search:more", cursor, "--json"], None);
    assert_eq!(next.stdout, repeated.stdout);
    let next: Value = serde_json::from_slice(&next.stdout).unwrap();
    assert_ne!(first["cards"][0]["fetch"], next["cards"][0]["fetch"]);
}

#[tokio::test]
async fn obsolete_algorithm_settings_cannot_change_ingest_or_search() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("fixed.db");
    let record = serde_json::from_value(serde_json::json!({
        "id":"scoped", "body":"# Cache policy\nCache invalidation policy controls cached response lifetime.\n",
        "scope":"example/repository", "state":"accepted", "kind":"doc"
    })).unwrap();
    drop(linn::store::Store::build(&db, vec![record]).await.unwrap());
    let invoke = |args: &[&str], obsolete: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_linn"));
        command.arg("--store").arg(&db).args(args);
        for (name, value) in [
            ("LINN_POS_PAIRS", "0"),
            ("LINN_ORDERED_PAIR_WEIGHT", "0"),
            ("LINN_HEADING_TREE", "null-scope-only"),
        ] {
            command.env_remove(name);
            if obsolete {
                command.env(name, value);
            }
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    let baseline = invoke(&["search", "cache invalidation", "--json"], false);
    invoke(&["index:rebuild"], true);
    let actual = invoke(&["search", "cache invalidation", "--json"], true);
    let normalize = |bytes: &[u8]| {
        let mut value: Value = serde_json::from_slice(bytes).unwrap();
        // The cursor carries wall clock; preserve whether another call exists.
        if value["cursor"].is_string() {
            value["cursor"] = Value::String("clock-bearing cursor".into());
        }
        value
    };
    assert_eq!(normalize(&baseline.stdout), normalize(&actual.stdout));
    let store = linn::store::Store::open(&db).await.unwrap();
    for table in ["pos_relations", "nav_trees"] {
        let mut rows = store
            .db
            .query(&format!("SELECT count(*) FROM {table}"), ())
            .await
            .unwrap();
        assert!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap() > 0,
            "{table}"
        );
    }
}

#[test]
fn list_filters_paginates_and_validates_input() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("list.db");
    let mut ids = Vec::new();
    for body in ["First useful discovery.", "Second useful discovery."] {
        let output = cli(
            &db,
            &[
                "add",
                "-",
                "--unscoped",
                "--stale-after",
                "2099-01-01T00:00:00Z",
                "--json",
            ],
            Some(body),
        );
        assert!(output.status.success());
        ids.push(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["record"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let output = cli(
        &db,
        &[
            "list", "--kind", "note", "--since", "7d", "--limit", "1", "--json",
        ],
        None,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(first["records"].as_array().unwrap().len(), 1);
    let next = first["next"].as_str().unwrap();
    let output = cli(&db, &["list", "--cursor", next, "--json"], None);
    assert!(output.status.success());
    let second: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_ne!(
        first["records"][0]["record"],
        second["records"][0]["record"]
    );
    assert!(second["next"].is_null());
    let empty = cli(&db, &["list", "--expiring-within", "1d", "--json"], None);
    assert_eq!(empty.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&empty.stdout).unwrap()["records"],
        serde_json::json!([])
    );
    for args in [
        vec!["list", "--since", "🦀"],
        vec!["list", "--limit", "0"],
        vec!["list", "--cursor", "invalid"],
        vec!["list", "--cursor", next, "--kind", "note"],
    ] {
        assert_eq!(cli(&db, &args, None).status.code(), Some(2));
    }
    assert!(
        cli(&db, &["add:close", &ids[0], "--reason", "correct"], None)
            .status
            .success()
    );
    assert_eq!(
        cli(&db, &["list", "--cursor", next], None).status.code(),
        Some(2)
    );
}

#[tokio::test]
async fn git_email_attribution_is_captured_for_notes_documents_and_closes() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("attribution.db");
    let config = tmp.path().join("gitconfig");
    std::fs::write(&config, "[user]\nemail = global@example.org\n").unwrap();
    let write = |args: &[&str], body: Option<&str>| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
            .args(["--store", db.to_str().unwrap(), "--json"])
            .args(args)
            .current_dir(tmp.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", &config)
            .env("GIT_CONFIG_COUNT", "0")
            .env("GIT_AUTHOR_EMAIL", "not-the-configured-email@example.org")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(body) = body {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(body.as_bytes())
                .unwrap();
        } else {
            drop(child.stdin.take());
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["record"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let global = write(&["add", "-", "--unscoped"], Some("A global-config note."));
    for args in [
        vec!["init", "-q"],
        vec!["config", "--local", "user.email", "local@example.org"],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(tmp.path())
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(tmp.path().join("note.md"), "Original file content.").unwrap();
    let local = write(&["add", "note.md"], None);
    let close = write(&["add:close", &local, "--reason", "correct"], None);
    assert!(
        Command::new("git")
            .args(["config", "--local", "--unset", "user.email"])
            .current_dir(tmp.path())
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(&config, "").unwrap();
    let unknown = write(&["add", "-", "--unscoped"], Some("No configured email."));
    let mut store = linn::storage::Store::open(&db).await.unwrap();
    for rebuild in [false, true] {
        if rebuild {
            store.rebuild().await.unwrap();
        }
        for (id, expected) in [
            (&global, Some("global@example.org")),
            (&local, Some("local@example.org")),
            (&close, Some("local@example.org")),
            (&unknown, None),
        ] {
            let mut rows = store
                .db
                .query(
                    "SELECT writer,envelope FROM records WHERE id=?",
                    [id.as_str()],
                )
                .await
                .unwrap();
            let row = rows.next().await.unwrap().unwrap();
            let sql: Option<String> = row.get(0).unwrap();
            assert_eq!(sql.as_deref(), expected);
            let envelope: Value = serde_json::from_slice(&row.get::<Vec<u8>>(1).unwrap()).unwrap();
            assert_eq!(
                envelope.get("writer"),
                Some(&serde_json::to_value(expected).unwrap())
            );
        }
    }
}

#[test]
fn reference_filters_share_execution_and_display_resolution() {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("refs.db");
    let body = "# First\nResolver bridge exact details.\n\n# Second\nResolver bridge more details.\n\n# Third\nResolver bridge further details.\n";
    let added = cli(&db, &["add", "-", "--json"], Some(body));
    assert!(added.status.success());
    let id = serde_json::from_slice::<Value>(&added.stdout).unwrap()["record"]
        .as_str()
        .unwrap()
        .to_owned();
    let initial = cli(&db, &["search", "resolver bridge", "--json"], None);
    assert!(initial.status.success());
    let result: Value = serde_json::from_slice(&initial.stdout).unwrap();
    let fetch = result["cards"][0]["fetch"].as_str().unwrap();
    let hash = fetch.split(':').next().unwrap();
    for reference in [id.as_str(), fetch, hash] {
        for filter in [format!("id:{reference}"), format!("id:\"{reference}\"")] {
            let query = format!("resolver bridge {filter}");
            let output = cli(&db, &["search", &query, "--json"], None);
            assert!(
                output.status.success(),
                "{query}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let page: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert!(
                page["cards"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|card| card["record"] == id)
            );
            if let Some(cursor) = page["cursor"].as_str() {
                let value: Value = serde_json::from_slice(
                    &URL_SAFE_NO_PAD
                        .decode(cursor.strip_prefix("s1.").unwrap())
                        .unwrap(),
                )
                .unwrap();
                assert_eq!(value[0], query);
                let more = cli(&db, &["search:more", cursor, "--json"], None);
                assert!(more.status.success());
            }
        }
    }
    assert!(cli(&db, &["add:forget", &id], None).status.success());
    assert!(cli(&db, &["index:check"], None).status.success());
}

#[tokio::test]
async fn writer_filters_list_search_and_replay_without_reassigning_unknowns() {
    use linn::storage::Store;
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("writers.db");
    let mut records = Vec::new();
    for i in 0..12 {
        records.push(serde_json::from_value(serde_json::json!({
            "id":format!("record-{i:02}"),"body":format!("Routing marker integration number {i}."),
            "kind":"note","writer":if i%2==0 {"a@example.test"} else {"b@example.test"},
            "created":"2026-11-10T00:00:00Z"
        })).unwrap());
    }
    records.push(
        serde_json::from_value(
            serde_json::json!({"id":"null","body":"Routing marker unknown.","writer":null}),
        )
        .unwrap(),
    );
    records.push(
        serde_json::from_value(serde_json::json!({"id":"legacy","body":"Routing marker legacy."}))
            .unwrap(),
    );
    drop(Store::build(&db, records).await.unwrap());
    let out = cli(
        &db,
        &[
            "list",
            "--writer",
            "a@example.test",
            "--limit",
            "2",
            "--json",
        ],
        None,
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let page: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(page["records"][0]["record"], "record-10");
    let cursor = page["next"].as_str().unwrap();
    assert!(
        !cli(
            &db,
            &["list", "--cursor", cursor, "--writer", "a@example.test"],
            None
        )
        .status
        .success()
    );
    let out = cli(&db, &["list", "--cursor", cursor, "--json"], None);
    let next: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(next["records"][0]["record"], "record-06");
    for query in [
        "routing marker writer:a@example.test",
        "routing marker writer:\"a@example.test\"",
    ] {
        let out = cli(&db, &["search", query, "--json"], None);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut page: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert!(!page["cards"].as_array().unwrap().is_empty());
        loop {
            for key in ["cards", "hints", "related"] {
                for item in page[key].as_array().into_iter().flatten() {
                    let id = item["record"].as_str().unwrap();
                    let number = id
                        .strip_prefix("record-")
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    assert_eq!(number % 2, 0);
                }
            }
            let Some(cursor) = page["cursor"].as_str() else {
                break;
            };
            let out = cli(&db, &["search:more", cursor, "--json"], None);
            page = serde_json::from_slice(&out.stdout).unwrap();
        }
    }
    for query in [
        "routing marker writer:missing@example.test",
        "routing marker writer:A@example.test",
        "routing marker writer:a@example.test writer:b@example.test",
    ] {
        let out = cli(&db, &["search", query, "--json"], None);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let page: Value = serde_json::from_slice(&out.stdout).unwrap();
        for key in ["cards", "hints", "related"] {
            assert!(page[key].as_array().is_none_or(Vec::is_empty));
        }
    }
}
