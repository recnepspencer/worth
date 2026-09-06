use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

#[test]
fn unwind_after_materialization_preserves_head_custody_without_claiming_movement() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "materialized-unwind");
    let baseline_pins = owner.state.retention.active_component_obligation_count();
    let _injection = crate::branch::publication_unwind::arm_materialized();
    assert!(catch_unwind(AssertUnwindSafe(
        || ready.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    ))
    .is_err());
    assert_eq!(cell.atomic_snapshot(), *expected.snapshot());
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.history.len(), 2);
    assert_eq!(owner.recovery_record_count(), 1);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(
        record.retention_posture(),
        crate::recovery::ProductUnpublishedRetentionPosture::ProductHeadPinsRetained
    );
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(record.owner_effect_count(), 1);
    let commit = record.successor_commit().unwrap().clone();
    assert!(owner.state.history.lookup(&commit).is_some());
    assert!(owner
        .recover_performed_publication(&commit)
        .unwrap()
        .is_none());
    assert_eq!(
        record.live_obligation_count(),
        4,
        "delivery's additional protection has already released"
    );
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        baseline_pins
    );
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    let reclaimed = owner
        .state
        .history
        .reclaim_batch(crate::history::CompositeHistoryReclamationRequest::new(
            owner.owner_identity(),
            vec![commit.clone()],
            1,
        ))
        .unwrap();
    assert_eq!(reclaimed.reclaimed_commits(), &[commit]);
    assert_eq!(owner.state.history.len(), 1);
}

#[test]
fn unwind_after_actual_cas_recovers_the_original_full_performed_facts() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "committed-unwind");
    let attempt = ready.attempt_identity().clone();
    let _injection = crate::branch::publication_unwind::arm();
    let result = catch_unwind(AssertUnwindSafe(|| {
        ready.publish(
            &cell,
            CompositeLateCancellationPosture::RequestedAfterProductMovement,
        )
    }));
    assert!(
        result.is_err(),
        "the actual committed boundary injected the unwind"
    );
    let actual = cell.atomic_snapshot();
    assert_ne!(&actual, expected.snapshot());
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 0);
    let performed = owner
        .recover_performed_publication(actual.selected_commit())
        .unwrap()
        .unwrap();
    assert_eq!(performed.attempt_identity(), &attempt);
    assert_eq!(performed.old_product_head(), expected.snapshot());
    assert_eq!(performed.new_product_head(), &actual);
    assert!(performed
        .component_results()
        .relational_commit_result()
        .is_some());
    assert_eq!(
        performed
            .component_results()
            .relational_publication_identity()
            .as_ref(),
        actual.commit().relational_publication_identity()
    );
    assert_eq!(
        performed.late_cancellation(),
        CompositeLateCancellationPosture::RequestedAfterProductMovement
    );
    let counters = performed.cost_counters();
    assert_eq!(counters.cas_attempts(), 1);
    assert_eq!(counters.cas_wins(), 1);
    assert_eq!(counters.cas_losses(), 0);
    assert_eq!(counters.history_slots_installed(), 1);
    assert_eq!(counters.cancellation_observations(), 1);
    assert!(owner
        .recover_performed_publication(actual.selected_commit())
        .unwrap()
        .is_none());
    drop(performed);
    let recovered = owner
        .recover_performed_publication(actual.selected_commit())
        .unwrap()
        .unwrap();
    assert_eq!(recovered.cost_counters(), counters);
    assert_eq!(recovered.new_product_head(), &actual);
    recovered.consume();
    assert!(owner
        .recover_performed_publication(actual.selected_commit())
        .unwrap()
        .is_none());
    assert_eq!(cell.atomic_snapshot(), actual);
}

#[test]
fn caller_drop_and_concurrent_recovery_share_one_delivery_lane() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "exclusive-delivery");
    let RuntimeWorldPublicationOutcome::Performed(performed) =
        ready.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    else {
        panic!("CAS must perform")
    };
    let identity = performed.commit().identity().clone();
    let before_metadata = owner.state.history.metadata_ledger();
    assert!(owner
        .recover_performed_publication(&identity)
        .unwrap()
        .is_none());
    drop(performed);
    let barrier = std::sync::Barrier::new(4);
    let winners = std::thread::scope(|scope| {
        let barrier = &barrier;
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let owner = &owner;
                let identity = &identity;
                scope.spawn(move || {
                    barrier.wait();
                    owner.recover_performed_publication(identity).unwrap()
                })
            })
            .collect();
        threads
            .into_iter()
            .filter_map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        winners.len(),
        1,
        "successful claims stay live across all contenders"
    );
    assert_eq!(owner.state.history.metadata_ledger(), before_metadata);
    assert_eq!(cell.atomic_snapshot().selected_commit(), &identity);
    for winner in winners {
        winner.consume();
    }
    assert!(owner
        .recover_performed_publication(&identity)
        .unwrap()
        .is_none());
}

#[test]
fn delivery_protects_retired_history_and_releases_its_envelope_charge_once() {
    use crate::history::CompositeHistoryReclamationRequest;
    use crate::lifecycle::RuntimeWorldBranchService;
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().unwrap();
    let ready = ready_relational(&fixture, &owner, &expected, "delivery-reclamation");
    let RuntimeWorldPublicationOutcome::Performed(performed) =
        ready.publish(&cell, CompositeLateCancellationPosture::NotRequested)
    else {
        panic!("CAS must perform")
    };
    let identity = performed.commit().identity().clone();
    let root = expected.selected_commit().clone();
    assert!(owner
        .retire_product_branch(&expected)
        .unwrap()
        .owner_retirement_work()
        .is_empty());
    drop(cell);
    drop(expected);
    let request = || {
        CompositeHistoryReclamationRequest::new(
            owner.owner_identity(),
            vec![identity.clone(), root.clone()],
            2,
        )
    };
    let before = owner.state.history.metadata_ledger();
    let protected = owner.state.history.reclaim_batch(request()).unwrap();
    assert_eq!(protected.skipped_protected(), 1);
    assert_eq!(protected.skipped_with_descendant_dependencies(), 1);
    assert_eq!(owner.state.history.metadata_ledger(), before);
    drop(performed);
    let reclaimed = owner.state.history.reclaim_batch(request()).unwrap();
    assert_eq!(reclaimed.reclaimed_commits(), &[identity.clone(), root]);
    assert_eq!(owner.state.history.metadata_ledger().total_occupancy(), 0);
    assert!(owner
        .recover_performed_publication(&identity)
        .unwrap()
        .is_none());
}
