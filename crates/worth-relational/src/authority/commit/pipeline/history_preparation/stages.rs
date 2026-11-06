//! One original, funded predecessor is consumed before its successor exists.
use super::super::phase_backing::CommitPhaseBacking;
use super::super::{boundary_validation, history_binding, mutation_execution};
use super::PreparedCommitExecution;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::InvariantPreparationControl;
use worth_execution::ExecutionRequest;

#[inline(never)]
pub(super) fn validate_boundary(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    source: CommitPhaseBacking<PreparedCommitExecution>,
    control: &InvariantPreparationControl<'_, '_>,
) -> Result<
    CommitPhaseBacking<boundary_validation::BoundaryValidatedCommitExecution>,
    TransactionCommitError,
> {
    boundary_validation::validate_commit_boundary(request, runtime, source, control)
}

#[inline(never)]
pub(super) fn mutate(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    source: CommitPhaseBacking<boundary_validation::BoundaryValidatedCommitExecution>,
    control: &InvariantPreparationControl<'_, '_>,
) -> Result<CommitPhaseBacking<mutation_execution::MutatedCommitExecution>, TransactionCommitError>
{
    mutation_execution::mutate_commit_execution(request, runtime, source, control)
}

#[inline(never)]
pub(super) fn bind(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    source: CommitPhaseBacking<mutation_execution::MutatedCommitExecution>,
) -> Result<CommitPhaseBacking<history_binding::HistoryBoundCommitExecution>, TransactionCommitError>
{
    history_binding::bind_commit_history(request, runtime, source)
}
