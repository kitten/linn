//! Ordered membership proposals and coordinator acceptance proofs.
//! @ref LLP 0008#protocol-resolutions
use crate::error::{Error, Result};
use iroh::{PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

pub const MAX_MEMBERS: usize = 256;
pub const MAX_EVENT_BYTES: usize = 16 * 1024;
const SAFE_INTEGER: u64 = (1 << 53) - 1;
const EVENT_DOMAIN: &[u8] = b"linn mesh event v1\0";
const RECEIPT_DOMAIN: &[u8] = b"linn mesh receipt v1\0";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub version: u8,
    pub service: String,
    pub id: String,
    pub heads: Vec<String>,
    pub action: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Genesis {
        actor: String,
        root: String,
        receipt_key: String,
    },
    Invite {
        actor: String,
        key: String,
        receipt_key: String,
        not_before: u64,
        expires: u64,
    },
    Redeem {
        actor: String,
        invitation: String,
    },
    Cancel {
        actor: String,
        invitation: String,
    },
    Revoke {
        actor: String,
        subject: String,
    },
}

impl Action {
    pub fn actor(&self) -> &str {
        match self {
            Self::Genesis { actor, .. }
            | Self::Invite { actor, .. }
            | Self::Redeem { actor, .. }
            | Self::Cancel { actor, .. }
            | Self::Revoke { actor, .. } => actor,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub event: Event,
    pub signatures: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub version: u8,
    pub service: String,
    pub event: String,
    pub accepted: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    pub proposal: Proposal,
    pub receipt: Receipt,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub issuer: String,
    pub key: String,
    pub not_before: u64,
    pub expires: u64,
    pub closed: bool,
}

/// Derived from verified history. Never trust an unsigned remote snapshot.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Membership {
    pub head: String,
    pub service: String,
    pub receipt_key: String,
    pub roots: BTreeSet<String>,
    pub members: BTreeSet<String>,
    pub revoked: BTreeSet<String>,
    pub invitations: BTreeMap<String, Invitation>,
}

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("mesh membership: {message}"))
}

pub fn canonical(value: &impl Serialize) -> Result<Vec<u8>> {
    serde_jcs::to_vec(value).map_err(|_| invalid("cannot canonicalize message"))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn unhex<const N: usize>(value: &str) -> Result<[u8; N]> {
    if value.len() != N * 2
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid("invalid lowercase hex value"));
    }
    let mut result = [0; N];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| invalid("invalid hex value"))?;
    }
    Ok(result)
}

pub fn endpoint(key: &SecretKey) -> String {
    hex(key.public().as_bytes())
}
pub fn random_id() -> String {
    hex(&SecretKey::generate().to_bytes())
}

fn message(domain: &[u8], value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = domain.to_vec();
    bytes.extend(canonical(value)?);
    Ok(bytes)
}

fn verify(key: &str, signature: &str, domain: &[u8], value: &impl Serialize) -> Result<()> {
    let key = PublicKey::from_bytes(&unhex(key)?).map_err(|_| invalid("invalid public key"))?;
    key.verify(
        &message(domain, value)?,
        &Signature::from_bytes(&unhex(signature)?),
    )
    .map_err(|_| super::failure::Kind::Authentication.error("invalid membership signature"))
}

impl Proposal {
    pub fn sign(event: Event, keys: &[&SecretKey]) -> Result<Self> {
        let message = message(EVENT_DOMAIN, &event)?;
        Ok(Self {
            event,
            signatures: keys
                .iter()
                .map(|key| (endpoint(key), hex(&key.sign(&message).to_bytes())))
                .collect(),
        })
    }

    pub fn hash(&self) -> Result<String> {
        Ok(hex(&Sha256::digest(canonical(self)?)))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_EVENT_BYTES {
            return Err(invalid("event too large"));
        }
        // Typed deserialization rejects duplicate fields as well as unknown ones.
        let proposal = serde_json::from_slice(bytes)?;
        if canonical(&proposal)? != bytes {
            return Err(invalid("canonical JSON required"));
        }
        Ok(proposal)
    }

