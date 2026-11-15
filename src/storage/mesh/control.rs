//! Durable control state shares the replica's schema-fenced connection.
use super::*;
use crate::mesh::failure::Kind;
use crate::mesh::membership::{
    Acceptance, Action, Membership, Proposal, canonical, endpoint, unhex,
};

// Only current leaves are cached. Historical states are rebuilt from immutable
// receipts when needed (for an old invitation or conflicting branch), never
// copied into permanent storage again. Iterative replay avoids recursive depth.
async fn state(tx: &libsql::Connection, head: &str) -> Result<Option<Membership>> {
    enum Work {
        Load(String),
        Apply(Box<Acceptance>),
    }
    let mut work = vec![Work::Load(head.into())];
    let mut values: Vec<Membership> = Vec::new();
    let mut visiting = std::collections::BTreeSet::new();
    while let Some(step) = work.pop() {
        match step {
            Work::Load(hash) => {
                let mut rows = tx
                    .query(
                        "SELECT state FROM mesh_membership_states WHERE head=?",
                        [hash.as_str()],
                    )
                    .await?;
                if let Some(row) = rows.next().await? {
                    values.push(serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?);
                    continue;
                }
                drop(rows);
                if !visiting.insert(hash.clone()) {
                    return Err(Error::Invalid("cyclic stored membership history".into()));
                }
                let mut rows = tx
                    .query(
                        "SELECT envelope FROM mesh_events WHERE hash=?",
                        [hash.as_str()],
                    )
                    .await?;
                let Some(row) = rows.next().await? else {
                    if hash == head {
                        return Ok(None);
                    }
                    return Err(Error::Invalid(
                        "stored membership predecessor is missing".into(),
                    ));
                };
                let event: Acceptance = serde_json::from_slice(&row.get::<Vec<u8>>(0)?)?;
                if event.proposal.hash()? != hash {
                    return Err(Error::Invalid("stored membership hash changed".into()));
                }
                let parents = event.proposal.event.heads.clone();
                work.push(Work::Apply(Box::new(event)));
                work.extend(parents.into_iter().rev().map(Work::Load));
            }
            Work::Apply(event) => {
                let parents = values.split_off(
                    values
                        .len()
                        .checked_sub(event.proposal.event.heads.len())
                        .ok_or_else(|| {
                            Error::Invalid("stored membership predecessor is missing".into())
                        })?,
                );
                let verified = event.evaluate(&parents.iter().collect::<Vec<_>>())?;
                visiting.remove(&verified.head);
                values.push(verified);
            }
        }
    }
    Ok(values.pop())
}

impl Replica {
    pub(crate) async fn mesh_known_event(&self, hash: &str) -> Result<bool> {
        let mut rows = self
            .db
            .query("SELECT 1 FROM mesh_events WHERE hash=?", [hash])
            .await?;
        Ok(rows.next().await?.is_some())
    }
    pub async fn mesh_disable(&self) -> Result<()> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        tx.execute("UPDATE mesh_identity SET enabled=0 WHERE singleton=1", ())
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn mesh_authorize(&self, peer: &str) -> Result<()> {
        let identity = self
            .mesh_identity()
            .await?
            .ok_or_else(|| Error::Invalid("mesh is not enrolled".into()))?;
        if identity.conflict {
            return Err(Kind::Conflict.error("membership history is conflicted"));
        }
        if !identity.enabled || identity.pending_leave {
            return Err(Kind::Authentication.error("mesh transport authorization is disabled"));
        }
        let state = self
            .mesh_membership(None)
            .await?
            .ok_or_else(|| Error::Invalid("missing verified membership".into()))?;
        if !state.members.contains(peer) || !state.members.contains(&endpoint(&identity.key)) {
            return Err(Kind::Authentication.error("peer is not an authorized mesh member"));
        }
        Ok(())
    }

