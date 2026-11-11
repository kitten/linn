use linn::{
    query::Query,
    record::Record,
    store::{Context, Store},
};
fn rec(id: &str, body: &str, path: Option<&str>) -> Record {
    serde_json::from_value(serde_json::json!({"id":id,"body":body,"kind":"note","state":"accepted","source_path":path,"scope":"other/repo"})).unwrap()
}
#[tokio::test]
async fn filename_promotion_and_forget_survive_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("store.db");
    let mut store = Store::build(
        &path,
        vec![
            rec("a", "Alpha routing decisions.", Some("one/config.nix")),
            rec("b", "Beta deployment decisions.", Some("two/config.nix")),
            rec("c", "A unique source body.", Some("unique.nix")),
        ],
    )
    .await
    .unwrap();
    let ctx = Context {
        scope: Some("my/repo".into()),
        calls_left: 0,
        ..Context::default()
    };
    let calls = store.agent("\"UNIQUE.NIX\"", &ctx).await.unwrap();
    assert_eq!(calls[0].cards[0].record, "c");
    assert_eq!(calls[0].cards[0].text, "A unique source body.");
    let calls = store
        .agent("\"config.nix\" beta deployment", &ctx)
        .await
        .unwrap();
    assert_eq!(calls[0].cards[0].record, "b");
    store.integrity().await.expect("before forget");
    store.forget("a").await.unwrap();
    store.integrity().await.unwrap();
    drop(store);
    let mut store = Store::open(&path).await.unwrap();
    store.integrity().await.unwrap();
    let calls = store.agent("\"config.nix\"", &ctx).await.unwrap();
    assert_eq!(calls[0].cards[0].record, "b");
    assert!(store.rows.values().all(|b| b.record != "a"));
    let mut rows = store
        .db
        .query("SELECT scope FROM records", ())
        .await
        .unwrap();
    while let Some(row) = rows.next().await.unwrap() {
        assert_eq!(row.get_value(0).unwrap(), libsql::Value::Null);
    }
}
#[tokio::test]
async fn identifiers_and_two_porter_tokenizers_stay_distinct() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::build(
        &dir.path().join("store.db"),
        vec![
            rec("heading", "# MEM KEY\nordinary ordering body", None),
            rec("body", "MEM_KEY body and foo-bar", None),
        ],
    )
    .await
    .unwrap();
    let hits = store.fts(false, "{body}:\"MEM_KEY\"").await.unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(store.rows[hits.iter().next().unwrap()].record, "body");
    let stem = store.porter("foo-bar").await.unwrap();
    assert_eq!(stem, "bar");
    let hits = store.fts(true, "\"foo-bar\"").await.unwrap();
    assert_eq!(hits.len(), 1);
    assert!(Query::parse("scope:else/repo ordering").is_ok());
    let calls = store.agent("ordering", &Context::default()).await.unwrap();
    assert!(calls[0].cards.iter().any(|c| c.record == "heading"));
}
#[test]
fn index_crlf_and_cr_cuts_preserve_bytes() {
    let body = format!("{}\r\r- second\r\n- third\n", "x".repeat(1390));
    let spans = linn::sections::block_spans(&body, 0..body.len(), 1400);
    assert_eq!(
        spans.iter().map(|s| &body[s.clone()]).collect::<String>(),
        body
    );
    assert!(spans.len() > 1);
    // Vertical tabs and form feeds are NOT boundaries for bytes.splitlines.
    let text = "a\x0bb\x0cc";
    assert_eq!(linn::sections::block_spans(text, 0..text.len(), 2).len(), 1);
}

#[tokio::test]
async fn forgetting_a_close_keeps_projection_until_rebuild() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("store.db");
    let target = rec("target", "Original claim.", None);
    let close:Record=serde_json::from_value(serde_json::json!({"id":"close","body":"","state":"accepted","close":{"target":"target","reason":"expire","replacement":null}})).unwrap();
    let mut store = Store::build(&path, vec![target, close]).await.unwrap();
    assert!(!store.hint_eligible("target"));
    store.forget("close").await.unwrap();
    assert!(!store.hint_eligible("target"));
    drop(store);
    let mut store = Store::open(&path).await.unwrap();
    store.resolve_loaded("target", None).await.unwrap();
    assert!(!store.hint_eligible("target"));
    store.rebuild().await.unwrap();
    assert!(store.hint_eligible("target"));
    store.integrity().await.unwrap();
    let duplicate = rec("close", "Resurrection must be ignored.", None);
    drop(store);
    let store = Store::build(&path, vec![duplicate]).await.unwrap();
    assert!(!store.records.contains_key("close"));
}

#[tokio::test]
async fn shared_body_hash_prefers_the_callers_scope_case_insensitively() {
    let tmp = tempfile::tempdir().unwrap();
    let mut mine = rec("z", "The same immutable body.", None);
    mine.scope = Some("Kitten/Linn".into());
    let other = rec("a", "The same immutable body.", None);
    let hash = mine.body_hash();
    let store = Store::build(&tmp.path().join("store.db"), vec![other, mine])
        .await
        .unwrap();
    assert_eq!(
        store
            .resolve(&hash[..12], Some("kitten/linn"))
            .unwrap()
            .0
            .id,
        "z"
    );
}
