CREATE INDEX IF NOT EXISTS sections_record ON sections(record,id);
CREATE INDEX IF NOT EXISTS vectors_order ON vectors(id);
CREATE INDEX IF NOT EXISTS record_source_path ON records(json_extract(CAST(envelope AS TEXT),'$.source_path'));
CREATE INDEX IF NOT EXISTS records_body_hash ON records(body_hash,id);
CREATE TABLE IF NOT EXISTS record_headers(record TEXT PRIMARY KEY,data BLOB,body_bytes INTEGER,body_nonblank INTEGER,body_hash TEXT) WITHOUT ROWID;
CREATE TRIGGER IF NOT EXISTS headers_records_ai AFTER INSERT ON records BEGIN DELETE FROM projections WHERE key='record-headers-v1'; END;
CREATE TRIGGER IF NOT EXISTS headers_records_au AFTER UPDATE ON records BEGIN DELETE FROM projections WHERE key='record-headers-v1'; DELETE FROM record_headers WHERE record=old.id; END;
CREATE TRIGGER IF NOT EXISTS headers_records_ad AFTER DELETE ON records BEGIN DELETE FROM projections WHERE key='record-headers-v1'; DELETE FROM record_headers WHERE record=old.id; END;
CREATE TABLE IF NOT EXISTS record_index_totals(record TEXT PRIMARY KEY,blocks INTEGER,length INTEGER) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS record_basenames(name TEXT,record TEXT,PRIMARY KEY(name,record)) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS record_basename_record ON record_basenames(record);
CREATE TRIGGER IF NOT EXISTS basename_records_ai AFTER INSERT ON records BEGIN DELETE FROM projections WHERE key='basenames-v1'; END;
CREATE TRIGGER IF NOT EXISTS basename_records_au AFTER UPDATE ON records BEGIN DELETE FROM projections WHERE key='basenames-v1'; DELETE FROM record_basenames WHERE record=old.id; END;
CREATE TRIGGER IF NOT EXISTS basename_records_ad AFTER DELETE ON records BEGIN DELETE FROM projections WHERE key='basenames-v1'; DELETE FROM record_basenames WHERE record=old.id; END;
CREATE TABLE IF NOT EXISTS pos_relations(block INTEGER PRIMARY KEY,record TEXT NOT NULL,body_hash TEXT NOT NULL,identity TEXT NOT NULL,generation BLOB NOT NULL,data BLOB NOT NULL);
CREATE INDEX IF NOT EXISTS pos_record ON pos_relations(record,block);
CREATE TABLE IF NOT EXISTS pos_pair_lookup(left_key TEXT,right_key TEXT,block INTEGER,PRIMARY KEY(left_key,right_key,block)) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS pos_pair_block ON pos_pair_lookup(block);
CREATE TRIGGER IF NOT EXISTS pos_pairs_ai AFTER INSERT ON pos_relations BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 INSERT OR IGNORE INTO pos_pair_lookup
 SELECT json_extract(r.value,'$.left'),json_extract(r.value,'$.right'),new.block
 FROM json_each(CAST(new.data AS TEXT)) u,json_each(u.value,'$[2]') r;
END;
CREATE TRIGGER IF NOT EXISTS pos_pairs_ad AFTER DELETE ON pos_relations BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_pair_lookup WHERE block=old.block;
END;
DROP TRIGGER IF EXISTS pos_pairs_au;
CREATE TRIGGER IF NOT EXISTS pos_pairs_au_v2 AFTER UPDATE OF block,record,body_hash,identity,data ON pos_relations BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_pair_lookup WHERE block=old.block;
 INSERT OR IGNORE INTO pos_pair_lookup
 SELECT json_extract(r.value,'$.left'),json_extract(r.value,'$.right'),new.block
 FROM json_each(CAST(new.data AS TEXT)) u,json_each(u.value,'$[2]') r;
