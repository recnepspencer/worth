use std::sync::{mpsc, Arc};
use std::time::Duration;

use crate::lifecycle::{
    RuntimeWorldCloseDenial, RuntimeWorldOwnerLifecycleObservation, RuntimeWorldRecoveryService,
};
use crate::publication::{
    CompositeAttemptProgress, RelationalAttemptProgress, SignalAttemptProgress,
};
use crate::recovery::ProductUnpublishedNextAction;

use super::settlement_catalog_tests::{relational_attempt, setup, successor_basis};

const RECOVERY_CLOSE_TEST_TIMEOUT: Duration = Duration::from_secs(2);

#[test]
fn updating_recovery_blocks_close_until_cleanup() {
    let (fixture, owner, expected) = setup();
    let mut attempt = relational_attempt(&fixture, &owner, expected.clone());
    attempt.begin_owner_execution();

    let performed = fixture.perform_relational_owner_change();
    let successor_basis = successor_basis(&owner, &expected, performed.next_basis().clone(), None);
    let retained = attempt
        .settle(CompositeAttemptProgress::new(
            RelationalAttemptProgress::performed(performed),
            SignalAttemptProgress::untouched(),
        ))
        .ready(successor_basis)
        .expect_err("unsettled Relational work enters retained recovery");
    let handle = retained.recovery_handle();
    drop(retained);

    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.recovery_handles(), vec![handle.clone()]);
    assert!(owner.inspect_recovery(&handle).is_some());
    // SPEC-P4-008: an installed record no longer denies close; it is exposed in
    // the terminal report. Closing here would end the world this proof still
    // needs, so the installed-record exposure is pinned by
    // `close_exposes_every_retained_record_in_its_terminal_report`. The
    // updating-record denial below is the half that survives.
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );

    let (reached_tx, reached_rx) = mpsc::sync_channel(1);
    let pause = super::install_test_recovery_update_pause(handle.clone(), reached_tx);
    let effects = owner
        .inspect_recovery(&handle)
        .expect("installed recovery supplies the continuation capability");
    let worker_owner = Arc::clone(&owner);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let outcome = RuntimeWorldRecoveryService::continue_effects(worker_owner.as_ref(), effects);
        finished_tx
            .send(outcome)
            .expect("the recovery proof still owns its completion receiver");
    });

    reached_rx
        .recv_timeout(RECOVERY_CLOSE_TEST_TIMEOUT)
        .expect("recovery reaches the catalog-update boundary");
    assert_updating_record_denies_close_but_stays_named(&owner, &handle);

    drop(pause);
    let continuation = finished_rx
        .recv_timeout(RECOVERY_CLOSE_TEST_TIMEOUT)
        .expect("recovery settlement finishes after the update boundary is released")
        .expect("the real Relational settlement authority completes recovery");
    worker.join().expect("recovery worker does not panic");
    assert!(!continuation
        .actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    assert_settled_record_closes_only_after_cleanup(&owner, &handle);
}

/// A record inside its own catalog update is still the owner's obligation: it
/// stays counted and named by identity, it is deliberately not inspectable
/// while it is being rewritten, and it is the one thing close still refuses,
/// because no report row could honestly describe a record mid-rewrite.
fn assert_updating_record_denies_close_but_stays_named(
    owner: &super::settlement_catalog_tests::TestOwner,
    handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
) {
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.recovery_handles(), vec![handle.clone()]);
    assert!(
        owner.inspect_recovery(handle).is_none(),
        "the updating record is absent from the installed map but remains reported by identity"
    );
    assert_eq!(owner.state.operation.active(), 1);
    assert_eq!(
        owner
            .state
            .operation
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .recovery_active,
        1
    );
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert_eq!(
        owner
            .close()
            .expect_err("an updating recovery record denies close"),
        RuntimeWorldCloseDenial::InFlightCriticalSection
    );
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open,
        "close denial must not enter Closing while recovery updates"
    );
}

/// Once the update completes the record is installed and settled again, and
/// only an explicit cleanup removes the close obligation it represents.
fn assert_settled_record_closes_only_after_cleanup(
    owner: &super::settlement_catalog_tests::TestOwner,
    handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
) {
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    let inspected = owner
        .inspect_recovery(handle)
        .expect("settled recovery remains installed until cleanup");
    assert_eq!(
        inspected.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::Settled
    );
    drop(inspected);

    assert!(owner.cleanup_recovery_handle(handle).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
    let _report = owner
        .close()
        .expect("cleanup removes the final recovery close obligation");
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
}
