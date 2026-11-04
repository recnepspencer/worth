//! Consumed draft frames expire before candidate symbol binding and later phases.
use super::authority_context::AuthoritativeCommitContext;
use super::draft_execution::PreparedCommitExecution;
use super::history_binding::HistoryBoundCommitExecution;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;

// Only the actual prepared draft remains while the persistent symbol tree binds.
// Boundary/mutation locals belong to the later function, not this deep call.
#[inline(never)]
pub(super) fn prepare_bound_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<HistoryBoundCommitExecution, TransactionCommitError> {
    let prepared = prepare_draft(execution, runtime, context)?;
    let round_runtime = runtime.with_candidate_symbols(prepared.candidate_symbols());
    finish_bound_history(execution, &round_runtime, prepared, control)
}

#[inline(never)]
fn prepare_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<PreparedCommitExecution, TransactionCommitError> {
    let admitted = super::execution_admission::admit_commit_execution(runtime, context)?;
    super::draft_execution::prepare_commit_execution(execution, runtime, admitted)
}

#[inline(never)]
fn finish_bound_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    prepared: PreparedCommitExecution,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<HistoryBoundCommitExecution, TransactionCommitError> {
    let boundary = super::boundary_validation::validate_commit_boundary(
        execution, runtime, prepared, control,
    )?;
    let mutated =
        super::mutation_execution::mutate_commit_execution(execution, runtime, boundary, control)?;
    super::history_binding::bind_commit_history(execution, runtime, mutated)
}
