//! Proofs for the terminal close report.
//!
//! Close settles what it can, exposes every retained owner obligation it
//! cannot, and denies only a critical section that is still in flight.

use std::sync::Arc;

use crate::lifecycle::{RuntimeWorldCloseDenial, RuntimeWorldOwnerLifecycleObservation};
use crate::publication::CompositeLateCancellationPosture;
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedNextAction};

use super::admission_race::{
    wait_until_close_holds_operation_admission, wait_until_close_is_admitting,
};
use super::product_cas_loss::{
    resolve_one_race, resolve_one_race_after_a_completed_publication, ResolvedRace,
};
use super::publication::TestOwner;

/// SPEC-P4-008. A live `ProductUnpublishedOwnerEffects` record does not refuse
/// close; it becomes a report row naming its identity, cause, live obligation
/// counts, and derived next actions, and it survives the close that named it.
#[test]
fn close_exposes_every_retained_record_in_its_terminal_report() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);
    let owner = race.owner;
    let handle = race.retained.recovery_handle();
    let expected_actions = race.retained.next_actions().to_vec();
    let expected_identity = race.retained.identity().clone();
    let expected_obligations = race.retained.live_obligation_count();
    assert_eq!(owner.recovery_record_count(), 1);

    let report = owner
        .close()
        .expect("a retained record is exposed by close, never refused");

    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        report.retained_records().len(),
        1,
        "close names every retained record exactly once"
    );
    let row = &report.retained_records()[0];
    assert_eq!(row.identity(), &expected_identity);
    assert_eq!(row.cause(), ProductUnpublishedCause::ProductPublicationLost);
    assert_eq!(
        row.live_component_obligations(),
        2,
        "the retained record still holds its exact relational and signal pins"
    );
    assert_eq!(
        row.live_component_obligations() + row.live_composite_obligations(),
        expected_obligations,
        "the report's split must sum to the record's own live obligation count"
    );
    assert_eq!(row.next_actions(), expected_actions.as_slice());
    assert!(row
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ReleaseObligations));
    assert!(!expected_actions.contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    assert_eq!(
        report.settled_records(),
        0,
        "an enumerated record is exposure, never settlement: close settled none"
    );

    assert_eq!(
        owner.recovery_record_count(),
        1,
        "exposure is never a discarded owner obligation"
    );
    assert!(
        owner.inspect_recovery(&handle).is_some(),
        "the named record is still inspectable after the close that named it"
    );
}

/// Close preserves exact pins held by retained history and partial records.
/// A completed publication does not make its predecessor history reclaimable.
#[test]
fn close_preserves_history_and_retained_record_component_pins() {
    let race = resolve_one_race_after_a_completed_publication(
        CompositeLateCancellationPosture::NotRequested,
    );
    let owner = race.owner;
    let handle = race.retained.recovery_handle();
    let obligations_before = owner.state.retention.active_component_obligation_count();
    let pins_before = owner.state.retention.unique_pin_count();

    let report = owner
        .close()
        .expect("a retained record is exposed by close, never refused");
    let pins_after = owner.state.retention.unique_pin_count();

    assert_eq!(
        report.released_unique_component_pins(),
        0,
        "retained predecessor history still owns the exact relational pin"
    );
    assert_eq!(
        report.released_unique_component_pins(),
        pins_before - pins_after,
        "the report counts the pins close actually released"
    );
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        obligations_before,
        "reclamation removes only released entries, never a pin the retained record still owes"
    );
    assert_eq!(
        report.retained_records()[0].live_component_obligations(),
        2,
        "the retained record still reports the exact pins it kept"
    );
    assert!(
        owner.inspect_recovery(&handle).is_some(),
        "the retained record survives the reclamation that skipped its pins"
    );
}

/// CLS-001. A pre-movement CAS loser released its history slot but still
/// occupies its recovery slot, so its row reports the exact pin pair as the
/// component half and that slot as the composite half. Both halves are the
/// record's own count read by scope, never a component charge subtracted from
/// a total that counted the pins differently.
#[test]
fn close_reports_a_pre_movement_loser_as_its_pin_pair_plus_its_recovery_slot() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);
    assert_eq!(race.retained.successor_commit(), None);
    assert_eq!(race.retained.live_obligation_count(), 3);

    let report = race
        .owner
        .close()
        .expect("a retained record is exposed by close, never refused");

    let row = &report.retained_records()[0];
    assert_eq!(
        row.live_component_obligations(),
        2,
        "the loser still holds its exact relational and signal pins"
    );
    assert_eq!(
        row.live_composite_obligations(),
        1,
        "the loser installed no successor, so the recovery slot it occupies is its one composite obligation"
    );
}

