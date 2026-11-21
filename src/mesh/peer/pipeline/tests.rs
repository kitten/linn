use super::*;
use crate::storage::sync::Entry;

fn entry(bytes: u64) -> InventoryEntry {
    InventoryEntry {
        entry: Entry {
            id: "id".into(),
            hash: Some("hash".into()),
        },
        frame_bytes: bytes,
        envelope_bytes: bytes / 2,
    }
}

#[tokio::test]
async fn failed_publication_keeps_the_reserved_error_cleanup_instead_of_panicking() {
    let (store, mut jobs) = store::channel();
    let mut finish = Some(store.prepare(Step::Finish(None)).await.unwrap());
    let (result, ()) = tokio::join!(publish_batch(&store, &mut finish), async {
        let publication = jobs.recv().await.unwrap();
        assert!(matches!(publication.step, Step::Finish(None)));
        assert!(
            publication
                .reply
                .send(Err(Kind::Publication.error("injected publication failure")))
                .is_ok()
        );
    });
    assert_eq!(Kind::of(&result.err().unwrap()), Kind::Publication);
    assert!(finish.is_some());
    let (cleanup, ()) = tokio::join!(finish.take().unwrap().submit(), async {
        let publication = jobs.recv().await.unwrap();
        assert!(matches!(publication.step, Step::Finish(None)));
        assert!(publication.reply.send(Ok(Value::Done)).is_ok());
    });
    assert!(cleanup.is_ok());
}

#[test]
fn a_clean_forget_recheck_cannot_erase_the_error_that_truncated_a_needs_prefix() {
    let mut entries = vec![entry(100), entry(100), entry(100)];
    let mut error = None;
    retain_needed(
        &mut entries,
        0,
        Prefix {
            decisions: vec![true, true],
            error: Some(Kind::Conflict.error("later conflict")),
        },
        &mut error,
    )
    .unwrap();
    assert_eq!(entries.len(), 2);
    retain_needed(
        &mut entries,
        0,
        Prefix {
            decisions: vec![false, true],
            error: None,
        },
        &mut error,
    )
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(Kind::of(error.as_ref().unwrap()), Kind::Conflict);
    retain_needed(
        &mut entries,
        0,
        Prefix {
            decisions: vec![],
            error: Some(Kind::Publication.error("earlier storage failure")),
        },
        &mut error,
    )
    .unwrap();
    assert!(entries.is_empty());
    assert_eq!(Kind::of(error.as_ref().unwrap()), Kind::Publication);
}

#[test]
fn byte_window_binds_before_the_count_bound_and_large_items_progress_alone() {
    let filling: VecDeque<_> = (0..GROUP_ITEMS - 1)
        .map(|_| Credit {
            entry: entry(GROUP_BYTES / GROUP_ITEMS as u64),
            received: false,
        })
        .collect();
    let (_, reply) = oneshot::channel();
    let committing = Commit {
        reply,
        entries: (0..GROUP_ITEMS)
            .map(|_| entry(GROUP_BYTES / GROUP_ITEMS as u64))
            .collect(),
    };
    assert!(!PageTransfer::can_credit(
        &entry(GROUP_BYTES / GROUP_ITEMS as u64),
        &filling,
        Some(&committing),
        &Batch::default()
    ));
    assert!(PageTransfer::can_credit(
        &entry(1024),
        &filling,
        Some(&committing),
        &Batch::default()
    ));
    let large = entry(WINDOW / DECODE_FACTOR + 1);
    assert!(PageTransfer::can_credit(
        &large,
        &VecDeque::new(),
        None,
        &Batch::default()
    ));
    let one = VecDeque::from([Credit {
        entry: large,
        received: false,
    }]);
    assert!(!PageTransfer::can_credit(
        &entry(1024),
        &one,
        None,
        &Batch::default()
    ));
    let batch = Batch {
        items: PAGE - 1,
        ..Batch::default()
    };
    assert!(PageTransfer::can_credit(
        &entry(1024),
        &VecDeque::new(),
        None,
        &batch
    ));
    assert!(!PageTransfer::can_credit(
        &entry(1024),
        &VecDeque::from([Credit {
            entry: entry(1024),
            received: false
        }]),
        None,
        &batch
    ));
}

#[tokio::test]
async fn a_closed_credit_queue_preserves_the_readers_real_failure_kind() {
    let (requests, expected) = mpsc::channel(1);
    drop(expected);
    let (replies, responses) = mpsc::channel(1);
    replies
        .send(Err(Kind::Publication.error("source publication failed")))
        .await
        .unwrap();
    let completed = tokio::spawn(async {});
    let abort = completed.abort_handle();
    completed.await.unwrap();
    let mut reader = Reader {
        requests,
        replies: responses,
        abort,
        active: Arc::default(),
        done: None,
    };
    assert_eq!(
        Kind::of(&reader.expect(Expected::Finish).unwrap_err()),
        Kind::Publication
    );
}

#[test]
fn get_credit_reserves_the_remaining_publication_bytes_before_requesting() {
    let batch = Batch {
        items: 16,
        bytes: BATCH_BYTES - 1024,
        ..Batch::default()
    };
    let filling = VecDeque::from([Credit {
        entry: entry(1024),
        received: false,
    }]);
    assert!(PageTransfer::can_credit(
        &entry(1024),
        &filling,
        None,
        &batch
    ));
    assert!(!PageTransfer::can_credit(
        &entry(1026),
        &filling,
        None,
        &batch
    ));

    let oversized = entry(2 * (BATCH_BYTES + 1));
    assert!(!batch.can_add(0, 0, &oversized));
    assert!(PageTransfer::can_credit(
        &oversized,
        &VecDeque::new(),
        None,
        &Batch::default()
    ));
}

#[test]
fn inventory_sizes_cursors_and_terminal_flags_are_validated_before_credit() {
    let mut page = InventoryPage {
        entries: vec![entry(100)],
        done: true,
    };
    assert!(validate(&page, false, None).is_ok());
    assert!(validate(&page, false, Some("id")).is_err());
    assert!(validate(&page, true, None).is_err());
    page.entries[0].envelope_bytes = 101;
    assert!(validate(&page, false, None).is_err());
    page.entries.clear();
    page.done = false;
    assert!(validate(&page, false, None).is_err());
}
#[test]
fn metadata_rows_are_compact_and_page_cardinality_is_bounded_during_decoding() {
    let row = entry(512);
    let encoded = serde_json::to_string(&row).unwrap();
    assert_eq!(encoded, "[\"id\",\"hash\",512,256]");
    let row: InventoryEntry = serde_json::from_str(&encoded).unwrap();
    assert_eq!(row.frame_bytes, 512);
    for count in [PAGE, PAGE + 1] {
        let page = InventoryPage {
            entries: (0..count).map(|_| entry(512)).collect(),
            done: true,
        };
        let encoded = serde_json::to_vec(&page).unwrap();
        assert_eq!(
            serde_json::from_slice::<InventoryPage>(&encoded).is_ok(),
            count == PAGE
        );
    }
}