    pub async fn mesh_authorize_chunk(
        &self,
        peer: &str,
        id: &str,
        hash: Option<&str>,
    ) -> Result<()> {
        self.mesh_authorize(peer).await?;
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = match hash {
            Some(hash) => tx.query("SELECT 1 FROM records WHERE id=? AND envelope_hash=? AND NOT EXISTS(SELECT 1 FROM tombstones WHERE id=?)", params![id, hash, id]).await?,
            None => tx.query("SELECT 1 FROM tombstones WHERE id=?", [id]).await?,
        };
        if rows.next().await?.is_none() {
            return Err(Error::Invalid(
                "source record changed or was forgotten; outgoing frame cancelled".into(),
            ));
        }
        Ok(())
    }
    pub async fn mesh_pin_receipt_key(&self, key: &str) -> Result<()> {
        private_store(&self.path)?;
        unhex::<32>(key)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query(
                "SELECT receipt_key FROM mesh_identity WHERE singleton=1",
                (),
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("mesh identity is not initialized".into()))?;
        if row.get::<Option<String>>(0)?.is_some_and(|old| old != key) {
            return Err(Error::Invalid(
                "mesh service receipt key changed; explicit reprovisioning is required".into(),
            ));
        }
        drop(rows);
        tx.execute(
            "UPDATE mesh_identity SET receipt_key=? WHERE singleton=1",
            [key],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mesh_membership(&self, head: Option<&str>) -> Result<Option<Membership>> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let owned;
        let head = match head {
            Some(head) => head,
            None => {
                let mut rows = tx
                    .query("SELECT head FROM mesh_identity WHERE singleton=1", ())
                    .await?;
                owned = match rows.next().await? {
                    Some(row) => row.get::<Option<String>>(0)?,
                    None => None,
                };
                let Some(head) = owned.as_deref() else {
                    return Ok(None);
                };
                head
            }
        };
        state(&tx, head).await
    }

    pub async fn mesh_pending(&self) -> Result<Option<Proposal>> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut rows = tx
            .query("SELECT envelope FROM mesh_pending WHERE singleton=1", ())
            .await?;
        rows.next()
            .await?
            .map(|row| Proposal::decode(&row.get::<Vec<u8>>(0)?))
            .transpose()
    }

    /// Persist before issuing a mutation so an uncertain reply can retry exact bytes.
    pub async fn mesh_prepare(&self, proposal: &Proposal) -> Result<()> {
        private_store(&self.path)?;
        proposal.verify_signatures()?;
        let bytes = canonical(proposal)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query("SELECT envelope FROM mesh_pending WHERE singleton=1", ())
            .await?;
        if let Some(row) = rows.next().await? {
            if row.get::<Vec<u8>>(0)? != bytes {
                return Err(Error::Invalid(
                    "a mesh proposal has an uncertain outcome; resolve it before preparing another"
                        .into(),
                ));
            }
        } else {
            tx.execute(
                "INSERT INTO mesh_pending(singleton,envelope) VALUES(1,?)",
                [bytes],
            )
            .await?;
        }
        drop(rows);
        tx.commit().await?;
        Ok(())
    }

