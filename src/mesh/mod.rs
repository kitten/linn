//! Device-authenticated mesh control and peer synchronization.
pub mod client;
pub(crate) mod control_stream;
pub(crate) mod discovery;
pub mod failure;
pub mod jobs;
pub mod membership;
pub mod pairing;
pub(crate) mod peer;
pub(crate) mod progress;
pub(crate) mod proof;
pub mod status;
pub mod store;
pub(crate) mod wire;
