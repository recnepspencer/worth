use crate::facade::history::BranchId;
use crate::facade::mvcc::{
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
    RelationalTransactionIntent,
};
use crate::tests::support::*;

/// The performed witness is not the owner of the remaining settlement work.
/// Dropping it may record abandonment and nothing else.
#[test]
fn dropping_the_performed_witness_leaves_settlement_recoverable() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "abandoned-witness-anchor");
    let performed = perform_main_write(&runtime, "abandoned-witness");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let record = runtime
        .publication_binding()
        .pending_settlement(commit_id)
        .expect("movement installed its runtime-owned settlement record");

    drop(performed);

    assert_eq!(
        record.abandoned_capability_count(),
        1,
        "abandoning the witness is observable"
    );
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        1,
        "abandoning the witness may not release the obligation"
    );
    let mut blocked = test_owner_begin_transaction_for_main(&runtime);
    blocked
        .push_batch(
            batch_create("blocked-after-abandonment"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(runtime
        .prepare_branch_transaction(
            blocked,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err()
        .detail()
        .contains("requires explicit owner settlement"));

    let repaired = runtime
        .repair_pending_publication_settlement(commit_id)
        .expect("the runtime still owns the work its lost witness only reported");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);
    drop(record);
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        0,
        "repair without a witness closes the published snapshot it opened"
    );

    let child = create_entity_outcome(&runtime, "child-after-abandoned-witness");
    assert_eq!(child.commit.parents, vec![commit_id]);
    assert_eq!(durable_appends(&runtime, commit_id), 1);
    release_test_commit_snapshot(&runtime, &child);
}

/// Terminal settlement is exactly once. Asking again returns the same receipt
/// and appends nothing.
#[test]
fn repeated_terminal_settlement_returns_one_receipt_and_one_durable_append() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "repeated-settlement-anchor");
    let performed = perform_main_write(&runtime, "repeated-settlement");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the witness holder settles once");
    assert_eq!(settled.commit.commit_id, commit_id);

    let repeated = runtime
        .repair_pending_publication_settlement(commit_id)
        .expect("a settled identity answers from its terminal receipt");
    assert_eq!(repeated.commit_id, commit_id);
    assert_eq!(repeated, settled.commit);
    assert_eq!(
        durable_appends(&runtime, commit_id),
        1,
        "asking again performs no second durable effect"
    );
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);
    release_test_commit_snapshot(&runtime, &settled);
}

/// Immediate settlement and commit-identity repair share one executor gate, so
/// whichever runs second observes the terminal answer instead of repeating it.
#[test]
fn repair_and_witness_settlement_execute_exactly_once() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "single-execution-anchor");
    let performed = perform_main_write(&runtime, "single-execution");
    let commit_id = performed.canonical_commit().commit.commit_id;

    let repaired = runtime
        .repair_pending_publication_settlement(commit_id)
        .expect("repair runs the one terminal effect while the witness is still held");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);

    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the witness holder still receives its exact commit result");
    assert_eq!(settled.commit.commit_id, commit_id);
    assert_eq!(settled.commit, repaired);
    assert_eq!(
        durable_appends(&runtime, commit_id),
        1,
        "two settlement callers produce exactly one durable append"
    );

    let child = create_entity_outcome(&runtime, "child-after-single-execution");
    assert_eq!(child.commit.parents, vec![commit_id]);
    release_test_commit_snapshot(&runtime, &settled);
    release_test_commit_snapshot(&runtime, &child);
}

/// Owner loss retires every pending settlement exactly once instead of leaving
/// an unbounded registry behind a dead runtime.
#[test]
fn owner_loss_retires_every_pending_settlement_exactly_once() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "owner-loss-settlement-anchor");
    let performed = perform_main_write(&runtime, "owner-loss-settlement");
    drop(performed);
    let binding = runtime.publication_binding();
    assert!(!binding.settlement_admission_is_closed());
    assert_eq!(binding.pending_settlement_count(), 1);

    drop(runtime);

    assert!(
        binding.settlement_admission_is_closed(),
        "a lost owner admits no further settlement"
    );
    assert_eq!(
        binding.pending_settlement_count(),
        0,
        "owner loss drains its pending registry"
    );
    assert_eq!(
        binding.pending_settlement_owner_loss_count(),
        1,
        "each retired pending settlement is accounted exactly once"
    );
}

