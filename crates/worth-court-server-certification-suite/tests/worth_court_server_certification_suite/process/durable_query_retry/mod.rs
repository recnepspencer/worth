//! Public Server calls use the same genuine Home application retry owner.
mod child_reexecution;
mod runtime;
use runtime::*;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;

#[test]
fn restarted_server_returns_the_original_receipt_after_later_history() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(contacts.clone());
    let first_server = server(home.runtime());
    let live = match execute(&first_server, "original-key", 8, "request-1").unwrap() {
        Outcome::Committed {
            receipt,
            result: Ok(value),
        } => {
            assert_eq!(value, serde_json::json!(8));
            receipt
        }
        other => panic!("fresh Server mutation commits: {other:?}"),
    };
    let original = match execute(&first_server, "original-key", 8, "request-2").unwrap() {
        Outcome::RecoveredCommitted(original) => original,
        other => panic!(
            "Query reconstructs the original, including changed request decoration: {other:?}"
        ),
    };
    assert_eq!(live.commit_reference(), original.commit_reference());
    assert_eq!(live.changed_record_count(), original.changed_record_count());
    assert!(matches!(
        execute(&first_server, "later-key", 9, "request-3").unwrap(),
        Outcome::Committed { .. }
    ));
    drop(first_server);
    let restarted = server(home.runtime());
    let retry = match execute(&restarted, "original-key", 8, "request-4").unwrap() {
        Outcome::RecoveredCommitted(original) => original,
        other => panic!("a new Server has no process receipt authority: {other:?}"),
    };
    assert_eq!(retry, original);
    assert_eq!(balance(home.runtime()), 9);
    assert_contacts(&contacts, 2);
}

#[test]
fn server_changed_intent_refuses_without_reentering_the_handler() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(contacts.clone());
    let server = server(home.runtime());
    assert!(matches!(
        execute(&server, "one-key", 8, "request-1").unwrap(),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        execute(&server, "one-key", 10, "request-2").unwrap(),
        Outcome::IdempotencyIntentDrift
    ));
    assert_contacts(&contacts, 1);
    assert_eq!(balance(home.runtime()), 8);
}

#[test]
fn concurrent_server_identical_keys_publish_one_original_occurrence() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = open(contacts.clone());
    let server = server(home.runtime());
    let before = home
        .runtime()
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits();
    let fresh = std::sync::atomic::AtomicUsize::new(0);
    let receipts = std::thread::scope(|scope| {
        (0..4)
            .map(|_| {
                scope.spawn(|| {
                    match execute(&server, "concurrent-key", 8, "same-request").unwrap() {
                        Outcome::Committed { .. } => {
                            fresh.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            match execute(&server, "concurrent-key", 8, "fresh-response").unwrap() {
                                Outcome::RecoveredCommitted(receipt) => receipt,
                                other => panic!(
                                    "the winning caller reads its durable original: {other:?}"
                                ),
                            }
                        }
                        Outcome::RecoveredCommitted(receipt) => receipt,
                        other => panic!("identical callers share one Query occurrence: {other:?}"),
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    for receipt in &receipts[1..] {
        assert_eq!(receipt, &receipts[0]);
    }
    assert_eq!(fresh.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(
        home.runtime()
            .runtime()
            .world_history_snapshot_for_test()
            .installed_commits(),
        before + 1
    );
    let prepared = contacts.load(std::sync::atomic::Ordering::SeqCst);
    let Outcome::RecoveredCommitted(replay) =
        execute(&server, "concurrent-key", 8, "sequential-replay").unwrap()
    else {
        panic!("sequential Server retry reads its original");
    };
    assert_eq!(replay, receipts[0]);
    assert_contacts(&contacts, prepared);
    assert_eq!(balance(home.runtime()), 8);
}
