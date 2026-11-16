//! Explicit enrollment and membership operations; never transfers original records.
use super::{
    client::{ControlClient, ControlError},
    membership::{
        Acceptance, Action, Event, Membership, Proposal, canonical, endpoint, hex, random_id, unhex,
    },
    store::{self, ControlStore, Step, Value},
};
use crate::{
    error::{Error, Result},
    storage::mesh::Identity,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use iroh::SecretKey;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Command {
    Sync,
    Pair {
        service: Option<String>,
        invitation: Option<String>,
        cancel: Option<String>,
        #[serde(default)]
        initial_sync: bool,
    },
    Status,
    Revoke {
        peer: String,
    },
    Leave,
    Reset,
}
impl Command {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Pair {
                service,
                invitation,
                cancel,
                initial_sync,
            } => {
                if let Some(service) = service {
                    crate::storage::mesh::service_origin(service)?;
                }
                if invitation.as_ref().is_some_and(|s| s.len() > 32 * 1024) {
                    return Err(Error::Invalid("invitation token is too large".into()));
                }
                if invitation.is_some() && cancel.is_some() {
                    return Err(Error::Invalid(
                        "redemption cannot be combined with cancellation".into(),
                    ));
                }
                if cancel.is_some() && service.is_some() {
                    return Err(Error::Invalid(
                        "cancellation cannot be combined with service".into(),
                    ));
                }
                if *initial_sync && invitation.is_none() {
                    return Err(Error::Invalid(
                        "first sync requires joining an invitation".into(),
                    ));
                }
                if let Some(id) = cancel {
                    unhex::<32>(id)?;
                }
            }
            Self::Revoke { peer } => {
                unhex::<32>(peer)?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn mutates(&self) -> bool {
        !matches!(self, Self::Status)
    }
    pub fn accepts(&self, output: &Output) -> bool {
        match (self, output) {
            (Self::Sync, Output::Synced(_)) => true,
            (Self::Status, Output::Status(_)) => true,
            (
                Self::Pair {
                    invitation: None,
                    cancel: None,
                    ..
                },
                Output::Invitation { .. },
            ) => true,
            (
                Self::Pair {
                    invitation: Some(_),
                    initial_sync,
                    ..
                },
                Output::Changed {
                    operation: Change::Paired,
                    initial_sync: report,
                    initial_sync_error: error,
                    ..
                },
            ) => {
                if *initial_sync {
                    report.is_some() != error.is_some()
                } else {
                    report.is_none() && error.is_none()
                }
            }
            (
                Self::Pair {
                    cancel: Some(_), ..
                },
                Output::Changed {
                    operation,
                    initial_sync: None,
                    initial_sync_error: None,
                    ..
                },
            ) => *operation == Change::Cancelled,
            (
                Self::Revoke { .. },
                Output::Changed {
                    operation,
                    initial_sync: None,
                    initial_sync_error: None,
                    ..
                },
            ) => *operation == Change::Revoked,
            (
                Self::Leave,
                Output::Changed {
                    operation,
                    initial_sync: None,
                    initial_sync_error: None,
                    ..
                },
            ) => *operation == Change::Left,
            (
                Self::Reset,
                Output::Changed {
                    operation,
                    initial_sync: None,
                    initial_sync_error: None,
                    ..
                },
            ) => *operation == Change::IdentityReset,
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    #[serde(default)]
    pub transport: super::status::Transport,
    pub endpoint: Option<String>,
    pub service: Option<String>,
    pub head: Option<String>,
    pub enabled: bool,
    pub pending_leave: bool,
    pub conflict: bool,
    pub members: Vec<String>,
    pub revoked: Vec<String>,
}
impl Status {
    pub(crate) fn is_admitted(&self) -> bool {
        self.enabled
            && !self.pending_leave
            && !self.conflict
            && self
                .endpoint
                .as_ref()
                .is_some_and(|key| unhex::<32>(key).is_ok() && self.members.contains(key))
            && self
                .head
                .as_ref()
                .is_some_and(|head| unhex::<32>(head).is_ok())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Output {
    Synced(super::jobs::Report),
    Invitation {
        token: String,
        invitation: String,
        expires: u64,
        members: Vec<String>,
    },
    Status(Status),
    Changed {
        operation: Change,
        status: Status,
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_sync: Option<Box<super::jobs::Report>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_sync_error: Option<Box<crate::error::ApplicationError>>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Change {
    Paired,
    Revoked,
    Left,
    Cancelled,
    IdentityReset,
}
impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Paired => "paired",
            Self::Revoked => "revoked",
            Self::Left => "left",
            Self::Cancelled => "cancelled",
            Self::IdentityReset => "identity-reset",
        })
    }
}
impl Output {
    fn changed(operation: Change, status: Status) -> Self {
        Self::Changed {
            operation,
            status,
            initial_sync: None,
            initial_sync_error: None,
        }
    }
    pub fn complete(&self) -> bool {
        match self {
            Self::Synced(report) => report.complete,
            Self::Changed {
                initial_sync,
                initial_sync_error,
                ..
            } => initial_sync_error.is_none() && initial_sync.as_ref().is_none_or(|r| r.complete),
            _ => true,
        }
    }
}
// Secret-bearing invitation tokens deliberately have no Debug implementation.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Token {
    version: u8,
    invitation: Acceptance,
    secret: String,
}

fn now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::Invalid("invalid local clock".into()))?
        .as_secs())
}
fn proposal(
    identity: &Identity,
    heads: Vec<String>,
    action: Action,
    additional: Option<&SecretKey>,
) -> Result<Proposal> {
    let mut heads = heads;
    heads.sort();
    let event = Event {
        version: 1,
        service: identity.service.clone(),
        id: random_id(),
        heads,
        action,
    };
    let mut keys = vec![&identity.key];
    if let Some(additional) = additional {
        keys.push(additional);
    }
    Proposal::sign(event, &keys)
}

