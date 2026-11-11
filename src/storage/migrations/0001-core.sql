CREATE TABLE IF NOT EXISTS metadata(version INTEGER NOT NULL);
INSERT INTO metadata SELECT 1 WHERE NOT EXISTS(SELECT 1 FROM metadata);
CREATE TABLE IF NOT EXISTS records(id TEXT PRIMARY KEY,scope TEXT CHECK(scope IS NULL),original_scope TEXT,state TEXT,body BLOB NOT NULL,body_hash TEXT NOT NULL,envelope BLOB NOT NULL,envelope_hash TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS tombstones(id TEXT PRIMARY KEY);
CREATE TABLE IF NOT EXISTS projections(key TEXT PRIMARY KEY,value BLOB NOT NULL);
CREATE TABLE IF NOT EXISTS sections(id INTEGER PRIMARY KEY,record TEXT NOT NULL,heading TEXT NOT NULL,body TEXT NOT NULL,parts TEXT NOT NULL,start INTEGER,end INTEGER);
CREATE INDEX IF NOT EXISTS sections_record ON sections(record,id);
CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(heading,body,parts,content='sections',content_rowid='id',tokenize="unicode61 tokenchars '_-'");
CREATE VIRTUAL TABLE IF NOT EXISTS stem USING fts5(text,tokenize="porter unicode61 tokenchars '_-'");
CREATE VIRTUAL TABLE IF NOT EXISTS vocab USING fts5vocab(fts,'instance');
CREATE VIRTUAL TABLE IF NOT EXISTS stem_vocab USING fts5vocab(stem,'instance');
CREATE TABLE IF NOT EXISTS vectors(id INTEGER PRIMARY KEY,record TEXT,emb F32_BLOB(512));
CREATE INDEX IF NOT EXISTS vectors_order ON vectors(id);
CREATE TRIGGER IF NOT EXISTS sections_ai AFTER INSERT ON sections BEGIN
 INSERT INTO fts(rowid,heading,body,parts) VALUES(new.id,new.heading,new.body,new.parts);
END;
CREATE TRIGGER IF NOT EXISTS sections_ad BEFORE DELETE ON sections BEGIN
 INSERT INTO fts(fts,rowid,heading,body,parts) VALUES('delete',old.id,old.heading,old.body,old.parts);
 DELETE FROM stem WHERE rowid=old.id; DELETE FROM vectors WHERE id=old.id;
END;
INSERT INTO fts(fts,rank) VALUES('secure-delete',1);
INSERT INTO stem(stem,rank) VALUES('secure-delete',1);
