use super::*;

const SERVICE: &str = "https://mesh.example.test";

#[test]
fn shared_worker_fixtures_have_identical_hashes_signatures_and_membership() {
    #[derive(Deserialize)]
    struct Vector {
        accepted: Acceptance,
        state: Membership,
    }
    let vectors: Vec<Vector> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/mesh-membership-v1.json"
    ))
    .unwrap();
    let mut states = BTreeMap::<String, Membership>::new();
    for vector in vectors {
        let parents: Vec<_> = vector
            .accepted
            .proposal
            .event
            .heads
            .iter()
            .map(|head| states.get(head).unwrap())
            .collect();
        let state = vector.accepted.evaluate(&parents).unwrap();
        assert_eq!(state, vector.state);
        assert_eq!(
            Proposal::decode(&canonical(&vector.accepted.proposal).unwrap()).unwrap(),
            vector.accepted.proposal
        );
        states.insert(state.head.clone(), state);
    }
}

struct TestMesh {
    receipt: SecretKey,
    owner: SecretKey,
    log: Vec<Acceptance>,
    state: Membership,
}
impl TestMesh {
    fn new(receipt: &SecretKey, seed: u8) -> Self {
        let owner = SecretKey::from_bytes(&[seed; 32]);
        let event = Event {
            version: 1,
            service: SERVICE.into(),
            id: hex(&[seed; 32]),
            heads: vec![],
            action: Action::Genesis {
                actor: endpoint(&owner),
                root: hex(&[seed; 32]),
                receipt_key: endpoint(receipt),
            },
        };
        let first =
            Acceptance::sign(Proposal::sign(event, &[&owner]).unwrap(), 1000, receipt).unwrap();
        let state = first.evaluate(&[]).unwrap();
        Self {
            receipt: receipt.clone(),
            owner,
            log: vec![first],
            state,
        }
    }
    fn event(&self, action: Action) -> Event {
        Event {
            version: 1,
            service: SERVICE.into(),
            id: random_id(),
            heads: vec![self.state.head.clone()],
            action,
        }
    }
    fn apply(&mut self, action: Action, keys: &[&SecretKey], time: u64) -> Acceptance {
        let accepted = Acceptance::sign(
            Proposal::sign(self.event(action), keys).unwrap(),
            time,
            &self.receipt,
        )
        .unwrap();
        self.state = accepted.evaluate(&[&self.state]).unwrap();
        self.log.push(accepted.clone());
        accepted
    }
    fn invite(&mut self, key: &SecretKey) -> String {
        let owner = self.owner.clone();
        self.apply(
            Action::Invite {
                actor: endpoint(&owner),
                key: endpoint(key),
                receipt_key: endpoint(&self.receipt),
                not_before: 1000,
                expires: 1600,
            },
            &[&owner],
            1001,
        )
        .proposal
        .hash()
        .unwrap()
    }
    fn redeem(&mut self, invitation: &str, actor: &SecretKey, delegation: &SecretKey) {
        self.apply(
            Action::Redeem {
                actor: endpoint(actor),
                invitation: invitation.into(),
            },
            &[actor, delegation],
            1002,
        );
    }
}

#[test]
fn device_and_service_signatures_are_independently_required() {
    let receipt = SecretKey::generate();
    let mesh = TestMesh::new(&receipt, 1);
    let first = &mesh.log[0];
    let mut forged = first.clone();
    forged.proposal.event.action = Action::Genesis {
        actor: endpoint(&SecretKey::generate()),
        root: hex(&[1; 32]),
        receipt_key: endpoint(&receipt),
    };
    assert!(forged.evaluate(&[]).is_err());
    let mut forged = first.clone();
    forged.receipt.accepted += 1;
    assert!(forged.evaluate(&[]).is_err());
    let mut forged = first.clone();
    forged.signature = hex(&SecretKey::generate().sign(b"wrong").to_bytes());
    assert!(forged.evaluate(&[]).is_err());
    let mut forged = first.clone();
    forged
        .proposal
        .signatures
        .insert(endpoint(&receipt), forged.signature.clone());
    assert!(forged.evaluate(&[]).is_err());
}

#[test]
fn compromised_receipt_signer_cannot_redeem_or_substitute_a_device() {
    let receipt = SecretKey::generate();
    let mut mesh = TestMesh::new(&receipt, 1);
    let delegated = SecretKey::generate();
    let invitation = mesh.invite(&delegated);
    let joining = SecretKey::generate();
    let attacker = SecretKey::generate();
    let event = mesh.event(Action::Redeem {
        actor: endpoint(&attacker),
        invitation: invitation.clone(),
    });
    // Even a valid service receipt cannot replace the delegated signature.
    for keys in [vec![&attacker], vec![&attacker, &receipt]] {
        let forged = Acceptance::sign(
            Proposal::sign(event.clone(), &keys).unwrap(),
            1002,
            &receipt,
        )
        .unwrap();
        assert!(forged.evaluate(&[&mesh.state]).is_err());
    }
    let genuine = Proposal::sign(
        mesh.event(Action::Redeem {
            actor: endpoint(&joining),
            invitation,
        }),
        &[&joining, &delegated],
    )
    .unwrap();
    assert!(
        Acceptance::sign(genuine.clone(), 1002, &receipt)
            .unwrap()
            .evaluate(&[&mesh.state])
            .is_ok()
    );
    let mut substituted = genuine;
    substituted.event.action = event.action;
    substituted.signatures.remove(&endpoint(&joining));
    substituted.signatures.insert(
        endpoint(&attacker),
        hex(&attacker
            .sign(&message(EVENT_DOMAIN, &substituted.event).unwrap())
            .to_bytes()),
    );
    assert!(
        Acceptance::sign(substituted, 1002, &receipt)
            .unwrap()
            .evaluate(&[&mesh.state])
            .is_err()
    );
}