async fn accept(store: &impl ControlStore, accepted: Acceptance) -> Result<String> {
    if let Value::Accepted(state) = store.step(Step::Accept(accepted)).await? {
        Ok(state)
    } else {
        Err(Error::Invalid("unexpected mesh acceptance reply".into()))
    }
}

pub async fn status(store: &impl ControlStore) -> Result<Status> {
    let identity = store::identity(store).await?;
    let state = if identity.is_some() {
        store::membership(store, None).await?
    } else {
        None
    };
    Ok(Status {
        transport: Default::default(),
        endpoint: identity.as_ref().map(|id| endpoint(&id.key)),
        service: identity.as_ref().map(|id| id.service.clone()),
        head: identity.as_ref().and_then(|id| id.head.clone()),
        enabled: identity.as_ref().is_some_and(|id| id.enabled),
        pending_leave: identity.as_ref().is_some_and(|id| id.pending_leave),
        conflict: identity.as_ref().is_some_and(|id| id.conflict),
        members: state
            .as_ref()
            .map(|s| s.members.iter().cloned().collect())
            .unwrap_or_default(),
        revoked: state
            .as_ref()
            .map(|s| s.revoked.iter().cloned().collect())
            .unwrap_or_default(),
    })
}

/// Capture a finite remote head. Never chase later changes while downloading.
pub async fn refresh(
    store: &impl ControlStore,
    client: &ControlClient,
    reader: &SecretKey,
    invitation: Option<&str>,
) -> Result<Membership> {
    tokio::time::timeout(
        std::time::Duration::from_secs(300),
        refresh_inner(store, client, reader, invitation),
    )
    .await
    .map_err(|_| ControlError::Unavailable)?
}
async fn refresh_inner(
    store: &impl ControlStore,
    client: &ControlClient,
    reader: &SecretKey,
    invitation: Option<&str>,
) -> Result<Membership> {
    require_identity(store).await?;
    let known = store::membership(store, None).await?;
    let mut staging = None;
    let mut cursor = None;
    let mut target = None;
    loop {
        let page = client
            .history(
                reader,
                invitation,
                cursor.as_deref(),
                known.as_ref().map(|state| state.head.as_str()),
            )
            .await?;
        if target.is_none() {
            if let Some(known) = &known
                && known.head == page.head
            {
                return Ok(known.clone());
            }
            if invitation.is_none() {
                staging = Some(super::proof::Staging::new(store).await?);
            }
            target = Some(page.head);
        }
        let target = target.as_ref().unwrap();
        for event in page.events {
            let head = event.proposal.hash()?;
            if let Some(staging) = staging.as_mut() {
                staging.accept(store, event).await?;
            } else {
                accept(store, event).await?;
            }
            if &head == target {
                if let Some(staging) = staging {
                    staging.finish()?;
                }
                return store::membership(store, Some(head))
                    .await?
                    .ok_or_else(|| Error::Invalid("missing verified membership head".into()));
            }
        }
        if !page.more {
            if let Some(staging) = staging {
                staging.finish()?;
            }
            return store::membership(store, Some(target.clone()))
                .await?
                .ok_or_else(|| {
                    Error::Invalid("mesh history is incomplete; no records were exchanged".into())
                });
        }
        cursor = page.after;
    }
}

