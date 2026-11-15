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
}
