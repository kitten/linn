use super::jobs::{Phase, Report};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transport {
    pub ready: bool,
    pub relay_connected: bool,
    pub local_discovery: bool,
    pub addresses: Vec<String>,
    pub error: Option<String>,
    pub jobs: Vec<Job>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: String,
    pub phase: Phase,
    pub report: Report,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "phase",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Progress {
    Admitted(super::pairing::Status),
    Sync(Job),
    Checked(Probe),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub peer: String,
    pub result: Reachability,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum Reachability {
    Reachable,
    Failed {
        kind: super::failure::Kind,
        message: String,
    },
    Unknown {
        message: String,
    },
}
impl std::fmt::Display for Reachability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reachable => f.write_str("reachable (transport only)"),
            Self::Failed { message, .. } | Self::Unknown { message } => f.write_str(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_version_failure_is_structured_and_fails_the_command() {
        let peer = "a".repeat(64);
        let probe = Probe {
            peer: peer.clone(),
            result: Reachability::Failed {
                kind: crate::mesh::failure::Kind::Version,
                message: format!("update linn on peer {peer}, then retry"),
            },
        };
        let value = serde_json::to_value(&probe).unwrap();
        assert_eq!(value["result"]["kind"], "version");
        assert_eq!(value["peer"], peer);
        let mut status: crate::mesh::pairing::Status = serde_json::from_value(serde_json::json!({
            "endpoint": null, "service": null, "head": null, "enabled": true,
            "pending_leave": false, "conflict": false, "members": [], "revoked": []
        }))
        .unwrap();
        status.reachability = Some(std::collections::BTreeMap::from([(peer, probe.result)]));
        assert!(!crate::mesh::pairing::Output::Status(status).successful());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Traversal {
    pub round: u8,
    pub level: u16,
    pub direct_complete: bool,
    pub discovery_expired: bool,
    pub reached: usize,
    pub no_known_direct_routes: Vec<(String, String)>,
    pub pending: Vec<Contact>,
    pub relay: Vec<Relay>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub id: u64,
    pub source: String,
    pub target: String,
    pub level: u16,
    pub routes: super::scheduler::Routes,
    pub refresh: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relay {
    pub relay: String,
    pub target: String,
    pub outcome: super::scheduler::RelayState,
    pub refresh_used: bool,
}
