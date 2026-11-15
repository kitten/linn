-- Membership snapshots are derived caches, not immutable history.
DELETE FROM mesh_membership_states WHERE head IN (SELECT parent FROM mesh_successors);
-- Components only merge: one representative root identifies an event's ancestry.
UPDATE mesh_events SET roots=json_array(json_extract(roots,'$[0]'));
CREATE INDEX mesh_history_root ON mesh_events(json_extract(roots,'$[0]'));
