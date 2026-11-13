//! Local RPC schema; independent of CLI parsing and SSH framing.
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
pub(super) const COMPATIBILITY: u32 = 1;
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
    pub fn validate_reply(&self, reply: &ReplyEnvelope) -> Result<()> {
        use Operation::*;
        let valid = match &self.operation {
            Search { .. } | More { .. } => matches!(reply.value, Value::Search(_)),
            Show { .. } => matches!(reply.value, Value::Show(_)),
            Outline { .. } => matches!(reply.value, Value::Outline(_) | Value::OutlineNotCurrent),
            Add(_) | Close { .. } => matches!(reply.value, Value::Added(_)),
            Forget { .. } => matches!(reply.value, Value::Forgotten(_)),
            List(_) => matches!(reply.value, Value::List { .. }),
            Export(_) => matches!(reply.value, Value::Exported),
            Import => matches!(reply.value, Value::Imported(_)),
            Sync { .. } => matches!(reply.value, Value::Synced(_)),
            SyncHello { version } => matches!(&reply.value,
                Value::SyncFrame(crate::sync::protocol::Response::Hello { version: actual }) if actual == version),
            SyncFrame(request) => {
                matches!(&reply.value, Value::SyncFrame(response) if sync_reply_matches(request, response))
            }
            Check => matches!(reply.value, Value::Checked),
            Rebuild => matches!(reply.value, Value::Rebuilt),
            Pair(_) => matches!(reply.value, Value::Paired(_)),
            RemoveRemote { .. } => matches!(reply.value, Value::Removed(_)),
            Remotes => matches!(reply.value, Value::Remotes(_)),
        };
        if !valid || reply.output != matches!(self.operation, Export(_)) {
            return Err(Error::Invalid(
                "daemon reply does not match the requested operation".into(),
            ));
        }
        Ok(())
    }
}
fn sync_reply_matches(
    request: &crate::sync::protocol::Request,
    response: &crate::sync::protocol::Response,
) -> bool {
    use crate::sync::protocol::{Request, Response};
    match request {
        Request::Hello { version } => {
            matches!(response, Response::Hello { version: actual } if actual == version)
        }
        Request::Inventory { .. } => matches!(response, Response::Entries { .. }),
        Request::Get { .. } => matches!(response, Response::Item { .. }),
        Request::Needs { .. } => matches!(response, Response::Needed { .. }),
        Request::Apply { .. } => matches!(response, Response::Applied { .. }),
        Request::Finish => matches!(response, Response::Done),
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
    Status,
    Stop,
    State(DaemonState),
    Ready {
        token: String,
        error: Option<String>,
    },
}
