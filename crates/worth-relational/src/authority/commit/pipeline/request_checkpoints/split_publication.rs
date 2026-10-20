use crate::tests::support::runtime_with_test_schema;
use crate::transactions::data::WorkerIntentBatch;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn split_publication_cancelled_request_releases_candidate_and_records_discard() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let slots = runtime
        .preparation_runtime_snapshot()
        .published_snapshot_count();
    let candidates = runtime.publication_binding().prepared_candidate_count();
    let cancellation = CancellationSource::new();
    let backing =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&backing);
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("split"),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = runtime
        .validate_branch_transaction(&execution, transaction)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&execution, proposal)
        .unwrap();
    assert_eq!(
        runtime.publication_binding().prepared_candidate_count(),
        candidates + 1
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        slots + 1
    );
    let scope = crate::inspection::mvcc::cost::RelationalMvccCostScope::capture(
        &runtime,
        vec![identity.clone()],
    );
    cancellation.cancel();
    let outcome = runtime
        .publication_port()
        .compare_and_publish(&execution, candidate);
    assert!(
        matches!(
            outcome,
            crate::mvcc::RelationalPublicationOutcome::RequestStopped(
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                ),
            )
        ),
        "a cancelled request retains its typed cause on the split path"
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
    assert_eq!(
        runtime.publication_binding().prepared_candidate_count(),
        candidates
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        slots
    );
    assert_eq!(
        runtime
            .observe_mvcc_counters(&scope)
            .unwrap()
            .sharing_cost_delta()
            .candidate_discards,
        1
    );
    assert_eq!(
        runtime
            .observe_mvcc_counters(&scope)
            .unwrap()
            .sharing_cost_delta()
            .publication_attempts,
        0
    );
}

#[test]
fn foreign_candidate_leaves_the_target_owner_and_its_counters_unchanged() {
    let source = runtime_with_test_schema();
    let target = runtime_with_test_schema();
    let backing = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&backing);
    let source_identity = source.main_branch_identity();
    let (source_head, basis) = source.observe_branch(&source_identity).unwrap();
    let target_identity = target.main_branch_identity();
    let target_head = target.observe_branch(&target_identity).unwrap().0;
    let target_slots = target
        .preparation_runtime_snapshot()
        .published_snapshot_count();
    let source_slots = source
        .preparation_runtime_snapshot()
        .published_snapshot_count();
    let scope = crate::inspection::mvcc::cost::RelationalMvccCostScope::capture(
        &target,
        vec![target_identity.clone()],
    );
    let transaction = source
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate = source
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    assert_eq!(source.publication_binding().prepared_candidate_count(), 1);
    let outcome = target
        .publication_port()
        .compare_and_publish(&execution, candidate);
    assert!(
        matches!(outcome, crate::mvcc::RelationalPublicationOutcome::Denied(
        crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
            expected_runtime_instance_id, actual_runtime_instance_id,
        }
    ) if expected_runtime_instance_id == target.runtime_instance_id()
        && actual_runtime_instance_id == source.runtime_instance_id()),
        "{outcome:?}"
    );
    let cost = target
        .observe_mvcc_counters(&scope)
        .unwrap()
        .sharing_cost_delta();
    assert_eq!(cost.publication_attempts, 0);
    assert_eq!(cost.candidate_discards, 0);
    assert_eq!(target.publication_binding().prepared_candidate_count(), 0);
    assert_eq!(
        target
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        target_slots
    );
    assert_eq!(
        target.observe_branch(&target_identity).unwrap().0,
        target_head
    );
    // Consuming the foreign value releases the source's registration and slot;
    // it does not record a discard against the target owner's branch.
    assert_eq!(source.publication_binding().prepared_candidate_count(), 0);
    assert_eq!(
        source
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        source_slots
    );
    assert_eq!(
        source.observe_branch(&source_identity).unwrap().0,
        source_head
    );
}
