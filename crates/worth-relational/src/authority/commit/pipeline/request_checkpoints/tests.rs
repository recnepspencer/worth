use super::{observation, CommitStage};
use crate::tests::support::runtime_with_test_schema;
use crate::transactions::data::{
    CommitExecutionDenialKind, TransactionCommitError, WorkerIntentBatch,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

fn transaction(
    runtime: &crate::runtime::RelationalRuntime,
    batches: usize,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    for index in 0..batches {
        transaction
            .push_batch(
                WorkerIntentBatch::new(format!("stage-{index}")),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    transaction
}

#[test]
fn cancellation_between_validation_and_publication_stops_without_publishing() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let cancellation = Arc::new(CancellationSource::new());
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let visits = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&visits);
    let cancel_at_publication = Arc::clone(&cancellation);
    let publication = runtime.publication_binding();
    let preparation = runtime.preparation_runtime_snapshot();
    let before_candidates = publication.prepared_candidate_count();
    let before_slots = preparation.published_snapshot_count();
    let candidates_at_stage = Arc::new(AtomicUsize::new(0));
    let candidates_seen = Arc::clone(&candidates_at_stage);
    let slots_at_stage = Arc::new(AtomicUsize::new(0));
    let slots_seen = Arc::clone(&slots_at_stage);
    let result = observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                seen.fetch_add(1, Ordering::Relaxed);
                candidates_seen.store(publication.prepared_candidate_count(), Ordering::Relaxed);
                slots_seen.store(preparation.published_snapshot_count(), Ordering::Relaxed);
                cancel_at_publication.cancel();
            }
        }),
        || runtime.commit_branch_transaction(&execution, transaction(&runtime, 1)),
    );
    // If the consult disappears this assertion fails; no thread waits for an event.
    assert_eq!(
        visits.load(Ordering::Relaxed),
        1,
        "publication consult must reach its observer"
    );
    assert_eq!(
        candidates_at_stage.load(Ordering::Relaxed),
        before_candidates + 1
    );
    assert_eq!(slots_at_stage.load(Ordering::Relaxed), before_slots + 1);
    assert_eq!(
        runtime.publication_binding().prepared_candidate_count(),
        before_candidates
    );
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .published_snapshot_count(),
        before_slots
    );
    let TransactionCommitError::Execution { denial, .. } =
        result.expect_err("cancelled request cannot publish")
    else {
        panic!("interruption retains its typed request cause")
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, before);
}

#[test]
fn a_completed_empty_commit_keeps_its_zero_data_work_charge() {
    let runtime = runtime_with_test_schema();
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let (result, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            runtime.commit_branch_transaction(&execution, transaction(&runtime, 1))
        })
        .unwrap();
    let result = result.unwrap();
    assert!(result.changed_records.is_empty());
    // An empty batch supplies no rows, mutations, or patch fragments; the
    // fixed-size publication metadata has no data-work fee on the base.
    assert_eq!(report.charged_work(), 0);
}
