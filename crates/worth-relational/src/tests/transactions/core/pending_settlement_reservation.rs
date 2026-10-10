use std::sync::{Arc, Barrier};

use crate::facade::history::BranchId;
use crate::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalInterruptionBoundary, RelationalOperationControl,
    RelationalOperationInterruption, RelationalTransactionIntent,
};
use crate::tests::support::*;

/// The pending-settlement record is installed before the branch reference can
/// move, so a stale attempt must give back the exact reservation it installed.
#[test]
fn stale_publication_releases_its_pending_settlement_reservation() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "stale-reservation-anchor");
    let loser = prepared_main_write(&runtime, "stale-loser");
    let winner = prepared_main_write(&runtime, "stale-winner");

    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(winner)
    else {
        panic!("the first publisher from a shared basis performs");
    };
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        1,
        "movement is reported only after its settlement record exists"
    );
    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the winner settles");
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);

    let reference_before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let contacts_before = runtime
        .publication_binding()
        .pending_settlement_contact_count();
    let crate::mvcc::RelationalPublicationOutcome::Stale(_) =
        runtime.publication_port().compare_and_publish(loser)
    else {
        panic!("the second publisher from the same basis is stale");
    };

    assert_eq!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        reference_before,
        "a stale attempt performs no movement"
    );
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        0,
        "a stale attempt releases the exact reservation it installed"
    );
    assert!(
        runtime
            .publication_binding()
            .pending_settlement_contact_count()
            > contacts_before,
        "the stale attempt did reach the registry before releasing"
    );
    release_test_commit_snapshot(&runtime, &settled);
}

/// Interruption between the pre-effect reservation and the linearization point
/// is a no-movement path, so it must not retain settlement state either.
#[test]
fn interrupted_publication_releases_its_pending_settlement_reservation() {
    for interruption in [
        RelationalOperationInterruption::Cancelled,
        RelationalOperationInterruption::TimedOut,
    ] {
        let runtime = runtime_with_test_schema();
        create_entity(&runtime, "interrupted-reservation-anchor");
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let control = RelationalOperationControl::uninterrupted().with_injected_interruption(
            RelationalInterruptionBoundary::BeforeCriticalSection,
            interruption,
            1,
        );
        let mut transaction = runtime
            .begin_branch_transaction_with_control(
                &basis,
                RelationalTransactionIntent::ordinary(),
                control,
            )
            .unwrap();
        transaction
            .push_batch(
                batch_create("interrupted-before-critical-section"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let candidate = runtime
            .prepare_branch_transaction(
                transaction,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let reference_before = runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap();

        let crate::mvcc::RelationalPublicationOutcome::Interrupted(event) =
            runtime.publication_port().compare_and_publish(candidate)
        else {
            panic!("an injected critical-section interruption must not move the reference");
        };

        assert_eq!(event.interruption(), interruption);
        assert_eq!(
            event.boundary(),
            RelationalInterruptionBoundary::BeforeCriticalSection
        );
        assert_eq!(
            runtime
                .branch_reference_state(&BranchId("main".to_owned()))
                .unwrap(),
            reference_before,
        );
        assert_eq!(
            runtime.publication_binding().pending_settlement_count(),
            0,
            "an interrupted attempt leaks no pre-effect settlement reservation"
        );
    }
}

/// Published-snapshot capacity is bounded once, at preparation. Every prepared
/// candidate already holds one of the configured handles, so exhaustion is
/// reported before a candidate exists and long before the pre-effect settlement
/// reservation: the registry needs no second capacity check to stay bounded.
#[test]
fn published_snapshot_capacity_defers_at_preparation_without_reaching_settlement() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4096,
            max_published_snapshot_handles: 2,
            max_active_snapshot_handles: 4_096,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
        })
        .build();
    let first = create_entity_outcome(&runtime, "capacity-first");
    let second = create_entity_outcome(&runtime, "capacity-second");
    let before = test_owner_main_basis(&runtime).unwrap();
    let contacts_before = runtime
        .publication_binding()
        .pending_settlement_contact_count();

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("capacity-third"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        runtime.prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ),
        Err(
            crate::transactions::data::TransactionCommitError::PublicationDeferred {
                deferred:
                    crate::mvcc::RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted {
                        maximum_handles: 2,
                    },
                ..
            }
        )
    ));

    assert_eq!(
        runtime
            .publication_binding()
            .pending_settlement_contact_count(),
        contacts_before,
        "capacity is refused at preparation, so settlement admission is never contacted"
    );
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        0,
        "a candidate that never existed retains no settlement reservation"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before.descriptor(),
        "capacity exhaustion performs no movement"
    );
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

/// The reservation is installed before the effect, not after it. Pausing at the
/// linearization point observes a branch reference that has already moved, and
/// its settlement record must already exist at that instant.
#[test]
fn the_pending_settlement_record_exists_before_the_moved_head_is_observable() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "pre-effect-install-anchor");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::uninterrupted()
        .with_post_linearization_pause(Arc::clone(&reached), Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("pre-effect-install"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let reference_before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let binding = runtime.publication_binding();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    reached.wait();
    // Both observations are taken before releasing the paused publisher, so a
    // failing assertion cannot strand it on the barrier.
    let reference_at_pause = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let pending_at_pause = binding.pending_settlement_count();
    release.wait();

    assert_ne!(
        reference_at_pause, reference_before,
        "the pause observes a branch reference that already crossed linearization"
    );
    assert_eq!(
        pending_at_pause, 1,
        "movement is never observable before the settlement record that recovers it"
    );

    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = publisher.join().unwrap()
    else {
        panic!("an uncontended paused candidate performs");
    };
    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the paused movement settles through its installed record");
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);
    release_test_commit_snapshot(&runtime, &settled);
}

fn prepared_main_write(
    runtime: &RelationalRuntime,
    name: &str,
) -> PreparedRelationalCommitCandidate {
    let basis = test_owner_main_basis(runtime).expect("main basis is admitted");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("transaction binds");
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate prepares")
}
