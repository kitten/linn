//! Read-only replica source requests and final-delta validation.
use super::*;
pub(super) struct ReplicaSource<'a> {
    pub peer: &'a Peer,
    pub remote: &'a str,
    pub run: &'a str,
    pub round: u8,
    pub delta: bool,
    pub job: Arc<SignedRun>,
    pub stream: source::Stream,
    pub cutoff: u64,
    pub progress: Progress,
    pub delta_started: bool,
    pub revision: Option<u64>,
}
impl ReplicaSource<'_> {
    pub async fn serve(&mut self, send: &mut SendStream, recv: &mut RecvStream) -> Result<()> {
        loop {
            let frame = wire::receive_request(recv).await?;
            {
                let runs = self.peer.runs.lock().unwrap();
                let entry = runs
                    .get(self.run)
                    .ok_or_else(|| Kind::Authentication.error("mesh peer: unknown manual run"))?;
                if !entry.serves_round(self.remote, self.round) {
                    return Err(Kind::Authentication
                        .error("mesh peer: replica request is outside the active job phase"));
                }
            }
            let source = self
                .peer
                .runs
                .lock()
                .unwrap()
                .get(self.run)
                .and_then(|entry| entry.coordinated.clone());
            match frame {
                Frame::DeltaInventory { tombstones, after } if self.round == 2 => {
                    if self
                        .delta_inventory(send, source, tombstones, after)
                        .await?
                    {
                        return Ok(());
                    }
                }
                Frame::Request(Request::Inventory { tombstones, after }) => {
                    self.inventory(send, tombstones, after).await?
                }
                Frame::Request(Request::Get { id }) => self.get(send, id).await?,
                Frame::Request(Request::Finish) => return self.finish(send, source).await,
                _ => return Err(protocol("replica request expected")),
            }
        }
    }
    async fn delta_inventory(
        &mut self,
        send: &mut SendStream,
        source: Option<Arc<coordinated::Node>>,
        tombstones: bool,
        after: Option<String>,
    ) -> Result<bool> {
        if !self.delta || self.stream.fallback {
            return Err(protocol("delta cannot replay a fallback source"));
        }
        let current = source.as_ref().and_then(|node| node.delta_revision());
        if !self.delta_started {
            self.revision = current;
        }
        self.delta_started = true;
        let response = match source
            .as_ref()
            .filter(|_| current.is_some() && current == self.revision)
            .and_then(|node| node.delta_page(self.remote, tombstones, after.as_deref()))
        {
            Some((entries, done)) => {
                let Value::Inventory(page) = self
                    .peer
                    .store
                    .step(Step::Sizes {
                        peer: self.remote.into(),
                        entries,
                    })
                    .await?
                else {
                    return Err(protocol("sized delta inventory expected"));
                };
                Frame::DeltaEntries {
                    entries: page.entries,
                    done,
                }
            }
            None => {
                self.stream.fallback()?;
                Frame::DeltaUnavailable
            }
        };
        let unavailable = matches!(response, Frame::DeltaUnavailable);
        wire::send_progress(send, response, &self.progress).await?;
        if unavailable {
            return Ok(true);
        }
        Ok(false)
    }
    async fn inventory(
        &self,
        send: &mut SendStream,
        tombstones: bool,
        after: Option<String>,
    ) -> Result<()> {
        if self.delta {
            return Err(protocol(
                "delta source cannot switch to full inventories on the same stream",
            ));
        }
        let Value::Inventory(page) = self
            .peer
            .store
            .step(Step::Inventory {
                peer: self.remote.into(),
                tombstones,
                after,
                cutoff: self.cutoff,
            })
            .await?
        else {
            return Err(protocol("sized inventory expected"));
        };
        wire::send_progress(send, Frame::InventoryPage(page), &self.progress).await?;
        Ok(())
    }
    async fn get(&self, send: &mut SendStream, id: String) -> Result<()> {
        let scope = self.stream.scope.clone();
        #[cfg(test)]
        wire::metrics::source_read(&self.peer.local);
        match self
            .peer
            .store
            .step(Step::Get {
                peer: self.remote.into(),
                job: self.job.clone(),
                scope: scope.clone(),
                id,
            })
            .await?
        {
            Value::Body(body) => {
                wire::send_body(
                    send,
                    body,
                    &self.peer.store,
                    self.remote,
                    &self.job,
                    &self.progress,
                    &scope,
                )
                .await?;
                if let Some(entry) = self.peer.runs.lock().unwrap().get_mut(self.run) {
                    entry.report.sent += 1;
                }
            }
            Value::Item(None) => {
                wire::send_progress(
                    send,
                    Frame::Response(Response::Item { item: None }),
                    &self.progress,
                )
                .await?
            }
            _ => return Err(protocol("source item expected")),
        }
        Ok(())
    }
    async fn finish(
        &mut self,
        send: &mut SendStream,
        source: Option<Arc<coordinated::Node>>,
    ) -> Result<()> {
        if self.delta_started {
            self.stream.fallback()?;
            if !source.as_ref().is_some_and(|node| {
                node.delta_eligible(self.remote)
                    && self.revision.is_some()
                    && node.delta_revision() == self.revision
            }) {
                wire::send_progress(send, Frame::DeltaUnavailable, &self.progress).await?;
                return Ok(());
            }
        }
        self.peer
            .store
            .step(Step::Finish(Some(self.remote.into())))
            .await?;
        wire::send_progress(send, Frame::Response(Response::Done), &self.progress).await?;
        Ok(())
    }
}