    pub fn verify_signatures(&self) -> Result<()> {
        if canonical(self)?.len() > MAX_EVENT_BYTES
            || self.signatures.is_empty()
            || self.signatures.len() > 2
        {
            return Err(invalid("invalid event size or signature count"));
        }
        for (key, signature) in &self.signatures {
            verify(key, signature, EVENT_DOMAIN, &self.event)?;
        }
        Ok(())
    }

    /// Verify against the named predecessor states at the coordinator's commit time.
    /// The caller must also compare those heads with its *current* durable heads.
    pub fn evaluate(&self, parents: &[&Membership], accepted: u64) -> Result<Membership> {
        self.verify_signatures()?;
        let event = &self.event;
        if event.version != 1 {
            return Err(super::failure::Kind::Version.error("unsupported membership version"));
        }
        if accepted > SAFE_INTEGER
            || event.service.len() > 2048
            || crate::storage::mesh::service_origin(&event.service)? != event.service
        {
            return Err(invalid("invalid version, time or service origin"));
        }
        unhex::<32>(&event.id)?;
        unhex::<32>(event.action.actor())?;
        if event.heads.len() > 2 || event.heads.windows(2).any(|p| p[0] >= p[1]) {
            return Err(invalid("heads must be sorted and distinct"));
        }
        for head in &event.heads {
            unhex::<32>(head)?;
        }
        let heads: BTreeSet<_> = parents.iter().map(|p| &p.head).collect();
        if heads.len() != parents.len() || heads.iter().copied().ne(event.heads.iter()) {
            return Err(invalid("missing or unexpected predecessor"));
        }
        let actor = event.action.actor();
        let mut required = BTreeSet::from([actor.to_owned()]);
        let mut state = if let Action::Genesis {
            root, receipt_key, ..
        } = &event.action
        {
            if !parents.is_empty() {
                return Err(invalid("genesis has predecessors"));
            }
            unhex::<32>(root)?;
            unhex::<32>(receipt_key)?;
            Membership {
                head: String::new(),
                service: event.service.clone(),
                receipt_key: receipt_key.clone(),
                roots: BTreeSet::from([root.clone()]),
                members: BTreeSet::from([actor.to_owned()]),
                revoked: BTreeSet::new(),
                invitations: BTreeMap::new(),
            }
        } else {
            let Some(first) = parents.first() else {
                return Err(invalid("missing predecessor"));
            };
            if !matches!(event.action, Action::Redeem { .. }) && parents.len() != 1 {
                return Err(invalid("ordinary action needs one head"));
            }
            let mut state = (**first).clone();
            for parent in parents {
                if parent.service != event.service || parent.receipt_key != state.receipt_key {
                    return Err(invalid("service or receipt key changed"));
                }
            }
            if parents.len() == 2 {
                let second = parents[1];
                if !state.roots.is_disjoint(&second.roots) {
                    return Err(invalid("overlapping mesh histories"));
                }
                state.roots.extend(second.roots.iter().cloned());
                state.members.extend(second.members.iter().cloned());
                state.revoked.extend(second.revoked.iter().cloned());
                for (id, invitation) in &second.invitations {
                    if state
                        .invitations
                        .insert(id.clone(), invitation.clone())
                        .is_some()
                    {
                        return Err(invalid("duplicate invitation history"));
                    }
                }
                state
                    .members
                    .retain(|member| !state.revoked.contains(member));
            }
            match &event.action {
                Action::Redeem { invitation, .. } => {
                    let source = parents
                        .iter()
                        .find(|parent| parent.invitations.contains_key(invitation))
                        .ok_or_else(|| invalid("unknown invitation"))?;
                    let invite = state
                        .invitations
                        .get_mut(invitation)
                        .ok_or_else(|| invalid("missing invitation"))?;
                    if invite.closed
                        || accepted < invite.not_before
                        || accepted >= invite.expires
                        || !state.members.contains(&invite.issuer)
                        || state.revoked.contains(actor)
                    {
                        return Err(super::failure::Kind::Authentication
                            .error("expired, closed or unauthorized invitation"));
                    }
                    if parents.len() == 2 {
                        if source.members.contains(actor)
                            || !parents
                                .iter()
                                .any(|p| p.head != source.head && p.members.contains(actor))
                        {
                            return Err(invalid("merge actor must belong to the other component"));
                        }
                    } else if state.members.contains(actor) {
                        return Err(invalid("endpoint already enrolled"));
                    }
                    required.insert(invite.key.clone());
                    invite.closed = true;
                    state.members.insert(actor.to_owned());
                }
                Action::Invite {
                    key,
                    receipt_key,
                    not_before,
                    expires,
                    ..
                } => {
                    if !state.members.contains(actor) || receipt_key != &state.receipt_key {
                        return Err(
                            super::failure::Kind::Authentication.error("unauthorized invitation")
                        );
                    }
                    unhex::<32>(key)?;
                    if *expires > SAFE_INTEGER
                        || expires.checked_sub(*not_before) != Some(600)
                        || accepted < *not_before
                        || accepted >= *expires
                    {
                        return Err(invalid("invalid invitation window"));
                    }
                    if key == actor || state.members.contains(key) || state.revoked.contains(key) {
                        return Err(invalid("delegated key must be fresh"));
                    }
                    if state.invitations.values().any(|invite| &invite.key == key) {
                        return Err(invalid("delegated key was already used"));
                    }
                    state.invitations.insert(
                        self.hash()?,
                        Invitation {
                            issuer: actor.into(),
                            key: key.clone(),
                            not_before: *not_before,
                            expires: *expires,
                            closed: false,
                        },
                    );
                }
                Action::Cancel { invitation, .. } => {
                    if !state.members.contains(actor) {
                        return Err(
                            super::failure::Kind::Authentication.error("actor is not a member")
                        );
                    }
                    let invitation = state
                        .invitations
                        .get_mut(invitation)
                        .ok_or_else(|| invalid("unknown invitation"))?;
                    if invitation.closed {
                        return Err(invalid("invitation already closed"));
                    }
                    invitation.closed = true;
                }
                Action::Revoke { subject, .. } => {
                    if !state.members.contains(actor) || !state.members.remove(subject) {
                        return Err(super::failure::Kind::Authentication
                            .error("actor or subject is not a member"));
                    }
                    state.revoked.insert(subject.clone());
                }
                Action::Genesis { .. } => unreachable!(),
            }
            state
        };
        if required.iter().ne(self.signatures.keys()) {
            return Err(super::failure::Kind::Authentication.error("wrong event signers"));
        }
        if state.members.len() > MAX_MEMBERS {
            return Err(invalid("mesh exceeds member limit"));
        }
        state.head = self.hash()?;
        Ok(state)
    }
}