END;
DROP TRIGGER IF EXISTS pos_records_ai;
DROP TRIGGER IF EXISTS pos_records_au;
DROP TRIGGER IF EXISTS pos_records_ad;
DROP TRIGGER IF EXISTS pos_sections_ai;
DROP TRIGGER IF EXISTS pos_sections_au;
DROP TRIGGER IF EXISTS pos_sections_ad;
CREATE TRIGGER IF NOT EXISTS pos_records_ai_v2 AFTER INSERT ON records BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE record=new.id;
END;
CREATE TRIGGER IF NOT EXISTS pos_records_au_v2 AFTER UPDATE ON records BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE record=old.id OR record=new.id;
END;
CREATE TRIGGER IF NOT EXISTS pos_records_ad_v2 AFTER DELETE ON records BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE record=old.id;
END;
CREATE TRIGGER IF NOT EXISTS pos_sections_ai_v2 AFTER INSERT ON sections BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE block=new.id;
END;
CREATE TRIGGER IF NOT EXISTS pos_sections_au_v2 AFTER UPDATE ON sections BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE block=old.id OR block=new.id;
END;
CREATE TRIGGER IF NOT EXISTS pos_sections_ad_v2 AFTER DELETE ON sections BEGIN
 DELETE FROM projections WHERE key='pos-pair-lookup-v1';
 DELETE FROM pos_relations WHERE block=old.id;
END;

