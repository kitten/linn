//! Native, patched-Iroh interoperability gate. Never contacts a public default relay.
use iroh::{
    Endpoint, EndpointAddr, NetReportConfig, RelayConfig, RelayMap, RelayMode, RelayUrl, SecretKey,
    Watcher, endpoint::presets,
};
use linn::mesh::{
    client::ControlClient,
    membership::{Action, Event, Membership, Proposal, endpoint as key_id, random_id},
};
use std::{
    error::Error,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::timeout;

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
const ALPN: &[u8] = b"linn/relay-interop/1";
const PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

#[tokio::test]
async fn selecting_one_relay_does_not_claim_a_successful_connection() -> TestResult {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let relay: RelayUrl = format!("http://{}/", listener.local_addr()?).parse()?;
    let endpoint = endpoint(SecretKey::generate(), relay.clone()).await?;
    assert!(
        timeout(Duration::from_secs(1), endpoint.online())
            .await
            .is_err()
    );
    let statuses = endpoint.home_relay_status().get();
    assert_eq!(
        statuses.len(),
        1,
        "the sole relay should be selected without a probe"
    );
    assert_eq!(statuses[0].url(), &relay);
    assert!(
        !statuses[0].is_connected(),
        "a pending handshake is not online"
    );
    endpoint.close().await;
    Ok(())
}

/// Run against a local or staging Worker using ordinary signed enrollment.
#[tokio::test]
#[ignore = "requires an explicitly configured Worker"]
async fn native_iroh_relay_only_roundtrip() -> TestResult {
    if std::env::var_os("LINN_RELAY_TEST_HEARTBEATS").is_some() {
        return relay_heartbeat_tolerance().await;
    }
    for peer_count in [2, 10] {
        relay_roundtrip(peer_count).await?;
    }
    Ok(())
}

async fn relay_heartbeat_tolerance() -> TestResult {
    let relay: RelayUrl = std::env::var("LINN_RELAY_TEST_URL")?.parse()?;
    let key = SecretKey::generate();
    enroll(relay.as_str().trim_end_matches('/'), &[&key]).await?;
    let endpoint = endpoint(key, relay).await?;
    let outcome = async {
        timeout(Duration::from_secs(10), endpoint.online()).await?;
        let start = std::time::Instant::now();
        let mut status = endpoint.home_relay_status();
        timeout(Duration::from_secs(90), async {
            loop {
                let statuses = status.updated().await.map_err(|_| "relay watcher closed")?;
                if !statuses.iter().any(|s| s.is_connected()) {
                    return Ok::<_, &str>(());
                }
            }
        })
        .await??;
        assert!(
            start.elapsed() >= Duration::from_secs(70),
            "delayed periodic Pong must not disconnect the client"
        );
        timeout(Duration::from_secs(10), endpoint.online()).await?;
        tokio::time::sleep(Duration::from_millis(250)).await;
        Ok::<_, Box<dyn Error + Send + Sync>>(())
    }
    .await;
    endpoint.close().await;
    outcome
}

async fn relay_roundtrip(peer_count: usize) -> TestResult {
    let relay: RelayUrl = std::env::var("LINN_RELAY_TEST_URL")?.parse()?;
    let server_key = SecretKey::generate();
    let client_key = SecretKey::generate();
    let idle_keys = (2..peer_count)
        .map(|_| SecretKey::generate())
        .collect::<Vec<_>>();
    let keys: Vec<_> = [&server_key, &client_key]
        .into_iter()
        .chain(idle_keys.iter())
        .collect();
    enroll(relay.as_str().trim_end_matches('/'), &keys).await?;

    let server = endpoint(server_key, relay.clone()).await?;
    let client = endpoint(client_key, relay.clone()).await?;
    let mut idle = Vec::new();
    for key in idle_keys {
        idle.push(endpoint(key, relay.clone()).await?);
    }
    let outcome = timeout(Duration::from_secs(60), async {
        tokio::try_join!(
            timeout(Duration::from_secs(30), server.online()),
            timeout(Duration::from_secs(30), client.online()),
        )?;
        for endpoint in &idle {
            timeout(Duration::from_secs(30), endpoint.online()).await?;
        }
        // Exercise control heartbeats and hibernation/wake before any QUIC data.
        let mut status = server.home_relay_status();
        let stable = async {
            loop {
                let statuses = status.updated().await.map_err(|_| "relay watcher closed")?;
                if !statuses.iter().any(|s| s.is_connected()) {
                    return Err::<(), _>("relay disconnected during idle");
                }
            }
        };
        assert!(
            timeout(Duration::from_secs(35), stable).await.is_err(),
            "idle relay must stay connected"
        );
        if std::env::var_os("LINN_RELAY_TEST_INSPECT").is_some() {
            let disconnected = async {
                loop {
                    let statuses = status.updated().await.map_err(|_| "relay watcher closed")?;
                    if !statuses.iter().any(|s| s.is_connected()) {
                        return Ok::<_, &str>(());
                    }
                }
            };
            let disconnect = async {
                reqwest::get(relay.join("/_inspect?disconnect")?.as_str())
                    .await?
                    .error_for_status()?;
                Ok::<_, Box<dyn Error + Send + Sync>>(())
            };
            let (lost, requested) = timeout(Duration::from_secs(10), async {
                tokio::join!(disconnected, disconnect)
            })
            .await?;
            lost?;
            requested?;
            timeout(Duration::from_secs(15), server.online()).await?;
            timeout(Duration::from_secs(15), client.online()).await?;
        }
        let receive = async {
            let connection = server
                .accept()
                .await
                .ok_or("no incoming connection")?
                .await?;
            if connection.remote_id() != client.id() {
                return Err("wrong client identity".into());
            }
            let (mut send, mut recv) = connection.accept_bi().await?;
            let payload = recv.read_to_end(PAYLOAD_BYTES).await?;
            if payload != vec![0x5a; PAYLOAD_BYTES] {
                return Err("payload mismatch".into());
            }
            send.write_all(&payload).await?;
            send.finish()?;
            // Keep the receiver alive until the client has received its echo.
            connection.closed().await;
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        };
        let transmit = async {
            let address = EndpointAddr::new(server.id()).with_relay_url(relay);
            let connection = client.connect(address, ALPN).await?;
            if connection.remote_id() != server.id() {
                return Err("wrong server identity".into());
            }
            let (mut send, mut recv) = connection.open_bi().await?;
            send.write_all(&vec![0x5a; PAYLOAD_BYTES]).await?;
            send.finish()?;
            let echo = recv.read_to_end(PAYLOAD_BYTES).await?;
            if echo != vec![0x5a; PAYLOAD_BYTES] {
                return Err("echo mismatch".into());
            }
            connection.close(0u32.into(), b"interop complete");
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        };
        tokio::try_join!(receive, transmit)?;
        Ok::<_, Box<dyn Error + Send + Sync>>(())
    })
    .await;
    for endpoint in idle {
        endpoint.close().await;
    }
    client.close().await;
    server.close().await;
    outcome??;
    eprintln!("relay-only interop passed: {peer_count} peers, idle heartbeats and 8 MiB echo");
    Ok(())
}

async fn endpoint(
    key: SecretKey,
    relay: RelayUrl,
) -> Result<Endpoint, Box<dyn Error + Send + Sync>> {
    Ok(Endpoint::builder(presets::Minimal)
        .secret_key(key)
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .clear_address_lookup()
        .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
            relay, None,
        )])))
        .net_report_config(NetReportConfig::minimal())
        .bind()
        .await?)
}