impl Acceptance {
    pub fn verify_receipt(&self, key: &str) -> Result<()> {
        if self.receipt.version != 1
            || self.receipt.event != self.proposal.hash()?
            || self.receipt.service != self.proposal.event.service
            || self.receipt.accepted > SAFE_INTEGER
        {
            return Err(invalid("receipt does not bind this event"));
        }
        verify(key, &self.signature, RECEIPT_DOMAIN, &self.receipt)
    }
    /// Service-side only. Membership still derives from device signatures.
    pub fn sign(proposal: Proposal, accepted: u64, receipt_key: &SecretKey) -> Result<Self> {
        let receipt = Receipt {
            version: 1,
            service: proposal.event.service.clone(),
            event: proposal.hash()?,
            accepted,
        };
        let signature = hex(&receipt_key
            .sign(&message(RECEIPT_DOMAIN, &receipt)?)
            .to_bytes());
        Ok(Self {
            proposal,
            receipt,
            signature,
        })
    }

    pub fn evaluate(&self, parents: &[&Membership]) -> Result<Membership> {
        let state = self.proposal.evaluate(parents, self.receipt.accepted)?;
        if self.receipt.version != 1
            || self.receipt.event != state.head
            || self.receipt.service != state.service
        {
            return Err(invalid("receipt does not bind this event"));
        }
        self.verify_receipt(&state.receipt_key)?;
        Ok(state)
    }
}
