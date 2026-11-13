use super::*;
use tokio::io::AsyncWriteExt;
#[tokio::test]
async fn framing_rejects_oversize_and_truncation() {
    let (mut tx, mut rx) = tokio::io::duplex(8);
    tx.write_u32((wire::MAX_FRAME + 1) as u32).await.unwrap();
    assert!(wire::receive::<Frame>(&mut rx).await.is_err());
    let (mut tx, mut rx) = tokio::io::duplex(16);
    tx.write_u32(12).await.unwrap();
    tx.write_all(b"{}").await.unwrap();
    drop(tx);
    assert!(wire::receive::<Frame>(&mut rx).await.is_err());
}
#[tokio::test]
async fn streamed_values_have_no_single_frame_record_limit() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("lw-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("s");
    let accept = listener(&path).unwrap();
    let target = path.clone();
    let client = async move {
        let mut stream = Stream::connect(target.to_fs_name::<GenericFilePath>().unwrap())
            .await
            .unwrap();
        for n in [
            0,
            wire::CHUNK_BYTES - 2,
            wire::CHUNK_BYTES - 1,
            wire::MAX_FRAME + 1,
        ] {
            let value: String = receive_json(&mut stream).await.unwrap();
            assert_eq!(value.len(), n);
            assert!(value.bytes().all(|b| b == b'x'));
        }
    };
    let server = async {
        let mut stream = accept.accept().await.unwrap();
        for n in [
            0,
            wire::CHUNK_BYTES - 2,
            wire::CHUNK_BYTES - 1,
            wire::MAX_FRAME + 1,
        ] {
            send_json(&mut stream, "x".repeat(n)).await.unwrap();
        }
    };
    tokio::join!(client, server);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn private_files_reject_symlinks_and_insecure_modes() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    std::fs::write(&target, b"").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
    let link = dir.path().join("link");
    symlink(&target, &link).unwrap();
    assert!(platform::private_file(&link).is_err());
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(platform::private_file(&target).is_err());
}
#[test]
fn socket_keys_are_short_and_distinguish_routing_paths() {
    let a = platform::Identity {
        account: 1,
        path: "/a/store.db".into(),
    };
    let b = platform::Identity {
        path: "/b/store.db".into(),
        ..a.clone()
    };
    assert_eq!(platform::key(&a).unwrap(), "xjqkf45vfjzexen7zxbdvgodby");
    assert_ne!(platform::key(&a).unwrap(), platform::key(&b).unwrap());
    assert!(platform::check_socket(Path::new(&format!("/tmp/{}", "x".repeat(200)))).is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn published_owner_survives_missing_initiating_client() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("la-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let store = dir.path().join("store.db");
    let location = Location::new(&store).unwrap();
    let config = Bootstrap {
        endpoint: Endpoint {
            identity: location.id.clone(),
            path: dir.path().join("s"),
            build: String::new(),
        },
        root: location.root,
        store: location.path,
        ready: dir.path().join("initiator-already-gone"),
        token: "test".into(),
    };
    tokio::task::LocalSet::new()
        .run_until(async {
            let task = tokio::task::spawn_local(daemon(config));
            timeout(WAIT, async {
                loop {
                    if let Ok(state) = control(&store, false).await
                        && state.running
                    {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert!(!control(&store, true).await.unwrap().running);
            task.await.unwrap().unwrap();
        })
        .await;
}

#[test]
fn routing_missing_stores_and_dangling_aliases_never_creates_database() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("nested/store.db");
    let alias = dir.path().join("alias.db");
    std::os::unix::fs::symlink("nested/store.db", &alias).unwrap();
    let direct = Location::new(&store).unwrap();
    let indirect = Location::new(&alias).unwrap();
    assert_eq!(direct.id, indirect.id);
    assert_eq!(direct.key, indirect.key);
    assert!(!store.exists());
    assert_eq!(platform::file_identity(&store).unwrap(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn admission_is_bounded_before_upload_and_control_bypasses_request_slots() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("lc-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = dir.path().join("s");
    let accept = listener(&socket).unwrap();
    let identity = platform::identity(&dir.path().join("store.db")).unwrap();
    let requests = std::sync::Arc::new(Semaphore::new(1));
    let (queue, _jobs) = mpsc::channel(1);
    let (stop, _stopping) = mpsc::channel(1);
    tokio::task::LocalSet::new().run_until(async {
        let server_id = identity.clone();
        let server = tokio::task::spawn_local(async move {
            let mut clients = Vec::new();
            for _ in 0..3 {
                let stream = accept.accept().await.unwrap();
                clients.push(tokio::task::spawn_local(client(stream, server_id.clone(),
                    queue.clone(), stop.clone(), requests.clone())));
            }
            for task in clients { let _ = task.await.unwrap(); }
        });
        let connect = |control| {
            let socket = &socket;
            let identity = &identity;
            async move {
                let mut stream = Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap()).await.unwrap();
                wire::send(&mut stream, &Frame::Hello(Hello { version: VERSION, build: String::new(), compatibility: COMPATIBILITY, schema: crate::storage::SCHEMA_VERSION,
                    identity: identity.clone(), store: identity.path.clone(), control })).await.unwrap();
                stream
            }
        };
        let mut stalled = connect(false).await;
        assert!(matches!(wire::receive(&mut stalled).await.unwrap(), Frame::Accepted));
        let mut overflow = connect(false).await;
        assert!(matches!(wire::receive(&mut overflow).await.unwrap(), Frame::Rejected(message) if message.contains("not executed")));
        let mut control = connect(true).await;
        assert!(matches!(wire::receive(&mut control).await.unwrap(), Frame::Accepted));
        wire::send(&mut control, &Frame::Status).await.unwrap();
        assert!(matches!(wire::receive(&mut control).await.unwrap(), Frame::State(state) if state.running));
        drop(stalled);
        server.await.unwrap();
    }).await;
}

#[tokio::test(flavor = "current_thread")]
async fn response_delivery_barrier_remains_held_until_a_stalled_stream_is_closed() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("ld-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = dir.path().join("s");
    let accept = listener(&socket).unwrap();
    let identity = platform::identity(&dir.path().join("store.db")).unwrap();
    let (queue, mut jobs) = mpsc::channel(1);
    let (stop, _stopping) = mpsc::channel(1);
    tokio::task::LocalSet::new()
        .run_until(async {
            let server_id = identity.clone();
            let server = tokio::task::spawn_local(async move {
                let stream = accept.accept().await.unwrap();
                client(
                    stream,
                    server_id,
                    queue,
                    stop,
                    std::sync::Arc::new(Semaphore::new(1)),
                )
                .await
            });
            let mut stream =
                Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap())
                    .await
                    .unwrap();
            wire::send(
                &mut stream,
                &Frame::Hello(Hello {
                    version: VERSION,
                    build: String::new(),
                    compatibility: COMPATIBILITY,
                    schema: crate::storage::SCHEMA_VERSION,
                    identity: identity.clone(),
                    store: identity.path,
                    control: false,
                }),
            )
            .await
            .unwrap();
            assert!(matches!(
                wire::receive(&mut stream).await.unwrap(),
                Frame::Accepted
            ));
            wire::send(&mut stream, &Frame::Invoke { id: 1 })
                .await
                .unwrap();
            send_json(
                &mut stream,
                Invocation {
                    operation: Operation::SyncHello { version: 1 },
                    caller: Caller {
                        scope: None,
                        writer: None,
                        diagnostics: Default::default(),
                    },
                },
            )
            .await
            .unwrap();
            wire::send(&mut stream, &Frame::End).await.unwrap();
            let job = jobs.recv().await.unwrap();
            let mut delivered = job.delivered;
            let output = tempfile::tempfile().unwrap();
            output.set_len(64 * 1024 * 1024).unwrap();
            job.reply
                .send(Ok(Reply {
                    value: Value::Removed("test".into()),
                    warnings: vec![],
                    output: Some(output),
                }))
                .ok()
                .unwrap();
            assert!(matches!(
                wire::receive(&mut stream).await.unwrap(),
                Frame::Reply { id: 1 }
            ));
            let envelope: ReplyEnvelope = receive_json(&mut stream).await.unwrap();
            assert!(envelope.output);
            tokio::time::sleep(Duration::from_millis(20)).await;
            assert!(
                matches!(
                    delivered.try_recv(),
                    Err(oneshot::error::TryRecvError::Empty)
                ),
                "the dispatcher must not pass the barrier while bytes remain queued"
            );
            drop(stream);
            assert!(server.await.unwrap().is_err());
            let _ = timeout(WAIT, delivered).await.unwrap();
        })
        .await;
}

#[test]
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn native_descriptor_list_includes_sparse_high_numbers() {
    use std::os::fd::{AsRawFd, FromRawFd};
    let file = tempfile::tempfile().unwrap();
    let high = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 4096) };
    assert!(high >= 4096);
    let high = unsafe { std::os::fd::OwnedFd::from_raw_fd(high) };
    let descriptors = platform::open_descriptors().unwrap();
    assert!(descriptors.contains(&file.as_raw_fd()));
    assert!(descriptors.contains(&high.as_raw_fd()));
}

#[tokio::test]
async fn rpc_rejects_invalid_replies_and_preserves_mutation_uncertainty() {
    use crate::error::{ApplicationError, ErrorCode};
    for case in 0..10 {
        let (mut caller, mut daemon) = tokio::io::duplex(1024);
        let request = Invocation {
            operation: if case >= 8 {
                Operation::Export(crate::application::ExportOptions {
                    scope: None,
                    kind: None,
                    since: None,
                    id: vec![],
                    all_states: false,
                })
            } else {
                Operation::Rebuild
            },
            caller: Caller {
                diagnostics: Default::default(),
                scope: None,
                writer: None,
            },
        };
        let client = exchange(&mut caller, request, None);
        let server = async {
            assert!(matches!(
                wire::receive(&mut daemon).await.unwrap(),
                Frame::Invoke { id: 1 }
            ));
            let _: Invocation = receive_json(&mut daemon).await.unwrap();
            receive_file(&mut daemon, &mut tempfile::tempfile().unwrap())
                .await
                .unwrap();
            match case {
                2 => wire::send(&mut daemon, &Frame::Reply { id: 2 })
                    .await
                    .unwrap(),
                3 => {
                    wire::send(&mut daemon, &Frame::Reply { id: 1 })
                        .await
                        .unwrap();
                    wire::send(&mut daemon, &Frame::Chunk(STANDARD.encode(b"{")))
                        .await
                        .unwrap();
                    wire::send(&mut daemon, &Frame::End).await.unwrap();
                }
                4 => {}
                5 => wire::send(&mut daemon, &Frame::Rejected("busy; not executed".into()))
                    .await
                    .unwrap(),
                6 => {
                    let mut error =
                        ApplicationError::new(Error::Invalid("invalid request".into()), false);
                    error.diagnostics.push("preserved diagnostic".into());
                    wire::send(&mut daemon, &Frame::Failure(error))
                        .await
                        .unwrap();
                }
                _ => {
                    wire::send(&mut daemon, &Frame::Reply { id: 1 })
                        .await
                        .unwrap();
                    send_json(
                        &mut daemon,
                        ReplyEnvelope {
                            value: match case {
                                1 => Value::Checked,
                                8 | 9 => Value::Exported,
                                _ => Value::Rebuilt,
                            },
                            warnings: vec![],
                            output: case >= 7,
                        },
                    )
                    .await
                    .unwrap();
                    if case == 9 {
                        let mut file = tempfile::tempfile().unwrap();
                        file.write_all(b"original bytes").unwrap();
                        file.rewind().unwrap();
                        send_file(&mut daemon, &mut file).await.unwrap();
                    }
                }
            }
            drop(daemon);
        };
        let (result, ()) = tokio::join!(client, server);
        if case == 0 || case == 9 {
            let reply = result.unwrap();
            if case == 9 {
                let mut bytes = Vec::new();
                reply.output.unwrap().read_to_end(&mut bytes).unwrap();
                assert_eq!(bytes, b"original bytes");
            }
        } else {
            let Err(Error::Application(error)) = result else {
                panic!("case {case} did not produce a typed error")
            };
            assert_eq!(
                error.mutation_may_have_committed,
                !matches!(case, 5 | 6 | 8),
                "case {case}"
            );
            if case == 6 {
                assert_eq!(error.diagnostics, ["preserved diagnostic"]);
                assert!(matches!(error.code, ErrorCode::Invalid));
            }
            if case == 3 {
                assert!(matches!(error.code, ErrorCode::Invalid));
            }
            if case == 4 || case == 8 {
                assert!(matches!(error.code, ErrorCode::Io));
            }
        }
    }
}

#[test]
#[cfg(target_os = "macos")]
fn routing_uses_native_spelling_on_case_insensitive_filesystems() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("MixedCase");
    std::fs::create_dir(&root).unwrap();
    let alias = dir.path().join("mixedcase");
    if !alias.exists() {
        return; // Case-sensitive volumes correctly treat these as different paths.
    }
    let missing = root.join("missing.db");
    assert_eq!(
        platform::resolve_store(&missing).unwrap(),
        platform::resolve_store(&alias.join("missing.db")).unwrap()
    );
    assert!(!missing.exists());
    let store = root.join("Store.db");
    std::fs::write(&store, b"").unwrap();
    let first = Location::new(&store).unwrap();
    let second = Location::new(&alias.join("store.DB")).unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(first.key, second.key);
}

#[tokio::test]
async fn handshake_checks_compatibility_and_schema_but_keeps_legacy_control() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("lv-")
        .tempdir_in(std::env::var_os("LINN_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into()))
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = dir.path().join("s");
    let accept = listener(&socket).unwrap();
    let identity = platform::identity(&dir.path().join("store.db")).unwrap();
    let (queue, _jobs) = mpsc::channel(1);
    let (stop, _stopping) = mpsc::channel(1);
    let schema = crate::storage::SCHEMA_VERSION;
    for (version, compatibility, schema, build, control, accepted) in [
        (
            VERSION,
            Some(COMPATIBILITY),
            Some(schema),
            "different-build-a",
            false,
            true,
        ),
        (
            VERSION,
            Some(COMPATIBILITY),
            Some(schema),
            "different-build-b",
            false,
            true,
        ),
        (
            VERSION,
            Some(COMPATIBILITY),
            Some(schema + 1),
            "",
            false,
            false,
        ),
        (
            VERSION,
            Some(COMPATIBILITY + 1),
            Some(schema),
            "",
            false,
            false,
        ),
        (
            VERSION,
            Some(COMPATIBILITY - 1),
            Some(schema),
            "",
            false,
            false,
        ),
        (VERSION, None, Some(schema), "old-build", false, false),
        (VERSION, Some(COMPATIBILITY), None, "", false, false),
        (VERSION, None, None, "old-build", true, true),
        (
            VERSION,
            Some(COMPATIBILITY + 1),
            Some(schema + 1),
            "future-build",
            true,
            true,
        ),
        (
            VERSION + 1,
            Some(COMPATIBILITY),
            Some(schema),
            "",
            true,
            false,
        ),
    ] {
        let server = async {
            let stream = accept.accept().await.unwrap();
            let _ = client(
                stream,
                identity.clone(),
                queue.clone(),
                stop.clone(),
                std::sync::Arc::new(Semaphore::new(1)),
            )
            .await;
        };
        let caller = async {
            let mut stream =
                Stream::connect(socket.as_path().to_fs_name::<GenericFilePath>().unwrap())
                    .await
                    .unwrap();
            let mut hello = serde_json::json!({"version":version,"build":build,"identity":identity,"store":identity.path,"control":control});
            if let Some(schema) = schema {
                hello["schema"] = schema.into();
            }
            if let Some(compatibility) = compatibility {
                hello["compatibility"] = compatibility.into();
            }
            wire::send(&mut stream, &serde_json::json!({"Hello":hello}))
                .await
                .unwrap();
            let frame: Frame = wire::receive(&mut stream).await.unwrap();
            assert_eq!(matches!(frame, Frame::Accepted), accepted);
            if accepted && control {
                wire::send(&mut stream, &Frame::Status).await.unwrap();
                assert!(
                    matches!(wire::receive(&mut stream).await.unwrap(), Frame::State(state) if state.running)
                );
            }
        };
        tokio::join!(server, caller);
    }
    std::fs::remove_file(socket).unwrap();
}

#[test]
fn json_streaming_does_not_require_a_temporary_directory() {
    if std::env::var_os("LINN_TEST_RPC_NO_TMPDIR").is_none() {
        let dir = tempfile::tempdir().unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runtime::tests::json_streaming_does_not_require_a_temporary_directory",
            ])
            .env("LINN_TEST_RPC_NO_TMPDIR", "1")
            .env("TMPDIR", dir.path().join("missing"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        for n in [
            0,
            wire::CHUNK_BYTES - 2,
            wire::CHUNK_BYTES - 1,
            wire::MAX_FRAME + 1,
        ] {
            let (mut tx, mut rx) = tokio::io::duplex(1024);
            let (sent, received) = tokio::join!(
                send_json(&mut tx, "x".repeat(n)),
                receive_json::<String>(&mut rx)
            );
            sent.unwrap();
            assert_eq!(received.unwrap(), "x".repeat(n));
        }
        let (mut tx, mut rx) = tokio::io::duplex(1024);
        let sender =
            tokio::spawn(async move { send_json(&mut tx, "x".repeat(wire::MAX_FRAME + 1)).await });
        assert!(matches!(
            wire::receive::<Frame>(&mut rx).await.unwrap(),
            Frame::Chunk(_)
        ));
        drop(rx);
        assert!(
            timeout(Duration::from_secs(2), sender)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
    });
}

#[test]
fn native_paths_round_trip_across_startup_handshake_and_file_input() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let path = PathBuf::from(OsStr::from_bytes(b"/native-\xff/path"));
    let identity = platform::Identity {
        account: 42,
        path: path.clone(),
    };
    let config = Bootstrap {
        endpoint: Endpoint {
            identity: identity.clone(),
            path: path.clone(),
            build: String::new(),
        },
        root: path.clone(),
        store: path.clone(),
        ready: path.clone(),
        token: "test".into(),
    };
    let encoded = serde_json::to_vec(&config).unwrap();
    let decoded: Bootstrap = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.endpoint.identity, identity);
    for actual in [
        decoded.endpoint.path,
        decoded.root,
        decoded.store,
        decoded.ready,
    ] {
        assert_eq!(actual, path);
    }
    let hello = Hello {
        version: VERSION,
        compatibility: COMPATIBILITY,
        build: String::new(),
        schema: crate::storage::SCHEMA_VERSION,
        identity: identity.clone(),
        store: path.clone(),
        control: false,
    };
    let decoded: Hello = serde_json::from_slice(&serde_json::to_vec(&hello).unwrap()).unwrap();
    assert_eq!(decoded.identity, identity);
    assert_eq!(decoded.store, path);
    for repository in [None, Some(path.clone())] {
        let input = crate::application::FileInput {
            path: path.clone(),
            repository: repository.clone(),
        };
        let decoded: crate::application::FileInput =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
        assert_eq!(decoded.path, path);
        assert_eq!(decoded.repository, repository);
    }
    let lossy = platform::Identity {
        path: path.to_string_lossy().into_owned().into(),
        ..identity.clone()
    };
    assert_ne!(
        platform::key(&identity).unwrap(),
        platform::key(&lossy).unwrap()
    );
    // UTF-8 routing must keep the key used by older owners, to avoid split ownership.
    let ascii = platform::Identity {
        account: 1,
        path: "/a/store.db".into(),
    };
    assert_eq!(
        serde_json::to_value(&ascii).unwrap(),
        serde_json::json!({"account":1,"path":"/a/store.db"})
    );
}

#[test]
fn native_path_codec_rejects_foreign_encodings_and_retains_legacy_endpoint_shape() {
    assert!(
        serde_json::from_value::<crate::application::FileInput>(serde_json::json!({
            "path":{"Windows":[47,100]}, "repository":null
        }))
        .is_err()
    );
    #[derive(Deserialize)]
    struct LegacyEndpoint {
        identity: platform::Identity,
        path: PathBuf,
        build: String,
    }
    let endpoint = Endpoint {
        identity: platform::Identity {
            account: 1,
            path: "/store.db".into(),
        },
        path: "/runtime/s".into(),
        build: String::new(),
    };
    let old: LegacyEndpoint =
        serde_json::from_slice(&serde_json::to_vec(&endpoint).unwrap()).unwrap();
    assert_eq!(old.identity, endpoint.identity);
    assert_eq!(old.path, endpoint.path);
    assert!(old.build.is_empty());
}
