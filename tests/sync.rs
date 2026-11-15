use linn::storage::{
    Store,
    record::hash,
    sync::{Item, Replica},
};
use std::path::Path;
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
async fn merge(source: &Replica, target: &Replica, scope: Option<&str>) -> usize {
    let mut count = 0;
    for tombstones in [true, false] {
        let mut after = None;
        loop {
            let entries = source
                .inventory(tombstones, after.as_deref(), scope)
                .await
                .unwrap();
            if entries.is_empty() {
                break;
            }
            after = entries.last().map(|e| e.id.clone());
            for entry in entries {
                if target.needs(&entry).await.unwrap() {
                    let item = source.get(&entry.id, scope).await.unwrap().unwrap();
                    count += usize::from(target.apply(&item, scope).await.unwrap());
                }
            }
        }
    }
    count
}
async fn pull(local: &Replica, path: &Path, scope: Option<&str>) -> usize {
    merge(&Replica::open(path).await.unwrap(), local, scope).await
}
async fn push(local: &Replica, path: &Path, scope: Option<&str>) -> usize {
    let target = Replica::open(path).await.unwrap();
    let count = merge(local, &target, scope).await;
    target.finish().await.unwrap();
    count
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
async fn three_replica_item_merges_converge_and_retry_without_changes() {
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
    assert_eq!(pull(&local, &b, None).await, 1);
    assert_eq!(pull(&local, &c, None).await, 2);
    local.finish().await.unwrap();
    assert_eq!(push(&local, &b, None).await, 3);
    assert_eq!(push(&local, &c, None).await, 2);
    for replica in [&local, &peer_b, &peer_c] {
        assert_eq!(replica.inventory(false, None, None).await.unwrap().len(), 3);
        assert_eq!(replica.inventory(true, None, None).await.unwrap().len(), 1);
    }
    assert_eq!(pull(&local, &b, None).await, 0);
    assert_eq!(push(&local, &c, None).await, 0);
}
#[tokio::test]
async fn scoped_selection_never_transfers_unselected_bodies() {
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
    let scope = Some("org/repo");
    assert_eq!(pull(&local, &b, scope).await, 1);
    local.finish().await.unwrap();
    assert!(local.get("private", None).await.unwrap().is_none());
    assert!(
        local
            .apply(&item("outside", "Out of scope", "other"), scope)
            .await
            .is_err()
    );
    assert_eq!(push(&local, &b, scope).await, 0);
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