/// Interruption immediately after the linearization point is not a
/// no-movement path, so the settlement record must survive it.
#[test]
fn interruption_after_movement_retains_its_pending_settlement() {
    for interruption in [
        RelationalOperationInterruption::Cancelled,
        RelationalOperationInterruption::TimedOut,
    ] {
        let runtime = runtime_with_test_schema();
        create_entity(&runtime, "post-movement-interruption-anchor");
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let control = RelationalOperationControl::uninterrupted().with_injected_interruption(
            RelationalInterruptionBoundary::AfterLinearization,
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
                batch_create("interrupted-after-linearization"),
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

        let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
            runtime.publication_port().compare_and_publish(candidate)
        else {
            panic!("late interruption cannot erase performed movement");
        };

        assert_ne!(
            runtime
                .branch_reference_state(&BranchId("main".to_owned()))
                .unwrap(),
            reference_before,
        );
        assert_eq!(
            performed
                .late_interruption()
                .expect("late interruption is attached to the performed witness")
                .interruption(),
            interruption
        );
        assert_eq!(
            runtime.publication_binding().pending_settlement_count(),
            1,
            "movement that happened keeps its recoverable settlement record"
        );
        let settled = runtime
            .settle_performed_publication(performed)
            .expect("a late-interrupted movement remains settleable");
        assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);
        release_test_commit_snapshot(&runtime, &settled);
    }
}

/// Commit-identity repair can drive a record into durability deferral while the
/// performed witness is still live, so the repair that finally succeeds may not
/// close the published snapshot that witness is still owed.
#[test]
fn durability_repair_leaves_a_live_witness_its_usable_published_snapshot() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "live-witness-repair-anchor");
    let performed = perform_main_write(&runtime, "live-witness-repair");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let occupied_before = runtime
        .preparation_runtime_snapshot()
        .published_snapshot_count();

    runtime.durability.arm_append_failure();
    assert!(matches!(
        runtime
            .repair_pending_publication_settlement(commit_id)
            .expect_err("the injected append fault defers this commit-identity repair"),
        crate::publication::data::DeferredPublicationSettlementError::DurableAppend(_)
    ));
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        1,
        "a durability-deferred record is retained for repair"
    );
    assert_eq!(
        runtime.visibility.published_snapshot_handle_count(),
        1,
        "the deferred lane keeps the published snapshot it already opened"
    );

    let repaired = runtime
        .repair_pending_publication_settlement(commit_id)
        .expect("the retried repair performs the one missing durable append");
    assert_eq!(repaired.commit_id, commit_id);
    assert_eq!(runtime.publication_binding().pending_settlement_count(), 0);

    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the still-live witness receives its exact commit result");
    assert_eq!(settled.commit, repaired);
    assert!(
        runtime
            .read_truth()
            .read_snapshot(&settled.snapshot)
            .is_some(),
        "durability repair may not close a snapshot its live witness still owns"
    );
    assert!(
        runtime
            .snapshots()
            .release_snapshot(&settled.snapshot)
            .is_ok(),
        "the claiming witness took the release obligation with its result"
    );
    assert!(
        runtime
            .snapshots()
            .release_snapshot(&settled.snapshot)
            .is_err(),
        "one published handle is released exactly once"
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        occupied_before - 1,
        "the repaired settlement leaks no published-snapshot capacity"
    );
    assert_eq!(
        durable_appends(&runtime, commit_id),
        1,
        "a deferred then repaired then witnessed settlement appends exactly once"
    );
}

fn perform_main_write(
    runtime: &RelationalRuntime,
    name: &str,
) -> crate::mvcc::PerformedRelationalCommit {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate prepares");
    match runtime.publication_port().compare_and_publish(candidate) {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("an uncontended candidate performs: {outcome:?}"),
    }
}

fn durable_appends(
    runtime: &RelationalRuntime,
    commit_id: crate::history::data::CommitId,
) -> usize {
    runtime
        .durability()
        .durable_log()
        .iter()
        .filter(|entry| entry.envelope().commit.commit_id == commit_id)
        .count()
}
