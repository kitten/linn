//! FIFO fetch credits and two bounded, durably retired Apply groups.
use super::*;
use crate::{
    mesh::{buffers::Body, store::Prepared},
    storage::sync::{InventoryEntry, InventoryPage, PAGE, Prefix},
};
use std::collections::VecDeque;
use tokio::sync::{mpsc, oneshot};

// Fixed bounds validated by the matched-coverage native transfer fixture.
// @ref LLP 0008#manual-mesh-jobs
const GROUP_ITEMS: usize = 32;
const GROUP_BYTES: u64 = 512 * 1024;
const BATCH_BYTES: u64 = 4 * 1024 * 1024;
const FLUSH: Duration = Duration::from_millis(2);
const WINDOW: u64 = 128 * 1024 * 1024;
// Include raw/decoded originals, validation's JSON tree and bounded task metadata.
const DECODE_FACTOR: u64 = 128;

enum Expected {
    Page,
    Item { body: Body, entry: InventoryEntry },
    Finish,
}
enum Received {
    Page(Option<InventoryPage>),
    Item(wire::BodyResponse),
    Finish(bool),
}
struct Reader {
    requests: mpsc::Sender<Expected>,
    replies: mpsc::Receiver<Result<Received>>,
    abort: tokio::task::AbortHandle,
    active: Arc<Mutex<Option<Body>>>,
    done: Option<oneshot::Receiver<()>>,
}
impl Drop for Reader {
    fn drop(&mut self) {
        self.abort.abort();
        if let Some(body) = self.active.lock().unwrap().take() {
            body.discard_incomplete();
        }
    }
}
struct ReadTask {
    recv: Option<RecvStream>,
    expected: mpsc::Receiver<Expected>,
    replies: Option<mpsc::Sender<Result<Received>>>,
    active: Arc<Mutex<Option<Body>>>,
    done: Option<oneshot::Sender<()>>,
}
impl Drop for ReadTask {
    fn drop(&mut self) {
        if let Some(mut recv) = self.recv.take() {
            let _ = recv.stop(Kind::Cancelled.code().into());
        }
        self.expected.close();
        while self.expected.try_recv().is_ok() {}
        if let Some(body) = self.active.lock().unwrap().take() {
            body.discard_incomplete();
        }
        drop(self.replies.take());
        if let Some(done) = self.done.take() {
            let _ = done.send(());
        }
    }
}
impl Reader {
    async fn start(peer: &Peer, recv: RecvStream, progress: Progress, delta: bool) -> Self {
        let (requests, expected) = mpsc::channel::<Expected>(PAGE + 2);
        let (replies, responses) = mpsc::channel(PAGE + 2);
        let (finished, done) = oneshot::channel();
        let active = Arc::new(Mutex::new(None));
        let mut task = ReadTask {
            recv: Some(recv),
            expected,
            replies: Some(replies),
            active: active.clone(),
            done: Some(finished),
        };
        let mut tasks = peer.readers.lock().await;
        while tasks.try_join_next().is_some() {}
        let abort = tasks.spawn(async move {
            while let Some(request) = task.expected.recv().await {
                let recv = task.recv.as_mut().unwrap();
                let response = match request {
                    Expected::Item { body, entry } => {
                        *task.active.lock().unwrap() = Some(body.clone());
                        let result = wire::receive_body(recv, body, &entry, &progress, delta)
                            .await
                            .map(Received::Item);
                        task.active.lock().unwrap().take();
                        result
                    }
                    Expected::Page => match wire::receive_progress(recv, &progress).await {
                        Ok(Frame::InventoryPage(page)) => Ok(Received::Page(Some(page))),
                        Ok(Frame::DeltaEntries { entries, done }) => {
                            Ok(Received::Page(Some(InventoryPage { entries, done })))
                        }
                        Ok(Frame::DeltaUnavailable) => Ok(Received::Page(None)),
                        Ok(_) => Err(protocol("inventory page expected")),
                        Err(error) => {
                            Err(Kind::of(&error).error(format!("mesh inventory response: {error}")))
                        }
                    },
                    Expected::Finish => match wire::receive_progress(recv, &progress).await {
                        Ok(Frame::Response(Response::Done)) => Ok(Received::Finish(true)),
                        Ok(Frame::DeltaUnavailable) => Ok(Received::Finish(false)),
                        Ok(_) => Err(protocol("replica completion expected")),
                        Err(error) => {
                            Err(Kind::of(&error).error(format!("mesh finish response: {error}")))
                        }
                    },
                };
                let failed = response.is_err();
                if task.replies.as_ref().unwrap().send(response).await.is_err() || failed {
                    break;
                }
            }
        });
        Self {
            requests,
            replies: responses,
            abort,
            active,
            done: Some(done),
        }
    }
    async fn stop(&mut self) {
        self.abort.abort();
        if let Some(done) = &mut self.done {
            let _ = done.await;
            self.done = None;
        }
    }
    fn expect(&mut self, expected: Expected) -> Result<()> {
        match self.requests.try_send(expected) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Closed(_)) => {
                // The reader queued its typed error before closing credits.
                // Registered cells still own every earlier complete response.
                while let Ok(reply) = self.replies.try_recv() {
                    reply?;
                }
                Err(protocol("credited-response reader ended without a result"))
            }
            Err(mpsc::error::TrySendError::Full(_)) => Err(protocol("fetch-credit bound exceeded")),
        }
    }
    async fn next(&mut self) -> Result<Received> {
        self.replies
            .recv()
            .await
            .ok_or_else(|| protocol("credited-response reader ended"))?
    }
}

