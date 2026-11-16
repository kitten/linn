# Native client control heartbeats

Sources are the published iroh-relay 1.4.1 crate (MIT OR Apache-2.0), SHA-256
`7e5589e7c1475700686603a970275687f760df853bc6525c7eb1347aa31d7230`, with its
BSD-3-Clause notice for inherited code. The Cargo patch keeps the pinned version.

`src/protos/streams.rs` adds a native client-only WebSocket adapter mode:
canonical eight-byte application Pings become WebSocket control Pings, and
eight-byte control Pongs become application Pong events for Iroh's existing
random-token health tracker. `src/client/conn.rs` enables it for native clients.
Server adapters default to upstream behavior, as do browser clients.
Application Pongs, datagrams and handshakes are unchanged. In particular, replies
to server application checkpoints remain binary, ordered application messages;
control Pongs must never acknowledge the relay's application send window.

See LLP 0008's Worker architecture and resource limits for the contract. Remove
this local patch when an adopted upstream release supplies equivalent behavior.

Adapter regression tests: `nix develop -c cargo test --manifest-path
vendor/iroh-relay/Cargo.toml --lib control_health_tests`. The relay interop gate
also asserts that native health pings never enter Worker JavaScript while
application checkpoint acknowledgements do.
