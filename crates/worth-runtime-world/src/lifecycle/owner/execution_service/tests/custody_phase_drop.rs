use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;
use crate::recovery::ProductUnpublishedRetentionPosture;

#[test]
fn settlement_drop_retains_both_real_owner_results_without_binding_pins() {
    let (fixture, owner, expected) = setup();
    let baseline = owner.state.retention.active_component_obligation_count();
    let settlement = settled(execute_with_empty_signal(
        &owner,
        prepare_both_owners(&fixture, &owner, &expected, "settlement-drop"),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    let (_, results) = settlement.progress().ready_results().unwrap();
    let relational = results.relational_publication_identity().unwrap();
    let signal = results.signal_publication_identity().unwrap();
    drop(settlement);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(record.owner_effect_count(), 2);
    assert_eq!(
        record.component_results().relational_publication_identity(),
        Some(relational)
    );
    assert_eq!(
        record.component_results().signal_publication_identity(),
        Some(signal)
    );
    assert!(record
        .component_results()
        .relational_commit_result()
        .is_some());
    assert_eq!(record.successor_basis(), Some(&successor));
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert!(record.successor_commit().is_none());
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline
    );
    assert_eq!(owner.state.history.reserved_len(), 1);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 2);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(
        owner.state.branches.root_cell().unwrap().atomic_snapshot(),
        *expected.snapshot()
    );
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
}

#[test]
fn actual_signal_apply_unwind_keeps_the_already_settled_relational_effect() {
    let (fixture, owner, expected) = setup();
    let baseline = owner.state.retention.active_component_obligation_count();
    let prepared = prepare_both_owners(&fixture, &owner, &expected, "signal-apply-unwind");
    let cancellation = RuntimeWorldCancellationSource::new();
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            prepared,
            &mut (),
            &cancellation.token(),
            |_| panic!("the real Signal apply callback unwinds before Signal movement"),
        )
    }));
    assert!(unwind.is_err());
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_retains_only_the_relational_effect(&record, ProductUnpublishedCause::CallerAbandoned);
    let result = record.component_results();
    assert!(result.relational_publication_identity().is_some());
    assert!(result.relational_commit_result().is_some());
    assert!(result.signal_publication_identity().is_none());
    assert!(record.successor_basis().is_none());
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert_eq!(
        owner.state.branches.root_cell().unwrap().atomic_snapshot(),
        *expected.snapshot()
    );
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline
    );
    assert_eq!(owner.state.operation.active(), 0);
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
}

#[test]
fn invalid_ready_basis_unwind_preserves_the_original_exact_successor() {
    let (fixture, owner, expected) = setup();
    let settlement = settled(execute_without_signal(
        &owner,
        prepare_relational(&fixture, &owner, &expected, "invalid-ready-basis"),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    let (_, evidence) = settlement.progress().ready_results().unwrap();
    let identity = evidence.relational_publication_identity().unwrap();
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        settlement.ready(expected.basis().clone())
    }));
    assert!(unwind.is_err());
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_retains_only_the_relational_effect(&record, ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(record.successor_basis(), Some(&successor));
    assert_eq!(
        record.component_results().relational_publication_identity(),
        Some(identity)
    );
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert!(record.successor_commit().is_none());
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.operation.active(), 0);
}
