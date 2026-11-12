//! Indexed metadata browsing; independent of retrieval.
//! @ref LLP 0004#linn-list
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListQuery {
    pub now: String,
    pub kind: Option<String>,
    #[serde(default)]
    pub writer: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    pub since: Option<String>,
    pub expires_before: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListedRecord {
    pub record: String,
    pub label: String,
    pub kind: String,
    pub created: String,
    pub stale_after: Option<String>,
}

pub struct RecordPage {
    pub records: Vec<ListedRecord>,
    pub next: Option<(String, String)>,
    pub publication: Vec<u8>,
}

impl Store {
    pub async fn list_records(
        &self,
        query: &ListQuery,
        after: Option<&(String, String)>,
        limit: usize,
        publication: Option<&[u8]>,
    ) -> Result<RecordPage> {
        self.ensure_usable()?;
        if !(1..=100).contains(&limit)
            || query
                .kind
                .as_deref()
                .is_some_and(|k| !matches!(k, "note" | "doc"))
        {
            return Err(Error::Invalid("invalid listing options".into()));
        }
        let snapshot = self.db.transaction().await?;
        let result = async {
            let mut rows = self
                .db
                .query("SELECT value FROM projections WHERE key='structure'", ())
                .await?;
            let stamp: Vec<u8> = rows
                .next()
                .await?
                .ok_or_else(|| Error::Invalid("missing publication".into()))?
                .get(0)?;
            if stamp != self.structure_stamp || publication.is_some_and(|p| p != stamp) {
                return Err(Error::Invalid("store changed; start a new list".into()));
            }
            drop(rows);
            let expiry = query.expires_before.is_some();
            let mut records = Vec::new();
            // Two scalar seeks let SQLite use both expression and record-id keys.
            let ranges: &[bool] = if after.is_some() {
                &[true, false]
            } else {
                &[false]
            };
            for same_key in ranges {
                let (sql, values) =
                    list_statement(query, after, limit + 1 - records.len(), *same_key);
                let mut rows = self.db.query(&sql, values).await?;
                while let Some(row) = rows.next().await? {
                    let expires: String = row.get(4)?;
                    records.push(ListedRecord {
                        record: row.get(0)?,
                        label: row.get(1)?,
                        kind: row.get(2)?,
                        created: row.get(3)?,
                        stale_after: (!expires.is_empty()).then_some(expires),
                    });
                }
                if records.len() > limit {
                    break;
                }
            }
            let more = records.len() > limit;
            records.truncate(limit);
            let next = more.then(|| {
                let last = records.last().unwrap();
                (
                    if expiry {
                        last.stale_after.clone().unwrap()
                    } else {
                        last.created.clone()
                    },
                    last.record.clone(),
                )
            });
            Ok(RecordPage {
                records,
                next,
                publication: stamp,
            })
        }
        .await;
        snapshot.rollback().await?;
        result
    }
}

fn list_statement(
    query: &ListQuery,
    after: Option<&(String, String)>,
    limit: usize,
    same_key: bool,
) -> (String, Vec<libsql::Value>) {
    let created = if query.writer.is_some() {
        "coalesce(json_extract(CAST(w.envelope AS TEXT),'$.created'),'')"
    } else {
        "coalesce(json_extract(CAST(h.data AS TEXT),'$.created'),'')"
    };
    let kind = "coalesce(json_extract(CAST(h.data AS TEXT),'$.kind'),'')";
    let expiry = query.expires_before.is_some();
    let expiry_key = if query.writer.is_some() {
        "coalesce(json_extract(CAST(w.envelope AS TEXT),'$.stale_after'),'')"
    } else {
        "r.expires"
    };
    let from = if query.writer.is_some() {
        if expiry {
            "records w INDEXED BY records_writer_expiry CROSS JOIN record_headers h ON h.record=w.id CROSS JOIN record_read_index r ON r.record=w.id"
        } else {
            "records w INDEXED BY records_writer_created CROSS JOIN record_headers h ON h.record=w.id CROSS JOIN record_read_index r ON r.record=w.id"
        }
    } else if expiry {
        "record_read_index r INDEXED BY record_expiry_list CROSS JOIN record_headers h ON h.record=r.record"
    } else if query.kind.is_some() {
        "record_headers h INDEXED BY record_kind_created CROSS JOIN record_read_index r ON r.record=h.record"
    } else {
        "record_headers h INDEXED BY record_created CROSS JOIN record_read_index r ON r.record=h.record"
    };
    let mut sql = format!(
        "SELECT h.record,coalesce(json_extract(CAST(h.data AS TEXT),'$.label'),''),{kind},{created},r.expires FROM {from} WHERE (r.eligible&1)!=0"
    );
    let mut values: Vec<libsql::Value> = Vec::new();
    if let Some(writer) = &query.writer {
        sql.push_str(" AND w.writer=?");
        values.push(writer.clone().into());
    }
    if let Some(scope) = &query.scope {
        sql.push_str(" AND EXISTS (SELECT 1 FROM records scoped WHERE scoped.id=h.record AND scoped.original_scope=? COLLATE NOCASE)");
        values.push(scope.clone().into());
    }
    if expiry {
        sql.push_str(&format!(" AND {expiry_key}>? AND {expiry_key}<=?"));
        values.push(query.now.clone().into());
        values.push(query.expires_before.clone().unwrap().into());
    } else {
        sql.push_str(" AND (r.expires='' OR r.expires>?)");
        values.push(query.now.clone().into());
    }
    if let Some(value) = &query.kind {
        sql.push_str(&format!(" AND {kind}=?"));
        values.push(value.clone().into());
    }
    if let Some(value) = &query.since {
        sql.push_str(&format!(" AND {created}>=?"));
        values.push(value.clone().into());
    }
    let key = if expiry { expiry_key } else { created };
    let id = if query.writer.is_some() {
        "w.id"
    } else if expiry {
        "r.record"
    } else {
        "h.record"
    };
    let operator = if expiry { ">" } else { "<" };
    if let Some((time, record)) = after {
        if same_key {
            sql.push_str(&format!(" AND {key}=? AND {id}{operator}?"));
            values.extend([time.clone().into(), record.clone().into()]);
        } else {
            sql.push_str(&format!(" AND {key}{operator}?"));
            values.push(time.clone().into());
        }
    }
    let direction = if expiry { "ASC" } else { "DESC" };
    if same_key {
        sql.push_str(&format!(" ORDER BY {id} {direction} LIMIT ?"));
    } else {
        sql.push_str(&format!(
            " ORDER BY {key} {direction},{id} {direction} LIMIT ?"
        ));
    }
    values.push((limit as i64).into());

    (sql, values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scope_filters_original_scope_before_pagination() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scope.db");
        let records = [
            serde_json::json!({"id":"a","body":"Alpha","scope":"Owner/Repo"}),
            serde_json::json!({"id":"b","body":"Beta","scope":"owner/repo"}),
            serde_json::json!({"id":"c","body":"Other","scope":"other/repo"}),
            serde_json::json!({"id":"d","body":"Unscoped"}),
        ]
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap())
        .collect();
        drop(Store::build(&path, records).await.unwrap());
        let store = Store::open(&path).await.unwrap();
        let query = ListQuery {
            now: "2026-11-12T00:00:00Z".into(),
            kind: None,
            writer: None,
            scope: Some("OWNER/REPO".into()),
            since: None,
            expires_before: None,
        };
        let first = store.list_records(&query, None, 1, None).await.unwrap();
        assert_eq!(first.records[0].record, "b");
        let next = store
            .list_records(&query, first.next.as_ref(), 1, Some(&first.publication))
            .await
            .unwrap();
        assert_eq!(next.records[0].record, "a");
        assert!(next.next.is_none());
        assert!(store.records.is_empty());
    }

    #[tokio::test]
    async fn indexed_pages_filter_lifecycle_and_stay_metadata_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("listing.db");
        let records = [
            serde_json::json!({"id":"a","kind":"note","label":"A","body":"Original amber","created":"2026-11-01T00:00:00Z","stale_after":"2026-11-12T00:00:00Z"}),
            serde_json::json!({"id":"b","kind":"note","body":"Original cobalt","created":"2026-11-01T00:00:00Z","stale_after":"2026-11-12T00:00:00Z"}),
            serde_json::json!({"id":"c","kind":"doc","body":"Original copper","created":"2026-11-02T00:00:00Z"}),
            serde_json::json!({"id":"unknown","kind":"note","body":"Original timeless"}),
            serde_json::json!({"id":"expired","kind":"note","body":"Expired","created":"2026-11-03T00:00:00Z","stale_after":"2026-11-11T00:00:00Z"}),
            serde_json::json!({"id":"candidate","kind":"note","body":"Candidate","state":"candidate","created":"2026-11-03T00:00:00Z"}),
            serde_json::json!({"id":"parent","kind":"note","body":"Parent","created":"2026-11-03T00:00:00Z"}),
            serde_json::json!({"id":"child","kind":"note","body":"Child","parent":"parent","created":"2026-11-02T00:00:00Z"}),
            serde_json::json!({"id":"closed","kind":"note","body":"Closed","created":"2026-11-03T00:00:00Z"}),
            serde_json::json!({"id":"close","body":"","close":{"target":"closed","reason":"correct"}}),
        ].into_iter().map(|v| serde_json::from_value(v).unwrap()).collect();
        drop(Store::build(&path, records).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        assert!(store.records.is_empty());
        let mut query = ListQuery {
            now: "2026-11-11T00:00:00Z".into(),
            kind: None,
            writer: None,
            scope: None,
            since: None,
            expires_before: None,
        };
        let first = store.list_records(&query, None, 2, None).await.unwrap();
        assert_eq!(
            first
                .records
                .iter()
                .map(|r| r.record.as_str())
                .collect::<Vec<_>>(),
            ["child", "c"]
        );
        let second = store
            .list_records(&query, first.next.as_ref(), 2, Some(&first.publication))
            .await
            .unwrap();
        assert_eq!(
            second
                .records
                .iter()
                .map(|r| r.record.as_str())
                .collect::<Vec<_>>(),
            ["b", "a"]
        );
        let third = store
            .list_records(&query, second.next.as_ref(), 2, Some(&first.publication))
            .await
            .unwrap();
        assert_eq!(third.records[0].record, "unknown");
        assert!(third.next.is_none());

        query.kind = Some("note".into());
        query.since = Some("2026-11-01T00:00:00Z".into());
        let notes = store.list_records(&query, None, 20, None).await.unwrap();
        assert_eq!(
            notes
                .records
                .iter()
                .map(|r| r.record.as_str())
                .collect::<Vec<_>>(),
            ["child", "b", "a"]
        );
        query.expires_before = Some("2026-11-12T00:00:00Z".into());
        let soon = store.list_records(&query, None, 1, None).await.unwrap();
        assert_eq!(soon.records[0].record, "a");
        let next = store
            .list_records(&query, soon.next.as_ref(), 1, Some(&soon.publication))
            .await
            .unwrap();
        assert_eq!(next.records[0].record, "b");
        assert!(next.next.is_none());
        assert!(store.records.is_empty());
        assert!(store.rows.is_empty());
        assert!(store.encoder.is_none());

        for (kind, expiry, expected) in [
            (None, None, "record_created"),
            (Some("note".to_string()), None, "record_kind_created"),
            (
                None,
                Some("2026-12-01T00:00:00Z".to_string()),
                "record_expiry_list",
            ),
        ] {
            query.kind = kind;
            query.since = None;
            query.expires_before = expiry;
            for same_key in [true, false] {
                let (sql, values) = list_statement(
                    &query,
                    Some(&("2026-11-12T00:00:00Z".into(), "b".into())),
                    2,
                    same_key,
                );
                let mut rows = store
                    .db
                    .query(&format!("EXPLAIN QUERY PLAN {sql}"), values)
                    .await
                    .unwrap();
                let mut plan = String::new();
                while let Some(row) = rows.next().await.unwrap() {
                    plan.push_str(&row.get::<String>(3).unwrap());
                    plan.push('\n');
                }
                assert!(plan.contains(expected), "{plan}");
                assert!(
                    !plan.contains("SCAN h") && !plan.contains("SCAN r"),
                    "{plan}"
                );
                assert!(!plan.contains("TEMP B-TREE"), "{plan}");
            }
        }
        store.forget("a").await.unwrap();
        assert!(
            store
                .list_records(&query, first.next.as_ref(), 2, Some(&first.publication))
                .await
                .is_err()
        );
        assert!(
            !store
                .list_records(&query, None, 20, None)
                .await
                .unwrap()
                .records
                .iter()
                .any(|r| r.record == "a")
        );
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;

    #[tokio::test]
    async fn writer_indexes_seek_lists_and_search_before_materialization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("writers.db");
        let records = [
            serde_json::json!({"id":"a","body":"Routing marker alpha.","writer":"a@example.test","created":"2026-11-10T00:00:00Z","stale_after":"2026-12-01T00:00:00Z"}),
            serde_json::json!({"id":"b","body":"Routing marker beta.","writer":"a@example.test","created":"2026-11-10T00:00:00Z","stale_after":"2026-12-01T00:00:00Z"}),
            serde_json::json!({"id":"other","body":"Routing marker gamma.","writer":"b@example.test"}),
            serde_json::json!({"id":"unknown","body":"Routing marker delta.","writer":null}),
            serde_json::json!({"id":"legacy","body":"Routing marker epsilon."}),
        ].into_iter().map(|v|serde_json::from_value(v).unwrap()).collect();
        drop(Store::build(&path, records).await.unwrap());
        let mut store = Store::open(&path).await.unwrap();
        let mut query = ListQuery {
            now: "2026-11-11T00:00:00Z".into(),
            kind: None,
            writer: Some("a@example.test".into()),
            scope: None,
            since: None,
            expires_before: None,
        };
        for expiry in [false, true] {
            query.expires_before = expiry.then(|| "2027-01-01T00:00:00Z".into());
            let first = store.list_records(&query, None, 1, None).await.unwrap();
            let second = store
                .list_records(&query, first.next.as_ref(), 1, Some(&first.publication))
                .await
                .unwrap();
            assert_eq!(first.records[0].record, if expiry { "a" } else { "b" });
            assert_eq!(second.records[0].record, if expiry { "b" } else { "a" });
            assert!(second.next.is_none());
            for after in [None, first.next.as_ref()] {
                for same in [false, true] {
                    if after.is_none() && same {
                        continue;
                    }
                    let (sql, values) = list_statement(&query, after, 2, same);
                    let mut rows = store
                        .db
                        .query(&format!("EXPLAIN QUERY PLAN {sql}"), values)
                        .await
                        .unwrap();
                    let mut plan = String::new();
                    while let Some(row) = rows.next().await.unwrap() {
                        plan.push_str(&row.get::<String>(3).unwrap());
                        plan.push('\n');
                    }
                    assert!(
                        plan.contains(if expiry {
                            "records_writer_expiry"
                        } else {
                            "records_writer_created"
                        }),
                        "{plan}"
                    );
                    assert!(plan.contains("writer=?"), "{plan}");
                    assert!(
                        !plan.contains("SCAN ") && !plan.contains("TEMP B-TREE"),
                        "{plan}"
                    );
                }
            }
        }
        assert!(store.records.is_empty());
        let q = crate::retrieval::query::Query::parse("routing writer:a@example.test").unwrap();
        for stem in [false, true] {
            let (sql, values) =
                super::super::writer_fts_statement(stem, "routing", "a@example.test");
            let mut rows = store
                .db
                .query(&format!("EXPLAIN QUERY PLAN {sql}"), values)
                .await
                .unwrap();
            let mut plan = String::new();
            while let Some(row) = rows.next().await.unwrap() {
                plan.push_str(&row.get::<String>(3).unwrap());
                plan.push('\n');
            }
            assert!(
                plan.contains("records_writer_created") && plan.contains("writer=?"),
                "{plan}"
            );
            assert!(
                !plan.contains("SCAN w") && !plan.contains("SCAN b"),
                "{plan}"
            );
            let hits = store.writer_fts_ids(stem, "routing", &q).await.unwrap();
            // The Porter index stores "rout", so a raw routing term need not occur there.
            if !stem {
                assert_eq!(hits.len(), 2);
            }
            assert!(store.records.is_empty());
        }
        query.writer = Some("A@example.test".into());
        assert!(
            store
                .list_records(&query, None, 20, None)
                .await
                .unwrap()
                .records
                .is_empty()
        );
    }
}