// Field order is part of cancellation cleanup: stop the reader, enqueue the
// filling prefix, release the empty spare, then enqueue publication. The writer
// executes these reserved messages even when their response receivers are gone.
struct Pipeline {
    reader: Reader,
    filling: Prepared,
    spare: Option<Prepared>,
    finish: Option<Prepared>,
}
impl Drop for Pipeline {
    fn drop(&mut self) {
        self.reader.abort.abort();
    }
}
#[derive(Default)]
struct Batch {
    changed: u64,
    items: usize,
    bytes: u64,
}
impl Batch {
    fn can_add(&self, pending_count: usize, pending_bytes: u64, entry: &InventoryEntry) -> bool {
        let items = self.items + pending_count;
        let bytes = self.bytes + pending_bytes;
        items == 0
            || (items < PAGE && bytes < BATCH_BYTES && bytes + entry.envelope_bytes <= BATCH_BYTES)
    }
    fn accept(
        &mut self,
        peer: &Peer,
        run: &str,
        entries: &[InventoryEntry],
        value: Value,
    ) -> Result<()> {
        let Value::Decisions(prefix) = value else {
            return Err(protocol("committed Apply decisions expected"));
        };
        if prefix.decisions.len() > entries.len()
            || (prefix.error.is_none() && prefix.decisions.len() != entries.len())
        {
            return Err(protocol("invalid Apply prefix"));
        }
        let mut received = 0;
        for (entry, changed) in entries.iter().zip(prefix.decisions) {
            if changed {
                received += 1;
                self.changed += 1;
                self.items += 1;
                self.bytes += entry.envelope_bytes;
            }
        }
        if let Some(entry) = peer.runs.lock().unwrap().get_mut(run) {
            entry.report.received += received;
        }
        if let Some(node) = peer
            .runs
            .lock()
            .unwrap()
            .get(run)
            .and_then(|entry| entry.coordinated.clone())
        {
            node.refresh_delta();
        }
        if let Ok(progress) = peer.progress(run) {
            progress.advance();
        }
        match prefix.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
struct Credit {
    entry: InventoryEntry,
    received: bool,
}
struct Commit {
    reply: oneshot::Receiver<Result<Value>>,
    entries: Vec<InventoryEntry>,
}
impl Commit {
    async fn finish(&mut self) -> Result<Value> {
        (&mut self.reply)
            .await
            .map_err(|_| protocol("group commit reply lost"))?
    }
}
fn apply_step(remote: &str, job: &Arc<SignedRun>) -> Step {
    Step::ApplyGroup {
        peer: remote.into(),
        job: job.clone(),
        items: Vec::new(),
    }
}
async fn publish_batch(store: &Handle, finish: &mut Option<Prepared>) -> Result<()> {
    // Keep failure/cancellation publication reserved even if this Finish fails.
    let replacement = store.prepare(Step::Finish(None)).await?;
    finish.replace(replacement).unwrap().submit().await?;
    Ok(())
}
fn reservation(entry: &InventoryEntry) -> u64 {
    entry
        .frame_bytes
        .saturating_mul(DECODE_FACTOR)
        .saturating_add(4096)
}
fn validate(page: &InventoryPage, tombstones: bool, after: Option<&str>) -> Result<()> {
    if page.entries.len() > PAGE
        || (!page.done && page.entries.is_empty())
        || page
            .entries
            .windows(2)
            .any(|entries| entries[0].entry.id >= entries[1].entry.id)
        || page.entries.iter().any(|entry| {
            after.is_some_and(|after| entry.entry.id.as_str() <= after)
                || entry.entry.hash.is_none() != tombstones
                || entry.frame_bytes <= 4
                || entry.frame_bytes > (crate::sync::protocol::MAX_FRAME + 4) as u64
                || entry.envelope_bytes > entry.frame_bytes
        })
    {
        return Err(protocol("invalid sized inventory page"));
    }
    Ok(())
}

impl Peer {
    pub(super) async fn pull_pipeline(
        &self,
        connection: &Connection,
        remote: &str,
        run: &str,
        operation: (u8, u64),
        delta: bool,
    ) -> Result<(u64, bool)> {
        let progress = self.progress(run)?;
        let job = Arc::new(
            self.runs
                .lock()
                .unwrap()
                .get(run)
                .ok_or_else(|| protocol("unknown run"))?
                .job
                .clone(),
        );
        let (mut send, recv) = connection.open_bi().await.map_err(Kind::connection)?;
        wire::send(
            &mut send,
            Frame::Replica {
                run: run.into(),
                round: operation.0,
                context: operation.1,
                delta,
            },
        )
        .await?;
        // Reserve publication first so reverse local destruction preserves FIFO.
        let finish = self.store.prepare(Step::Finish(None)).await?;
        let filling = self.store.prepare(apply_step(remote, &job)).await?;
        let spare = self.store.prepare(apply_step(remote, &job)).await?;
        let mut pipeline = Pipeline {
            reader: Reader::start(self, recv, progress, delta).await,
            filling,
            spare: Some(spare),
            finish: Some(finish),
        };
        let mut batch = Batch::default();
        let result = self
            .pipeline_stream(&mut pipeline, &mut send, remote, &job, &mut batch, delta)
            .await;
        let result = match result {
            Err(error)
                if delta
                    && matches!(
                        Kind::of(&error),
                        Kind::Offline | Kind::Timeout | Kind::Unresolved
                    ) =>
            {
                Ok(true)
            }
            other => other,
        };
        pipeline.reader.stop().await;
        // Every page has retired both groups before returning, including errors.
        pipeline.filling.discard();
        let published = pipeline.finish.take().unwrap().submit().await;
        published?;
        Ok((batch.changed, result?))
    }
    async fn request_page(
        &self,
        pipeline: &mut Pipeline,
        send: &mut SendStream,
        tombstones: bool,
        after: Option<String>,
        delta: bool,
    ) -> Result<()> {
        pipeline.reader.expect(Expected::Page)?;
        wire::send(
            send,
            if delta {
                Frame::DeltaInventory { tombstones, after }
            } else {
                Frame::Request(Request::Inventory { tombstones, after })
            },
        )
        .await
    }
    async fn pipeline_stream(
        &self,
        pipeline: &mut Pipeline,
        send: &mut SendStream,
        remote: &str,
        job: &Arc<SignedRun>,
        batch: &mut Batch,
        delta: bool,
    ) -> Result<bool> {
        for tombstones in [true, false] {
            let mut after = None;
            self.request_page(pipeline, send, tombstones, None, delta)
                .await?;
            let mut prefetched = None;
            loop {
                let page = match prefetched.take() {
                    Some(page) => page,
                    None => match pipeline.reader.next().await? {
                        Received::Page(Some(page)) => page,
                        Received::Page(None) if delta => return Ok(true),
                        _ => return Err(protocol("inventory page expected")),
                    },
                };
                if let Err(error) = validate(&page, tombstones, after.as_deref()) {
                    if delta {
                        return Ok(true);
                    } else {
                        return Err(error);
                    }
                }
                after = page.entries.last().map(|entry| entry.entry.id.clone());
                if !page.done {
                    self.request_page(pipeline, send, tombstones, after.clone(), delta)
                        .await?;
                }
                let Value::Needs { prefix, epoch } = self
                    .store
                    .step(Step::Needs {
                        peer: remote.into(),
                        entries: page
                            .entries
                            .iter()
                            .map(|entry| entry.entry.clone())
                            .collect(),
                    })
                    .await?
                else {
                    return Err(protocol("page Needs decisions expected"));
                };
                let mut entries = page.entries;
                let mut needs_error = None;
                retain_needed(&mut entries, 0, prefix, &mut needs_error)?;
                if (PageTransfer {
                    peer: self,
                    remote,
                    job,
                    batch,
                    delta,
                })
                .run(pipeline, send, entries, epoch, &mut prefetched)
                .await?
                {
                    return Ok(true);
                }
                if let Some(error) = needs_error {
                    return Err(error);
                }
                if page.done {
                    break;
                }
            }
        }
        pipeline.reader.expect(Expected::Finish)?;
        wire::send(send, Frame::Request(Request::Finish)).await?;
        match pipeline.reader.next().await? {
            Received::Finish(true) => Ok(delta
                && !self
                    .runs
                    .lock()
                    .unwrap()
                    .get(&job.run.id)
                    .and_then(|entry| entry.coordinated.clone())
                    .is_some_and(|node| node.delta_baseline_valid())),
            Received::Finish(false) if delta => Ok(true),
            _ => Err(protocol("replica completion expected")),
        }
    }
}

struct PageTransfer<'a> {
    peer: &'a Peer,
    remote: &'a str,
    job: &'a Arc<SignedRun>,
    batch: &'a mut Batch,
    delta: bool,
}
fn retain_needed(
    entries: &mut Vec<InventoryEntry>,
    issued: usize,
    prefix: Prefix<bool>,
    error: &mut Option<Error>,
) -> Result<()> {
    let remaining = entries.len() - issued;
    if prefix.decisions.len() > remaining
        || (prefix.error.is_none() && prefix.decisions.len() != remaining)
    {
        return Err(protocol("invalid Needs prefix"));
    }
    let suffix: Vec<_> = entries
        .drain(issued..)
        .zip(prefix.decisions)
        .filter_map(|(entry, needed)| needed.then_some(entry))
        .collect();
    entries.extend(suffix);
    // A new failure is earlier in the remaining prefix. A clean recheck of
    // that prefix must not erase the error which originally truncated it.
    if prefix.error.is_some() {
        *error = prefix.error;
    }
    Ok(())
}
impl PageTransfer<'_> {
    async fn run(
        &mut self,
        pipeline: &mut Pipeline,
        send: &mut SendStream,
        mut entries: Vec<InventoryEntry>,
        mut epoch: u64,
        prefetched: &mut Option<InventoryPage>,
    ) -> Result<bool> {
        let mut issued = 0;
        let mut filling = VecDeque::<Credit>::new();
        let mut commit: Option<Commit> = None;
        let mut flush_at = None;
        let mut failure = None;
        let mut suffix_error = None;
        let mut fallback = false;
        let result = async {
        loop {
            if failure.is_none() && !fallback && issued < entries.len() && self.peer.store.buffers.epoch() != epoch {
                let Value::Needs { prefix, epoch: fresh } = self.peer.store.step(Step::Needs {
                    peer: self.remote.into(),
                    entries: entries[issued..].iter().map(|entry| entry.entry.clone()).collect(),
                }).await? else { return Err(protocol("fresh Needs prefix expected")); };
                retain_needed(&mut entries, issued, prefix, &mut suffix_error)?;
                epoch = fresh;
            }
            while failure.is_none() && !fallback && issued < entries.len()
                && Self::can_credit(&entries[issued], &filling, commit.as_ref(), self.batch) {
                let entry = entries[issued].clone();
                let body = match self.peer.store.buffers.receive(epoch, entry.entry.clone()) {
                    Ok(body) => body,
                    Err(_) => break, // Recheck Needs under the new forget epoch.
                };
                pipeline.filling.push(body.clone());
                filling.push_back(Credit { entry: entry.clone(), received: false });
                issued += 1;
                pipeline.reader.expect(Expected::Item { body, entry: entry.clone() })?;
                let request = Frame::Request(Request::Get { id: entry.entry.id });
                if let Err(error) = wire::send(send, request).await {
                    failure = Some(error);
                    break;
                }
            }
            if failure.is_some() {
                pipeline.reader.stop().await;
                pipeline.filling.remove_missing(|index| { filling.remove(index); });
                let valid = pipeline.filling.complete_prefix();
                for credit in filling.iter_mut().take(valid) { credit.received = true; }
                filling.truncate(valid);
                pipeline.filling.truncate(valid);
            }
            let ready = filling.iter().take_while(|credit| credit.received).count();
            let ready_bytes: u64 = filling.iter().take(ready).map(|credit| credit.entry.frame_bytes).sum();
            let all_received = ready == filling.len();
            let stop = failure.is_some() || fallback || issued == entries.len();
            let flush = ready != 0 && (ready >= GROUP_ITEMS || ready_bytes >= GROUP_BYTES || (stop && all_received)
                || flush_at.is_some_and(|at| tokio::time::Instant::now() >= at));
            if commit.is_none() && flush {
                let mut prepared = pipeline.spare.take().unwrap();
                pipeline.filling.move_prefix(&mut prepared, ready);
                let entries = filling.drain(..ready).map(|credit| credit.entry).collect();
                commit = Some(Commit { reply: prepared.start(), entries });
                flush_at = None;
            }
            if filling.is_empty() && commit.is_none() {
                if let Some(error) = failure { return Err(error); }
                if fallback { return Ok(true); }
                if issued == entries.len() {
                    return match suffix_error { Some(error) => Err(error), None => Ok(false) };
                }
                if !self.batch.can_add(0, 0, &entries[issued]) {
                    publish_batch(&self.peer.store, &mut pipeline.finish).await?;
                    self.batch.items = 0;
                    self.batch.bytes = 0;
                    continue;
                }
                // Only a concurrent forget can have prevented an otherwise empty
                // window from issuing a valid item; refresh its Needs next turn.
                if self.peer.store.buffers.epoch() != epoch { continue; }
            }
            let waiting = filling.iter().any(|credit| !credit.received);
            let alarm = async {
                if let Some(at) = flush_at { tokio::time::sleep_until(at).await; }
                else { std::future::pending::<()>().await; }
            };
            tokio::select! {
                reply = async { commit.as_mut().unwrap().finish().await }, if commit.is_some() => {
                    let completed = commit.take().unwrap();
                    let accepted = reply.and_then(|value| {
                        self.batch.accept(self.peer, &self.job.run.id, &completed.entries, value)
                    });
                    if let Err(error) = accepted {
                        // The first failing Apply precedes every item in filling.
                        pipeline.reader.abort.abort();
                        pipeline.filling.discard();
                        filling.clear();
                        return Err(error);
                    }
                    pipeline.spare = Some(self.peer.store.prepare(apply_step(self.remote, self.job)).await?);
                },
                reply = pipeline.reader.next(), if waiting => {
                    match reply {
                        Ok(Received::Page(Some(page))) if prefetched.is_none() => *prefetched = Some(page),
                        Ok(Received::Page(None)) if self.delta => fallback = true,
                        Ok(Received::Item(response)) => {
                            let index = filling.iter().position(|credit| !credit.received).unwrap();
                            match response {
                                wire::BodyResponse::Item => {
                                    filling[index].received = true;
                                    flush_at.get_or_insert_with(|| tokio::time::Instant::now() + FLUSH);
                                }
                                wire::BodyResponse::Missing if !self.delta => {
                                    filling.remove(index);
                                    pipeline.filling.remove(index);
                                }
                                wire::BodyResponse::Missing | wire::BodyResponse::Unavailable
                                | wire::BodyResponse::Mismatch if self.delta => {
                                    fallback = true;
                                    pipeline.reader.abort.abort();
                                    let valid = filling.iter().take_while(|credit| credit.received).count();
                                    filling.truncate(valid); pipeline.filling.truncate(valid);
                                }
                                wire::BodyResponse::Mismatch => {
                                    failure = Some(Kind::Conflict.error("mesh item does not match inventory"));
                                },
                                _ => failure = Some(protocol("unexpected delta-unavailable reply")),
                            }
                        }
                        Err(error) if self.delta && matches!(
                            Kind::of(&error), Kind::Offline | Kind::Timeout | Kind::Unresolved
                        ) => {
                            fallback = true;
                            pipeline.reader.abort.abort();
                            let valid = filling.iter().take_while(|credit| credit.received).count();
                            filling.truncate(valid); pipeline.filling.truncate(valid);
                        }
                        Err(error) => failure = Some(error),
                        _ => failure = Some(protocol("unexpected credited response")),
                    }
                },
                _ = alarm, if commit.is_none() && ready != 0 => (),
            }
        }
        }.await;
        self.retire(pipeline, commit, filling, result).await
    }
    async fn retire(
        &mut self,
        pipeline: &mut Pipeline,
        commit: Option<Commit>,
        mut filling: VecDeque<Credit>,
        result: Result<bool>,
    ) -> Result<bool> {
        if result.is_err() || matches!(result, Ok(true)) {
            pipeline.reader.stop().await;
            pipeline.filling.remove_missing(|index| {
                filling.remove(index);
            });
        }
        if let Some(mut pending) = commit {
            let applied = pending.finish().await.and_then(|value| {
                self.batch
                    .accept(self.peer, &self.job.run.id, &pending.entries, value)
            });
            if let Err(error) = applied {
                pipeline.filling.discard();
                return Err(error);
            }
        }
        if !filling.is_empty() {
            let replacement = self
                .peer
                .store
                .prepare(apply_step(self.remote, self.job))
                .await?;
            let pending = std::mem::replace(&mut pipeline.filling, replacement);
            let metadata: Vec<_> = filling.into_iter().map(|credit| credit.entry).collect();
            let applied = pending.submit().await.and_then(|value| {
                self.batch
                    .accept(self.peer, &self.job.run.id, &metadata, value)
            });
            if let Err(error) = applied
                && !matches!(&error, Error::Mesh(error) if !error.local && error.kind == Kind::Cancelled)
            {
                return Err(error);
            }
        }
        result
    }
    fn can_credit(
        entry: &InventoryEntry,
        filling: &VecDeque<Credit>,
        commit: Option<&Commit>,
        batch: &Batch,
    ) -> bool {
        let committed = commit.map_or(&[][..], |commit| commit.entries.as_slice());
        let pending_count = filling.len() + committed.len();
        let pending_original: u64 = filling
            .iter()
            .map(|credit| credit.entry.envelope_bytes)
            .chain(committed.iter().map(|entry| entry.envelope_bytes))
            .sum();
        if filling.len() >= GROUP_ITEMS || !batch.can_add(pending_count, pending_original, entry) {
            return false;
        }
        if pending_count == 0 {
            return true;
        } // One independently valid large item.
        if entry.frame_bytes > GROUP_BYTES
            || filling
                .iter()
                .any(|credit| credit.entry.frame_bytes > GROUP_BYTES)
        {
            return false;
        }
        let filling_bytes: u64 = filling.iter().map(|credit| credit.entry.frame_bytes).sum();
        if filling_bytes + entry.frame_bytes > GROUP_BYTES {
            return false;
        }
        let reserved: u64 = filling
            .iter()
            .map(|credit| reservation(&credit.entry))
            .chain(committed.iter().map(reservation))
            .sum();
        reserved.saturating_add(reservation(entry)) <= WINDOW
    }
}

#[cfg(test)]
mod tests;
