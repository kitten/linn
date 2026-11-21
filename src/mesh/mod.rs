//! Device-authenticated mesh control and peer synchronization.
pub(crate) mod buffers;
pub mod client;
mod connections;
pub(crate) mod control_stream;
mod delta;
pub(crate) mod discovery;
pub mod discovery_job;
pub mod failure;
pub mod jobs;
pub mod membership;
pub mod pairing;
pub(crate) mod peer;
pub(crate) mod progress;
pub(crate) mod proof;
pub mod scheduler;
pub mod status;
pub mod store;
pub(crate) mod wire;