async fn enroll(service: &str, keys: &[&SecretKey]) -> TestResult {
    async fn commit(
        client: &ControlClient,
        service: &str,
        parent: Option<&Membership>,
        action: Action,
        signers: &[&SecretKey],
    ) -> TestResult<Membership> {
        let proposal = Proposal::sign(
            Event {
                version: 1,
                service: service.into(),
                id: random_id(),
                heads: parent.map(|s| vec![s.head.clone()]).unwrap_or_default(),
                action,
            },
            signers,
        )?;
        let accepted = client.commit(&proposal).await?;
        Ok(accepted.evaluate(&parent.into_iter().collect::<Vec<_>>())?)
    }
    let client = ControlClient::new(service)?;
    let receipt = client.service().await?.receipt_key;
    let owner = keys[0];
    let mut state = commit(
        &client,
        service,
        None,
        Action::Genesis {
            actor: key_id(owner),
            root: random_id(),
            receipt_key: receipt.clone(),
        },
        &[owner],
    )
    .await?;
    for joining in keys.iter().skip(1) {
        let delegated = SecretKey::generate();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        state = commit(
            &client,
            service,
            Some(&state),
            Action::Invite {
                actor: key_id(owner),
                key: key_id(&delegated),
                receipt_key: receipt.clone(),
                not_before: now,
                expires: now + 600,
            },
            &[owner],
        )
        .await?;
        let invitation = state.head.clone();
        state = commit(
            &client,
            service,
            Some(&state),
            Action::Redeem {
                actor: key_id(joining),
                invitation,
            },
            &[joining, &delegated],
        )
        .await?;
    }
    Ok(())
}
