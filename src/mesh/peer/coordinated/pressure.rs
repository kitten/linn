//! Per-edge reopening stays separate from the discovery route ledger.
use super::*;
const IDLE_RELEASE: u32 = 0x4d00;
pub(super) fn was_idle_release(connection: &Connection) -> bool {
    matches!(connection.close_reason(), Some(iroh::endpoint::ConnectionError::ApplicationClosed(close)) if close.error_code.into_inner() == u64::from(IDLE_RELEASE))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Purpose {
    Expansion,
    Data { round: u8, upward: bool },
    Terminal,
}
impl Purpose {
    pub(super) fn of(envelope: &Envelope, local: &str) -> Result<Self> {
        Ok(match &envelope.command {
            Command::Contact { .. } | Command::Detach { .. } => Self::Expansion,
            Command::Pull { round, child, .. } => Self::Data {
                round: *round,
                upward: child.is_some(),
            },
            Command::Expose { round, readers } => {
                let parent = if envelope.route.len() > 1 {
                    &envelope.route[envelope.route.len() - 2]
                } else {
                    local
                };
                Self::Data {
                    round: *round,
                    upward: readers.iter().any(|reader| reader == parent),
                }
            }
            Command::Stop => Self::Terminal,
        })
    }
    fn index(self) -> Result<usize> {
        match self {
            Self::Expansion => Ok(0),
            Self::Data {
                round: round @ 1..=2,
                upward,
            } => Ok(1 + usize::from(round - 1) * 2 + usize::from(!upward)),
            Self::Terminal => Ok(5),
            _ => Err(protocol("unsupported reopening round")),
        }
    }
}
#[derive(Default)]
pub(super) struct Budget([bool; 6]);
impl Budget {
    pub(super) fn issue(&mut self, purpose: Purpose) -> Result<()> {
        let used = &mut self.0[purpose.index()?];
        if *used {
            return Err(Kind::Capacity.error("recorded edge reopening allowance exhausted"));
        }
        *used = true;
        Ok(())
    }
}

pub(super) struct Edge {
    session: tokio::sync::Mutex<Option<Arc<Session>>>,
    budget: Arc<Mutex<Budget>>,
    active: AtomicUsize,
    touched: Mutex<tokio::time::Instant>,
}
pub(super) struct Lease {
    edge: Arc<Edge>,
    session: Option<Arc<Session>>,
    owner: Weak<Peer>,
}
impl std::ops::Deref for Lease {
    type Target = Session;
    fn deref(&self) -> &Session {
        self.session.as_ref().unwrap()
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        *self.edge.touched.lock().unwrap() = tokio::time::Instant::now();
        self.edge.active.fetch_sub(1, Ordering::AcqRel);
        if let Some(peer) = self.owner.upgrade() {
            peer.connection_idle.notify_waiters();
        }
    }
}
impl Edge {
    pub(super) fn new(session: Arc<Session>, budget: Arc<Mutex<Budget>>) -> Arc<Self> {
        Arc::new(Self {
            session: tokio::sync::Mutex::new(Some(session)),
            budget,
            active: AtomicUsize::new(0),
            touched: Mutex::new(tokio::time::Instant::now()),
        })
    }
    pub(super) async fn acquire(
        self: &Arc<Self>,
        node: &Node,
        target: &str,
        purpose: Purpose,
    ) -> Result<Lease> {
        self.active.fetch_add(1, Ordering::AcqRel);
        let mut lease = Lease {
            edge: self.clone(),
            session: None,
            owner: node.owner.clone(),
        };
        let mut session = self.session.lock().await;
        if session.is_none() {
            self.budget.lock().unwrap().issue(purpose)?;
            *session = Some(node.reopen(target, purpose).await?);
        }
        lease.session = session.clone();
        Ok(lease)
    }
    pub(super) fn release_idle(&self) -> bool {
        if self.active.load(Ordering::Acquire) != 0 {
            return false;
        }
        let Ok(mut session) = self.session.try_lock() else {
            return false;
        };
        if self.active.load(Ordering::Acquire) != 0 {
            return false;
        }
        if let Some(session) = session.take() {
            session
                .link
                .connection
                .close(IDLE_RELEASE.into(), b"idle recorded edge released");

            true
        } else {
            false
        }
    }
    pub(super) fn touched(&self) -> tokio::time::Instant {
        *self.touched.lock().unwrap()
    }
}

impl Edge {
    pub(super) async fn cancel(&self, id: u64) {
        if let Some(session) = &*self.session.lock().await {
            let _ = session.cancel.send(id);
        }
    }
}

pub(super) async fn reopen(node: &Node, target: &str, purpose: Purpose) -> Result<Arc<Session>> {
    let peer = node.peer()?;
    let job = &node.job;
    wire::metrics::reopened(&peer.local, purpose.index()?);
    let link = peer
        .connect_with_routes(
            target,
            Routes {
                direct: true,
                relay: true,
            },
            &startup::DialState::default(),
            Kind::Timeout,
        )
        .await?;
    let deadline = link.admission_deadline;
    let admission = async {
        let (mut send, mut recv) = link.connection.open_bi().await.map_err(Kind::connection)?;
        peer.handshake_client(&mut send, &mut recv, target).await?;
        let mut control = Control::new(send, recv);
        control.track_coordinated(peer.progress(&job.run.id).unwrap_or_default(), true);
        control
            .send(Frame::Begin {
                job: job.clone(),
                replaces_parent: None,
                continuation: true,
                terminal_only: purpose == Purpose::Terminal,
            })
            .await?;
        let Frame::Accepted {
            free,
            parent,
            duplicate: true,
            ..
        } = control.receive().await?
        else {
            return Err(protocol("recorded edge continuation expected"));
        };
        if parent.as_ref() != Some(&peer.local) {
            return Err(Kind::Authentication.error("recorded edge changed its parent"));
        }
        node.capacity.record(target, free);
        Ok(Session::new(
            link,
            control,
            node.capacity.clone(),
            job.run.cohort.clone(),
        ))
    };
    tokio::time::timeout_at(deadline, admission)
        .await
        .map_err(|_| Kind::Timeout.error("recorded edge admission timed out"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_allowance_is_independent_single_and_control_only() {
        let mut budget = Budget::default();
        for round in 1..=2 {
            for upward in [true, false] {
                let purpose = Purpose::Data { round, upward };
                budget.issue(purpose).unwrap();
                assert!(budget.issue(purpose).is_err());
            }
        }
        budget.issue(Purpose::Expansion).unwrap();
        budget.issue(Purpose::Terminal).unwrap();
        assert!(budget.issue(Purpose::Terminal).is_err());
        assert!(budget.issue(Purpose::Expansion).is_err());
        assert!(
            budget
                .issue(Purpose::Data {
                    round: 3,
                    upward: false
                })
                .is_err()
        );
        assert!(matches!(
            Purpose::of(
                &Envelope {
                    id: 1,
                    route: vec![],
                    command: Command::Stop
                },
                "root"
            )
            .unwrap(),
            Purpose::Terminal
        ));
        assert!(matches!(
            Purpose::of(
                &Envelope {
                    id: 2,
                    route: vec!["child".into()],
                    command: Command::Pull {
                        round: 2,
                        child: None,
                        delta: false,
                    }
                },
                "root"
            )
            .unwrap(),
            Purpose::Data {
                round: 2,
                upward: false
            }
        ));
    }
    #[test]
    fn routed_source_exposure_has_the_same_direction_at_each_hop() {
        let command = Command::Expose {
            round: 2,
            readers: vec!["parent".into()],
        };
        for (local, route) in [
            ("root", vec!["parent".into(), "child".into()]),
            ("parent", vec!["child".into()]),
        ] {
            assert_eq!(
                Purpose::of(
                    &Envelope {
                        id: 1,
                        route,
                        command: command.clone()
                    },
                    local
                )
                .unwrap(),
                Purpose::Data {
                    round: 2,
                    upward: true
                }
            );
        }
    }
}
