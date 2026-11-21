//! Native, patched-Iroh interoperability gate. Never contacts a public default relay.
use iroh::{
    Endpoint, EndpointAddr, NetReportConfig, RelayConfig, RelayMap, RelayMode, RelayUrl, SecretKey,
    Watcher, endpoint::presets,
};
use linn::mesh;
use std::{error::Error, time::Duration};
#[path = "support/relay.rs"]
mod relay_fixture;
use relay_fixture::enroll;
use tokio::time::timeout;

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;
const ALPN: &[u8] = b"linn/relay-interop/1";
const PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

/// Initial-route restrictions must not mutate shared path selection for another
/// connection, including when that other connection has already warmed it.
#[tokio::test]
#[ignore = "requires an explicitly configured Worker and loopback UDP"]
async fn native_correlated_initial_routes_preserve_other_connections() -> TestResult {
    use iroh::endpoint::{ConnectOptions, InitialPath};
    async fn udp(key: SecretKey, relay: Option<RelayUrl>) -> TestResult<Endpoint> {
        Ok(Endpoint::builder(presets::Minimal)
            .secret_key(key)
            .alpns(vec![ALPN.to_vec()])
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")?
            .clear_address_lookup()
            .relay_mode(match relay {
                Some(url) => RelayMode::custom([url]),
                None => RelayMode::Disabled,
            })
            .net_report_config(NetReportConfig::minimal())
            .bind()
            .await?)
    }
    let relay: RelayUrl = std::env::var("LINN_RELAY_TEST_URL")?.parse()?;
    for policy in [InitialPath::DirectOnly, InitialPath::RelayOnly] {
        let source_key = SecretKey::generate();
        let target_key = SecretKey::generate();
        enroll(
            relay.as_str().trim_end_matches('/'),
            &[&source_key, &target_key],
        )
        .await?;
        let (source, target, address) = if policy == InitialPath::DirectOnly {
            let source = endpoint(source_key, relay.clone()).await?;
            let target = endpoint(target_key, relay.clone()).await?;
            timeout(Duration::from_secs(10), async {
                tokio::join!(source.online(), target.online());
            })
            .await?;
            let address = EndpointAddr::new(target.id()).with_relay_url(relay.clone());
            (source, target, address)
        } else {
            let source = udp(source_key, Some(relay.clone())).await?;
            let target = udp(target_key, None).await?;
            let address = EndpointAddr::new(target.id()).with_ip_addr(target.bound_sockets()[0]);
            (source, target, address)
        };
        let (outgoing, incoming) = timeout(Duration::from_secs(3), async {
            tokio::try_join!(
                async {
                    Ok::<_, Box<dyn Error + Send + Sync>>(source.connect(address, ALPN).await?)
                },
                async {
                    Ok::<_, Box<dyn Error + Send + Sync>>(
                        target
                            .accept()
                            .await
                            .ok_or("no initial connection")?
                            .await?,
                    )
                },
            )
        })
        .await??;
        let destination = EndpointAddr::new(target.id()).with_relay_url(relay.clone());
        let destination = if policy == InitialPath::DirectOnly {
            // This source has no IP transport. Following its retained selected
            // relay path would reach the target and fail the assertion below.
            destination.with_ip_addr("127.0.0.1:9".parse::<std::net::SocketAddr>()?)
        } else {
            destination.with_ip_addr(target.bound_sockets()[0])
        };
        let probing = async {
            let attempt = async {
                source
                    .connect_with_opts(
                        destination,
                        ALPN,
                        ConnectOptions::new().with_initial_path(policy),
                    )
                    .await?
                    .await
                    .map_err(Box::<dyn Error + Send + Sync>::from)
            };
            tokio::select! {
                incoming = target.accept() => panic!("restricted attempt used the other connection's cached route: {incoming:?}"),
                result = timeout(Duration::from_millis(300), attempt) => assert!(result.is_err(), "restricted route unexpectedly completed: {result:?}"),
            }
        };
        let exchange = async {
            let (mut send, mut recv) = outgoing.open_bi().await?;
            send.write_all(b"existing connection remains healthy")
                .await?;
            send.finish()?;
            let (mut response, mut request) = incoming.accept_bi().await?;
            let bytes = request.read_to_end(100).await?;
            assert_eq!(bytes, b"existing connection remains healthy");
            response.write_all(&bytes).await?;
            response.finish()?;
            assert_eq!(recv.read_to_end(100).await?, bytes);
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        };
        let (_, result) = timeout(Duration::from_secs(3), async {
            tokio::join!(probing, exchange)
        })
        .await?;
        result?;
        outgoing.close(0u32.into(), b"route policy check");
        incoming.close(0u32.into(), b"route policy check");
        source.close().await;
        target.close().await;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires an explicitly configured Worker and loopback UDP"]
async fn native_correlated_absence_does_not_preclude_direct_success() -> TestResult {
    let relay: RelayUrl = std::env::var("LINN_RELAY_TEST_URL")?.parse()?;
    let source_key = SecretKey::generate();
    let target_key = SecretKey::generate();
    enroll(
        relay.as_str().trim_end_matches('/'),
        &[&source_key, &target_key],
    )
    .await?;
    let source = Endpoint::builder(presets::Minimal)
        .secret_key(source_key)
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")?
        .clear_address_lookup()
        .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
            relay.clone(),
            None,
        )])))
        .net_report_config(NetReportConfig::minimal())
        .bind()
        .await?;
    let target = Endpoint::builder(presets::Minimal)
        .secret_key(target_key)
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")?
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .bind()
        .await?;
    timeout(Duration::from_secs(10), source.online()).await?;
    let mut observation = source
        .observe_relay_dial(relay.clone(), target.id())
        .unwrap();
    let address = EndpointAddr::new(target.id())
        .with_relay_url(relay)
        .with_ip_addr(target.bound_sockets()[0]);
    timeout(Duration::from_secs(3), async {
        let (outgoing, incoming) = tokio::try_join!(
            async { Ok::<_, Box<dyn Error + Send + Sync>>(source.connect(address, ALPN).await?) },
            async {
                Ok::<_, Box<dyn Error + Send + Sync>>(
                    target.accept().await.ok_or("no direct dial")?.await?,
                )
            },
        )?;
        outgoing.close(0u32.into(), b"direct success");
        incoming.close(0u32.into(), b"direct success");
        Ok::<_, Box<dyn Error + Send + Sync>>(())
    })
    .await??;
    // A fast direct path need not send relay traffic at all. Any relay reply
    // must still describe absence, never override the successful direct dial.
    if let Ok(outcome) = timeout(Duration::from_millis(100), observation.outcome()).await {
        assert_eq!(
            outcome,
            Some(iroh::endpoint::RelayDialOutcome::NotRegistered)
        );
    }
    source.close().await;
    target.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires an explicitly configured Worker"]