async fn committed(
    store: &impl ControlStore,
    client: &ControlClient,
    proposal: Proposal,
) -> Result<Acceptance> {
    store.step(Step::Prepare(proposal.clone())).await?;
    match client.commit(&proposal).await {
        Ok(accepted) => {
            accept(store, accepted.clone()).await?;
            Ok(accepted)
        }
        Err(
            error @ (ControlError::StaleHead
            | ControlError::Unauthorized
            | ControlError::Rejected(_)),
        ) => {
            store.step(Step::Reject(proposal)).await?;
            Err(error.into())
        }
        Err(error) => Err(error.into()),
    }
}

async fn resume(store: &impl ControlStore, client: &ControlClient) -> Result<Option<Acceptance>> {
    let Value::Pending(pending) = store.step(Step::Pending).await? else {
        return Err(Error::Invalid(
            "unexpected pending mesh proposal reply".into(),
        ));
    };
    if let Some(pending) = pending {
        match committed(store, client, pending).await {
            Ok(accepted) => return Ok(Some(accepted)),
            Err(Error::MeshControl(ControlError::StaleHead)) => (),
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}

async fn create_identity(store: &impl ControlStore, service: &str) -> Result<Identity> {
    if let Value::Identity(Some(identity)) =
        store.step(Step::CreateIdentity(service.into())).await?
    {
        Ok(identity)
    } else {
        Err(Error::Invalid(
            "unexpected identity initialization reply".into(),
        ))
    }
}

async fn require_identity(store: &impl ControlStore) -> Result<Identity> {
    let identity = store::identity(store).await?.ok_or_else(|| {
        Error::Invalid(
            "this store is not paired; use linn sync:pair or redeem an invitation".into(),
        )
    })?;
    if identity.conflict {
        return Err(Error::Invalid(
            "mesh history conflicts; network authorization is suspended".into(),
        ));
    }
    Ok(identity)
}

async fn activate(store: &impl ControlStore) -> Result<()> {
    let identity = require_identity(store).await?;
    let head = identity
        .head
        .ok_or_else(|| Error::Invalid("enrollment has no verified membership head".into()))?;
    store.step(Step::Enable(head)).await?;
    Ok(())
}

// @ref LLP 0008#pairing-and-mesh-membership
fn invitation_service<'a>(requested: Option<&'a str>, enrolled: Option<&'a str>) -> &'a str {
    requested.or(enrolled).unwrap_or("https://linn.kitten.sh")
}

async fn create_invitation(store: &impl ControlStore, service: Option<String>) -> Result<Output> {
    let initial = store::identity(store).await?;
    let origin = invitation_service(
        service.as_deref(),
        initial.as_ref().map(|id| id.service.as_str()),
    );
    let identity = create_identity(store, origin).await?;
    if identity.pending_leave || identity.conflict {
        return Err(Error::Invalid("store has left or has conflicting membership; explicitly reset identity before enrolling again".into()));
    }
    let client = ControlClient::new(&identity.service)?;
    if identity.receipt_key.is_none() {
        store
            .step(Step::Pin(client.service().await?.receipt_key))
            .await?;
    }
    resume(store, &client).await?;
    let identity = require_identity(store).await?;
    if identity.head.is_none() {
        let genesis = proposal(
            &identity,
            vec![],
            Action::Genesis {
                actor: endpoint(&identity.key),
                root: random_id(),
                receipt_key: identity.receipt_key.clone().unwrap(),
            },
            None,
        )?;
        committed(store, &client, genesis).await?;
    }
    activate(store).await?;
    let delegated = SecretKey::generate();
    for _ in 0..3 {
        refresh(store, &client, &identity.key, None).await?;
        let current = require_identity(store).await?;
        let now = now()?;
        let event = proposal(
            &current,
            vec![
                current
                    .head
                    .clone()
                    .ok_or_else(|| Error::Invalid("missing membership head".into()))?,
            ],
            Action::Invite {
                actor: endpoint(&current.key),
                key: endpoint(&delegated),
                receipt_key: current.receipt_key.clone().unwrap(),
                not_before: now,
                expires: now + 600,
            },
            None,
        )?;
        match committed(store, &client, event).await {
            Ok(invitation) => {
                let id = invitation.proposal.hash()?;
                let token = URL_SAFE_NO_PAD.encode(canonical(&Token {
                    version: 1,
                    invitation,
                    secret: hex(&delegated.to_bytes()),
                })?);
                return Ok(Output::Invitation {
                    token,
                    invitation: id,
                    expires: now + 600,
                    members: status(store).await?.members,
                });
            }
            Err(Error::MeshControl(ControlError::StaleHead)) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(ControlError::StaleHead.into())
}

async fn redeem(
    store: &impl ControlStore,
    value: String,
    service_override: Option<String>,
) -> Result<Output> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value.trim())
        .map_err(|_| Error::Invalid("invalid mesh invitation".into()))?;
    let token: Token = serde_json::from_slice(&bytes)
        .map_err(|_| Error::Invalid("invalid mesh invitation".into()))?;
    if token.version != 1 || canonical(&token)? != bytes {
        return Err(Error::Invalid(
            "invalid invitation encoding or version".into(),
        ));
    }
    let Action::Invite {
        key,
        receipt_key,
        expires,
        ..
    } = &token.invitation.proposal.event.action
    else {
        return Err(Error::Invalid(
            "token does not contain an invitation".into(),
        ));
    };
    let delegated = SecretKey::from_bytes(&unhex(&token.secret)?);
    if endpoint(&delegated) != *key {
        return Err(Error::Invalid(
            "invitation key does not match its delegation".into(),
        ));
    }
    token.invitation.proposal.verify_signatures()?;
    token.invitation.verify_receipt(receipt_key)?;
    let origin = &token.invitation.proposal.event.service;
    if service_override.as_ref().is_some_and(|service| {
        crate::storage::mesh::service_origin(service).as_ref().ok() != Some(origin)
    }) {
        return Err(Error::Invalid("invitation names another service".into()));
    }
    let identity = create_identity(store, origin).await?;
    if identity.pending_leave || identity.conflict {
        return Err(Error::Invalid(
            "store has left or has conflicting membership; reset identity explicitly".into(),
        ));
    }
    store.step(Step::Pin(receipt_key.clone())).await?;
    let client = ControlClient::new(origin)?;
    let anchor = token.invitation.proposal.hash()?;
    if let Some(accepted) = resume(store, &client).await?
        && matches!(&accepted.proposal.event.action, Action::Redeem { invitation, .. } if invitation == &anchor)
    {
        activate(store).await?;
        return Ok(Output::changed(Change::Paired, status(store).await?));
    }
    if now()? >= *expires {
        return Err(Error::Invalid("invitation expired".into()));
    }
    for _ in 0..3 {
        let current = require_identity(store).await?;
        if current.head.is_some() {
            refresh(store, &client, &current.key, None).await?;
        }
        let offered = refresh(store, &client, &delegated, Some(&anchor)).await?;
        // The out-of-band token must be the same verified delegation in this proof.
        let pinned = store::membership(store, Some(anchor.clone()))
            .await?
            .ok_or_else(|| Error::Invalid("invitation trust anchor missing".into()))?;
        if pinned.service != offered.service
            || pinned.receipt_key != offered.receipt_key
            || !offered.invitations.contains_key(&anchor)
        {
            return Err(Error::Invalid(
                "invitation proof does not match its trust anchor".into(),
            ));
        }
        let current = require_identity(store).await?;
        let mut heads = vec![offered.head];
        if let Some(head) = current.head {
            let own = store::membership(store, Some(head.clone()))
                .await?
                .ok_or_else(|| Error::Invalid("missing local membership".into()))?;
            if !own.roots.is_disjoint(&offered.roots) {
                return Err(Error::Invalid(
                    "this store already belongs to the invited mesh".into(),
                ));
            }
            heads.push(head);
        }
        let event = proposal(
            &identity,
            heads,
            Action::Redeem {
                actor: endpoint(&identity.key),
                invitation: anchor.clone(),
            },
            Some(&delegated),
        )?;
        match committed(store, &client, event).await {
            Ok(_) => {
                activate(store).await?;
                return Ok(Output::changed(Change::Paired, status(store).await?));
            }
            Err(Error::MeshControl(ControlError::StaleHead)) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(ControlError::StaleHead.into())
}

pub async fn run(store: &impl ControlStore, command: Command) -> Result<Output> {
    command.validate()?;
    if matches!(
        command,
        Command::Pair {
            initial_sync: true,
            ..
        }
    ) {
        return Err(Error::Invalid(
            "first sync requires the resident peer transport".into(),
        ));
    }
    match command {
        Command::Sync => Err(Error::Invalid(
            "record synchronization requires the resident peer transport".into(),
        )),
        Command::Status => Ok(Output::Status(status(store).await?)),
        Command::Reset => {
            store.step(Step::Reset).await?;
            Ok(Output::changed(Change::IdentityReset, status(store).await?))
        }
        Command::Pair {
            service,
            invitation: Some(invitation),
            ..
        } => redeem(store, invitation, service).await,
        Command::Pair {
            service,
            invitation: None,
            cancel: None,
            ..
        } => create_invitation(store, service).await,
        command => {
            let identity = require_identity(store).await?;
            let leaving = matches!(command, Command::Leave);
            if leaving {
                store.step(Step::Leave).await?;
            }
            let client = ControlClient::new(&identity.service)?;
            resume(store, &client).await?;
            if leaving
                && store::membership(store, None)
                    .await?
                    .is_some_and(|state| !state.members.contains(&endpoint(&identity.key)))
            {
                return Ok(Output::changed(Change::Left, status(store).await?));
            }
            for _ in 0..3 {
                if let Err(error) = refresh(store, &client, &identity.key, None).await {
                    if leaving {
                        return Err(Error::Invalid("local transport disabled; remote revocation is outstanding because the service could not be reached or authorize this peer".into()));
                    }
                    return Err(error);
                }
                let current = require_identity(store).await?;
                let head = current
                    .head
                    .ok_or_else(|| Error::Invalid("store is not enrolled".into()))?;
                let (action, operation) = match &command {
                    Command::Revoke { peer } => (
                        Action::Revoke {
                            actor: endpoint(&identity.key),
                            subject: peer.clone(),
                        },
                        Change::Revoked,
                    ),
                    Command::Leave => (
                        Action::Revoke {
                            actor: endpoint(&identity.key),
                            subject: endpoint(&identity.key),
                        },
                        Change::Left,
                    ),
                    Command::Pair {
                        cancel: Some(invitation),
                        ..
                    } => (
                        Action::Cancel {
                            actor: endpoint(&identity.key),
                            invitation: invitation.clone(),
                        },
                        Change::Cancelled,
                    ),
                    _ => unreachable!(),
                };
                let event = proposal(&identity, vec![head], action, None)?;
                match committed(store, &client, event).await {
                    Ok(_) => {
                        return Ok(Output::changed(operation, status(store).await?));
                    }
                    Err(Error::MeshControl(ControlError::StaleHead)) => continue,
                    Err(error) => return Err(error),
                }
            }
            Err(ControlError::StaleHead.into())
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    struct NoStore;
    impl ControlStore for NoStore {
        async fn step(&self, _: Step) -> Result<Value> {
            panic!("invalid requests must not access the store");
        }
    }
    fn pair(cancel: Option<String>, initial_sync: bool) -> Command {
        Command::Pair {
            service: Some("https://mesh.example".into()),
            invitation: None,
            cancel,
            initial_sync,
        }
    }
    #[test]
    fn invitations_default_to_public_service_and_preserve_existing_enrollment() {
        assert_eq!(invitation_service(None, None), "https://linn.kitten.sh");
        assert_eq!(
            invitation_service(None, Some("https://custom.example")),
            "https://custom.example"
        );
        assert_eq!(
            invitation_service(Some("https://override.example"), None),
            "https://override.example"
        );
        assert_eq!(
            invitation_service(
                Some("https://override.example"),
                Some("https://custom.example")
            ),
            "https://override.example"
        );
        Command::Pair {
            service: None,
            invitation: None,
            cancel: None,
            initial_sync: false,
        }
        .validate()
        .unwrap();
    }

    #[tokio::test]
    async fn validation_rejects_conflicting_modes_before_io() {
        assert!(
            run(&NoStore, pair(Some("1".repeat(64)), false))
                .await
                .is_err()
        );
        assert!(run(&NoStore, pair(None, true)).await.is_err());
        assert!(
            run(
                &NoStore,
                Command::Revoke {
                    peer: "invalid".into()
                }
            )
            .await
            .is_err()
        );
        assert!(pair(None, false).validate().is_ok());
    }
}
