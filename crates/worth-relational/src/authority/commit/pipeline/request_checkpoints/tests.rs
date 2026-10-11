use super::{observation, CommitStage};
use crate::tests::support::runtime_with_test_schema;
use crate::transactions::data::{
    CommitExecutionDenialKind, TransactionCommitError, WorkerIntentBatch,
};
use std::sync::{Arc, Barrier};
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
    let rendezvous = Arc::new(Barrier::new(2));
    let at_publication = Arc::clone(&rendezvous);
    let result = observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                at_publication.wait();
                at_publication.wait();
            }
        }),
        || {
            std::thread::scope(|scope| {
                scope.spawn(|| {
                    rendezvous.wait();
                    cancellation.cancel();
                    rendezvous.wait();
                });
                runtime.commit_branch_transaction(&execution, transaction(&runtime, 1))
            })
        },
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
        .run(ExecutionWorkCeiling::new(u64::MAX), |_| {
            runtime.commit_branch_transaction(&execution, transaction(&runtime, 1))
        })
        .unwrap();
    let result = result.unwrap();
    assert!(result.changed_records.is_empty());
    // An empty batch supplies no rows, mutations, or patch fragments; the
    // fixed-size publication metadata has no data-work fee on the base.
    assert_eq!(report.charged_work(), 0);
}