#[test]
fn revoked_sponsor_cannot_redeem_but_prior_admissions_survive() {
    let receipt = SecretKey::generate();
    let mut mesh = TestMesh::new(&receipt, 1);
    let delegated = SecretKey::generate();
    let b = SecretKey::generate();
    let invitation = mesh.invite(&delegated);
    mesh.redeem(&invitation, &b, &delegated);
    let owner = mesh.owner.clone();
    let next = mesh.invite(&SecretKey::generate());
    mesh.apply(
        Action::Revoke {
            actor: endpoint(&b),
            subject: endpoint(&owner),
        },
        &[&b],
        1003,
    );
    assert!(mesh.state.members.contains(&endpoint(&b)));
    assert!(mesh.state.revoked.contains(&endpoint(&owner)));
    let invalid = Proposal::sign(
        mesh.event(Action::Revoke {
            actor: endpoint(&owner),
            subject: endpoint(&b),
        }),
        &[&owner],
    )
    .unwrap();
    assert!(invalid.evaluate(&[&mesh.state], 1004).is_err());
    let invalid = Proposal::sign(
        mesh.event(Action::Redeem {
            actor: endpoint(&SecretKey::generate()),
            invitation: next,
        }),
        &[&owner],
    )
    .unwrap();
    assert!(invalid.evaluate(&[&mesh.state], 1004).is_err());
}

#[test]
fn invitation_expiry_cancellation_and_reuse_never_grant_membership() {
    let receipt = SecretKey::generate();
    let mut mesh = TestMesh::new(&receipt, 1);
    let delegated = SecretKey::generate();
    let b = SecretKey::generate();
    let invitation = mesh.invite(&delegated);
    let join = Proposal::sign(
        mesh.event(Action::Redeem {
            actor: endpoint(&b),
            invitation: invitation.clone(),
        }),
        &[&b, &delegated],
    )
    .unwrap();
    assert!(join.evaluate(&[&mesh.state], 999).is_err());
    assert!(join.evaluate(&[&mesh.state], 1600).is_err());
    // Receipt verification uses acceptance time, never delivery time.
    let accepted = Acceptance::sign(join.clone(), 1599, &receipt).unwrap();
    assert!(accepted.evaluate(&[&mesh.state]).is_ok());
    let owner = mesh.owner.clone();
    let old_state = mesh.state.clone();
    mesh.apply(
        Action::Cancel {
            actor: endpoint(&owner),
            invitation: invitation.clone(),
        },
        &[&owner],
        1003,
    );
    assert!(join.evaluate(&[&mesh.state], 1004).is_err()); // Stale head.
    let join = Proposal::sign(
        mesh.event(Action::Redeem {
            actor: endpoint(&b),
            invitation,
        }),
        &[&b, &delegated],
    )
    .unwrap();
    assert!(join.evaluate(&[&mesh.state], 1004).is_err()); // Current head, closed invitation.
    assert!(accepted.evaluate(&[&old_state]).is_ok()); // Historical proof remains verifiable.
}

#[test]
fn merge_requires_both_heads_and_preserves_revocations() {
    let receipt = SecretKey::generate();
    let mut a = TestMesh::new(&receipt, 1);
    let mut b = TestMesh::new(&receipt, 2);
    let delegated = SecretKey::generate();
    let invitation = a.invite(&delegated);
    let actor = b.owner.clone();
    let mut event = a.event(Action::Redeem {
        actor: endpoint(&actor),
        invitation: invitation.clone(),
    });
    event.heads.push(b.state.head.clone());
    event.heads.sort();
    let proposal = Proposal::sign(event, &[&actor, &delegated]).unwrap();
    let merged = proposal.evaluate(&[&a.state, &b.state], 1002).unwrap();
    assert_eq!(merged.roots.len(), 2);
    assert_eq!(merged.members.len(), 2);
    assert!(proposal.evaluate(&[&a.state], 1002).is_err());
    // Revocation in either input prevents the other component restoring that key.
    let attacker = SecretKey::generate();
    let delegation = SecretKey::generate();
    let invite = b.invite(&delegation);
    b.redeem(&invite, &attacker, &delegation);
    b.apply(
        Action::Revoke {
            actor: endpoint(&attacker),
            subject: endpoint(&actor),
        },
        &[&attacker],
        1003,
    );
    let mut event = proposal.event.clone();
    event.heads = vec![a.state.head.clone(), b.state.head.clone()];
    event.heads.sort();
    assert!(
        Proposal::sign(event, &[&actor, &delegated])
            .unwrap()
            .evaluate(&[&a.state, &b.state], 1004)
            .is_err()
    );
}

#[test]
fn malformed_duplicate_fields_unknown_fields_and_missing_parents_fail() {
    let mesh = TestMesh::new(&SecretKey::generate(), 1);
    let proposal = &mesh.log[0].proposal;
    let bytes = canonical(proposal).unwrap();
    assert_eq!(Proposal::decode(&bytes).unwrap(), *proposal);
    let unknown = String::from_utf8(bytes.clone()).unwrap().replacen(
        "\"event\":{",
        "\"event\":{\"extra\":1,",
        1,
    );
    assert!(Proposal::decode(unknown.as_bytes()).is_err());
    let duplicate = String::from_utf8(bytes).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(Proposal::decode(duplicate.as_bytes()).is_err());
    let owner = mesh.owner.clone();
    let mut event = mesh.event(Action::Revoke {
        actor: endpoint(&owner),
        subject: endpoint(&owner),
    });
    event.heads.push(event.heads[0].clone());
    assert!(
        Proposal::sign(event, &[&owner])
            .unwrap()
            .evaluate(&[&mesh.state], 1003)
            .is_err()
    );
}
