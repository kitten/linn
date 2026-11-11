//! A separate test process starts with no previously initialized SQLite handles.
#[tokio::test]
async fn tokenizer_before_store_uses_libsql_initialization() {
    assert_eq!(linn::nlp::porter::word("running").unwrap(), "run");
    let db = libsql::Builder::new_local(":memory:")
        .build()
        .await
        .unwrap();
    let connection = db.connect().unwrap();
    connection
        .execute("CREATE TABLE probe(id INTEGER)", ())
        .await
        .unwrap();
}
