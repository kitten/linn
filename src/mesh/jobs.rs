//! Finite manual run identities and per-phase outcomes.
use super::membership::{Membership, canonical, endpoint, hex, unhex};
use crate::error::{Error, Result};
use iroh::{PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};

const DOMAIN: &[u8] = b"linn mesh job v1\0";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub version: u8,
    pub id: String,
    pub initiator: String,
    pub head: String,
    pub cohort: Vec<String>,
    pub max_hops: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedRun {
    pub run: Run,
    pub signature: String,
    pub path: Vec<String>,
}
impl SignedRun {
    pub fn new(key: &SecretKey, membership: &Membership, id: String) -> Result<Self> {
        unhex::<32>(&id)?;
        let run = Run {
            version: 1,
            id,
            initiator: endpoint(key),
            head: membership.head.clone(),
            cohort: membership.members.iter().cloned().collect(),
            max_hops: 16,
        };
        let mut message = DOMAIN.to_vec();
        message.extend(canonical(&run)?);
        Ok(Self {
            path: vec![run.initiator.clone()],
            signature: hex(&key.sign(&message).to_bytes()),
            run,
        })
    }
    pub fn verify(&self, frozen: &Membership, current: &Membership) -> Result<()> {
        let run = &self.run;
        unhex::<32>(&run.id)?;
        if run.version != 1 {
            return Err(super::failure::Kind::Version.error("unsupported mesh job version"));
        }
        if run.max_hops == 0
            || run.max_hops > 16
            || self.path.is_empty()
            || self.path.len() > usize::from(run.max_hops) + 1
            || self.path.first() != Some(&run.initiator)
            || self
                .path
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.path.len()
            || self.path.iter().any(|id| !run.cohort.contains(id))
            || run.head != frozen.head
            || run.cohort.len() > 256
            || run.cohort.iter().ne(frozen.members.iter())
            || !frozen.members.contains(&run.initiator)
            || !current.members.contains(&run.initiator)
            || frozen.roots.is_disjoint(&current.roots)
        {
            return Err(super::failure::Kind::Authentication
                .error("invalid manual mesh job authority or cohort"));
        }
        let public = PublicKey::from_bytes(&unhex(&run.initiator)?)
            .map_err(|_| Error::Invalid("invalid job initiator".into()))?;
        let mut message = DOMAIN.to_vec();
        message.extend(canonical(run)?);
        public
            .verify(&message, &Signature::from_bytes(&unhex(&self.signature)?))
            .map_err(|_| super::failure::Kind::Authentication.error("invalid manual job signature"))
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Collecting,
    Collected,
    Distributing,
    Complete,
    Failed,
    Cancelled,
}
impl Phase {
    pub(crate) fn is_terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Failed | Self::Cancelled)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub peer: String,
    pub phase: Phase,
    pub kind: super::failure::Kind,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub run: String,
    pub cohort: Vec<String>,
    pub collected: Vec<String>,
    pub distributed: Vec<String>,
    pub failures: Vec<Failure>,
    pub received: u64,
    pub complete: bool,
}
impl Report {
    pub fn new(run: &Run, local: &str) -> Self {
        Self {
            run: run.id.clone(),
            cohort: run.cohort.clone(),
            collected: vec![local.into()],
            ..Self::default()
        }
    }
    pub fn merge_collected(&mut self, other: &Self) {
        self.collected.extend(other.collected.iter().cloned());
        self.failures.extend(other.failures.iter().cloned());
        self.normalize();
    }
    pub fn merge_distributed(&mut self, other: &Self) {
        self.distributed.extend(other.distributed.iter().cloned());
        self.failures.extend(other.failures.iter().cloned());
        self.normalize();
    }
    pub fn normalize(&mut self) {
        self.collected.sort();
        self.collected.dedup();
        self.distributed.sort();
        self.distributed.dedup();
        self.failures.retain(|failure| match failure.phase {
            Phase::Collecting => !self.collected.contains(&failure.peer),
            Phase::Distributing => !self.distributed.contains(&failure.peer),
            _ => true,
        });
        self.failures.sort_by(|a, b| {
            (&a.peer, a.phase, &a.kind, &a.message).cmp(&(&b.peer, b.phase, &b.kind, &b.message))
        });
        self.failures
            .dedup_by(|a, b| a.peer == b.peer && a.phase == b.phase);
        self.complete = self.failures.is_empty()
            && self
                .cohort
                .iter()
                .all(|peer| self.collected.contains(peer) && self.distributed.contains(peer));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forwarding_paths_are_unique_authorized_and_hop_bounded() {
        let keys: Vec<_> = (0..20).map(|_| SecretKey::generate()).collect();
        let state = Membership {
            head: "0".repeat(64),
            service: "https://mesh.example".into(),
            receipt_key: "0".repeat(64),
            roots: std::collections::BTreeSet::from(["1".repeat(64)]),
            members: keys.iter().map(endpoint).collect(),
            revoked: Default::default(),
            invitations: Default::default(),
        };
        let mut job =
            SignedRun::new(&keys[0], &state, crate::mesh::membership::random_id()).unwrap();
        assert!(job.verify(&state, &state).is_ok());
        job.path = keys.iter().take(17).map(endpoint).collect();
        assert!(job.verify(&state, &state).is_ok());
        job.path.push(endpoint(&keys[17]));
        assert!(job.verify(&state, &state).is_err());
        job.path = vec![endpoint(&keys[0]), endpoint(&keys[1]), endpoint(&keys[0])];
        assert!(job.verify(&state, &state).is_err());
        job.path = vec![endpoint(&keys[1])];
        assert!(job.verify(&state, &state).is_err());
        job.path = vec![endpoint(&keys[0])];
        job.run.max_hops = 1; // Changing the forwarding budget invalidates the signature.
        assert!(job.verify(&state, &state).is_err());
    }
}
