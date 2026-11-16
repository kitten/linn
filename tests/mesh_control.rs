//! Stock native client against real workerd or an explicit staging deployment.
use iroh::{Endpoint, EndpointAddr, RelayMode, SecretKey, endpoint::presets};
use linn::{
    mesh::{
        client::{ControlClient, ControlError},
        membership::{Action, Event, Proposal, endpoint, random_id},
    },
    storage::sync::Replica,
};
use std::{
    error::Error,
    os::unix::fs::PermissionsExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
type TestResult = Result<(), Box<dyn Error + Send + Sync>>;
const ALPN: &[u8] = b"linn/control-interop/1";

fn proposal(
    service: &str,
    heads: Vec<String>,
    action: Action,
    keys: &[&SecretKey],
) -> linn::error::Result<Proposal> {
    let mut heads = heads;
    heads.sort();
    Proposal::sign(
        Event {
            version: 1,
            service: service.into(),
            id: random_id(),
            heads,
            action,
        },
        keys,
    )
}

#[tokio::test]
#[ignore = "requires an explicitly configured Worker"]
async fn signed_membership_and_private_native_relay() -> TestResult {
    let service = std::env::var("LINN_RELAY_TEST_URL")?
        .trim_end_matches('/')
        .to_owned();
    let client = ControlClient::new(&service)?;
    let info = client.service().await?;
    let root = tempfile::tempdir()?;
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700))?;
    let a = Replica::open_private_mesh(&root.path().join("a.db")).await?;
    let b = Replica::open_private_mesh(&root.path().join("b.db")).await?;
    let a_id = a.mesh_create_identity(&service).await?;
    let b_id = b.mesh_create_identity(&service).await?;
    for replica in [&a, &b] {
        replica.mesh_pin_receipt_key(&info.receipt_key).await?;
    }
    let first = proposal(
        &service,
        vec![],
        Action::Genesis {
            actor: endpoint(&a_id.key),
            root: random_id(),
            receipt_key: info.receipt_key.clone(),
        },
        &[&a_id.key],
    )?;
    a.mesh_prepare(&first).await?;
    let accepted = client.commit(&first).await?;
    let genesis = a.mesh_accept(&accepted).await?;
    let unchanged = client
        .history(&a_id.key, None, None, Some(&genesis.head))
        .await?;
    assert_eq!(unchanged.head, genesis.head);
    assert!(unchanged.events.is_empty() && !unchanged.more);
    a.mesh_enable(&genesis.head).await?;
    let delegated = SecretKey::generate();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let invite = proposal(
        &service,
        vec![genesis.head],
        Action::Invite {
            actor: endpoint(&a_id.key),
            key: endpoint(&delegated),
            receipt_key: info.receipt_key,
            not_before: now,
            expires: now + 600,
        },
        &[&a_id.key],
    )?;
    a.mesh_prepare(&invite).await?;
    let accepted_invite = client.commit(&invite).await?;
    let offered = a.mesh_accept(&accepted_invite).await?;
    let mut cursor = None;
    let mut found_anchor = false;
    loop {
        let page = client
            .history(&delegated, Some(&offered.head), cursor.as_deref(), None)
            .await?;
        for accepted in &page.events {
            let state = b.mesh_accept(accepted).await?;
            found_anchor |= state.head == offered.head;
        }
        if found_anchor {
            break;
        }
        if !page.more {
            return Err("invitation anchor missing from history".into());
        }
        cursor = page.after;
    }
    let join = proposal(
        &service,
        vec![offered.head.clone()],
        Action::Redeem {
            actor: endpoint(&b_id.key),
            invitation: offered.head,
        },
        &[&b_id.key, &delegated],
    )?;
    b.mesh_prepare(&join).await?;
    let accepted_join = client.commit(&join).await?;
    let joined = b.mesh_accept(&accepted_join).await?;
    b.mesh_enable(&joined.head).await?;
    a.mesh_accept(&accepted_join).await?;
    assert_eq!(client.commit(&join).await?, accepted_join);
    assert_eq!(joined.members.len(), 2);
    assert!(a.mesh_pending().await?.is_none());
    assert!(b.mesh_pending().await?.is_none());

    let relay = service.parse()?;
    let server = Endpoint::builder(presets::Minimal)
        .secret_key(a_id.key.clone())
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .clear_address_lookup()
        .relay_mode(RelayMode::custom([relay]))
        .bind()
        .await?;
    let relay = service.parse()?;
    let peer = Endpoint::builder(presets::Minimal)
        .secret_key(b_id.key.clone())
        .alpns(vec![ALPN.to_vec()])
        .clear_ip_transports()
        .clear_address_lookup()
        .relay_mode(RelayMode::custom([relay]))
        .bind()
        .await?;
    let exchange = tokio::time::timeout(Duration::from_secs(30), async {
        tokio::join!(server.online(), peer.online());
        let receive = async {
            let connection = server.accept().await.ok_or("no connection")?.await?;
            assert_eq!(connection.remote_id(), peer.id());
            let (mut send, mut recv) = connection.accept_bi().await?;
            let bytes = recv.read_to_end(65536).await?;
            assert_eq!(bytes, vec![0x51; 65536]);
            send.write_all(&bytes).await?;
            send.finish()?;
            connection.closed().await;
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        };
        let send = async {
            let connection = peer
                .connect(
                    EndpointAddr::new(server.id()).with_relay_url(service.parse()?),
                    ALPN,
                )
                .await?;
            let (mut send, mut recv) = connection.open_bi().await?;
            send.write_all(&vec![0x51; 65536]).await?;
            send.finish()?;
            assert_eq!(recv.read_to_end(65536).await?, vec![0x51; 65536]);
            connection.close(0u32.into(), b"control interop complete");
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        };
        tokio::try_join!(receive, send)?;
        Ok::<_, Box<dyn Error + Send + Sync>>(())
    })
    .await;
    peer.close().await;
    server.close().await;
    exchange??;

    let revoke_a = proposal(
        &service,
        vec![joined.head.clone()],
        Action::Revoke {
            actor: endpoint(&b_id.key),
            subject: endpoint(&a_id.key),
        },
        &[&b_id.key],
    )?;
    let revoke_b = proposal(
        &service,
        vec![joined.head],
        Action::Revoke {
            actor: endpoint(&a_id.key),
            subject: endpoint(&b_id.key),
        },
        &[&a_id.key],
    )?;
    let (left, right) = tokio::join!(client.commit(&revoke_a), client.commit(&revoke_b));
    let (accepted, winner, loser) = match (left, right) {
        (Ok(accepted), Err(ControlError::StaleHead)) => (accepted, &b_id.key, &a_id.key),
        (Err(ControlError::StaleHead), Ok(accepted)) => (accepted, &a_id.key, &b_id.key),
        _ => return Err("concurrent revocations did not yield exactly one winner".into()),
    };
    let state = a.mesh_accept(&accepted).await?;
    b.mesh_accept(&accepted).await?;
    assert_eq!(state.members.len(), 1);
    assert!(matches!(
        client.history(loser, None, None, None).await,
        Err(ControlError::Unauthorized)
    ));
    let leave = proposal(
        &service,
        vec![state.head],
        Action::Revoke {
            actor: endpoint(winner),
            subject: endpoint(winner),
        },
        &[winner],
    )?;
    let final_event = client.commit(&leave).await?;
    a.mesh_accept(&final_event).await?;
    b.mesh_accept(&final_event).await?;
    eprintln!(
        "native signed admission, verified history, private Iroh relay, retry and ordered revocation passed"
    );
    Ok(())
}
