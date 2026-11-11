CREATE TABLE sync_remotes(name TEXT PRIMARY KEY,origin TEXT NOT NULL,scope TEXT);
CREATE TABLE sync_state(key TEXT PRIMARY KEY,value INTEGER NOT NULL);
CREATE INDEX sync_scope_records ON records(original_scope COLLATE NOCASE,id);
