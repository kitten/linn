//! Typed application operations, independent of CLI and transport.
//! @ref LLP 0007#application-boundary
pub(crate) mod input;
pub(crate) mod listing;
use crate::{
    error::{Error, Result},
    retrieval::query::Query,
    storage::{
        Call, Card, Context, Store,
        record::{Record, hash},
        sections,
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
pub use input::{FileInput, InputTask};
pub use listing::ListOptions;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};
#[derive(Serialize, Deserialize)]
pub struct Caller {
    #[serde(default)]
    pub diagnostics: crate::diagnostics::Diagnostics,
    pub scope: Option<String>,
    pub writer: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct AddOptions {
    pub label: Option<String>,
    pub input_name: Option<String>,
    pub input: InputTask,
    pub parent: Option<String>,
    pub stale_after: Option<String>,
    pub unscoped: bool,
}
#[derive(Serialize, Deserialize)]
pub struct ExportOptions {
    pub scope: Option<String>,
    pub kind: Option<String>,
    pub since: Option<String>,
    pub id: Vec<String>,
    pub all_states: bool,
}
#[derive(Serialize, Deserialize)]
pub enum Operation {
    Search {
        query: String,
    },
    More {
        cursor: String,
    },
    Show {
        reference: String,
        all: bool,
    },
    Outline {
        reference: String,
    },
    Add(AddOptions),
    Close {
        reference: String,
        reason: String,
        replacement: Option<String>,
    },
    Forget {
        reference: String,
    },
    List(ListOptions),
    Export(ExportOptions),
    Import,
    Sync {
        environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    },
    SyncHello {
        version: u32,
    },
    Check,
    Rebuild,
    Pair(crate::storage::sync::Remote),
    RemoveRemote {
        name: String,
    },
    Remotes,
    SyncFrame(crate::sync::protocol::Request),
}
impl Operation {
    pub(crate) fn takes_input(&self) -> bool {
        matches!(self, Self::Add(_) | Self::Import)
    }
    pub fn mutates(&self) -> bool {
        match self {
            Self::Add(_)
            | Self::Close { .. }
            | Self::Forget { .. }
            | Self::Sync { .. }
            | Self::Import
            | Self::Rebuild
            | Self::Pair(_)
            | Self::RemoveRemote { .. } => true,
            Self::SyncFrame(request) => match request {
                crate::sync::protocol::Request::Apply { .. }
                | crate::sync::protocol::Request::Finish => true,
                crate::sync::protocol::Request::Hello { .. }
                | crate::sync::protocol::Request::Inventory { .. }
                | crate::sync::protocol::Request::Get { .. }
                | crate::sync::protocol::Request::Needs { .. } => false,
            },
            Self::Search { .. }
            | Self::More { .. }
            | Self::Show { .. }
            | Self::Outline { .. }
            | Self::List(_)
            | Self::Export(_)
            | Self::SyncHello { .. }
            | Self::Check
            | Self::Remotes => false,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct RecordLabel {
    pub label: String,
    pub kind: String,
    pub body_hash: String,
}
impl RecordLabel {
    pub fn body_hash(&self) -> &str {
        &self.body_hash
    }
}
#[derive(Serialize, Deserialize)]
pub struct SearchPage {
    pub call: Option<Call>,
    pub records: BTreeMap<String, RecordLabel>,
    pub number: usize,
    pub cursor: Option<String>,
    pub more: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Shown {
    pub fetch: String,
    pub record: String,
    pub label: String,
    pub heading: String,
    pub text: String,
    pub truncated: bool,
    pub characters: usize,
    pub superseded_by: Option<String>,
    pub reason: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub enum Value {
    Search(SearchPage),
    Show(Shown),
    Outline(Vec<Card>),
    OutlineNotCurrent,
    List {
        records: Vec<crate::storage::listing::ListedRecord>,
        next: Option<String>,
    },
    Added(String),
    Forgotten(String),
    Checked,
    Rebuilt,
    Exported,
    Imported(crate::storage::bundle::Imported),
    Synced(crate::sync::Counts),
    Paired(String),
    Removed(String),
    Remotes(Vec<crate::storage::sync::Remote>),
    SyncFrame(crate::sync::protocol::Response),
}
pub struct Reply {
    pub value: Value,
    pub warnings: Vec<String>,
    pub output: Option<File>,
}
impl From<Value> for Reply {
    fn from(value: Value) -> Self {
        Self {
            value,
            warnings: Vec::new(),
            output: None,
        }
    }
}
pub struct Application {
    path: PathBuf,
    store: Option<Store>,
    replica: Option<crate::storage::sync::Replica>,
}
impl Application {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            store: None,
            replica: None,
        }
    }
    async fn store(&mut self, rebuild: bool) -> Result<&mut Store> {
        if let Some(s) = &self.store
            && s.verify_source_version(&s.db).await.is_err()
        {
            self.store = None;
        }
        if rebuild {
            self.store = None;
        }
        if self.store.is_none() {
            let started = std::time::Instant::now();
            self.store = Some(if rebuild {
                Store::open_for_rebuild(&self.path).await?
            } else {
                Store::open(&self.path).await?
            });
            if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
                crate::diagnostics::emit(format!(
                    "linn application: {}",
                    json!({"phase":"store-open","ms":started.elapsed().as_secs_f64()*1000.})
                ));
            }
        }
        Ok(self.store.as_mut().unwrap())
    }
    async fn replica(&mut self) -> Result<&crate::storage::sync::Replica> {
        if self.replica.is_none() {
            let started = std::time::Instant::now();
            self.replica = Some(crate::storage::sync::Replica::open(&self.path).await?);
            if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
                crate::diagnostics::emit(format!(
                    "linn application: {}",
                    json!({
                        "phase":"replica-open", "ms":started.elapsed().as_secs_f64()*1000.
                    })
                ));
            }
        }
        Ok(self.replica.as_ref().unwrap())
    }
    pub async fn execute(
        &mut self,
        op: Operation,
        caller: Caller,
        input: impl Into<Option<File>>,
    ) -> Result<Reply> {
        let mut input = input.into();
        let flags = caller.diagnostics.clone();
        let mutates = op.mutates();
        let (result, lines) = crate::diagnostics::capture(flags, async {
            let started = std::time::Instant::now();
            let result = self.execute_inner(op, caller, &mut input).await;
            if crate::diagnostics::enabled("LINN_STARTUP_TIMINGS") {
                crate::diagnostics::emit(format!(
                    "linn application: {}",
                    json!({"phase":"operation","ms":started.elapsed().as_secs_f64()*1000.})
                ));
            }
            result
        })
        .await;
        match result {
            Ok(mut reply) => {
                reply.warnings.extend(lines);
                Ok(reply)
            }
            Err(error) => {
                self.store = None;
                self.replica = None;
                let mut error = crate::error::ApplicationError::new(error, mutates);
                error.diagnostics = lines;
                Err(error.into())
            }
        }
    }
    async fn execute_inner(
        &mut self,
        op: Operation,
        caller: Caller,
        input: &mut Option<File>,
    ) -> Result<Reply> {
        use Operation::*;
        if op.takes_input() && input.is_none() {
            return Err(Error::Invalid("missing operation input".into()));
        }
        let ctx = Context {
            scope: caller.scope,
            now: now(),
            ..Context::default()
        };
        match op {
            SyncHello { version } => {
                if version != 1 {
                    return Err(Error::Invalid("incompatible sync protocol".into()));
                }
                return Ok(
                    Value::SyncFrame(crate::sync::protocol::Response::Hello { version: 1 }).into(),
                );
            }
            Sync { environment } => {
                self.store = None;
                let replica = self.replica().await?;
                return Ok(Value::Synced(synchronize(replica, Some(&environment)).await?).into());
            }
            Import => {
                self.store = None;
                let staged = crate::storage::bundle::stage(&mut BufReader::new(
                    input
                        .as_mut()
                        .ok_or_else(|| Error::Invalid("missing import input".into()))?,
                ))?;
                let replica = self.replica().await?;
                return Ok(Value::Imported(
                    crate::storage::bundle::import_bundle(replica, staged).await?,
                )
                .into());
            }
            Pair(_) | RemoveRemote { .. } | Remotes | SyncFrame(_) => {
                if op.mutates() {
                    self.store = None;
                }
                let replica = self.replica().await?;
                let value = match op {
                    Pair(remote) => {
                        if remote.name.is_empty()
                            || !remote
                                .name
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                            || remote.scope.as_ref().is_some_and(|s| s.trim().is_empty())
                        {
                            return Err(Error::Invalid(
                                "invalid remote name or empty scope".into(),
                            ));
                        }
                        crate::sync::ssh::Origin::parse(&remote.origin)?;
                        replica.pair(&remote).await?;
                        Value::Paired(remote.name)
                    }
                    RemoveRemote { name } => {
                        replica.remove(&name).await?;
                        Value::Removed(name)
                    }
                    Remotes => Value::Remotes(replica.remotes().await?),
                    SyncFrame(frame) => {
                        Value::SyncFrame(crate::sync::protocol::handle(replica, frame).await?)
                    }
                    _ => unreachable!(),
                };
                return Ok(value.into());
            }
            _ => {}
        }
        let prepared = if let Add(options) = &op {
            Some(input::prepare(&options.input)?)
        } else {
            None
        };
        let store = self.store(matches!(op, Rebuild)).await?;
        let snapshot = if matches!(
            op,
            Search { .. } | More { .. } | Show { .. } | Outline { .. }
        ) {
            Some(store.application_snapshot().await?)
        } else {
            None
        };
        store.set_query_clock(&ctx.now);
        let result = execute_store(store, op, caller.writer, input, ctx, prepared).await;
        if let Some(snapshot) = snapshot {
            snapshot.rollback().await?;
        }
        result
    }
}
async fn execute_store(
    store: &mut Store,
    op: Operation,
    writer: Option<String>,
    input: &mut Option<File>,
    ctx: Context,
    prepared: Option<input::Prepared>,
) -> Result<Reply> {
    use Operation::*;
    let scope = ctx.scope.clone();
    let value = match op {
        Search { query } => Value::Search(search(store, &query, &ctx, 0).await?),
        More { cursor: token } => {
            let bytes = URL_SAFE_NO_PAD
                .decode(
                    token
                        .strip_prefix("s1.")
                        .ok_or_else(|| Error::Invalid("invalid cursor".into()))?,
                )
                .map_err(|_| Error::Invalid("invalid cursor".into()))?;
            let (q, scope, next, now): (String, Option<String>, usize, String) =
                serde_json::from_slice(&bytes)?;
            Value::Search(search(store, &q, &Context { scope, now, ..ctx }, next).await?)
        }
        List(options) => {
            let (records, next) = listing::run(store, options, &ctx.now).await?;
            Value::List { records, next }
        }
        Export(options) => {
            let since = listing::since(options.since.as_deref(), &ctx.now)?;
            let mut ids = Vec::new();
            for reference in options.id {
                ids.push(store.export_reference(&reference, scope.as_deref()).await?);
            }
            let selection = crate::storage::bundle::Selection {
                scope: options.scope,
                kind: options.kind,
                since,
                ids,
                all_states: options.all_states,
                now: ctx.now,
            };
            let mut output = tempfile::tempfile()?;
            store.export_bundle(&selection, &mut output).await?;
            output.seek(SeekFrom::Start(0))?;
            return Ok(Reply {
                value: Value::Exported,
                warnings: Vec::new(),
                output: Some(output),
            });
        }
        Show { reference, all } => {
            let rid = store
                .resolve_metadata(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.load_record_sections(&rid).await?;
            let lifecycle = store.lifecycle_references(&rid).await?;
            let (record, span) = store.resolve(&reference, scope.as_deref())?;
            let fetch = format!("{}:v1:{}:{}", record.body_hash(), span[0], span[1]);
            if !store.validity.eligible(record, &ctx.now) {
                let child = lifecycle.iter().map(|id| &store.records[id]).find(|r| {
                    r.state == "accepted"
                        && r.close.is_none()
                        && r.parent.as_ref() == Some(&record.id)
                });
                let close = lifecycle.iter().map(|id| &store.records[id]).find(|r| {
                    r.state == "accepted" && r.close.as_ref().is_some_and(|c| c.target == record.id)
                });
                let reason = if let Some(child) = child {
                    format!("superseded by {}", child.id)
                } else if let Some(close) = close {
                    format!("closed by {}", close.id)
                } else {
                    "expired or not accepted".into()
                };
                return Ok(Value::Show(Shown {
                    fetch,
                    record: record.id.clone(),
                    label: record.label.clone(),
                    heading: String::new(),
                    text: String::new(),
                    truncated: false,
                    characters: 0,
                    superseded_by: child.map(|r| r.id.clone()),
                    reason: Some(reason),
                })
                .into());
            }
            let (shown, characters) = store.display_span(&rid, span, all).await?;
            let truncated = !all && characters > 1500;
            let heading = store.sections[&record.id]
                .iter()
                .find(|s| s.span.start <= span[0] && span[0] < s.span.end)
                .map(|s| s.path.join(" > "))
                .unwrap_or_default();
            Value::Show(Shown {
                fetch,
                record: record.id.clone(),
                label: record.label.clone(),
                heading,
                text: shown,
                truncated,
                characters,
                superseded_by: None,
                reason: None,
            })
        }
        Outline { reference } => {
            let rid = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.load_record_sections(&rid).await?;
            let (record, _) = store.resolve(&reference, scope.as_deref())?;
            if !store.validity.eligible(record, &ctx.now) {
                return Ok(Value::OutlineNotCurrent.into());
            }
            let sections = &store.sections[&record.id];
            let top = sections
                .iter()
                .filter(|s| s.level > 0)
                .map(|s| s.level)
                .min()
                .unwrap_or(1);
            let mut cards = Vec::new();
            for sec in sections
                .iter()
                .filter(|s| s.level <= top + 1 && !s.span.is_empty())
                .take(20)
            {
                let span = if sec.span.len() > 1500 {
                    sections::hint_spans(record.body.as_bytes(), sec.span.clone())[0].clone()
                } else {
                    sec.span.clone()
                };
                let text = String::from_utf8_lossy(&record.body.as_bytes()[span.clone()]);
                let excerpt = text
                    .lines()
                    .find(|s| !s.trim().is_empty() && !s.trim_start().starts_with('#'))
                    .unwrap_or("")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(160)
                    .collect();
                cards.push(Card {
                    record: record.id.clone(),
                    heading: sec.path.join(" > "),
                    span: [span.start, span.end],
                    text: excerpt,
                    fetch: Some(format!(
                        "{}:v1:{}:{}",
                        record.body_hash(),
                        span.start,
                        span.end
                    )),
                    ..Card::default()
                });
            }
            Value::Outline(cards)
        }

        Add(options) => {
            let prepared = prepared.unwrap();
            let mut body = String::new();
            input
                .as_mut()
                .ok_or_else(|| Error::Invalid("missing add input".into()))?
                .read_to_string(&mut body)?;
            let body_hash = hash(body.as_bytes());
            let source = prepared.source(&body_hash);
            let label = options.label.unwrap_or_else(|| {
                if prepared.kind == "doc" {
                    let sections = sections::parse_sections(&body);
                    let level = sections
                        .iter()
                        .filter(|s| s.level > 0)
                        .map(|s| s.level)
                        .min()
                        .unwrap_or(1);
                    let top = sections
                        .iter()
                        .filter(|s| s.level == level)
                        .collect::<Vec<_>>();
                    if top.len() == 1 {
                        return top[0].heading.clone();
                    }
                    options.input_name.unwrap_or_default()
                } else {
                    body.lines().next().unwrap_or("").chars().take(80).collect()
                }
            });
            let parent = match options.parent {
                Some(p) => Some(
                    store
                        .resolve_loaded(&p, scope.as_deref())
                        .await?
                        .0
                        .id
                        .clone(),
                ),
                None => None,
            };
            let id = format!(
                "{}-{}",
                &body_hash[..16],
                time::OffsetDateTime::now_utc().unix_timestamp_nanos()
            );
            let mut warnings = Vec::new();
            let link = regex::Regex::new(r"\]\(linn:([^)]+)\)").unwrap();
            for captures in link.captures_iter(&body) {
                if store
                    .resolve_loaded(&captures[1], scope.as_deref())
                    .await
                    .is_err()
                {
                    warnings.push(format!("Unresolved link: linn:{}", &captures[1]));
                }
            }
            let record = Record {
                id: id.clone(),
                body,
                scope: if options.unscoped { None } else { scope },
                state: "accepted".into(),
                kind: prepared.kind.into(),
                label,
                source_path: prepared.path,
                parent,
                close: None,
                stale_after: options.stale_after,
                writer: Some(writer),
                source,
                created: now(),
                stored_body: None,
                projected_eligible: None,
            };
            store.add(record).await?;
            return Ok(Reply {
                value: Value::Added(id),
                warnings,
                output: None,
            });
        }
        Close {
            reference,
            reason,
            replacement,
        } => {
            if !["supersede", "correct", "expire"].contains(&reason.as_str()) {
                return Err(Error::Invalid("invalid close reason".into()));
            }
            let target = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            let replacement = match replacement {
                Some(p) => Some(
                    store
                        .resolve_loaded(&p, scope.as_deref())
                        .await?
                        .0
                        .id
                        .clone(),
                ),
                None => None,
            };
            let id = format!(
                "close-{}",
                time::OffsetDateTime::now_utc().unix_timestamp_nanos()
            );
            let record: Record = serde_json::from_value(
                json!({"id":id,"body":"","kind":"note","state":"accepted","writer":writer,"created":now(),"scope":scope,"close":crate::storage::record::Close{target,reason,replacement}}),
            )?;
            store.add(record).await?;
            Value::Added(id)
        }
        Forget { reference } => {
            let id = store
                .resolve_loaded(&reference, scope.as_deref())
                .await?
                .0
                .id
                .clone();
            store.forget(&id).await?;
            Value::Forgotten(id)
        }
        Check => {
            store.integrity().await?;
            Value::Checked
        }
        Rebuild => {
            store.rebuild().await?;
            Value::Rebuilt
        }
        _ => unreachable!(),
    };
    Ok(value.into())
}
pub fn now() -> String {
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .unwrap()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
fn cursor(q: &str, scope: &Option<String>, next: usize, now: &str) -> String {
    format!(
        "s1.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&(q, scope, next, now)).unwrap())
    )
}
async fn search(store: &mut Store, q: &str, ctx: &Context, start: usize) -> Result<SearchPage> {
    let parsed = Query::parse(q).map_err(Error::Invalid)?;
    if parsed
        .filters
        .iter()
        .any(|(k, _)| !["scope", "under", "path", "kind", "id", "writer"].contains(&k.as_str()))
    {
        return Err(Error::Invalid("Query error: Unknown prefix".into()));
    }
    let original_query = q;
    let mut replacements = std::collections::HashMap::new();
    for (k, v) in &parsed.filters {
        if k == "id" {
            let rid = store
                .resolve_loaded(v, ctx.scope.as_deref())
                .await?
                .0
                .id
                .clone();
            if rid.contains('"') {
                return Err(Error::Invalid("record ID contains a quote and cannot be used as a query filter; use show instead".into()));
            }
            replacements.insert((k.clone(), v.clone()), rid);
        }
    }
    let q = crate::retrieval::query::rewrite_filter_values(original_query, &replacements);
    let parsed = Query::parse(&q).map_err(Error::Invalid)?;
    let mut calls = store.agent_in_snapshot(&q, ctx).await?;
    let pointers = calls
        .iter()
        .flat_map(|call| call.hints.iter().chain(&call.related))
        .map(|p| (p.record.clone(), p.span[0]))
        .collect::<Vec<_>>();
    store.load_pointer_blocks(&pointers).await?;
    let mut seen = HashSet::new();
    for call in &mut calls {
        call.cards.retain(|c| {
            let yes = store
                .rows
                .get(&c.section)
                .is_some_and(|b| store.allowed(b, &parsed.filters));
            yes && seen.insert((c.record.clone(), c.span))
        });
        call.hints.retain(|p| pointer_allowed(store, p, &parsed));
        call.related.retain(|p| pointer_allowed(store, p, &parsed));
    }
    let selected =
        (start..calls.len()).find(|i| !calls[*i].cards.is_empty() || (*i == 0 && start == 0));
    let Some(i) = selected else {
        return Ok(SearchPage {
            call: None,
            records: BTreeMap::new(),
            number: 0,
            cursor: None,
            more: false,
        });
    };
    let call = &calls[i];
    let next = (i + 1..calls.len()).find(|j| !calls[*j].cards.is_empty());
    let token = next.map(|n| cursor(original_query, &ctx.scope, n, &ctx.now));
    let mut records = BTreeMap::new();
    for id in call
        .cards
        .iter()
        .map(|c| &c.record)
        .chain(call.hints.iter().chain(&call.related).map(|p| &p.record))
    {
        let r = &store.records[id];
        records.insert(
            id.clone(),
            RecordLabel {
                label: r.label.clone(),
                kind: r.kind.clone(),
                body_hash: r.body_hash(),
            },
        );
    }
    Ok(SearchPage {
        call: Some(call.clone()),
        records,
        number: i + 1,
        cursor: token,
        more: next.is_some(),
    })
}
fn pointer_allowed(store: &Store, p: &crate::storage::Pointer, q: &Query) -> bool {
    store
        .record_blocks(&p.record)
        .find(|b| b.start <= p.span[0] && p.span[0] < b.end)
        .is_some_and(|b| store.allowed(b, &q.filters))
}

/// Network coordination is independent of the store dispatcher and its transport.
pub async fn synchronize(
    replica: &impl crate::sync::LocalReplica,
    environment: Option<&[(std::ffi::OsString, std::ffi::OsString)]>,
) -> Result<crate::sync::Counts> {
    crate::sync::run_with_environment(replica, environment).await
}

#[cfg(test)]
mod resident_tests {
    use super::*;
    use crate::sync::protocol::{Request, Response};

    fn caller() -> Caller {
        Caller {
            scope: None,
            writer: None,
            diagnostics: crate::diagnostics::Diagnostics {
                startup: true,
                ..Default::default()
            },
        }
    }
    #[tokio::test]
    async fn readonly_replica_steps_reuse_the_connection_and_preserve_the_warm_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("resident.db");
        let record: Record = serde_json::from_value(json!({
            "id":"r", "body":"Original publication routing evidence.", "state":"accepted"
        }))
        .unwrap();
        drop(Store::build(&path, vec![record]).await.unwrap());
        let mut app = Application::new(path);
        let first = app
            .execute(
                Operation::Search {
                    query: "publication routing".into(),
                },
                caller(),
                None,
            )
            .await
            .unwrap();
        let Value::Search(first) = first.value else {
            panic!("expected search")
        };
        let expected = serde_json::to_value(first.call).unwrap();
        for (index, op) in [
            Operation::Remotes,
            Operation::SyncFrame(Request::Inventory {
                tombstones: false,
                after: None,
                scope: None,
            }),
            Operation::SyncFrame(Request::Get {
                id: "r".into(),
                scope: None,
            }),
            Operation::SyncFrame(Request::Needs { entries: vec![] }),
        ]
        .into_iter()
        .enumerate()
        {
            let reply = app.execute(op, caller(), None).await.unwrap();
            assert_eq!(
                reply.warnings.iter().any(|s| s.contains("replica-open")),
                index == 0
            );
            let reply = app
                .execute(
                    Operation::Search {
                        query: "publication routing".into(),
                    },
                    caller(),
                    None,
                )
                .await
                .unwrap();
            assert!(!reply.warnings.iter().any(|s| s.contains("store-open")));
            let Value::Search(page) = reply.value else {
                panic!("expected search")
            };
            assert_eq!(serde_json::to_value(page.call).unwrap(), expected);
        }
        let tombstone = crate::storage::sync::Item {
            id: "r".into(),
            envelope: None,
            envelope_hash: None,
            body_hash: None,
        };
        let reply = app
            .execute(
                Operation::SyncFrame(Request::Apply {
                    item: tombstone,
                    scope: None,
                }),
                caller(),
                None,
            )
            .await
            .unwrap();
        assert!(matches!(
            reply.value,
            Value::SyncFrame(Response::Applied { changed: true })
        ));
        app.execute(Operation::SyncFrame(Request::Finish), caller(), None)
            .await
            .unwrap();
        let reply = app
            .execute(
                Operation::Search {
                    query: "publication routing".into(),
                },
                caller(),
                None,
            )
            .await
            .unwrap();
        assert!(reply.warnings.iter().any(|s| s.contains("store-open")));
        assert!(
            app.execute(
                Operation::Show {
                    reference: "r".into(),
                    all: true
                },
                caller(),
                None
            )
            .await
            .is_err()
        );
    }
}
