ALTER TABLE records ADD COLUMN writer TEXT GENERATED ALWAYS AS (json_extract(CAST(envelope AS TEXT),'$.writer')) VIRTUAL;
