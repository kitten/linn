# Single relay without HTTPS probes

Sources are the published iroh 1.4.1 crate (MIT OR Apache-2.0), SHA-256
`e3689cab508af9bcef7816c1201b403595ec6aa20e7d15e1f9683bb4be4a5b17`.
MIT and Apache license files come from the upstream `v1.4.1` tag; the crate's
inherited BSD notice is retained too.

`src/net_report.rs` selects the sole configured relay when HTTPS probes are
explicitly disabled. There is no latency ranking to perform in this case.
The existing relay actor connects, authenticates, reports connection failures,
reconnects and checks health. No latency or successful connection is
fabricated. Empty and multi-relay maps, and HTTPS-probe-enabled selection, retain
upstream behavior.

Linn configures `NetReportConfig::minimal()` and a single `RelayConfig` with
`quic: None`: its Worker has no QUIC address-discovery listener. Direct peer IP
transports remain enabled. The native relay interoperability gate checks startup,
idle continuity and data exchange without HTTP probes. Workspace checks exclude
this dependency's own tests, as with the existing vendored dependencies.

Remove the corresponding changes when an adopted upstream version supplies
equivalent behavior. See LLP 0008 for the product contract.
