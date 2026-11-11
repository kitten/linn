use linn::storage::{
    Store,
    bundle::{Selection, import_bundle, stage},
    sync::{Item, Replica},
};
use serde_json::json;
use std::io::Cursor;

fn record(v: serde_json::Value) -> linn::storage::record::Record {
    serde_json::from_value(v).unwrap()
}
fn selection() -> Selection {
    Selection {
        now: "2026-11-11T00:00:00Z".into(),
        ..Default::default()
    }
}
fn items(bytes: &[u8]) -> Vec<serde_json::Value> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .filter_map(|v| v.get("item").cloned())
        .collect()
}
fn ids(bytes: &[u8]) -> Vec<String> {
    items(bytes)
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn filters_history_deletion_privacy_and_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let mut store = Store::build(&source, vec![
        record(json!({"id":"a","body":"Original amber text.","kind":"note","scope":"Org/Repo","created":"2026-11-10T00:00:00Z","writer":"a@example.test","source_path":"/does/not/exist"})),
        record(json!({"id":"b","body":"Original blue text.","kind":"doc","scope":"Org/Repo","created":"2026-11-09T00:00:00Z"})),
        record(json!({"id":"other","body":"Other project original.","scope":"Other/Repo","kind":"note"})),
        record(json!({"id":"expired","body":"Expired original.","stale_after":"2026-11-01T00:00:00Z"})),
        record(json!({"id":"candidate","body":"Candidate original.","state":"candidate"})),
        record(json!({"id":"parent","body":"Original older version.","scope":"Org/Repo","kind":"note"})),
        record(json!({"id":"child","body":"Original newer version.","parent":"parent","scope":"Other/Repo","kind":"note"})),
        record(json!({"id":"closed","body":"Original closed record."})),
        record(json!({"id":"close","body":"","close":{"target":"closed","reason":"correct"}})),
        record(json!({"id":"forgotten","body":"Original forgotten content."})),
    ]).await.unwrap();
    store.forget("forgotten").await.unwrap();
    let mut bytes = Vec::new();
    store.export_bundle(&selection(), &mut bytes).await.unwrap();
    assert_eq!(ids(&bytes), ["a", "b", "child", "other"]);
    let mut filter = selection();
    filter.scope = Some("org/repo".into());
    filter.kind = Some("note".into());
    filter.since = Some("2026-11-10T00:00:00Z".into());
    filter.ids = vec!["a".into(), "b".into(), "other".into()];
    bytes.clear();
    store.export_bundle(&filter, &mut bytes).await.unwrap();
    assert_eq!(ids(&bytes), ["a"]);
    let target = Replica::open(&dir.path().join("target.db")).await.unwrap();
    let imported = import_bundle(&target, stage(&mut Cursor::new(&bytes)).unwrap())
        .await
        .unwrap();
    assert_eq!(imported.changed, 1);
    assert_eq!(
        import_bundle(&target, stage(&mut Cursor::new(&bytes)).unwrap())
            .await
            .unwrap()
            .unchanged,
        1
    );
    let original = Replica::open(&source)
        .await
        .unwrap()
        .get("a", None)
        .await
        .unwrap()
        .unwrap();
    let copied = target.get("a", None).await.unwrap().unwrap();
    assert_eq!(original.envelope, copied.envelope);
    assert_eq!(original.envelope_hash, copied.envelope_hash);
    assert!(target.get("other", None).await.unwrap().is_none());

    let mut history = selection();
    history.all_states = true;
    history.scope = Some("Org/Repo".into());
    assert!(
        store
            .export_bundle(&history, &mut Vec::new())
            .await
            .unwrap_err()
            .to_string()
            .contains("witness")
    );
    history.scope = None;
    bytes.clear();
    store.export_bundle(&history, &mut bytes).await.unwrap();
    assert_eq!(ids(&bytes).len(), 10);
    assert_eq!(ids(&bytes)[0], "forgotten");
    import_bundle(&target, stage(&mut Cursor::new(&bytes)).unwrap())
        .await
        .unwrap();
    let full = Store::open(&dir.path().join("target.db")).await.unwrap();
    let mut current = Vec::new();
    full.export_bundle(&selection(), &mut current)
        .await
        .unwrap();
    let mut source_current = Vec::new();
    store
        .export_bundle(&selection(), &mut source_current)
        .await
        .unwrap();
    assert_eq!(current, source_current);
    full.integrity().await.unwrap();

    history.ids = vec!["forgotten".into()];
    bytes.clear();
    store.export_bundle(&history, &mut bytes).await.unwrap();
    assert_eq!(ids(&bytes), ["forgotten"]);
    history.scope = Some("Org/Repo".into());
    bytes.clear();
    store.export_bundle(&history, &mut bytes).await.unwrap();
    assert!(ids(&bytes).is_empty());
}

