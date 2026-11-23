//! Consumed draft, boundary and mutation phases do not occupy the next frame.
use super::authority_context::AuthoritativeCommitContext;
use super::draft_execution::PreparedCommitExecution;
use super::history_binding::HistoryBoundCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
mod stages;

#[inline(never)]
pub(super) fn prepare_bound_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<CommitPhaseBacking<HistoryBoundCommitExecution>, TransactionCommitError> {
    let prepared = prepare_draft(execution, runtime, context)?;
    let boundary = stages::validate_boundary(execution, prepared)?;
    let mutated = stages::mutate(execution, &runtime.services.instrumentation, boundary)?;
    stages::bind(execution, runtime.into(), mutated)
}

#[inline(never)]
fn prepare_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<CommitPhaseBacking<PreparedCommitExecution>, TransactionCommitError> {
    let admitted = super::execution_admission::admit_commit_execution(runtime.into(), context)?;
    super::draft_execution::prepare_commit_execution(execution, runtime, admitted)
}
