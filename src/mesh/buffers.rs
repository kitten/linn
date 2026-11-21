//! Original-byte ownership shared with the serialized secure-forget writer.
use super::failure::Kind;
use crate::{
    error::Result,
    storage::sync::{Entry, Item},
};
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::watch;

#[derive(Default)]
pub(crate) struct Buffers(Mutex<State>);
#[derive(Default)]
struct State {
    epoch: u64,
    cells: Vec<Weak<Cell>>,
}
struct Cell {
    entry: Entry,
    contents: Mutex<Payload>,
    erased: watch::Sender<bool>,
}
pub(super) enum Payload {
    Receiving(Vec<u8>),
    Item(Item),
    Encoded(Vec<u8>),
    Missing,
    Empty,
}
#[derive(Clone)]
pub struct Body(Arc<Cell>);
pub(crate) struct WeakBody(Weak<Cell>);
impl WeakBody {
    pub(crate) fn alive(&self) -> bool {
        self.0.strong_count() != 0
    }
    pub(crate) fn discard(&self) {
        if let Some(cell) = self.0.upgrade() {
            *cell.contents.lock().unwrap() = Payload::Empty;
            cell.erased.send_replace(true);
        }
    }
}
impl Buffers {
    #[cfg(test)]
    pub(crate) fn live_count(&self) -> usize {
        self.0
            .lock()
            .unwrap()
            .cells
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|cell| !matches!(*cell.contents.lock().unwrap(), Payload::Empty))
            .count()
    }
    pub(crate) fn epoch(&self) -> u64 {
        self.0.lock().unwrap().epoch
    }
    pub(super) fn receive(&self, epoch: u64, entry: Entry) -> Result<Body> {
        self.insert(epoch, entry, Payload::Receiving(Vec::new()))
    }
    pub(super) fn original(&self, item: Item) -> Body {
        let mut state = self.0.lock().unwrap();
        let entry = Entry {
            id: item.id.clone(),
            hash: item.envelope_hash.clone(),
        };
        Self::insert_locked(&mut state, entry, Payload::Item(item))
    }
    fn insert(&self, epoch: u64, entry: Entry, payload: Payload) -> Result<Body> {
        let mut state = self.0.lock().unwrap();
        if state.epoch != epoch {
            return Err(Body::erased_error());
        }
        Ok(Self::insert_locked(&mut state, entry, payload))
    }
    fn insert_locked(state: &mut State, entry: Entry, payload: Payload) -> Body {
        state.cells.retain(|cell| cell.strong_count() != 0);
        let cell = Arc::new(Cell {
            entry,
            contents: Mutex::new(payload),
            erased: watch::channel(false).0,
        });
        state.cells.push(Arc::downgrade(&cell));
        Body(cell)
    }
    pub(crate) fn forget(&self, id: &str) {
        let mut state = self.0.lock().unwrap();
        state.epoch += 1;
        state.cells.retain(|weak| {
            let Some(cell) = weak.upgrade() else {
                return false;
            };
            let mut contents = cell.contents.lock().unwrap();
            if cell.entry.id == id {
                *contents = Payload::Empty;
                cell.erased.send_replace(true);
            }
            true
        });
    }
}
impl Body {
    pub(crate) fn downgrade(&self) -> WeakBody {
        WeakBody(Arc::downgrade(&self.0))
    }
    pub(super) fn discard(&self) {
        *self.0.contents.lock().unwrap() = Payload::Empty;
    }
    pub(super) fn discard_incomplete(&self) {
        let mut contents = self.0.contents.lock().unwrap();
        if !matches!(*contents, Payload::Item(_) | Payload::Missing) {
            *contents = Payload::Empty;
        }
    }
    pub(super) fn entry(&self) -> Entry {
        self.0.entry.clone()
    }
    pub(super) fn complete(&self) -> bool {
        matches!(*self.0.contents.lock().unwrap(), Payload::Item(_))
    }
    pub(super) fn missing(&self) -> bool {
        matches!(*self.0.contents.lock().unwrap(), Payload::Missing)
    }
    pub(super) fn erased_error() -> crate::error::Error {
        Kind::Cancelled.local_error("local forget invalidated buffered original bytes")
    }
    pub(super) async fn erased(&self) {
        let mut erased = self.0.erased.subscribe();
        let _ = erased.wait_for(|erased| *erased).await;
    }
    pub(super) fn with<T>(&self, work: impl FnOnce(&mut Payload) -> Result<T>) -> Result<T> {
        let mut contents = self.0.contents.lock().unwrap();
        if matches!(*contents, Payload::Empty) {
            return Err(if *self.0.erased.borrow() {
                Self::erased_error()
            } else {
                Kind::Cancelled.error("buffer has no accepted original")
            });
        }
        work(&mut contents)
    }
    // Call only on the serialized writer: no forget can interleave with Apply.
    pub(crate) fn take(&self) -> Result<Item> {
        self.with(|payload| match std::mem::replace(payload, Payload::Empty) {
            Payload::Item(item) => Ok(item),
            Payload::Receiving(_) => {
                Err(Kind::Cancelled.error("response was incomplete before cleanup"))
            }
            _ => Err(Kind::Protocol.error("decoded original expected")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: &str) -> Item {
        Item {
            id: id.into(),
            envelope: Some("secret".into()),
            envelope_hash: Some("hash".into()),
            body_hash: None,
        }
    }
    #[tokio::test]
    async fn forget_drops_known_and_undecoded_bytes_and_fences_late_allocations() {
        let buffers = Buffers::default();
        let epoch = buffers.epoch();
        let raw = buffers
            .receive(
                epoch,
                Entry {
                    id: "a".into(),
                    hash: Some("hash".into()),
                },
            )
            .unwrap();
        raw.with(|payload| {
            *payload = Payload::Receiving(b"secret".to_vec());
            Ok(())
        })
        .unwrap();
        let forgotten = buffers.original(item("a"));
        let encoded = buffers.original(item("a"));
        encoded
            .with(|payload| {
                *payload = Payload::Encoded(b"secret".to_vec());
                Ok(())
            })
            .unwrap();
        let other = buffers.original(item("b"));
        buffers.forget("a");
        raw.erased().await;
        assert!(raw.with(|_| Ok(())).is_err());
        assert!(forgotten.take().is_err());
        assert!(encoded.with(|_| Ok(())).is_err());
        assert_eq!(other.take().unwrap().id, "b");
        assert!(
            buffers
                .receive(
                    epoch,
                    Entry {
                        id: "a".into(),
                        hash: None
                    }
                )
                .is_err()
        );
        let fresh = buffers
            .receive(
                buffers.epoch(),
                Entry {
                    id: "b".into(),
                    hash: None,
                },
            )
            .unwrap();
        assert!(fresh.with(|_| Ok(())).is_ok());
        drop((raw, forgotten, encoded, other, fresh));
        let _body = buffers.original(item("c"));
        assert_eq!(buffers.0.lock().unwrap().cells.len(), 1);
    }
}
