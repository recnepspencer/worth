use crate::transactions::data::{
    CommitExecutionDenialKind, TransactionCommitError, WorkerIntentBatch,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn strategy_consults_before_its_stale_basis_check() {
    let (runtime, lowered) = crate::commit_strategies::facade::lowered_commit_fixture();
    let cancellation = CancellationSource::new();
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        cancellation.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("move-head"),
            execution.allocation_policy(),
        )
        .unwrap();
    runtime
        .commit_branch_transaction(&execution, transaction)
        .unwrap();
    cancellation.cancel();
    let error = runtime
        .commit_strategies_authority()
        .validate_lowered_plan(&execution, &runtime, lowered)
        .unwrap_err();
    assert!(
        matches!(error, TransactionCommitError::Execution { denial, .. }
        if denial.kind == CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::Cancelled)),
        "cancellation outranks currency before any strategy work"
    );
}
