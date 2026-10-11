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
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
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
