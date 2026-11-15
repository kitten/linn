CREATE TABLE mesh_identity(
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    secret BLOB NOT NULL CHECK(length(secret)=32),
    service TEXT NOT NULL,
    receipt_key TEXT,
    head TEXT,
    enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
    pending_leave INTEGER NOT NULL DEFAULT 0 CHECK(pending_leave IN (0,1)),
    conflict INTEGER NOT NULL DEFAULT 0 CHECK(conflict IN (0,1))
);
CREATE TABLE mesh_events(
    hash TEXT PRIMARY KEY,
    envelope BLOB NOT NULL,
    roots TEXT NOT NULL
);
CREATE TABLE mesh_hints(
    endpoint TEXT PRIMARY KEY,
    envelope BLOB NOT NULL
);
CREATE TABLE mesh_membership_states(
    head TEXT PRIMARY KEY,
    state BLOB NOT NULL
);
CREATE TABLE mesh_successors(
    parent TEXT PRIMARY KEY,
    child TEXT NOT NULL
);
CREATE TABLE mesh_event_ids(
    id TEXT PRIMARY KEY,
    hash TEXT NOT NULL
);
CREATE TABLE mesh_roots(
    root TEXT PRIMARY KEY,
    genesis TEXT NOT NULL
);
CREATE TABLE mesh_conflicts(
    hash TEXT PRIMARY KEY,
    service TEXT NOT NULL,
    roots TEXT NOT NULL,
    envelope BLOB NOT NULL
);
CREATE TABLE mesh_pending(
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    envelope BLOB NOT NULL
);
