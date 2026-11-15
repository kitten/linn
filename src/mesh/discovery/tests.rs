use super::*;
use std::time::Duration;

#[test]
fn dns_records_require_membership_profile_and_valid_interface_scope() {
    let local = iroh::SecretKey::generate().public();
    let approved = iroh::SecretKey::generate().public();
    let unknown = iroh::SecretKey::generate().public();
    let make = |id: PublicKey, ip: &str, port, version: &str| {
        ServiceInfo::new(
            SERVICE,
            &id.to_z32(),
            &format!("{}.local.", id.to_z32()),
            ip,
            port,
            [("v", version)].as_slice(),
        )
        .unwrap()
        .as_resolved_service()
    };
    let mut state = State {
        lookup: MemoryLookup::new(),
        local,
        allowed: BTreeSet::from([approved]),
        seen: BTreeSet::new(),
        error: None,
    };
    state.resolved(&make(approved, "192.0.2.1", 12345, "1"));
    assert!(state.lookup.get_endpoint_info(approved).is_some());
    for info in [
        make(unknown, "192.0.2.2", 12345, "1"),
        make(local, "192.0.2.3", 12345, "1"),
        make(approved, "192.0.2.4", 12345, "2"),
    ] {
        state.resolved(&info);
    }
    assert_eq!(state.seen, BTreeSet::from([approved]));
    assert!(state.lookup.get_endpoint_info(unknown).is_none());
    state.event(ServiceEvent::ServiceRemoved(
        SERVICE.into(),
        format!("{}.{SERVICE}", approved.to_z32()),
    ));
    assert!(state.lookup.get_endpoint_info(approved).is_none());
    // A loopback record without local receive-interface metadata is unsafe.
    state.resolved(&make(approved, "127.0.0.1", 12345, "1"));
    assert!(state.lookup.get_endpoint_info(approved).is_none());
    state.allowed.clear();
    state.resolved(&make(approved, "192.0.2.1", 12345, "1"));
    assert!(state.seen.is_empty());
}
#[tokio::test]
#[ignore = "requires native loopback multicast"]
async fn loopback_rediscovery_survives_both_endpoints_restarting_with_new_ports() {
    use iroh::{RelayMode, SecretKey, endpoint::presets};
    let a_key = SecretKey::generate();
    let b_key = SecretKey::generate();
    let state = Membership {
        head: "0".repeat(64),
        service: "https://unreachable.invalid".into(),
        receipt_key: "0".repeat(64),
        roots: BTreeSet::new(),
        members: [&a_key, &b_key]
            .into_iter()
            .map(super::super::membership::endpoint)
            .collect(),
        revoked: BTreeSet::new(),
        invitations: Default::default(),
    };
    async fn round(
        a_key: &SecretKey,
        b_key: &SecretKey,
        state: &Membership,
    ) -> (SocketAddr, SocketAddr) {
        let a_lookup = MemoryLookup::new();
        let b_lookup = MemoryLookup::new();
        let a = Endpoint::builder(presets::Minimal)
            .secret_key(a_key.clone())
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .clear_address_lookup()
            .address_lookup(a_lookup.clone())
            .relay_mode(RelayMode::Disabled)
            .alpns(vec![b"linn/discovery-test/1".to_vec()])
            .bind()
            .await
            .unwrap();
        let b = Endpoint::builder(presets::Minimal)
            .secret_key(b_key.clone())
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .clear_address_lookup()
            .address_lookup(b_lookup.clone())
            .relay_mode(RelayMode::Disabled)
            .alpns(vec![b"linn/discovery-test/1".to_vec()])
            .bind()
            .await
            .unwrap();
        let a_local = LocalLookup::start(&a, a_lookup, state).unwrap();
        let b_local = LocalLookup::start(&b, b_lookup, state).unwrap();
        tokio::time::timeout(Duration::from_secs(15), async {
            while a_local.addresses(b.id()).is_empty() || b_local.addresses(a.id()).is_empty() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            let receive = async {
                let conn = b.accept().await.unwrap().await.unwrap();
                assert_eq!(conn.remote_id(), a.id());
                let (mut send, mut recv) = conn.accept_bi().await.unwrap();
                let bytes = recv.read_to_end(1024).await.unwrap();
                assert_eq!(bytes, b"offline loopback");
                send.write_all(&bytes).await.unwrap();
                send.finish().unwrap();
                conn.closed().await;
            };
            let transmit = async {
                let conn = a
                    .connect(
                        EndpointAddr::new(b.id()).with_addrs(
                            a_local
                                .addresses(b.id())
                                .into_iter()
                                .map(iroh::TransportAddr::Ip),
                        ),
                        b"linn/discovery-test/1",
                    )
                    .await
                    .unwrap();
                let (mut send, mut recv) = conn.open_bi().await.unwrap();
                send.write_all(b"offline loopback").await.unwrap();
                send.finish().unwrap();
                assert_eq!(recv.read_to_end(1024).await.unwrap(), b"offline loopback");
                conn.close(0u32.into(), b"done");
            };
            tokio::join!(receive, transmit);
        })
        .await
        .unwrap();
        let ports = (a.bound_sockets()[0], b.bound_sockets()[0]);
        drop(a_local);
        drop(b_local);
        a.close().await;
        b.close().await;
        ports
    }
    let (old_a, old_b) = round(&a_key, &b_key, &state).await;
    let _hold_a = std::net::UdpSocket::bind(old_a).unwrap();
    let _hold_b = std::net::UdpSocket::bind(old_b).unwrap();
    let (new_a, new_b) = round(&a_key, &b_key, &state).await;
    assert_ne!(old_a, new_a);
    assert_ne!(old_b, new_b);
}
