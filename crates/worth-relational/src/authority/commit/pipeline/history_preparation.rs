//! Consumed draft, boundary and mutation phases do not occupy the next frame.
use super::authority_context::AuthoritativeCommitContext;
use super::draft_execution::PreparedCommitExecution;
use super::history_binding::HistoryBoundCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;
mod stages;

#[inline(never)]
pub(super) fn prepare_bound_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<CommitPhaseBacking<HistoryBoundCommitExecution>, TransactionCommitError> {
    let prepared = prepare_draft(execution, runtime, context)?;
    let round_runtime = runtime.with_candidate_symbols(prepared.value().candidate_symbols());
    let boundary = stages::validate_boundary(execution, &round_runtime, prepared, control)?;
    let mutated = stages::mutate(execution, &round_runtime, boundary, control)?;
    stages::bind(execution, &round_runtime, mutated)
}

#[inline(never)]
fn prepare_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<CommitPhaseBacking<PreparedCommitExecution>, TransactionCommitError> {
    let admitted = super::execution_admission::admit_commit_execution(runtime, context)?;
    super::draft_execution::prepare_commit_execution(execution, runtime, admitted)
}
