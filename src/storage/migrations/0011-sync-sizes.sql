CREATE TABLE sync_sizes(
 ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
 id TEXT NOT NULL UNIQUE,
 hash TEXT,
 scope TEXT,
 frame_bytes INTEGER NOT NULL CHECK(frame_bytes>4),
 envelope_bytes INTEGER NOT NULL CHECK(envelope_bytes>=0)
);

-- This write-only view follows Get's tombstone/staging/published precedence.
-- Size the exact JSON response, including its four-byte length prefix.
CREATE VIEW sync_size_source AS
 SELECT id,envelope_hash AS hash,scope,
  4+length(CAST(json_object('type','response','body',json_object('result','item','item',
   json_object('id',id,'envelope',envelope,'envelope_hash',envelope_hash,'body_hash',body_hash))) AS BLOB)) AS frame_bytes,
  coalesce(length(CAST(envelope AS BLOB)),0) AS envelope_bytes
 FROM (
  SELECT id,NULL AS envelope,NULL AS envelope_hash,NULL AS body_hash,NULL AS scope FROM tombstones
  UNION ALL
  SELECT id,json_extract(item,'$.envelope'),json_extract(item,'$.envelope_hash'),json_extract(item,'$.body_hash'),json_extract(json_extract(item,'$.envelope'),'$.scope')
   FROM sync_incoming WHERE NOT EXISTS(SELECT 1 FROM tombstones t WHERE t.id=sync_incoming.id)
  UNION ALL
  SELECT id,CAST(envelope AS TEXT),envelope_hash,body_hash,original_scope FROM records
   WHERE NOT EXISTS(SELECT 1 FROM tombstones t WHERE t.id=records.id)
    AND NOT EXISTS(SELECT 1 FROM sync_incoming s WHERE s.id=records.id)
 );

INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source;

CREATE TRIGGER sync_sizes_records_insert AFTER INSERT ON records BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=NEW.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
END;
CREATE TRIGGER sync_sizes_records_update AFTER UPDATE OF id,envelope,envelope_hash,body_hash,original_scope ON records BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id IN(OLD.id,NEW.id)
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;
CREATE TRIGGER sync_sizes_records_delete AFTER DELETE ON records BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=OLD.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;
CREATE TRIGGER sync_sizes_incoming_insert AFTER INSERT ON sync_incoming BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=NEW.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
END;
CREATE TRIGGER sync_sizes_incoming_update AFTER UPDATE ON sync_incoming BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id IN(OLD.id,NEW.id)
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;
CREATE TRIGGER sync_sizes_incoming_delete AFTER DELETE ON sync_incoming BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=OLD.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;
CREATE TRIGGER sync_sizes_tombstones_insert AFTER INSERT ON tombstones BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=NEW.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
END;
CREATE TRIGGER sync_sizes_tombstones_update AFTER UPDATE ON tombstones BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id IN(OLD.id,NEW.id)
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;
CREATE TRIGGER sync_sizes_tombstones_delete AFTER DELETE ON tombstones BEGIN
 INSERT INTO sync_sizes(id,hash,scope,frame_bytes,envelope_bytes) SELECT * FROM sync_size_source WHERE id=OLD.id
 ON CONFLICT(id) DO UPDATE SET hash=excluded.hash,scope=excluded.scope,frame_bytes=excluded.frame_bytes,envelope_bytes=excluded.envelope_bytes;
 DELETE FROM sync_sizes WHERE id=OLD.id AND NOT EXISTS(SELECT 1 FROM sync_size_source WHERE id=OLD.id);
END;

CREATE INDEX sync_sizes_records ON sync_sizes(id) WHERE hash IS NOT NULL;
CREATE INDEX sync_sizes_tombstones ON sync_sizes(id) WHERE hash IS NULL;
CREATE INDEX sync_sizes_scope ON sync_sizes(scope COLLATE NOCASE,id) WHERE hash IS NOT NULL;
