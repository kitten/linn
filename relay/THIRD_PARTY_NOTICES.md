# Iroh relay wire protocol

`src/protocol.ts` is an independently written adapter for `iroh-relay` 1.4.1's
binary framing and authentication protocol. Protocol field definitions and domain
separation string follow n0-computer/iroh tag v1.4.1, under MIT OR Apache-2.0:

- https://github.com/n0-computer/iroh/blob/v1.4.1/iroh-relay/src/protos/common.rs
- https://github.com/n0-computer/iroh/blob/v1.4.1/iroh-relay/src/protos/handshake.rs
- https://github.com/n0-computer/iroh/blob/v1.4.1/iroh-relay/src/protos/relay.rs

Copyright (c) 2022 The Iroh Authors. The protocol implementation's licenses are
available at https://github.com/n0-computer/iroh/tree/v1.4.1.
Cryptographic hashing uses the separately installed MIT-licensed `@noble/hashes`;
signature verification uses the runtime's Web Crypto Ed25519 implementation.
