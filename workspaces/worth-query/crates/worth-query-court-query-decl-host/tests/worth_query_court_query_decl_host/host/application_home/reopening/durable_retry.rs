//! Filesystem retries compare original public receipt facts after actual process death.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;

mod process;
mod receipt_fields;

#[test]
fn lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint() {
    process::journey(false);
}

#[test]
fn lost_response_kill_reopen_retry_preserves_receipt_after_checkpoint() {
    process::journey(true);
}

#[test]
fn concurrent_identical_directory_keys_publish_one_original_receipt() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = super::super::pending_support::open(execution, contacts.clone());
    let receipts = std::thread::scope(|threads| {
        let threads = (0..4)
            .map(|_| {
                let runtime = home.runtime();
                threads.spawn(move || {
                    let (_, live) = scope();
                    match set(runtime, &live, 0x7265747279, 8).unwrap() {
                        Outcome::Committed { receipt, result: 8 }
                        | Outcome::AlreadyCommitted(receipt) => receipt,
                        other => {
                            panic!("identical key must resolve original publication: {other:?}")
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        threads
            .into_iter()
            .map(|thread| thread.join().expect("concurrent retry worker"))
            .collect::<Vec<_>>()
    });
    for receipt in &receipts[1..] {
        assert!(receipt.is_same_authoritative_commit(&receipts[0]));
        assert_eq!(
            receipt_fields::projection(receipt),
            receipt_fields::projection(&receipts[0])
        );
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 8);
    drop(receipts);
    drop(home);
}

#[test]
fn changed_intent_refuses_after_later_directory_publications() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = super::super::pending_support::open(execution, contacts.clone());
    let (_, live) = scope();
    let original = match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::Committed { receipt, result: 8 } => receipt,
        other => panic!("first key must publish: {other:?}"),
    };
    assert!(matches!(
        set(home.runtime(), &live, 0x726574727a, 9).unwrap(),
        Outcome::Committed { result: 9, .. }
    ));
    assert!(matches!(
        set(home.runtime(), &live, 0x7265747279, 10).unwrap(),
        Outcome::IdempotencyIntentDrift
    ));
    match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::AlreadyCommitted(receipt) => {
            assert_eq!(
                receipt_fields::projection(&receipt),
                receipt_fields::projection(&original)
            );
        }
        other => panic!("later history cannot replace the original receipt: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(balance(home.runtime(), &live), 9);
    drop(original);
    drop(home);
}
