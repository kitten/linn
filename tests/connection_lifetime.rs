//! @ref LLP 0001#connection-policy

#[tokio::test]
async fn native_handles_survive_owners_and_close_once() {
    // Re-exec just this small test with guarded allocations. The old double-close
    // then faults deterministically, instead of reading a still-mapped freed page.
    #[cfg(target_os = "macos")]
    if std::env::var_os("LINN_TEST_GUARD_MALLOC").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "native_handles_survive_owners_and_close_once"])
            .env("LINN_TEST_GUARD_MALLOC", "1")
            .env("DYLD_INSERT_LIBRARIES", "/usr/lib/libgmalloc.dylib")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "guarded connection teardown: {}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return;
    }

    for _ in 0..32 {
        let database = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        let connection = database.connect().unwrap();
        drop(database);
        connection
            .execute("CREATE TABLE t(value)", ())
            .await
            .unwrap();
        let transaction = connection.transaction().await.unwrap();
        let clone = connection.clone();
        drop(connection);
        transaction
            .execute("INSERT INTO t VALUES(7)", ())
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        let statement = clone.prepare("SELECT value FROM t").await.unwrap();
        let mut rows = statement.query(()).await.unwrap();
        drop(statement);
        drop(clone);
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            7
        );
        drop(rows);

        // No outstanding statements: both wrapper destructors run immediately.
        let database = libsql::Builder::new_local(":memory:")
            .build()
            .await
            .unwrap();
        drop(database.connect().unwrap());
    }
}