async fn native_correlated_dial_observations() -> TestResult {
    let relay: RelayUrl = std::env::var("LINN_RELAY_TEST_URL")?.parse()?;
    let source_key = SecretKey::generate();
    let target_key = SecretKey::generate();
    enroll(
        relay.as_str().trim_end_matches('/'),
        &[&source_key, &target_key],
    )
    .await?;
    let source = endpoint(source_key, relay.clone()).await?;
    timeout(Duration::from_secs(10), source.online()).await?;
    let delayed_gone = std::env::var_os("LINN_RELAY_TEST_DELAYED_GONE").is_some();
    if delayed_gone {
        reqwest::get(format!(
            "{}/_inspect?delayGone",
            relay.as_str().trim_end_matches('/')
        ))
        .await?
        .error_for_status()?;
    }
    let target_id = target_key.public();
    let mut observation = source.observe_relay_dial(relay.clone(), target_id).unwrap();
    let dial = source.connect(
        EndpointAddr::new(target_id).with_relay_url(relay.clone()),
        ALPN,
    );
    tokio::pin!(dial);
    let started = std::time::Instant::now();
    let old_relay = std::env::var_os("LINN_RELAY_TEST_OLD_RELAY").is_some();
    if old_relay {
        tokio::select! {
            result = observation.outcome() => panic!("old relay fabricated an observation: {result:?}"),
            result = &mut dial => panic!("absent dial ended before ordinary deadline: {result:?}"),
            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
        }
        println!(
            "measured {} fallback: no observation in {:?}",
            if std::env::var_os("LINN_RELAY_TEST_NO_REPLY").is_some() {
                "accepted-but-silent relay"
            } else {
                "old relay"
            },
            started.elapsed()
        );
    } else {
        let result = tokio::select! {
            result = observation.outcome() => result,
            result = &mut dial => panic!("absent dial ended without observation: {result:?}"),
            _ = tokio::time::sleep(Duration::from_secs(3)) => panic!("missing correlated reply"),
        };
        assert_eq!(
            result,
            Some(iroh::endpoint::RelayDialOutcome::NotRegistered)
        );
        println!(
            "measured cold registered-source absent-target observation: {:?}",
            started.elapsed()
        );
    }
    drop(observation);
    let target = endpoint(target_key, relay.clone()).await?;
    timeout(Duration::from_secs(10), target.online()).await?;
    let mut observation = source.observe_relay_dial(relay, target_id).unwrap();
    if delayed_gone {
        let state: serde_json::Value = reqwest::get(format!(
            "{}/_inspect",
            std::env::var("LINN_RELAY_TEST_URL")?.trim_end_matches('/')
        ))
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert!(
            state["delayedGone"].as_u64().unwrap() > 0,
            "the stale notice must actually be queued"
        );
        reqwest::get(format!(
            "{}/_inspect?releaseGone",
            std::env::var("LINN_RELAY_TEST_URL")?.trim_end_matches('/')
        ))
        .await?
        .error_for_status()?;
    }
    let exchange = async {
        let (outgoing, incoming) = tokio::try_join!(
            async { Ok::<_, Box<dyn Error + Send + Sync>>(dial.await?) },
            async {
                Ok::<_, Box<dyn Error + Send + Sync>>(
                    target.accept().await.ok_or("no incoming dial")?.await?,
                )
            },
        )?;
        outgoing.close(0u32.into(), b"observation check");
        incoming.close(0u32.into(), b"observation check");
        Ok::<_, Box<dyn Error + Send + Sync>>(())
    };
    let observed = async {
        if old_relay {
            assert!(
                timeout(Duration::from_millis(500), observation.outcome())
                    .await
                    .is_err()
            );
            None
        } else {
            observation.outcome().await
        }
    };
    let (result, observed) = timeout(Duration::from_secs(10), async {
        tokio::join!(exchange, observed)
    })
    .await?;
    result?;
    assert_eq!(
        observed,
        (!old_relay).then_some(iroh::endpoint::RelayDialOutcome::Registered)
    );
    target.close().await;
    source.close().await;
    Ok(())
}

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
    if std::env::var_os("LINN_RELAY_TEST_BACKOFF").is_some() {
        return relay_demand_backoff().await;
    }
    if std::env::var_os("LINN_RELAY_TEST_HEARTBEATS").is_some() {
        return relay_heartbeat_tolerance().await;
    }
    for peer_count in [2, 10] {
        relay_roundtrip(peer_count).await?;
    }
    Ok(())
}

