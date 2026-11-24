use super::authority_context::AuthoritativeCommitContext;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

pub(crate) fn execute_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    context: AuthoritativeCommitContext<crate::mvcc::RelationalTransactionValidationInput>,
) -> Result<CommitResult, TransactionCommitError> {
    super::consult_stage(execution, super::CommitStage::Admission)?;
    let preparation = runtime.preparation_runtime_snapshot();
    // A detached validated proposal retains its account; merge and replay
    // start theirs here. Every charged dispatch then spends the same ceiling.
    let preparation = if execution.is_leased() {
        preparation.with_commit_work_budget(context.commit_work_budget.clone())
    } else {
        preparation
    };
    let candidate = super::runtime_candidate_orchestration::prepare_consulted_authoritative_commit(
        execution,
        &preparation,
        context,
    )?;
    runtime.publish_prepared_candidate(execution, candidate)
}