#[tokio::test]
async fn damaged_bundles_conflicts_and_raw_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let replica = Replica::open(&source).await.unwrap();
    let raw =
        "{ \"id\":\"a\", \"body\":\"Original café 🦀.\", \"terms\":[\"unread\"], \"writer\":null }";
    let item = Item {
        id: "a".into(),
        envelope: Some(raw.into()),
        envelope_hash: Some(linn::storage::record::hash(raw.as_bytes())),
        body_hash: Some(linn::storage::record::hash("Original café 🦀.".as_bytes())),
    };
    replica.apply(&item, None).await.unwrap();
    replica.finish().await.unwrap();
    let store = Store::open(&source).await.unwrap();
    let mut bytes = Vec::new();
    store.export_bundle(&selection(), &mut bytes).await.unwrap();
    assert_eq!(items(&bytes)[0]["envelope"].as_str().unwrap(), raw);
    for damaged in [
        bytes[..bytes.len() - 1].to_vec(),
        bytes.split(|b| *b == b'\n').next().unwrap().to_vec(),
        [bytes.as_slice(), b"{}\n"].concat(),
        String::from_utf8(bytes.clone())
            .unwrap()
            .replace("Original", "Modified")
            .into_bytes(),
        String::from_utf8(bytes.clone())
            .unwrap()
            .replace("\"version\":1", "\"version\":2")
            .into_bytes(),
    ] {
        assert!(stage(&mut Cursor::new(damaged)).is_err());
    }
    let destination = dir.path().join("destination.db");
    drop(
        Store::build(
            &destination,
            vec![record(json!({"id":"a","body":"Conflicting original."}))],
        )
        .await
        .unwrap(),
    );
    let dest = Replica::open(&destination).await.unwrap();
    assert!(
        import_bundle(&dest, stage(&mut Cursor::new(&bytes)).unwrap())
            .await
            .unwrap_err()
            .to_string()
            .contains("conflict")
    );
    Store::open(&destination)
        .await
        .unwrap()
        .integrity()
        .await
        .unwrap();
}

fn cli(db: &std::path::Path, args: &[&str], input: Option<&[u8]>) -> std::process::Output {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_linn"))
        .arg("--store")
        .arg(db)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(bytes) = input {
        child.stdin.take().unwrap().write_all(bytes).unwrap();
    }
    child.wait_with_output().unwrap()
}

#[tokio::test]
async fn cli_files_stdio_references_and_no_clobber() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let body = "Original portable engineering knowledge.";
    drop(Store::build(&source, vec![record(json!({"id":"a","body":body,"kind":"note","scope":"org/project","created":"2026-11-10T00:00:00Z"}))]).await.unwrap());
    let export = dir.path().join("bundle.jsonl");
    let reference = format!(
        "{}:v1:0:8",
        &linn::storage::record::hash(body.as_bytes())[..12]
    );
    let out = cli(
        &source,
        &[
            "export",
            "--id",
            &reference,
            "--scope",
            "ORG/PROJECT",
            "--since",
            "2026-11-01T00:00:00Z",
            "--kind",
            "note",
            "--output",
            export.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        out.status.success(),
        "export exited with {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
    assert_eq!(
        std::fs::metadata(&export).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let bytes = std::fs::read(&export).unwrap();
    assert_eq!(ids(&bytes), ["a"]);
    assert!(
        !cli(
            &source,
            &["export", "--output", export.to_str().unwrap()],
            None
        )
        .status
        .success()
    );
    assert_eq!(bytes, std::fs::read(&export).unwrap());
    let stdout = cli(&source, &["export", "--json"], None);
    assert!(stdout.status.success());
    assert_eq!(ids(&stdout.stdout), ["a"]);
    let destination = dir.path().join("destination.db");
    let imported = cli(&destination, &["import", "-", "--json"], Some(&bytes));
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&imported.stdout).unwrap()["changed"],
        1
    );
    assert!(
        cli(&destination, &["import", export.to_str().unwrap()], None)
            .status
            .success()
    );
    assert!(cli(&destination, &["index:check"], None).status.success());
    let missing = dir.path().join("never-created.db");
    assert!(
        !cli(
            &missing,
            &["import", "-"],
            Some(b"{\"format\":\"linn-export\",\"version\":1}\n")
        )
        .status
        .success()
    );
    assert!(!missing.exists());
    assert!(
        !cli(&source, &["export", "--id", "missing"], None)
            .status
            .success()
    );
    assert!(
        !cli(&source, &["export", "--since", "-3d"], None)
            .status
            .success()
    );
}

#[tokio::test]
async fn conflict_publishes_earlier_items_and_tombstone_prevents_revival() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let source_store = Store::build(
        &source,
        vec![
            record(json!({"id":"a","body":"A new original."})),
            record(json!({"id":"z","body":"Conflicting later item."})),
        ],
    )
    .await
    .unwrap();
    let mut bytes = Vec::new();
    source_store
        .export_bundle(&selection(), &mut bytes)
        .await
        .unwrap();
    let target = dir.path().join("target.db");
    drop(
        Store::build(
            &target,
            vec![record(json!({"id":"z","body":"Existing original."}))],
        )
        .await
        .unwrap(),
    );
    let replica = Replica::open(&target).await.unwrap();
    assert!(
        import_bundle(&replica, stage(&mut Cursor::new(&bytes)).unwrap())
            .await
            .is_err()
    );
    let mut store = Store::open(&target).await.unwrap();
    store.integrity().await.unwrap();
    assert!(replica.get("a", None).await.unwrap().is_some());
    store.forget("a").await.unwrap();
    let mut only_a = selection();
    only_a.ids.push("a".into());
    bytes.clear();
    source_store
        .export_bundle(&only_a, &mut bytes)
        .await
        .unwrap();
    assert_eq!(
        import_bundle(&replica, stage(&mut Cursor::new(&bytes)).unwrap())
            .await
            .unwrap()
            .unchanged,
        1
    );
    assert!(
        replica
            .get("a", None)
            .await
            .unwrap()
            .unwrap()
            .envelope
            .is_none()
    );
}
