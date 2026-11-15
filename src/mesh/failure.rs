//! Stable report categories preserved across the local dispatcher boundary.
//! @ref LLP 0008#typed-job-failures
use crate::error::{Error, ErrorCode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Kind {
    Authentication = 0,
    Version = 1,
    Conflict = 2,
    Publication = 3,
    Timeout = 4,
    Offline = 5,
    Protocol = 6,
    Cancelled = 7,
    Capacity = 8,
    HopLimit = 9,
}
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct FailureError {
    pub kind: Kind,
    pub message: String,
}
impl Kind {
    // Reserved application codes shared by QUIC connection closes and stream resets.
    pub fn code(self) -> u32 {
        0x4c00 + self as u32
    }
    pub fn from_code(code: u64) -> Option<Self> {
        [
            Self::Authentication,
            Self::Version,
            Self::Conflict,
            Self::Publication,
            Self::Timeout,
            Self::Offline,
            Self::Protocol,
            Self::Cancelled,
            Self::Capacity,
            Self::HopLimit,
        ]
        .get(usize::try_from(code.checked_sub(0x4c00)?).ok()?)
        .copied()
    }
    pub fn connection(error: iroh::endpoint::ConnectionError) -> Error {
        use iroh::endpoint::ConnectionError;
        let kind = match &error {
            ConnectionError::ApplicationClosed(close) => {
                Self::from_code(close.error_code.into_inner()).unwrap_or(Self::Offline)
            }
            ConnectionError::VersionMismatch => Self::Version,
            ConnectionError::TransportError(error) => Self::transport(u64::from(error.code)),
            ConnectionError::ConnectionClosed(close) => {
                Self::transport(u64::from(close.error_code))
            }
            ConnectionError::TimedOut => Self::Timeout,
            ConnectionError::LocallyClosed => Self::Cancelled,
            _ => Self::Offline,
        };
        kind.error(error.to_string())
    }
    fn transport(code: u64) -> Self {
        if code == 0x100 + 120 {
            Self::Version
        }
        // TLS no_application_protocol.
        else if (0x100..=0x1ff).contains(&code) {
            Self::Authentication
        } else {
            Self::Protocol
        }
    }
    pub fn connect(error: iroh::endpoint::ConnectError) -> Error {
        use iroh::endpoint::{AuthenticationError, ConnectError, ConnectingError};
        match error {
            ConnectError::Connection { source, .. } => Self::connection(source),
            ConnectError::Connecting {
                source: ConnectingError::ConnectionError { source, .. },
                ..
            } => Self::connection(source),
            ConnectError::Connecting {
                source: ConnectingError::HandshakeFailure { source, .. },
                ..
            } => {
                let kind = if matches!(source, AuthenticationError::NoAlpn { .. }) {
                    Self::Version
                } else {
                    Self::Authentication
                };
                kind.error(source.to_string())
            }
            _ => Self::Offline.error(error.to_string()),
        }
    }
    pub fn write(error: iroh::endpoint::WriteError) -> Error {
        use iroh::endpoint::WriteError;
        match error {
            WriteError::ConnectionLost(error) => Self::connection(error),
            WriteError::Stopped(code) => Self::from_code(code.into_inner())
                .unwrap_or(Self::Offline)
                .error("remote mesh stream stopped"),
            _ => Self::Offline.error(error.to_string()),
        }
    }
    pub fn read(error: iroh::endpoint::ReadError) -> Error {
        use iroh::endpoint::ReadError;
        match error {
            ReadError::ConnectionLost(error) => Self::connection(error),
            ReadError::Reset(code) => Self::from_code(code.into_inner())
                .unwrap_or(Self::Offline)
                .error("remote mesh stream failed"),
            _ => Self::Offline.error(error.to_string()),
        }
    }
    pub fn read_exact(error: iroh::endpoint::ReadExactError) -> Error {
        use iroh::endpoint::ReadExactError;
        match error {
            ReadExactError::ReadError(error) => Self::read(error),
            ReadExactError::FinishedEarly(_) => Self::Protocol.error("truncated mesh frame"),
        }
    }
    pub fn error(self, message: impl Into<String>) -> Error {
        FailureError {
            kind: self,
            message: message.into(),
        }
        .into()
    }
    pub fn publication(error: Error) -> Error {
        if matches!(&error, Error::Schema { .. } | Error::Mesh(_))
            || matches!(&error, Error::Application(error) if matches!(error.code, ErrorCode::Schema | ErrorCode::Mesh(_)))
        {
            error
        } else {
            Self::Publication.error(error.to_string())
        }
    }
    pub fn of(error: &Error) -> Self {
        use super::client::ControlError;
        match error {
            Error::Mesh(error) => error.kind,
            Error::Application(error) => match error.code {
                ErrorCode::Mesh(kind) => kind,
                ErrorCode::Schema => Self::Version,
                ErrorCode::Store | ErrorCode::Annotation => Self::Publication,
                ErrorCode::Io => Self::Offline,
                ErrorCode::Invalid => Self::Protocol,
            },
            Error::MeshControl(error) => match error {
                ControlError::Unauthorized => Self::Authentication,
                ControlError::StaleHead => Self::Conflict,
                ControlError::Unavailable => Self::Offline,
                ControlError::Rejected(_) | ControlError::Protocol(_) => Self::Protocol,
            },
            Error::Schema { .. } => Self::Version,
            Error::Sql(_) | Error::Porter(_) => Self::Publication,
            Error::Io(error) if error.kind() == std::io::ErrorKind::TimedOut => Self::Timeout,
            Error::Io(_) => Self::Offline,
            Error::Json(_) | Error::Invalid(_) => Self::Protocol,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatcher_serialization_preserves_the_failure_category() {
        for kind in [
            Kind::Authentication,
            Kind::Version,
            Kind::Conflict,
            Kind::Publication,
            Kind::Timeout,
            Kind::Offline,
            Kind::Protocol,
            Kind::Cancelled,
            Kind::Capacity,
            Kind::HopLimit,
        ] {
            let error = crate::error::ApplicationError::new(kind.error("detail"), true);
            let bytes = serde_json::to_vec(&error).unwrap();
            let error = Error::Application(serde_json::from_slice(&bytes).unwrap());
            assert_eq!(Kind::of(&error), kind);
            assert_eq!(error.to_string(), "detail");
        }
        assert_eq!(
            Kind::of(&Error::Schema {
                found: 999,
                supported: 8
            }),
            Kind::Version
        );
        for (control, kind) in [
            (
                super::super::client::ControlError::Unauthorized,
                Kind::Authentication,
            ),
            (
                super::super::client::ControlError::StaleHead,
                Kind::Conflict,
            ),
            (
                super::super::client::ControlError::Unavailable,
                Kind::Offline,
            ),
        ] {
            let error = crate::error::ApplicationError::new(control.into(), true);
            assert_eq!(Kind::of(&Error::Application(error)), kind);
        }
        assert_eq!(
            Kind::of(&Kind::publication(Error::Invalid(
                "pending publication".into()
            ))),
            Kind::Publication
        );
        assert_eq!(
            Kind::of(&Kind::publication(Error::Schema {
                found: 999,
                supported: 8
            })),
            Kind::Version
        );
        assert_eq!(
            Kind::of(&Error::Io(std::io::ErrorKind::TimedOut.into())),
            Kind::Timeout
        );
    }
}
