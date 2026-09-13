use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;
use crate::publication::{
    ActiveAttemptCustody, CompositeAttemptProgress, RelationalAttemptProgress,
    SignalAttemptProgress,
};
use crate::recovery::ProductUnpublishedRetentionPosture;

// These isolate the custody owner using the record already registered by real
// preparation. Production phase Drop/unwind is also proved at its own seams.
fn register_attempt(
    attempt: crate::publication::ReservedCompositePublicationAttempt,
) -> ActiveAttemptCustody {
    attempt.into_parts().custody
}

fn register_settlement(settlement: OwnerExecutionSettlement) -> ActiveAttemptCustody {
    let successor = settlement.successor_basis().unwrap().clone();
    let (attempt, progress) = settlement.into_parts();
    let mut custody = register_attempt(attempt);
    custody.record_progress(progress);
    custody.record_successor(successor);
    custody
}

#[test]
fn active_custody_pre_effect_drop_releases_all_reservations() {
    let (_fixture, owner, expected) = setup();
    let custody = register_attempt(prepare_signal(&owner, &expected, None).into_attempt());
    assert_eq!(owner.state.operation.active(), 1);
    assert_eq!(owner.state.recovery.reserved_slots(), 1);
    assert_eq!(owner.recovery_record_count(), 0);
    drop(custody);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(
        owner
            .state
            .retention
            .reserved_in_flight_acquisition_capacity(),
        0
    );
    assert!(owner.recovery_handles().is_empty());
}

#[test]
fn active_custody_unbound_abandonment_keeps_exact_effects_without_acquiring_pins() {
    let (fixture, owner, expected) = setup();
    let baseline = owner.state.retention.active_component_obligation_count();
    let settlement = settled(execute_with_empty_signal(
        &owner,
        prepare_both_owners(&fixture, &owner, &expected, "custody-both"),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    let custody = register_settlement(settlement);
    drop(custody);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(
        owner.state.recovery.metadata_bytes(),
        crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
    );
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert_eq!(record.owner_effect_count(), 2);
    assert_eq!(
        record.successor_basis().unwrap().identity(),
        successor.identity()
    );
    assert!(record.successor_commit().is_none());
    assert_eq!(
        record.live_obligation_count(),
        4,
        "two reserved pins, reserved history capacity, and recovery slot"
    );
    assert_eq!(
        owner.state.history.reserved_len(),
        1,
        "unused history capacity remains owned until explicit cleanup"
    );
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline
    );
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 2);
    assert_eq!(
        owner
            .state
            .branches
            .root_cell()
            .unwrap()
            .atomic_snapshot()
            .commit()
            .identity(),
        expected.selected_commit()
    );
    assert!(
        owner.cleanup_recovery_handle(&handle).is_none(),
        "a live inspection owns a catalog view"
    );
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert!(
        owner.cleanup_recovery_handle(&handle).is_none(),
        "cleanup cannot release twice"
    );
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline
    );
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
}

#[test]
fn active_custody_resource_lease_restores_bound_pins_on_unwind() {
    let (_fixture, owner, expected) = setup();
    let baseline = owner.state.retention.active_component_obligation_count();
    let settlement = settled(execute_with_empty_signal(
        &owner,
        prepare_signal(&owner, &expected, None),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    let mut custody = register_settlement(settlement);
    let unwound = catch_unwind(AssertUnwindSafe(|| {
        custody.bind_publication_pins(&successor).unwrap();
        // An invalid second binding unwinds while the resources are leased.
        // The lease must restore the first binding before caller Drop runs.
        custody.bind_publication_pins(expected.basis()).unwrap();
    }));
    assert!(unwound.is_err());
    drop(custody);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::PublicationPinsRetained
    );
    assert_eq!(
        record.successor_basis().unwrap().identity(),
        successor.identity()
    );
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline + 4
    );
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline
    );
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.recovery_record_count(), 0);
}

#[test]
fn active_custody_concurrent_inspection_materializes_one_charged_record() {
    let (_fixture, owner, expected) = setup();
    let custody = register_settlement(settled(execute_with_empty_signal(
        &owner,
        prepare_signal(&owner, &expected, None),
    )));
    drop(custody);
    let handle = owner.recovery_handles().pop().unwrap();
    let views = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| owner.inspect_recovery(&handle).unwrap()))
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(views
        .iter()
        .all(|view| view.identity() == handle.identity()));
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(
        owner.state.recovery.metadata_bytes(),
        crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
    );
    drop(views);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
}

#[test]
fn active_custody_identity_repair_settles_only_the_recorded_relational_commit() {
    let (fixture, owner, expected) = setup();
    let mut attempt =
        prepare_relational(&fixture, &owner, &expected, "custody-identity-repair").into_attempt();
    attempt.begin_owner_execution();
    let candidate = attempt.take_relational_candidate().unwrap();
    let mut custody = register_attempt(attempt);
    let performed = match owner
        .state
        .relational
        .publication_port()
        .compare_and_publish(candidate)
    {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => {
            performed
        }
        other => panic!("real Relational publication must perform: {other:?}"),
    };
    let commit_identity = performed.commit_identity();
    custody.record_progress(CompositeAttemptProgress::new(
        RelationalAttemptProgress::settlement_required(
            commit_identity.clone(),
            performed.next_basis().clone(),
        ),
        SignalAttemptProgress::untouched(),
    ));
    drop(performed);
    drop(custody);
    let handle = owner.recovery_handles().pop().unwrap();
    assert!(owner.cleanup_recovery_handle(&handle).is_none());
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::SettlementRequired
    );
    assert!(
        record.successor_basis().is_none(),
        "no composite basis was admitted before caller loss"
    );
    crate::lifecycle::RuntimeWorldRecoveryService::continue_effects(owner.as_ref(), record)
        .unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(record.owner_effect_count(), 1);
    assert_eq!(
        record.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_eq!(
        owner
            .state
            .branches
            .root_cell()
            .unwrap()
            .atomic_snapshot()
            .commit()
            .identity(),
        expected.selected_commit()
    );
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
}

#[test]
fn active_custody_close_counts_reserved_history_before_any_inspection() {
    let (_fixture, owner, expected) = setup();
    let custody = register_settlement(settled(execute_with_empty_signal(
        &owner,
        prepare_signal(&owner, &expected, None),
    )));
    drop(custody);
    let report = owner
        .close()
        .expect("complete abandonment releases close admission");
    let rows = report.retained_records();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(rows[0].live_component_obligations(), 2);
    assert_eq!(rows[0].live_composite_obligations(), 2);
    assert_eq!(owner.state.history.reserved_len(), 1);
    let handle = owner.recovery_handles().pop().unwrap();
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
    assert_eq!(owner.recovery_record_count(), 0);
}