CREATE TABLE IF NOT EXISTS nav_lexemes(id INTEGER PRIMARY KEY,surface TEXT UNIQUE NOT NULL,norm TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS nav_norm ON nav_lexemes(norm,id);
CREATE TABLE IF NOT EXISTS nav_block_words(word INTEGER,block INTEGER,PRIMARY KEY(word,block)) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS nav_section_words(word INTEGER,section INTEGER,flags INTEGER,PRIMARY KEY(word,section)) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS nav_surface(word INTEGER PRIMARY KEY,df INTEGER,ord INTEGER);
CREATE TABLE IF NOT EXISTS nav_sections(id INTEGER PRIMARY KEY,record TEXT,ordinal INTEGER,data BLOB,own_nonempty INTEGER);
CREATE INDEX IF NOT EXISTS nav_sections_record ON nav_sections(record,ordinal);
CREATE TABLE IF NOT EXISTS nav_blocks(id INTEGER PRIMARY KEY,record TEXT,ordinal INTEGER,heading TEXT,path BLOB,start INTEGER,end INTEGER);
CREATE INDEX IF NOT EXISTS nav_blocks_record ON nav_blocks(record,id);
CREATE TABLE IF NOT EXISTS nav_trees(kind INTEGER,id INTEGER,record TEXT,PRIMARY KEY(kind,id)) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS nav_tree_rows(kind INTEGER,tree INTEGER,block INTEGER,PRIMARY KEY(kind,tree,block)) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS nav_tree_block ON nav_tree_rows(block,kind,tree);
CREATE TABLE IF NOT EXISTS nav_links(source TEXT,target TEXT,PRIMARY KEY(source,target)) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS nav_links_target ON nav_links(target,source);
CREATE TABLE IF NOT EXISTS fallback_artifacts(kind TEXT PRIMARY KEY,generation BLOB,model TEXT,metadata BLOB,rows INTEGER);
CREATE TABLE IF NOT EXISTS fallback_vectors(kind TEXT,part INTEGER,data BLOB,PRIMARY KEY(kind,part)) WITHOUT ROWID;
CREATE TRIGGER IF NOT EXISTS fallback_records_forget AFTER DELETE ON records BEGIN DELETE FROM fallback_vectors; DELETE FROM fallback_artifacts; END;
CREATE TRIGGER IF NOT EXISTS nav_records_ai AFTER INSERT ON records BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TRIGGER IF NOT EXISTS nav_records_ad AFTER DELETE ON records BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TRIGGER IF NOT EXISTS nav_records_au AFTER UPDATE ON records BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TRIGGER IF NOT EXISTS nav_records_forget AFTER DELETE ON records BEGIN
 DELETE FROM nav_block_words; DELETE FROM nav_section_words; DELETE FROM nav_surface;
 DELETE FROM nav_sections; DELETE FROM nav_blocks; DELETE FROM nav_tree_rows;
 DELETE FROM nav_trees; DELETE FROM nav_links; DELETE FROM nav_lexemes; DELETE FROM record_index_totals;
END;
CREATE TRIGGER IF NOT EXISTS nav_sections_ai AFTER INSERT ON sections BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TRIGGER IF NOT EXISTS nav_sections_ad AFTER DELETE ON sections BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TRIGGER IF NOT EXISTS nav_sections_au AFTER UPDATE ON sections BEGIN DELETE FROM projections WHERE key='structure'; END;
CREATE TABLE IF NOT EXISTS beam_terms(term TEXT PRIMARY KEY,ordinal INTEGER NOT NULL) WITHOUT ROWID;
CREATE TRIGGER IF NOT EXISTS beam_terms_ad AFTER DELETE ON records BEGIN DELETE FROM beam_terms; END;
CREATE TABLE IF NOT EXISTS section_means(block INTEGER PRIMARY KEY,ordinal INTEGER NOT NULL,emb BLOB NOT NULL);
CREATE TRIGGER IF NOT EXISTS section_means_ad AFTER DELETE ON sections BEGIN DELETE FROM section_means WHERE block=old.id; END;
CREATE TABLE IF NOT EXISTS record_read_index(record TEXT PRIMARY KEY,ordinal INTEGER UNIQUE,eligible INTEGER NOT NULL,expires TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS record_expiry_totals(expires TEXT PRIMARY KEY,blocks INTEGER NOT NULL,length INTEGER NOT NULL) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS nonempty_records ON record_index_totals(record DESC) WHERE blocks>0;
CREATE INDEX IF NOT EXISTS record_created ON record_headers(coalesce(json_extract(CAST(data AS TEXT),'$.created'),'') DESC,record DESC);
CREATE INDEX IF NOT EXISTS record_kind_created ON record_headers(coalesce(json_extract(CAST(data AS TEXT),'$.kind'),''),coalesce(json_extract(CAST(data AS TEXT),'$.created'),'') DESC,record DESC);
CREATE INDEX IF NOT EXISTS record_expiry_list ON record_read_index(expires,record) WHERE (eligible&1)!=0;
CREATE INDEX IF NOT EXISTS record_parent ON record_headers(json_extract(CAST(data AS TEXT),'$.parent'),record);
CREATE INDEX IF NOT EXISTS record_close_target ON record_headers(json_extract(CAST(data AS TEXT),'$.close.target'),record);
CREATE TRIGGER IF NOT EXISTS catalog_records_ai AFTER INSERT ON records BEGIN DELETE FROM projections WHERE key='record-read-v1'; END;
CREATE TRIGGER IF NOT EXISTS catalog_records_au AFTER UPDATE ON records BEGIN DELETE FROM projections WHERE key='record-read-v1'; END;
CREATE TRIGGER IF NOT EXISTS catalog_records_ad AFTER DELETE ON records BEGIN
 DELETE FROM projections WHERE key='record-read-v1'; DELETE FROM record_read_index; DELETE FROM record_expiry_totals;
END;
CREATE TRIGGER IF NOT EXISTS catalog_sections_ai AFTER INSERT ON sections BEGIN DELETE FROM projections WHERE key='record-read-v1'; END;
CREATE TRIGGER IF NOT EXISTS catalog_sections_au AFTER UPDATE ON sections BEGIN DELETE FROM projections WHERE key='record-read-v1'; END;
CREATE TRIGGER IF NOT EXISTS catalog_sections_ad AFTER DELETE ON sections BEGIN DELETE FROM projections WHERE key='record-read-v1'; END;
