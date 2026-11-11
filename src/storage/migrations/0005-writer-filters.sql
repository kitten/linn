CREATE INDEX records_writer_created ON records(writer,coalesce(json_extract(CAST(envelope AS TEXT),'$.created'),'') DESC,id DESC);
CREATE INDEX records_writer_expiry ON records(writer,coalesce(json_extract(CAST(envelope AS TEXT),'$.stale_after'),''),id);
