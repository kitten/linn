//! Standard DNS-SD address hints, independent of membership authorization.
//! @ref LLP 0008#offline-same-host-discovery
use crate::{
    error::{Error, Result},
    mesh::membership::{Membership, unhex},
};
use iroh::{Endpoint, EndpointAddr, PublicKey, address_lookup::memory::MemoryLookup};
use mdns_sd::{
    DaemonEvent, IfKind, ResolvedService, ScopedIp, ServiceDaemon, ServiceEvent, ServiceInfo,
};
use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Notify, task::JoinHandle};
const SERVICE: &str = "_linn._udp.local.";
const MAX_ADDRESSES: usize = 32;
const CACHE_BUDGET: i64 = 4096;

fn peer(name: &str) -> Option<PublicKey> {
    PublicKey::from_z32(name.strip_suffix(&format!(".{SERVICE}"))?).ok()
}
struct State {
    lookup: MemoryLookup,
    local: PublicKey,
    allowed: BTreeSet<PublicKey>,
    seen: BTreeSet<PublicKey>,
    error: Option<String>,
}
impl State {
    fn resolved(&mut self, info: &ResolvedService) {
        let Some(id) = peer(info.get_fullname()) else {
            return;
        };
        if id == self.local
            || !self.allowed.contains(&id)
            || info.get_port() == 0
            || info.get_property_val_str("v") != Some("1")
        {
            return;
        }
        let candidates = info.get_addresses().iter().filter_map(|ip| {
            let ScopedIp::V4(ip) = ip else {
                return None;
            };
            let addr = *ip.addr();
            if addr.is_unspecified() || addr.is_multicast() || addr.is_broadcast() {
                return None;
            }
            // A remote host's advertised loopback address must never become ours.
            if addr.is_loopback()
                && !ip
                    .interface_ids()
                    .iter()
                    .any(|i| i.get_addrs().iter().any(|ip| ip.is_loopback()))
            {
                return None;
            }
            Some(SocketAddr::new(addr.into(), info.get_port()))
        });
        let addresses: BTreeSet<_> = candidates
            .clone()
            .filter(|a| a.ip().is_loopback())
            .chain(candidates.filter(|a| !a.ip().is_loopback()))
            .take(MAX_ADDRESSES)
            .collect();
        self.lookup.remove_endpoint_info(id);
        if !addresses.is_empty() {
            self.lookup.set_endpoint_info(
                EndpointAddr::new(id)
                    .with_addrs(addresses.into_iter().map(iroh::TransportAddr::Ip)),
            );
            self.seen.insert(id);
        } else {
            self.seen.remove(&id);
        }
    }
    fn remove(&mut self, id: PublicKey) {
        self.lookup.remove_endpoint_info(id);
        self.seen.remove(&id);
    }
    fn clear(&mut self) {
        for id in std::mem::take(&mut self.seen) {
            self.lookup.remove_endpoint_info(id);
        }
        self.allowed.clear();
    }
    fn event(&mut self, event: ServiceEvent) {
        match event {
            ServiceEvent::ServiceResolved(info) => self.resolved(&info),
            ServiceEvent::ServiceRemoved(_, name) => {
                if let Some(id) = peer(&name) {
                    self.remove(id);
                }
            }
            _ => (),
        }
    }
}
pub(crate) struct LocalLookup {
    state: Arc<Mutex<State>>,
    daemon: ServiceDaemon,
    changed: Arc<Notify>,
    task: JoinHandle<()>,
}
impl LocalLookup {
    pub(crate) fn start(
        endpoint: &Endpoint,
        lookup: MemoryLookup,
        membership: &Membership,
    ) -> Result<Self> {
        let socket = endpoint
            .bound_sockets()
            .into_iter()
            .find(|s| s.is_ipv4() && (s.ip().is_unspecified() || s.ip() == Ipv4Addr::LOCALHOST))
            .ok_or_else(|| {
                Error::Invalid("local discovery needs a loopback-capable IPv4 socket".into())
            })?;
        let local = endpoint.id();
        let allowed = members(membership)?;
        let name = local.to_z32();
        let mut service = ServiceInfo::new(
            SERVICE,
            &name,
            &format!("{name}.local."),
            "127.0.0.1",
            socket.port(),
            [("v", "1")].as_slice(),
        )
        .map_err(|e| Error::Invalid(format!("mDNS service: {e}")))?;
        if socket.ip().is_unspecified() {
            service = service.enable_addr_auto();
            service.set_interfaces(vec![IfKind::IPv4]);
        } else {
            service.set_interfaces(vec![IfKind::LoopbackV4]);
        }
        let daemon =
            ServiceDaemon::new().map_err(|e| Error::Invalid(format!("mDNS startup: {e}")))?;
        let setup = || {
            daemon.disable_interface(IfKind::IPv6)?;
            if !socket.ip().is_unspecified() {
                daemon.disable_interface(IfKind::All)?;
            }
            daemon.enable_interface(IfKind::LoopbackV4)?;
            daemon.set_multicast_loop_v4(true)?;
            let monitor = daemon.monitor()?;
            let events = daemon.browse(SERVICE)?;
            daemon.register(service)?;
            Ok::<_, mdns_sd::Error>((events, monitor))
        };
        let (mut events, monitor) = match setup() {
            Ok(value) => value,
            Err(error) => {
                let _ = daemon.shutdown();
                return Err(Error::Invalid(format!("mDNS setup: {error}")));
            }
        };
        let state = Arc::new(Mutex::new(State {
            lookup,
            local,
            allowed,
            seen: BTreeSet::new(),
            error: None,
        }));
        let updates = state.clone();
        let changed = Arc::new(Notify::new());
        let refresh = changed.clone();
        let worker = daemon.clone();
        let task = tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let result: std::result::Result<(), String> = async { loop {
                tokio::select! {
                    event = events.recv_async() => updates.lock().unwrap().event(event.map_err(|e| e.to_string())?),
                    event = monitor.recv_async() => match event.map_err(|e| e.to_string())? {
                        DaemonEvent::Error(error) => updates.lock().unwrap().error = Some(format!("mDNS: {error}").chars().take(256).collect()),
                        DaemonEvent::NameChange(_) => return Err("mDNS endpoint identity has a name conflict".to_string()),
                        _ => (),
                    },
                    _ = refresh.notified() => { events = worker.browse(SERVICE).map_err(|e| e.to_string())?; },
                    _ = tick.tick() => {
                        let metrics = worker.get_metrics().map_err(|e| e.to_string())?;
                        let values = tokio::time::timeout(Duration::from_secs(2), metrics.recv_async()).await.map_err(|_| "mDNS metrics stalled".to_string())?.map_err(|e| e.to_string())?;
                        if values.iter().filter(|(key,_)| key.starts_with("cached-")).map(|(_, value)| *value).sum::<i64>() > CACHE_BUDGET {
                            return Err("mDNS cache budget exceeded".to_string());
                        }
                    }
                }
            }}.await;
            if let Err(error) = result {
                let mut state = updates.lock().unwrap();
                state.clear();
                state.error = Some(error.chars().take(256).collect());
            }
            let _ = worker.shutdown();
        });
        Ok(Self {
            state,
            daemon,
            changed,
            task,
        })
    }
    pub(crate) fn update_membership(&self, membership: &Membership) -> Result<()> {
        let allowed = members(membership)?;
        let mut state = self.state.lock().unwrap();
        if state.allowed != allowed {
            let removed: Vec<_> = state.seen.difference(&allowed).copied().collect();
            for id in removed {
                state.remove(id);
            }
            state.allowed = allowed;
            self.changed.notify_one();
        }
        Ok(())
    }
    pub(crate) fn error(&self) -> Option<String> {
        self.state.lock().unwrap().error.clone()
    }
    pub(crate) fn is_running(&self) -> bool {
        !self.task.is_finished()
    }
    pub(crate) fn stop(&self) {
        self.task.abort();
        let _ = self.daemon.shutdown();
        self.state.lock().unwrap().clear();
    }
    #[cfg(test)]
    fn addresses(&self, id: PublicKey) -> Vec<SocketAddr> {
        self.state
            .lock()
            .unwrap()
            .lookup
            .get_endpoint_info(id)
            .map(|i| i.to_endpoint_addr().ip_addrs().copied().collect())
            .unwrap_or_default()
    }
}
impl Drop for LocalLookup {
    fn drop(&mut self) {
        self.stop();
    }
}
fn members(state: &Membership) -> Result<BTreeSet<PublicKey>> {
    state
        .members
        .iter()
        .map(|id| {
            PublicKey::from_bytes(&unhex(id)?)
                .map_err(|_| Error::Invalid("invalid mesh endpoint key".into()))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn dns_records_require_membership_profile_and_valid_interface_scope() {
        let local = iroh::SecretKey::generate().public();
        let approved = iroh::SecretKey::generate().public();
        let unknown = iroh::SecretKey::generate().public();
        let make = |id: PublicKey, ip: &str, port, version: &str| {
            ServiceInfo::new(
                SERVICE,
                &id.to_z32(),
                &format!("{}.local.", id.to_z32()),
                ip,
                port,
                [("v", version)].as_slice(),
            )
            .unwrap()
            .as_resolved_service()
        };
        let mut state = State {
            lookup: MemoryLookup::new(),
            local,
            allowed: BTreeSet::from([approved]),
            seen: BTreeSet::new(),
            error: None,
        };
        state.resolved(&make(approved, "192.0.2.1", 12345, "1"));
        assert!(state.lookup.get_endpoint_info(approved).is_some());
        for info in [
            make(unknown, "192.0.2.2", 12345, "1"),
            make(local, "192.0.2.3", 12345, "1"),
            make(approved, "192.0.2.4", 12345, "2"),
        ] {
            state.resolved(&info);
        }
        assert_eq!(state.seen, BTreeSet::from([approved]));
        assert!(state.lookup.get_endpoint_info(unknown).is_none());
        state.event(ServiceEvent::ServiceRemoved(
            SERVICE.into(),
            format!("{}.{SERVICE}", approved.to_z32()),
        ));
        assert!(state.lookup.get_endpoint_info(approved).is_none());
        // A loopback record without local receive-interface metadata is unsafe.
        state.resolved(&make(approved, "127.0.0.1", 12345, "1"));
        assert!(state.lookup.get_endpoint_info(approved).is_none());
        state.allowed.clear();
        state.resolved(&make(approved, "192.0.2.1", 12345, "1"));
        assert!(state.seen.is_empty());
    }
    #[tokio::test]
    #[ignore = "requires native loopback multicast"]
    async fn loopback_rediscovery_survives_both_endpoints_restarting_with_new_ports() {
        use iroh::{RelayMode, SecretKey, endpoint::presets};
        let a_key = SecretKey::generate();
        let b_key = SecretKey::generate();
        let state = Membership {
            head: "0".repeat(64),
            service: "https://unreachable.invalid".into(),
            receipt_key: "0".repeat(64),
            roots: BTreeSet::new(),
            members: [&a_key, &b_key]
                .into_iter()
                .map(super::super::membership::endpoint)
                .collect(),
            revoked: BTreeSet::new(),
            invitations: Default::default(),
        };
        async fn round(
            a_key: &SecretKey,
            b_key: &SecretKey,
            state: &Membership,
        ) -> (SocketAddr, SocketAddr) {
            let a_lookup = MemoryLookup::new();
            let b_lookup = MemoryLookup::new();
            let a = Endpoint::builder(presets::Minimal)
                .secret_key(a_key.clone())
                .clear_ip_transports()
                .bind_addr("127.0.0.1:0")
                .unwrap()
                .clear_address_lookup()
                .address_lookup(a_lookup.clone())
                .relay_mode(RelayMode::Disabled)
                .alpns(vec![b"linn/discovery-test/1".to_vec()])
                .bind()
                .await
                .unwrap();
            let b = Endpoint::builder(presets::Minimal)
                .secret_key(b_key.clone())
                .clear_ip_transports()
                .bind_addr("127.0.0.1:0")
                .unwrap()
                .clear_address_lookup()
                .address_lookup(b_lookup.clone())
                .relay_mode(RelayMode::Disabled)
                .alpns(vec![b"linn/discovery-test/1".to_vec()])
                .bind()
                .await
                .unwrap();
            let a_local = LocalLookup::start(&a, a_lookup, state).unwrap();
            let b_local = LocalLookup::start(&b, b_lookup, state).unwrap();
            tokio::time::timeout(Duration::from_secs(15), async {
                while a_local.addresses(b.id()).is_empty() || b_local.addresses(a.id()).is_empty() {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
                let receive = async {
                    let conn = b.accept().await.unwrap().await.unwrap();
                    assert_eq!(conn.remote_id(), a.id());
                    let (mut send, mut recv) = conn.accept_bi().await.unwrap();
                    let bytes = recv.read_to_end(1024).await.unwrap();
                    assert_eq!(bytes, b"offline loopback");
                    send.write_all(&bytes).await.unwrap();
                    send.finish().unwrap();
                    conn.closed().await;
                };
                let transmit = async {
                    let conn = a
                        .connect(
                            EndpointAddr::new(b.id()).with_addrs(
                                a_local
                                    .addresses(b.id())
                                    .into_iter()
                                    .map(iroh::TransportAddr::Ip),
                            ),
                            b"linn/discovery-test/1",
                        )
                        .await
                        .unwrap();
                    let (mut send, mut recv) = conn.open_bi().await.unwrap();
                    send.write_all(b"offline loopback").await.unwrap();
                    send.finish().unwrap();
                    assert_eq!(recv.read_to_end(1024).await.unwrap(), b"offline loopback");
                    conn.close(0u32.into(), b"done");
                };
                tokio::join!(receive, transmit);
            })
            .await
            .unwrap();
            let ports = (a.bound_sockets()[0], b.bound_sockets()[0]);
            drop(a_local);
            drop(b_local);
            a.close().await;
            b.close().await;
            ports
        }
        let (old_a, old_b) = round(&a_key, &b_key, &state).await;
        let _hold_a = std::net::UdpSocket::bind(old_a).unwrap();
        let _hold_b = std::net::UdpSocket::bind(old_b).unwrap();
        let (new_a, new_b) = round(&a_key, &b_key, &state).await;
        assert_ne!(old_a, new_a);
        assert_ne!(old_b, new_b);
    }
}