async fn relay_demand_backoff() -> TestResult {
    async fn marker(directory: &std::path::Path, name: &str) -> TestResult {
        timeout(Duration::from_secs(65), async {
            while !directory.join(name).exists() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await?;
        Ok(())
    }
    async fn exchange(from: &Endpoint, address: EndpointAddr, to: &Endpoint) -> TestResult {
        let (outgoing, incoming) = timeout(Duration::from_secs(3), async {
            tokio::try_join!(
                async { Ok::<_, Box<dyn Error + Send + Sync>>(from.connect(address, ALPN).await?) },
                async {
                    Ok::<_, Box<dyn Error + Send + Sync>>(
                        to.accept().await.ok_or("no incoming connection")?.await?,
                    )
                },
            )
        })
        .await??;
        outgoing.close(0u32.into(), b"backoff check");
        incoming.close(0u32.into(), b"backoff check");
        Ok(())
    }
    let service = std::env::var("LINN_RELAY_TEST_URL")?;
    let backend: RelayUrl = std::env::var("LINN_RELAY_BACKOFF_BACKEND")?.parse()?;
    let directory = std::path::PathBuf::from(std::env::var("LINN_RELAY_BACKOFF_BARRIER")?);
    let target_key = SecretKey::generate();
    let keys: Vec<_> = (0..3).map(|_| SecretKey::generate()).collect();
    let enrollment: Vec<_> = std::iter::once(&target_key).chain(keys.iter()).collect();
    enroll(service.trim_end_matches('/'), &enrollment).await?;
    let target = endpoint(target_key, backend).await?;
    timeout(Duration::from_secs(10), target.online()).await?;
    let local = Endpoint::builder(presets::Minimal)
        .alpns(vec![ALPN.to_vec()])
        .clear_address_lookup()
        .relay_mode(RelayMode::Disabled)
        .bind()
        .await?;
    let port = local
        .bound_sockets()
        .iter()
        .find(|a| a.is_ipv4())
        .ok_or("no IPv4 socket")?
        .port();
    for (mode, key) in ["traffic", "network", "cancel"].into_iter().zip(keys) {
        let relay: RelayUrl =
            std::env::var(format!("LINN_RELAY_BACKOFF_{}", mode.to_uppercase()))?.parse()?;
        let client = Endpoint::builder(presets::Minimal)
            .secret_key(key)
            .alpns(vec![ALPN.to_vec()])
            .clear_address_lookup()
            .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
                relay.clone(),
                None,
            )])))
            .net_report_config(NetReportConfig::minimal())
            .bind()
            .await?;
        let outcome = async {
            marker(&directory, &format!("{mode}-ready")).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
            assert!(
                !client
                    .home_relay_status()
                    .get()
                    .iter()
                    .any(|s| s.is_connected())
            );
            if mode == "cancel" {
                return Ok::<_, Box<dyn Error + Send + Sync>>(());
            }
            // Direct local transport must stay usable during a long relay backoff.
            exchange(
                &client,
                EndpointAddr::new(local.id())
                    .with_ip_addr((std::net::Ipv4Addr::LOCALHOST, port).into()),
                &local,
            )
            .await?;
            assert!(
                !client
                    .home_relay_status()
                    .get()
                    .iter()
                    .any(|s| s.is_connected())
            );
            if mode == "traffic" {
                exchange(
                    &client,
                    EndpointAddr::new(target.id()).with_relay_url(relay),
                    &target,
                )
                .await?;
            } else {
                client.network_change().await;
                timeout(Duration::from_secs(3), client.online()).await?;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(())
        }
        .await;
        timeout(Duration::from_secs(1), client.close()).await?;
        outcome?;
    }
    target.close().await;
    local.close().await;
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

#[tokio::test]
#[ignore = "requires an explicitly configured local Worker relay"]
async fn native_registration_notification_old_relay_fallback_is_unknown() -> TestResult {
    let service = std::env::var("LINN_RELAY_TEST_URL")?
        .trim_end_matches('/')
        .to_owned();
    let keys = [SecretKey::generate(), SecretKey::generate()];
    enroll(&service, &[&keys[0], &keys[1]]).await?;
    let relay: RelayUrl = service.parse()?;
    let source = Endpoint::builder(presets::Minimal)
        .secret_key(keys[0].clone())
        .clear_ip_transports()
        .clear_address_lookup()
        .alpns(vec![ALPN.to_vec()])
        .relay_mode(RelayMode::Custom(RelayMap::from_iter([RelayConfig::new(
            relay.clone(),
            None,
        )])))
        .net_report_config(NetReportConfig::minimal())
        .bind()
        .await?;
    timeout(Duration::from_secs(10), source.online()).await?;
    let mut watch = source
        .watch_target_registration(relay, keys[1].public())
        .unwrap();
    assert_eq!(
        timeout(Duration::from_secs(2), watch.event()).await?,
        iroh::endpoint::TargetWatchEvent::Unknown
    );
    source.close().await;
    Ok(())
}
