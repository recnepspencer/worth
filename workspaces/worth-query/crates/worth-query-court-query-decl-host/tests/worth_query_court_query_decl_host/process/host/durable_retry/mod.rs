//! Filesystem retries compare original public receipt facts after actual process death.
use crate::host::application_home::{pending_support, reopening::opening_failed};
use pending_support::{balance, scope, set};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryAdvancementDenial,
    },
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::ApplicationBuildIdentity,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};

mod affinity;
mod child_reexecution;
mod fate_reopening;
mod receipt_fields;
mod request_stops;

#[test]
fn lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint() {
    child_reexecution::journey(false);
}

#[test]
fn lost_response_kill_reopen_retry_preserves_receipt_after_checkpoint() {
    child_reexecution::journey(true);
}

#[test]
fn concurrent_identical_directory_keys_publish_one_original_receipt() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = pending_support::open(execution, contacts.clone());
    let before = home
        .runtime()
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits();
    let fresh = AtomicUsize::new(0);
    let receipts = std::thread::scope(|threads| {
        let threads = (0..4)
            .map(|_| {
                let runtime = home.runtime();
                let fresh = &fresh;
                threads.spawn(move || {
                    let (_, live) = scope();
                    match set(runtime, &live, 0x7265747279, 8).unwrap() {
                        Outcome::Committed { receipt, result: 8 } => {
                            fresh.fetch_add(1, Ordering::SeqCst);
                            let recorded = match set(runtime, &live, 0x7265747279, 8).unwrap() {
                                Outcome::RecoveredCommitted(recorded) => recorded,
                                other => {
                                    panic!("fresh caller reads the durable original: {other:?}")
                                }
                            };
                            receipt_fields::assert_live_agrees(&receipt, &recorded);
                            recorded
                        }
                        Outcome::RecoveredCommitted(receipt) => receipt,
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
        assert_eq!(receipt, &receipts[0]);
        assert_eq!(
            receipt_fields::projection(receipt),
            receipt_fields::projection(&receipts[0])
        );
    }
    assert_eq!(fresh.load(Ordering::SeqCst), 1);
    assert_eq!(
        home.runtime()
            .runtime()
            .world_history_snapshot_for_test()
            .installed_commits(),
        before + 1
    );
    // Concurrent unseen requests may prepare pure candidates (the declaration
    // promises no external effect). The branch commit lane publishes one.
    let prepared = contacts.load(Ordering::SeqCst);
    let (_, replay_scope) = scope();
    let Outcome::RecoveredCommitted(replay) =
        set(home.runtime(), &replay_scope, 0x7265747279, 8).unwrap()
    else {
        panic!("sequential retry reads the existing original");
    };
    assert_eq!(replay, receipts[0]);
    assert_eq!(contacts.load(Ordering::SeqCst), prepared);
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
    let home = pending_support::open(execution, contacts.clone());
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
    let recorded = match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::RecoveredCommitted(recorded) => recorded,
        other => panic!("the original source must remain readable: {other:?}"),
    };
    receipt_fields::assert_live_agrees(&original, &recorded);
    match set(home.runtime(), &live, 0x7265747279, 8).unwrap() {
        Outcome::RecoveredCommitted(receipt) => {
            assert_eq!(
                receipt_fields::projection(&receipt),
                receipt_fields::projection(&recorded)
            );
        }
        other => panic!("later history cannot replace the original receipt: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(balance(home.runtime(), &live), 9);
    drop(original);
    drop(home);
}
