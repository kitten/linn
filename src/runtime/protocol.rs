//! Local RPC schema; independent of CLI parsing and peer framing.
//! @ref LLP 0007#local-protocol-and-ownership
use super::platform;
use crate::{
    application::{Caller, Operation, Value},
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// Keep lifecycle framing stable across application compatibility changes.
pub(super) const VERSION: u32 = 1;
// @ref LLP 0007#automatic-detached-lifecycle
pub(super) const COMPATIBILITY: u32 = 6;
#[derive(Serialize, Deserialize)]
pub(super) struct Hello {
    pub version: u32,
    // Retain the legacy field so status/stop can connect to pre-counter daemons.
    #[serde(default)]
    pub build: String,
    #[serde(default)]
    pub compatibility: u32,
    #[serde(default)]
    pub schema: usize,
    pub identity: platform::Identity,
    #[serde(with = "crate::native_path")]
    pub store: PathBuf,
    pub control: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Invocation {
    pub operation: Operation,
    pub caller: Caller,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplyEnvelope {
    pub value: Value,
    pub warnings: Vec<String>,
    pub output: bool,
}
impl Invocation {
    pub fn validate_progress(&self, progress: &crate::mesh::status::Progress) -> Result<()> {
        use crate::mesh::{pairing::Command, status::Progress};
        let valid = matches!(
            (&self.operation, progress),
            (
                Operation::Mesh(Command::Pair {
                    initial_sync: true,
                    invitation: Some(_),
                    ..
                }),
                _
            ) | (Operation::Mesh(Command::Sync), Progress::Sync(_))
        );
        let valid = valid
            && match progress {
                Progress::Admitted(status) => status.is_admitted(),
                Progress::Sync(job) => {
                    job.id == job.report.run
                        && crate::mesh::membership::unhex::<32>(&job.id).is_ok()
                }
            };
        if valid {
            Ok(())
        } else {
            Err(Error::Invalid("unexpected operation progress".into()))
        }
    }
    pub fn validate_reply(&self, reply: &ReplyEnvelope) -> Result<()> {
        use Operation::*;
        let valid = match &self.operation {
            Mesh(command) => matches!(&reply.value, Value::Mesh(output) if command.accepts(output)),
            Search { .. } | More { .. } => matches!(reply.value, Value::Search(_)),
            Show { .. } => matches!(reply.value, Value::Show(_)),
            Outline { .. } => matches!(reply.value, Value::Outline(_) | Value::OutlineNotCurrent),
            Add(_) | Close { .. } => matches!(reply.value, Value::Added(_)),
            Forget { .. } => matches!(reply.value, Value::Forgotten(_)),
            List(_) => matches!(reply.value, Value::List { .. }),
            Export(_) => matches!(reply.value, Value::Exported),
            Import => matches!(reply.value, Value::Imported(_)),
            Check => matches!(reply.value, Value::Checked),
            Rebuild => matches!(reply.value, Value::Rebuilt),
        };
        if !valid || reply.output != matches!(self.operation, Export(_)) {
            return Err(Error::Invalid(
                "daemon reply does not match the requested operation".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
pub struct DaemonState {
    pub running: bool,
    pub pid: Option<u32>,
}
impl DaemonState {
    pub(super) fn stopped() -> Self {
        Self {
            running: false,
            pid: None,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub(super) enum Frame {
    Hello(Hello),
    Accepted,
    Rejected(String),
    Invoke {
        id: u64,
    },
    Chunk(String),
    End,
    Reply {
        id: u64,
    },
    Failure(crate::error::ApplicationError),
    Progress(crate::mesh::status::Progress),
    Status,
    Stop,
    State(DaemonState),
    Ready {
        token: String,
        error: Option<String>,
    },
}
