//! Earlier phase values expire before publication artifacts need their stack frames.
use super::authority_context::AuthoritativeCommitContext;
use super::history_binding::HistoryBoundCommitExecution;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;

// This boundary releases consumed admission/draft frames before artifact assembly.
#[inline(never)]
pub(super) fn prepare_bound_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<HistoryBoundCommitExecution, TransactionCommitError> {
    let admitted = super::execution_admission::admit_commit_execution(runtime, context)?;
    let prepared = super::draft_execution::prepare_commit_execution(execution, runtime, admitted)?;
    let round_runtime = runtime.with_candidate_symbols(prepared.candidate_symbols());
    let runtime = &round_runtime;
    let boundary = super::boundary_validation::validate_commit_boundary(
        execution, runtime, prepared, control,
    )?;
    let mutated =
        super::mutation_execution::mutate_commit_execution(execution, runtime, boundary, control)?;
    super::history_binding::bind_commit_history(execution, runtime, mutated)
}