    /// Only after a definite non-acceptance, never after a timeout or lost response.
    pub async fn mesh_reject_pending(&self, proposal: &Proposal) -> Result<()> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        tx.execute(
            "DELETE FROM mesh_pending WHERE singleton=1 AND envelope=?",
            [canonical(proposal)?],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mesh_accept(&self, accepted: &Acceptance) -> Result<Membership> {
        if let Some(state) = self.mesh_accept_inner(accepted).await? {
            return Ok(state);
        }
        self.mesh_membership(Some(&accepted.proposal.hash()?))
            .await?
            .ok_or_else(|| Error::Invalid("missing verified membership state".into()))
    }

    /// Duplicate proof pages acknowledge their immutable hash without replaying
    /// old states. Only callers requesting a historical state pay for rebuilding it.
    pub async fn mesh_accept_event(&self, accepted: &Acceptance) -> Result<String> {
        self.mesh_accept_inner(accepted).await?;
        accepted.proposal.hash()
    }

    async fn mesh_accept_inner(&self, accepted: &Acceptance) -> Result<Option<Membership>> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx.query("SELECT secret,service,receipt_key,head,conflict FROM mesh_identity WHERE singleton=1", ()).await?;
        let identity = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("mesh identity is not initialized".into()))?;
        let secret: Vec<u8> = identity.get(0)?;
        let local = endpoint(&SecretKey::from_bytes(
            &secret
                .try_into()
                .map_err(|_| Error::Invalid("corrupt mesh identity".into()))?,
        ));
        let service: String = identity.get(1)?;
        let receipt_key: Option<String> = identity.get(2)?;
        let current: Option<String> = identity.get(3)?;
        if identity.get::<i64>(4)? != 0 {
            return Err(Kind::Conflict.error(
                "mesh membership history is conflicted; network authorization is suspended",
            ));
        }
        drop(rows);
        let hash = accepted.proposal.hash()?;
        let bytes = canonical(accepted)?;
        let mut rows = tx
            .query(
                "SELECT envelope FROM mesh_events WHERE hash=?",
                [hash.as_str()],
            )
            .await?;
        if let Some(row) = rows.next().await? {
            if row.get::<Vec<u8>>(0)? != bytes {
                return Err(Kind::Conflict.error("accepted event bytes changed"));
            }
            drop(rows);
            tx.execute(
                "DELETE FROM mesh_pending WHERE singleton=1 AND envelope=?",
                [canonical(&accepted.proposal)?],
            )
            .await?;
            tx.commit().await?;
            return Ok(None);
        }
        drop(rows);
        let mut parents = Vec::new();
        for head in &accepted.proposal.event.heads {
            parents.push(
                state(&tx, head)
                    .await?
                    .ok_or_else(|| Error::Invalid("mesh proof is missing a predecessor".into()))?,
            );
        }
        let verified = accepted.evaluate(&parents.iter().collect::<Vec<_>>())?;
        if verified.service != service || receipt_key.as_deref() != Some(&verified.receipt_key) {
            return Err(
                Kind::Authentication.error("mesh proof does not match the pinned service identity")
            );
        }
        let mut fork = false;
        let mut rows = tx
            .query(
                "SELECT hash FROM mesh_event_ids WHERE id=?",
                [accepted.proposal.event.id.as_str()],
            )
            .await?;
        if let Some(row) = rows.next().await? {
            fork |= row.get::<String>(0)? != hash;
        }
        drop(rows);
        for head in &accepted.proposal.event.heads {
            let mut rows = tx
                .query(
                    "SELECT child FROM mesh_successors WHERE parent=?",
                    [head.as_str()],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                fork |= row.get::<String>(0)? != hash;
            }
        }
        if let Action::Genesis { root, .. } = &accepted.proposal.event.action {
            let mut rows = tx
                .query(
                    "SELECT genesis FROM mesh_roots WHERE root=?",
                    [root.as_str()],
                )
                .await?;
            if let Some(row) = rows.next().await? {
                fork |= row.get::<String>(0)? != hash;
            }
        }
        if fork {
            tx.execute(
                "INSERT OR IGNORE INTO mesh_conflicts(hash,service,roots,envelope) VALUES(?,?,?,?)",
                params![
                    hash,
                    service.clone(),
                    serde_json::to_string(&verified.roots)?,
                    bytes
                ],
            )
            .await?;
            let affects_local = match current.as_deref() {
                Some(head) => state(&tx, head)
                    .await?
                    .is_some_and(|state| !state.roots.is_disjoint(&verified.roots)),
                None => false,
            };
            if affects_local {
                tx.execute(
                    "UPDATE mesh_identity SET conflict=1,enabled=0 WHERE singleton=1",
                    (),
                )
                .await?;
            }
            tx.commit().await?;
            return Err(Kind::Conflict.error("incompatible accepted mesh histories; authorization is suspended for the affected component"));
        }
        // A merge must not import a component already known to have forked.
        let mut rows = tx
            .query(
                "SELECT roots FROM mesh_conflicts WHERE service=?",
                [service.as_str()],
            )
            .await?;
        while let Some(row) = rows.next().await? {
            let roots: std::collections::BTreeSet<String> =
                serde_json::from_str(&row.get::<String>(0)?)?;
            if !roots.is_disjoint(&verified.roots) {
                return Err(Kind::Conflict.error("mesh proof includes a conflicted component"));
            }
        }
        drop(rows);
        tx.execute(
            "INSERT INTO mesh_events(hash,envelope,roots) VALUES(?,?,?)",
            params![
                hash.clone(),
                bytes,
                serde_json::to_string(&[verified
                    .roots
                    .first()
                    .ok_or_else(|| Error::Invalid("missing membership root".into()))?])?
            ],
        )
        .await?;
        tx.execute(
            "INSERT INTO mesh_event_ids(id,hash) VALUES(?,?)",
            params![accepted.proposal.event.id.clone(), hash.clone()],
        )
        .await?;
        tx.execute(
            "INSERT INTO mesh_membership_states(head,state) VALUES(?,?)",
            params![hash.clone(), canonical(&verified)?],
        )
        .await?;
        for parent in &accepted.proposal.event.heads {
            tx.execute(
                "DELETE FROM mesh_membership_states WHERE head=?",
                [parent.as_str()],
            )
            .await?;
            tx.execute(
                "INSERT INTO mesh_successors(parent,child) VALUES(?,?)",
                params![parent.clone(), hash.clone()],
            )
            .await?;
        }
        if let Action::Genesis { root, .. } = &accepted.proposal.event.action {
            tx.execute(
                "INSERT INTO mesh_roots(root,genesis) VALUES(?,?)",
                params![root.clone(), hash.clone()],
            )
            .await?;
        }
        let advances = current
            .as_ref()
            .is_some_and(|head| accepted.proposal.event.heads.contains(head));
        let enrolls = current.is_none()
            && accepted.proposal.event.action.actor() == local
            && matches!(
                accepted.proposal.event.action,
                Action::Genesis { .. } | Action::Redeem { .. }
            );
        if advances || enrolls {
            let active = verified.members.contains(&local);
            tx.execute("UPDATE mesh_identity SET head=?,enabled=CASE WHEN ? THEN enabled ELSE 0 END,pending_leave=CASE WHEN ? THEN pending_leave ELSE 0 END WHERE singleton=1", params![hash, active, active]).await?;
        }
        tx.execute(
            "DELETE FROM mesh_pending WHERE singleton=1 AND envelope=?",
            [canonical(&accepted.proposal)?],
        )
        .await?;
        tx.commit().await?;
        Ok(Some(verified))
    }

    pub async fn mesh_enable(&self, head: &str) -> Result<()> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let verified = state(&tx, head)
            .await?
            .ok_or_else(|| Error::Invalid("membership head is not verified".into()))?;
        let mut rows = tx
            .query(
                "SELECT secret,head,pending_leave,conflict FROM mesh_identity WHERE singleton=1",
                (),
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| Error::Invalid("mesh identity is not initialized".into()))?;
        let secret: Vec<u8> = row.get(0)?;
        let local = endpoint(&SecretKey::from_bytes(
            &secret
                .try_into()
                .map_err(|_| Error::Invalid("corrupt mesh identity".into()))?,
        ));
        if row.get::<Option<String>>(1)?.as_deref() != Some(head)
            || row.get::<i64>(2)? != 0
            || row.get::<i64>(3)? != 0
            || !verified.members.contains(&local)
        {
            return Err(Error::Invalid(
                "mesh transport cannot enable for this membership state".into(),
            ));
        }
        drop(rows);
        tx.execute("UPDATE mesh_identity SET enabled=1 WHERE singleton=1", ())
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mesh_leave_local(&self) -> Result<()> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Immediate).await?;
        let mut rows = tx
            .query(
                "SELECT secret,head FROM mesh_identity WHERE singleton=1",
                (),
            )
            .await?;
        let mut pending = true;
        if let Some(row) = rows.next().await? {
            let secret: Vec<u8> = row.get(0)?;
            let local = endpoint(&SecretKey::from_bytes(
                &secret
                    .try_into()
                    .map_err(|_| Error::Invalid("corrupt mesh identity".into()))?,
            ));
            if let Some(head) = row.get::<Option<String>>(1)? {
                pending = state(&tx, &head)
                    .await?
                    .is_none_or(|state| state.members.contains(&local));
            }
        }
        drop(rows);
        tx.execute(
            "UPDATE mesh_identity SET enabled=0,pending_leave=? WHERE singleton=1",
            [pending],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn mesh_history(&self, after: i64) -> Result<Vec<(i64, Acceptance)>> {
        private_store(&self.path)?;
        let tx = self.transaction(TransactionBehavior::Deferred).await?;
        let mut identity = tx
            .query("SELECT head FROM mesh_identity WHERE singleton=1", ())
            .await?;
        let head = match identity.next().await? {
            Some(row) => row.get::<Option<String>>(0)?,
            None => None,
        };
        drop(identity);
        let Some(head) = head else {
            return Ok(Vec::new());
        };
        let roots = state(&tx, &head)
            .await?
            .ok_or_else(|| Error::Invalid("missing verified mesh head".into()))?
            .roots;
        let mut events = Vec::new();
        for root in roots {
            let mut rows = tx.query(
                "SELECT rowid,envelope FROM mesh_events WHERE json_extract(roots,'$[0]')=? AND rowid>? ORDER BY rowid LIMIT 16",
                params![root, after]).await?;
            while let Some(row) = rows.next().await? {
                events.push((
                    row.get::<i64>(0)?,
                    serde_json::from_slice::<Acceptance>(&row.get::<Vec<u8>>(1)?)?,
                ));
            }
            events.sort_by_key(|(cursor, _)| *cursor);
            events.truncate(16);
        }
        events.sort_by_key(|(cursor, _)| *cursor);
        events.truncate(16);
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::membership::{Event, hex, random_id};
    use std::os::unix::fs::PermissionsExt;

    fn vectors() -> Vec<Acceptance> {
        let values: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/mesh-membership-v1.json"
        ))
        .unwrap();
        values
            .into_iter()
            .map(|value| serde_json::from_value(value["accepted"].clone()).unwrap())
            .collect()
    }

    async fn replica() -> (tempfile::TempDir, Replica) {
        let root = tempfile::tempdir().unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let replica = Replica::open_private_mesh(&root.path().join("store.db"))
            .await
            .unwrap();
        replica
            .mesh_create_identity("https://mesh.example.test")
            .await
            .unwrap();
        // The shared public fixtures use deterministic *test* endpoint seed 1.
        replica
            .db
            .execute("UPDATE mesh_identity SET secret=?", [vec![1u8; 32]])
            .await
            .unwrap();
        replica
            .mesh_pin_receipt_key(&endpoint(&SecretKey::from_bytes(&[9; 32])))
            .await
            .unwrap();
        (root, replica)
    }

    #[tokio::test]
    async fn accepted_history_pending_retries_and_revocation_survive_reopen() {
        let (root, replica) = replica().await;
        let events = vectors();
        replica.mesh_prepare(&events[0].proposal).await.unwrap();
        assert!(replica.mesh_pending().await.unwrap().is_some());
        drop(replica);
        let replica = Replica::open_private_mesh(&root.path().join("store.db"))
            .await
            .unwrap();
        assert_eq!(
            replica.mesh_pending().await.unwrap().unwrap(),
            events[0].proposal
        );
        let first = replica.mesh_accept(&events[0]).await.unwrap();
        assert!(replica.mesh_pending().await.unwrap().is_none());
        replica.mesh_enable(&first.head).await.unwrap();
        assert!(replica.mesh_identity().await.unwrap().unwrap().enabled);
        // Idempotent acceptance must also clear a retried pending proposal.
        replica.mesh_prepare(&events[0].proposal).await.unwrap();
        replica.mesh_accept(&events[0]).await.unwrap();
        assert!(replica.mesh_pending().await.unwrap().is_none());
        for event in &events[1..] {
            replica.mesh_accept(event).await.unwrap();
        }
        let identity = replica.mesh_identity().await.unwrap().unwrap();
        assert!(!identity.enabled);
        assert_eq!(
            identity.head.as_deref(),
            Some(events.last().unwrap().proposal.hash().unwrap().as_str())
        );
        assert!(replica.mesh_enable(&first.head).await.is_err());
        // A late historical retry cannot roll the current head back.
        replica.mesh_accept(&events[0]).await.unwrap();
        assert_eq!(
            replica.mesh_identity().await.unwrap().unwrap().head,
            identity.head
        );
        assert_eq!(replica.mesh_history(0).await.unwrap().len(), events.len());
        assert!(
            replica
                .mesh_pin_receipt_key(&endpoint(&SecretKey::generate()))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn leaf_caches_migrate_and_historical_proofs_remain_exact() {
        let (root, replica) = replica().await;
        let events = vectors();
        let mut states = std::collections::BTreeMap::new();
        for event in &events {
            let parents = event
                .proposal
                .event
                .heads
                .iter()
                .map(|h| &states[h])
                .collect::<Vec<_>>();
            let state = event.evaluate(&parents).unwrap();
            replica.mesh_accept_event(event).await.unwrap();
            states.insert(state.head.clone(), state);
        }
        // Recreate schema 7's per-event caches, then perform the actual migration.
        for state in states.values() {
            replica
                .db
                .execute(
                    "INSERT OR REPLACE INTO mesh_membership_states VALUES(?,?)",
                    params![state.head.clone(), canonical(state).unwrap()],
                )
                .await
                .unwrap();
        }
        replica
            .db
            .execute("DROP INDEX mesh_history_root", ())
            .await
            .unwrap();
        replica
            .db
            .execute_batch("DROP TABLE sync_incoming; PRAGMA user_version=7;")
            .await
            .unwrap();
        drop(replica);
        let replica = Replica::open_private_mesh(&root.path().join("store.db"))
            .await
            .unwrap();
        for (head, expected) in states {
            assert_eq!(
                replica.mesh_membership(Some(&head)).await.unwrap().unwrap(),
                expected
            );
        }
        for event in &events {
            replica.mesh_accept_event(event).await.unwrap();
        }
        let mut rows = replica
            .db
            .query("SELECT count(*) FROM mesh_membership_states", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            1
        );
        assert_eq!(
            replica
                .mesh_history(0)
                .await
                .unwrap()
                .into_iter()
                .map(|(_, e)| e)
                .collect::<Vec<_>>(),
            events
        );
    }

    #[tokio::test]
    async fn expired_invitation_history_uses_linear_persistent_cache_space() {
        let (_root, replica) = replica().await;
        let receipt = SecretKey::from_bytes(&[9; 32]);
        let owner = SecretKey::from_bytes(&[1; 32]);
        let genesis = vectors().remove(0);
        let mut head = replica.mesh_accept_event(&genesis).await.unwrap();
        for i in 0..320 {
            let now = 1000 + i * 601;
            let event = Event {
                version: 1,
                id: random_id(),
                service: "https://mesh.example.test".into(),
                heads: vec![head],
                action: Action::Invite {
                    actor: endpoint(&owner),
                    key: endpoint(&SecretKey::generate()),
                    receipt_key: endpoint(&receipt),
                    not_before: now,
                    expires: now + 600,
                },
            };
            let accepted =
                Acceptance::sign(Proposal::sign(event, &[&owner]).unwrap(), now, &receipt).unwrap();
            head = replica.mesh_accept_event(&accepted).await.unwrap();
        }
        let mut rows = replica.db.query("SELECT count(*),sum(length(state)),(SELECT sum(length(envelope)) FROM mesh_events) FROM mesh_membership_states", ()).await.unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<i64>(0).unwrap(), 1);
        assert!(row.get::<i64>(1).unwrap() < row.get::<i64>(2).unwrap());
        assert_eq!(
            replica
                .mesh_membership(Some(&head))
                .await
                .unwrap()
                .unwrap()
                .invitations
                .len(),
            320
        );
    }

    #[tokio::test]
    async fn only_a_valid_accepted_fork_suspends_the_component_and_preserves_evidence() {
        let (root, replica) = replica().await;
        let events = vectors();
        let genesis = replica.mesh_accept(&events[0]).await.unwrap();
        replica.mesh_enable(&genesis.head).await.unwrap();
        replica.mesh_accept(&events[1]).await.unwrap();
        let owner = SecretKey::from_bytes(&[1; 32]);
        let service = SecretKey::from_bytes(&[9; 32]);
        let event = Event {
            version: 1,
            id: random_id(),
            service: genesis.service.clone(),
            heads: vec![genesis.head],
            action: Action::Revoke {
                actor: endpoint(&owner),
                subject: endpoint(&owner),
            },
        };
        let accepted =
            Acceptance::sign(Proposal::sign(event, &[&owner]).unwrap(), 1002, &service).unwrap();
        let mut tampered = accepted.clone();
        tampered.signature = hex(&[0; 64]);
        assert!(replica.mesh_accept(&tampered).await.is_err());
        assert!(!replica.mesh_identity().await.unwrap().unwrap().conflict);
        assert!(replica.mesh_accept(&accepted).await.is_err());
        let identity = replica.mesh_identity().await.unwrap().unwrap();
        assert!(identity.conflict);
        assert!(!identity.enabled);
        assert!(replica.mesh_accept(&events[2]).await.is_err());
        drop(replica);
        let replica = Replica::open_private_mesh(&root.path().join("store.db"))
            .await
            .unwrap();
        assert!(replica.mesh_identity().await.unwrap().unwrap().conflict);
        let mut rows = replica
            .db
            .query("SELECT envelope FROM mesh_conflicts", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<Vec<u8>>(0)
                .unwrap(),
            canonical(&accepted).unwrap()
        );
        drop(rows);
        replica.mesh_reset_identity().await.unwrap();
        let first = &events[0];
        replica
            .mesh_create_identity(&first.proposal.event.service)
            .await
            .unwrap();
        let Action::Genesis { receipt_key, .. } = &first.proposal.event.action else {
            panic!("genesis fixture required")
        };
        replica.mesh_pin_receipt_key(receipt_key).await.unwrap();
        assert!(replica.mesh_accept(first).await.is_err());
    }

    #[tokio::test]
    async fn offline_leave_stays_disabled_and_identity_reset_preserves_original_deletions() {
        let (root, replica) = replica().await;
        let first = replica.mesh_accept(&vectors()[0]).await.unwrap();
        replica.mesh_enable(&first.head).await.unwrap();
        replica.mesh_leave_local().await.unwrap();
        assert!(replica.mesh_enable(&first.head).await.is_err());
        drop(replica);
        let replica = Replica::open_private_mesh(&root.path().join("store.db"))
            .await
            .unwrap();
        let identity = replica.mesh_identity().await.unwrap().unwrap();
        assert!(identity.pending_leave);
        assert!(!identity.enabled);
        replica
            .apply(
                &crate::storage::sync::Item {
                    id: "deleted-original".into(),
                    envelope: None,
                    envelope_hash: None,
                    body_hash: None,
                },
                None,
            )
            .await
            .unwrap();
        replica.mesh_reset_identity().await.unwrap();
        assert!(replica.mesh_identity().await.unwrap().is_none());
        assert!(replica.mesh_history(0).await.unwrap().is_empty());
        assert_eq!(
            replica.inventory(true, None, None).await.unwrap()[0].id,
            "deleted-original"
        );
    }
}
