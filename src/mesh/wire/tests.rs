use super::*;
use crate::{
    mesh::{control_stream::Control, store},
    storage::sync::Item,
};
use iroh::{
    Endpoint, EndpointAddr, RelayMode,
    endpoint::{Connection, presets},
};

async fn connection() -> (Endpoint, Endpoint, Connection, Connection) {
    async fn endpoint() -> Endpoint {
        Endpoint::builder(presets::Minimal)
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .clear_address_lookup()
            .relay_mode(RelayMode::Disabled)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .unwrap()
    }
    let a = endpoint().await;
    let b = endpoint().await;
    let address = EndpointAddr::new(b.id()).with_ip_addr(b.bound_sockets()[0]);
    let (client, server) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(a.connect(address, ALPN), async {
            b.accept().await.unwrap().await
        })
    })
    .await
    .unwrap();
    (a, b, client.unwrap(), server.unwrap())
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn partial_original_frames_reset_on_forget_or_cancellation_between_leases() {
    for cancel in [false, true] {
        let (a, b, client, server) = connection().await;
        let (handle, mut jobs) = store::channel();
        let (mut send_stream, _) = client.open_bi().await.unwrap();
        let envelope =
            serde_json::json!({"id":"original", "body":"x".repeat(256 * 1024)}).to_string();
        let item = Item {
            id: "original".into(),
            envelope_hash: Some(crate::record::hash(envelope.as_bytes())),
            body_hash: None,
            envelope: Some(envelope),
        };
        let sender = tokio::spawn(async move {
            send(
                &mut send_stream,
                Frame::Response(Response::Item { item: Some(item) }),
                &handle,
                "approved",
            )
            .await
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            let first = jobs.recv().await.unwrap();
            assert!(matches!(first.step, Step::Chunk { .. }));
            assert!(first.reply.send(Ok(store::Value::Done)).is_ok());
            first.delivered.await.unwrap_err();
            let next = jobs.recv().await.unwrap();
            // The receiver has seen a partial frame before the authorization changes.
            let (_, mut recv) = server.accept_bi().await.unwrap();
            let mut prefix = [0; 64];
            recv.read_exact(&mut prefix).await.unwrap();
            if cancel {
                sender.abort();
            } else {
                assert!(
                    next.reply
                        .send(Err(Error::Invalid("original forgotten".into())))
                        .is_ok()
                );
            }
            let result = sender.await;
            assert!(if cancel {
                result.unwrap_err().is_cancelled()
            } else {
                result.unwrap().is_err()
            });
            next.delivered.await.unwrap_err();
            assert!(recv.read_to_end(MAX_FRAME).await.is_err());
        })
        .await
        .unwrap();
        a.close().await;
        b.close().await;
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP and a real heartbeat interval"]
async fn idle_control_streams_stay_live_without_application_messages() {
    let (a, b, client, server) = connection().await;
    let (handle, _jobs) = store::channel();
    let (send, recv) = client.open_bi().await.unwrap();
    let mut outgoing = Control::new(send, recv, handle.clone(), b.id().to_string());
    outgoing.send(Frame::Distribute).await.unwrap();
    let (send, recv) = server.accept_bi().await.unwrap();
    let mut incoming = Control::new(send, recv, handle, a.id().to_string());
    assert!(matches!(
        incoming.receive().await.unwrap(),
        Frame::Distribute
    ));
    // Longer than the no-progress read deadline: only heartbeats keep it alive.
    tokio::time::sleep(WAIT + Duration::from_secs(5)).await;
    incoming.send(Frame::Distribute).await.unwrap();
    assert!(matches!(
        outgoing.receive().await.unwrap(),
        Frame::Distribute
    ));
    drop(outgoing);
    drop(incoming);
    a.close().await;
    b.close().await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn remote_failures_keep_their_categories_on_control_streams() {
    use crate::mesh::failure::Kind;
    for kind in [
        Kind::Authentication,
        Kind::Version,
        Kind::Conflict,
        Kind::Publication,
        Kind::Timeout,
        Kind::Offline,
    ] {
        for connection_close in [false, true] {
            let (a, b, client, server) = connection().await;
            let (handle, _jobs) = store::channel();
            let (mut send, mut recv) = client.open_bi().await.unwrap();
            super::send(&mut send, Frame::Distribute, &handle, "peer")
                .await
                .unwrap();
            let (server_send, mut server_recv) = server.accept_bi().await.unwrap();
            assert!(matches!(
                receive(&mut server_recv).await.unwrap(),
                Frame::Distribute
            ));
            let control = Control::new(server_send, server_recv, handle, a.id().to_string());
            if connection_close {
                server.close(kind.code().into(), b"typed failure");
            } else {
                control.fail(kind).await;
            }
            let error = receive(&mut recv).await.err().unwrap();
            assert_eq!(Kind::of(&error), kind);
            drop(control);
            a.close().await;
            b.close().await;
        }
    }
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn incompatible_alpn_is_reported_as_a_version_failure() {
    use crate::mesh::failure::Kind;
    let a = Endpoint::builder(presets::Minimal)
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .bind()
        .await
        .unwrap();
    let b = Endpoint::builder(presets::Minimal)
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .alpns(vec![b"obsolete".to_vec()])
        .bind()
        .await
        .unwrap();
    let address = EndpointAddr::new(b.id()).with_ip_addr(b.bound_sockets()[0]);
    let (client, _) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(a.connect(address, ALPN), async {
            let _ = b.accept().await.unwrap().await;
        })
    })
    .await
    .unwrap();
    assert_eq!(Kind::of(&Kind::connect(client.unwrap_err())), Kind::Version);
    a.close().await;
    b.close().await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn heartbeats_cannot_replace_application_progress() {
    use crate::mesh::{failure::Kind, progress::Progress};
    let (a, b, client, server) = connection().await;
    let (handle, _jobs) = store::channel();
    let (send, recv) = client.open_bi().await.unwrap();
    let mut outgoing = Control::with_idle(
        send,
        recv,
        handle.clone(),
        b.id().to_string(),
        Duration::from_millis(100),
    );
    outgoing.send(Frame::Distribute).await.unwrap();
    let (send, recv) = server.accept_bi().await.unwrap();
    let mut incoming = Control::new(send, recv, handle, a.id().to_string());
    assert!(matches!(
        incoming.receive().await.unwrap(),
        Frame::Distribute
    ));
    let local = Progress::default();
    outgoing.track(local.clone(), false);
    // Parent progress must not become local progress and echo back indefinitely.
    incoming
        .send(Frame::Progress { revision: 1 })
        .await
        .unwrap();
    let keepalive = tokio::spawn(async move {
        for _ in 0..100 {
            incoming
                .send(Frame::Progress { revision: 1 })
                .await
                .unwrap();
            incoming.send(Frame::Distribute).await.unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        incoming
    });
    let error = tokio::time::timeout(Duration::from_millis(350), async {
        loop {
            match outgoing.receive().await {
                Ok(Frame::Distribute) => (),
                Ok(_) => panic!("unexpected control frame"),
                Err(error) => break error,
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(Kind::of(&error), Kind::Timeout);
    assert_eq!(local.revision(), 0);
    keepalive.abort();
    let _ = keepalive.await;
    drop(outgoing);
    a.close().await;
    b.close().await;
}

#[tokio::test]
#[ignore = "requires native loopback UDP"]
async fn application_progress_extends_the_control_deadline() {
    use crate::mesh::progress::Progress;
    let (a, b, client, server) = connection().await;
    let (handle, _jobs) = store::channel();
    let (send, recv) = client.open_bi().await.unwrap();
    let mut outgoing = Control::with_idle(
        send,
        recv,
        handle.clone(),
        b.id().to_string(),
        Duration::from_millis(100),
    );
    outgoing.send(Frame::Distribute).await.unwrap();
    let (send, recv) = server.accept_bi().await.unwrap();
    let mut incoming = Control::new(send, recv, handle, a.id().to_string());
    assert!(matches!(
        incoming.receive().await.unwrap(),
        Frame::Distribute
    ));
    let local = Progress::default();
    outgoing.track(local.clone(), true);
    let progressing = tokio::spawn(async move {
        for revision in 1..=8 {
            incoming.send(Frame::Progress { revision }).await.unwrap();
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
        incoming.send(Frame::Distribute).await.unwrap();
        incoming
    });
    assert!(matches!(
        outgoing.receive().await.unwrap(),
        Frame::Distribute
    ));
    assert_eq!(local.revision(), 8);
    let incoming = progressing.await.unwrap();
    drop(incoming);
    drop(outgoing);
    a.close().await;
    b.close().await;
}
