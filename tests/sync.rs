use linn::{
    storage::{
        Store,
        record::hash,
        sync::{Entry, Item, Remote, Replica},
    },
    sync::{
        self,
        protocol::{self, Request, Response},
        ssh::Origin,
    },
};
use std::{
    io::{BufReader, Cursor},
    path::Path,
    process::{Child, Command, Stdio},
};
fn item(id: &str, body: &str, scope: &str) -> Item {
    // Deliberately noncanonical whitespace and an unknown field must survive.
    let envelope = format!(
        "{{ \"id\":{}, \"body\":{}, \"scope\":{}, \"unknown\":42 }}",
        serde_json::to_string(id).unwrap(),
        serde_json::to_string(body).unwrap(),
        serde_json::to_string(scope).unwrap()
    );
    Item {
        id: id.into(),
        envelope_hash: Some(hash(envelope.as_bytes())),
        body_hash: Some(hash(body.as_bytes())),
        envelope: Some(envelope),
    }
}
fn tombstone(id: &str) -> Item {
    Item {
        id: id.into(),
        envelope: None,
        envelope_hash: None,
        body_hash: None,
    }
}
fn remote() -> Remote {
    Remote {
        name: "peer".into(),
        origin: "ssh://peer".into(),
        scope: None,
    }
}
fn server(path: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_linn"))
        .arg("--store")
        .arg(path)
        .args(["sync:serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}
async fn pull(local: &Replica, path: &Path, r: &Remote) -> usize {
    let mut child = server(path);
    let mut w = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let n = sync::pull(local, r, &mut reader, &mut w).await.unwrap();
    drop(w);
    assert!(child.wait().unwrap().success());
    n
}
async fn push(local: &Replica, path: &Path, r: &Remote) -> usize {
    let mut child = server(path);
    let mut w = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let n = sync::push(local, r, &mut reader, &mut w).await.unwrap();
    drop(w);
    assert!(child.wait().unwrap().success());
    n
}
#[tokio::test]
async fn immutable_merge_conflicts_deletion_and_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let replica = Replica::open(&path).await.unwrap();
    let original = item("one", "Resolver integration evidence.", "org/repo");
    assert!(replica.apply(&original, None).await.unwrap());
    assert!(!replica.apply(&original, None).await.unwrap());
    assert!(Store::open(&path).await.is_err());
    assert!(
        replica
            .apply(&item("one", "Conflict.", "org/repo"), None)
            .await
            .is_err()
    );
    let mut damaged = item("two", "Other text", "org/repo");
    damaged.body_hash = Some("wrong".into());
    assert!(replica.apply(&damaged, None).await.is_err());
    replica.finish().await.unwrap();
    let mut store = Store::open(&path).await.unwrap();
    store.integrity().await.unwrap();
    store.rebuild().await.unwrap();
    store.integrity().await.unwrap();
    drop(store);
    assert_eq!(
        replica.get("one", None).await.unwrap().unwrap().envelope,
        original.envelope
    );
    assert!(replica.apply(&tombstone("one"), None).await.unwrap());
    assert!(!replica.apply(&original, None).await.unwrap());
    replica.finish().await.unwrap();
    let store = Store::open(&path).await.unwrap();
    store.integrity().await.unwrap();
    assert!(
        replica
            .get("one", None)
            .await
            .unwrap()
            .unwrap()
            .envelope
            .is_none()
    );
    assert!(
        replica
            .inventory(false, None, None)
            .await
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn three_replicas_pull_all_then_push_converge_and_retry_without_transfer() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.db");
    let b = dir.path().join("b.db");
    let c = dir.path().join("c.db");
    let local = Replica::open(&a).await.unwrap();
    let peer_b = Replica::open(&b).await.unwrap();
    let peer_c = Replica::open(&c).await.unwrap();
    for (replica, id) in [(&local, "a"), (&peer_b, "b"), (&peer_c, "c")] {
        replica
            .apply(
                &item(id, &format!("Original {id} resolver facts."), "repo"),
                None,
            )
            .await
            .unwrap();
    }
    peer_c.apply(&tombstone("gone"), None).await.unwrap();
    let r = remote();
    assert_eq!(pull(&local, &b, &r).await, 1);
    assert_eq!(pull(&local, &c, &r).await, 2);
    local.finish().await.unwrap();
    assert_eq!(push(&local, &b, &r).await, 3);
    assert_eq!(push(&local, &c, &r).await, 2);
    for replica in [&local, &peer_b, &peer_c] {
        assert_eq!(replica.inventory(false, None, None).await.unwrap().len(), 3);
        assert_eq!(replica.inventory(true, None, None).await.unwrap().len(), 1);
    }
    assert_eq!(pull(&local, &b, &r).await, 0);
    assert_eq!(push(&local, &c, &r).await, 0);
}
#[tokio::test]
async fn scoped_sync_never_transfers_unselected_bodies_and_configuration_is_local() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.db");
    let b = dir.path().join("b.db");
    let local = Replica::open(&a).await.unwrap();
    let peer = Replica::open(&b).await.unwrap();
    peer.apply(&item("shared", "Shared project bytes", "Org/Repo"), None)
        .await
        .unwrap();
    peer.apply(&item("private", "Other private project", "other"), None)
        .await
        .unwrap();
    peer.pair(&remote()).await.unwrap();
    let mut r = remote();
    r.scope = Some("org/repo".into());
    assert_eq!(pull(&local, &b, &r).await, 1);
    local.finish().await.unwrap();
    assert!(local.get("private", None).await.unwrap().is_none());
    assert!(local.remotes().await.unwrap().is_empty());
    assert!(
        local
            .apply(
                &item("outside", "Out of scope", "other"),
                r.scope.as_deref()
            )
            .await
            .is_err()
    );
    assert_eq!(push(&local, &b, &r).await, 0);
}
#[test]
fn origin_is_strict_and_remote_path_is_quoted() {
    for invalid in [
        "host:path",
        "https://host",
        "ssh://-x",
        "ssh://user:password@host",
        "ssh://host:0",
        "ssh://host:99999",
        "ssh://host?query",
        "ssh://host/%20",
        "ssh://host\ncommand",
    ] {
        assert!(Origin::parse(invalid).is_err(), "{invalid}");
    }
    let origin = Origin::parse("ssh://user@[::1]:2222/dir/it's $(touch nope).db").unwrap();
    assert_eq!(origin.destination, "ssh://user@[::1]:2222");
    assert_eq!(
        origin.remote_command(),
        "linn --store '/dir/it'\\''s $(touch nope).db' sync:serve --stdio"
    );
    assert_eq!(
        Origin::parse("ssh://alias").unwrap().remote_command(),
        "linn sync:serve --stdio"
    );
}
#[test]
fn protocol_limits_truncation_and_incompatible_version() {
    assert!(protocol::read::<Request>(&mut Cursor::new(b"{}")).is_err());
    assert!(
        protocol::read::<Request>(&mut Cursor::new(vec![b'x'; protocol::MAX_FRAME + 1])).is_err()
    );
    let mut stream = Vec::new();
    protocol::write(&mut stream, &Response::Hello { version: 999 }).unwrap();
    assert!(protocol::handshake(&mut Cursor::new(stream), &mut Vec::new()).is_err());
    let mut frame = Vec::new();
    protocol::write(
        &mut frame,
        &Entry {
            id: "one".into(),
            hash: None,
        },
    )
    .unwrap();
    assert_eq!(
        protocol::read::<Entry>(&mut Cursor::new(frame))
            .unwrap()
            .unwrap()
            .id,
        "one"
    );
}

#[tokio::test]
async fn inventories_are_keyset_paged_and_references_survive_sync() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paged.db");
    let replica = Replica::open(&path).await.unwrap();
    for i in 0..260 {
        replica
            .apply(&item(&format!("r{i:03}"), "", "repo"), None)
            .await
            .unwrap();
    }
    let first = replica.inventory(false, None, None).await.unwrap();
    assert_eq!(first.len(), 256);
    let second = replica
        .inventory(false, Some(&first.last().unwrap().id), None)
        .await
        .unwrap();
    assert_eq!(second.len(), 4);
    assert_eq!(second[0].id, "r256");
    let envelope = r#"{"id":"child","body":"Corrected procedure","parent":"r000","writer":"writer@example.org","scope":"repo"}"#;
    let child = Item {
        id: "child".into(),
        envelope: Some(envelope.into()),
        envelope_hash: Some(hash(envelope.as_bytes())),
        body_hash: Some(hash(b"Corrected procedure")),
    };
    replica.apply(&child, None).await.unwrap();
    replica.finish().await.unwrap();
    assert_eq!(
        replica
            .get("child", None)
            .await
            .unwrap()
            .unwrap()
            .envelope
            .as_deref(),
        Some(envelope)
    );
    let db = libsql::Builder::new_local(&path).build().await.unwrap();
    let conn = db.connect().unwrap();
    let mut rows = conn
        .query("SELECT writer FROM records WHERE id='child'", ())
        .await
        .unwrap();
    assert_eq!(
        rows.next()
            .await
            .unwrap()
            .unwrap()
            .get::<String>(0)
            .unwrap(),
        "writer@example.org"
    );
}

#[tokio::test]
async fn cli_orchestrates_all_pulls_before_pushes_and_stops_on_failed_pull() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.db");
    let b = dir.path().join("b quoted's.db");
    let c = dir.path().join("c.db");
    for (path, id) in [(&b, "from-b"), (&c, "from-c")] {
        let replica = Replica::open(path).await.unwrap();
        replica
            .apply(&item(id, "Original scoped content.", "repo"), None)
            .await
            .unwrap();
    }
    let executable = Path::new(env!("CARGO_BIN_EXE_linn"));
    let fake = dir.path().join("ssh");
    std::fs::write(&fake,r#"#!/bin/sh
for arg do
  case "$arg" in
    ssh://*) printf '%s\n' "$arg" >> "$SYNC_TEST_LOG"; case "$arg" in ssh://fail*) exit 255;; esac ;;
  esac
  last="$arg"
done
exec /bin/sh -c "$last"
"#).unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o700)).unwrap();
    let log = dir.path().join("ssh.log");
    let paths = std::env::join_paths(
        [
            dir.path().to_owned(),
            executable.parent().unwrap().to_owned(),
        ]
        .into_iter()
        .chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        )),
    )
    .unwrap();
    let cli = |args: &[&str]| {
        Command::new(executable)
            .arg("--store")
            .arg(&a)
            .args(args)
            .env("PATH", &paths)
            .env("SYNC_TEST_LOG", &log)
            .output()
            .unwrap()
    };
    for (name, path) in [("b", &b), ("c", &c)] {
        let origin = format!("ssh://peer{}", path.display());
        assert!(
            cli(&["sync:remote", "pair", name, &origin])
                .status
                .success()
        );
    }
    let output = cli(&["sync", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let counts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(counts["pulled"], 2);
    assert_eq!(counts["pushed"], 2);
    let calls = std::fs::read_to_string(&log).unwrap();
    assert_eq!(calls.lines().count(), 4);
    for path in [&a, &b, &c] {
        assert_eq!(
            Replica::open(path)
                .await
                .unwrap()
                .inventory(false, None, None)
                .await
                .unwrap()
                .len(),
            2
        );
    }
    // A failed pull must prevent even a single push to a healthy peer.
    assert!(
        cli(&["sync:remote", "pair", "z-failure", "ssh://fail"])
            .status
            .success()
    );
    let peer = Replica::open(&b).await.unwrap();
    peer.apply(
        &item("late", "New bytes before a failed pull.", "repo"),
        None,
    )
    .await
    .unwrap();
    std::fs::write(&log, "").unwrap();
    let failed = cli(&["sync"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("pull from z-failure failed"));
    Store::open(&a).await.unwrap().integrity().await.unwrap();
    assert!(
        Replica::open(&a)
            .await
            .unwrap()
            .get("late", None)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        Replica::open(&c)
            .await
            .unwrap()
            .get("late", None)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 3);
    assert!(
        cli(&["sync:remote", "remove", "z-failure"])
            .status
            .success()
    );
    assert!(
        !cli(&["sync:remote", "pair", "b", "ssh://other"])
            .status
            .success()
    );
    let output = cli(&["sync:remote", "list", "--json"]);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["remotes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn stdio_interruption_is_durable_and_sync_without_peers_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("interrupted.db");
    let mut child = server(&path);
    let mut writer = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    protocol::handshake(&mut reader, &mut writer).unwrap();
    assert!(matches!(
        protocol::exchange(
            &mut reader,
            &mut writer,
            &Request::Apply {
                item: item("one", "Remember this resolver detail.", "repo"),
                scope: None
            }
        )
        .unwrap(),
        Response::Applied { changed: true }
    ));
    drop(writer);
    assert!(child.wait().unwrap().success());
    assert!(Store::open(&path).await.is_err());
    let output = Command::new(env!("CARGO_BIN_EXE_linn"))
        .arg("--store")
        .arg(&path)
        .args(["sync", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Store::open(&path).await.unwrap().integrity().await.unwrap();
    let replica = Replica::open(&path).await.unwrap();
    assert!(replica.get("one", None).await.unwrap().is_some());
    let mut child = server(&path);
    let mut writer = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    assert!(
        protocol::exchange(&mut reader, &mut writer, &Request::Hello { version: 999 }).is_err()
    );
    drop(writer);
    assert!(!child.wait().unwrap().success());
}

#[tokio::test]
async fn rebuild_rejects_conflicting_originals_before_changing_indexes() {
    use linn::storage::record::Record;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("conflict.db");
    let original: Record = serde_json::from_value(serde_json::json!({
        "id":"same", "body":"Original alpha bytes", "label":"original", "state":"accepted"
    }))
    .unwrap();
    drop(Store::build(&path, vec![original.clone()]).await.unwrap());
    let database = libsql::Builder::new_local(&path).build().await.unwrap();
    let db = database.connect().unwrap();
    let mut rows = db
        .query("SELECT envelope_hash FROM records WHERE id='same'", ())
        .await
        .unwrap();
    let before: String = rows.next().await.unwrap().unwrap().get(0).unwrap();
    drop(rows);
    for metadata_only in [false, true] {
        let mut conflict = original.clone();
        if metadata_only {
            conflict.label = "different label".into();
        } else {
            conflict.body = "Replacement beta bytes".into();
        }
        let error = Store::build(&path, vec![conflict]).await.err().unwrap();
        assert!(
            error
                .to_string()
                .contains("immutable record conflict: same")
        );
        let mut rows = db.query("SELECT CAST(r.body AS TEXT),r.envelope_hash,s.body FROM records r JOIN sections s ON s.record=r.id", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<String>(0).unwrap(), original.body);
        assert_eq!(row.get::<String>(1).unwrap(), before);
        assert_eq!(row.get::<String>(2).unwrap(), original.body);
        drop(row);
        drop(rows);
        Store::open(&path).await.unwrap().integrity().await.unwrap();
    }
    // Identical input remains valid and raw-envelope tests cover noncanonical JSON.
    Store::build(&path, vec![original])
        .await
        .unwrap()
        .integrity()
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_writes_require_reopen_and_report_committed_publication() {
    use linn::storage::record::Record;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("failed.db");
    let make = |id: &str| -> Record {
        serde_json::from_value(serde_json::json!({"id":id,"body":"Original resolver evidence."}))
            .unwrap()
    };
    let mut store = Store::build(&path, vec![make("existing")]).await.unwrap();
    let db = libsql::Builder::new_local(&path).build().await.unwrap();
    let conn = db.connect().unwrap();
    conn.execute_batch("CREATE TRIGGER abort_record BEFORE INSERT ON records WHEN new.id='failed' BEGIN SELECT RAISE(ABORT,'injected abort'); END;").await.unwrap();
    // Reopen after installing the trigger so this probes insertion, not external-write invalidation.
    drop(store);
    store = Store::open(&path).await.unwrap();
    assert!(
        store
            .add(make("failed"))
            .await
            .unwrap_err()
            .to_string()
            .contains("injected abort")
    );
    assert!(
        store
            .add(make("later"))
            .await
            .unwrap_err()
            .to_string()
            .contains("discard")
    );
    assert!(store.agent("resolver", &Default::default()).await.is_err());
    drop(store);
    let mut store = Store::open(&path).await.unwrap();
    assert!(!store.contains_record("failed").await.unwrap());
    store.add(make("later")).await.unwrap();
    drop(store);
    conn.execute_batch("DROP TRIGGER abort_record; CREATE TRIGGER abort_publication BEFORE INSERT ON nav_sections BEGIN SELECT RAISE(ABORT,'publication abort'); END;").await.unwrap();
    let mut store = Store::open(&path).await.unwrap();
    assert!(
        store
            .add(make("committed"))
            .await
            .unwrap_err()
            .to_string()
            .contains("publication abort")
    );
    drop(store);
    assert!(Store::open(&path).await.is_err());
    assert!(
        Replica::open(&path)
            .await
            .unwrap()
            .get("committed", None)
            .await
            .unwrap()
            .is_some()
    );
    conn.execute_batch("DROP TRIGGER abort_publication;")
        .await
        .unwrap();
    Replica::open(&path).await.unwrap().finish().await.unwrap();
    Store::open(&path).await.unwrap().integrity().await.unwrap();
}

#[tokio::test]
async fn integrity_detects_original_hash_and_vector_damage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("integrity.db");
    let replica = Replica::open(&path).await.unwrap();
    replica
        .apply(&item("one", "Resolver bytes.", "repo"), None)
        .await
        .unwrap();
    replica.finish().await.unwrap();
    let store = Store::open(&path).await.unwrap();
    store.integrity().await.unwrap();
    let db = libsql::Builder::new_local(&path).build().await.unwrap();
    let conn = db.connect().unwrap();
    conn.execute("UPDATE records SET body_hash='wrong'", ())
        .await
        .unwrap();
    assert!(
        store
            .integrity()
            .await
            .unwrap_err()
            .to_string()
            .contains("original record")
    );
    conn.execute("UPDATE records SET body_hash=?", [hash(b"Resolver bytes.")])
        .await
        .unwrap();
    conn.execute("DELETE FROM vectors", ()).await.unwrap();
    assert!(
        store
            .integrity()
            .await
            .unwrap_err()
            .to_string()
            .contains("coverage mismatch")
    );
}