/// REV-D-003. The ledger check and the Open -> Closing flip are one window: a
/// reservation offered while close is draining must not be admitted, because no
/// report row could name it.
#[test]
fn close_admits_no_operation_between_its_ledger_check_and_its_state_flip() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);
    let owner = race.owner;

    // Park a close inside its own window: it takes the operation ledger, drains,
    // then blocks on the close contract this thread holds. Nothing waits on this
    // thread, so both gates are released below without a cycle.
    let ledger_gate = owner
        .state
        .operation
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let closing = spawn_close(&owner);
    wait_until_close_is_admitting(&owner);
    let contract_gate = owner
        .state
        .close
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    drop(ledger_gate);
    wait_until_close_holds_operation_admission(&owner);

    let reserving = spawn_recovery_reservation(&owner);
    drop(contract_gate);
    let report = closing
        .join()
        .expect("the close thread must not panic")
        .expect("a retained record is exposed by close, never refused");
    let admitted = reserving
        .join()
        .expect("the reserving thread must not panic");

    assert!(
        !admitted,
        "no operation may be admitted between close's ledger check and its state flip"
    );
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        report.retained_records().len(),
        1,
        "holding admission across the drain still reports every retained record"
    );
}

/// REV-D-006. A record in the `ReacquisitionPending` posture holds no issued
/// pins but does hold a reserved component pin pair, so its component half is a
/// pair and its composite half is the recovery slot it occupies plus the
/// successor history protection it still owes. The split is read from the
/// record, never inferred from the retained posture's shape.
#[cfg(feature = "test-operation-control")]
#[test]
fn close_reports_a_pending_record_as_a_reserved_component_pair() {
    let pending = super::publication::resolve_post_effect_retention_denial();
    assert_eq!(
        pending.retained.retention_posture(),
        crate::recovery::ProductUnpublishedRetentionPosture::ReacquisitionPending
    );
    assert_eq!(
        pending.owner.state.retention.reserved_unique_pin_capacity(),
        2,
        "the pending record holds a reserved pin pair, not issued pins"
    );

    let report = pending
        .owner
        .close()
        .expect("a pending retained record is exposed by close, never refused");

    let row = &report.retained_records()[0];
    assert_eq!(
        row.live_component_obligations(),
        2,
        "the reserved component pin pair is a component-scoped charge"
    );
    assert_eq!(
        row.live_composite_obligations(),
        2,
        "the composite half is the recovery slot and the installed successor history protection"
    );
    assert_eq!(
        row.live_component_obligations() + row.live_composite_obligations(),
        pending.retained.live_obligation_count(),
        "the report's split must sum to the record's own live obligation count"
    );
}

/// An operation parked inside its critical section is the only thing left that
/// close refuses. The retained record it leaves behind is enumerated instead.
#[test]
fn close_denies_only_an_undrainable_critical_section() {
    let race = resolve_one_race(CompositeLateCancellationPosture::NotRequested);
    let ResolvedRace {
        owner, retained, ..
    } = race;

    let parked = owner
        .reserve_recovery_operation_if_open_and_bootstrapped()
        .expect("an open bootstrapped owner admits a recovery critical section");
    assert_eq!(
        owner
            .close()
            .expect_err("a live critical section cannot be drained"),
        RuntimeWorldCloseDenial::InFlightCriticalSection
    );
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open,
        "a denied close must not enter Closing"
    );

    drop(parked);
    let report = owner
        .close()
        .expect("close succeeds once the critical section releases");
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        report.retained_records().len(),
        1,
        "the record that was never a denial is still a report row"
    );
    assert_eq!(report.retained_records()[0].identity(), retained.identity());
}

fn spawn_close(
    owner: &Arc<TestOwner>,
) -> std::thread::JoinHandle<
    Result<crate::lifecycle::RuntimeWorldCloseReport, RuntimeWorldCloseDenial>,
> {
    let owner = Arc::clone(owner);
    std::thread::spawn(move || owner.close())
}

fn spawn_recovery_reservation(owner: &Arc<TestOwner>) -> std::thread::JoinHandle<bool> {
    let owner = Arc::clone(owner);
    std::thread::spawn(move || {
        owner
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .is_ok()
    })
}
