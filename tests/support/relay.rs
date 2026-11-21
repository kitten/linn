//! Shared signed enrollment for native relay fixtures.
use crate::mesh::{
    client::ControlClient,
    membership::{Action, Event, Membership, Proposal, endpoint as key_id, random_id},
};
use iroh::SecretKey;
use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};
type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) async fn enroll(service: &str, keys: &[&SecretKey]) -> TestResult {
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
