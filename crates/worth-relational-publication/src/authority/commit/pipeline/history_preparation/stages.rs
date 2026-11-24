//! One original, funded predecessor is consumed before its successor exists.
use super::super::phase_backing::CommitPhaseBacking;
use super::super::{boundary_validation, history_binding, mutation_execution};
use super::PreparedCommitExecution;
use crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityInputs;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

#[inline(never)]
pub(super) fn validate_boundary(
    request: &ExecutionRequest<'_, '_>,
    source: CommitPhaseBacking<PreparedCommitExecution>,
) -> Result<
    CommitPhaseBacking<boundary_validation::BoundaryValidatedCommitExecution>,
    TransactionCommitError,
> {
    boundary_validation::validate_commit_boundary(request, source)
}

#[inline(never)]
pub(super) fn mutate(
    request: &ExecutionRequest<'_, '_>,
    instrumentation: &worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    source: CommitPhaseBacking<boundary_validation::BoundaryValidatedCommitExecution>,
) -> Result<CommitPhaseBacking<mutation_execution::MutatedCommitExecution>, TransactionCommitError>
{
    mutation_execution::mutate_commit_execution(request, instrumentation, source)
}

#[inline(never)]
pub(super) fn bind(
    request: &ExecutionRequest<'_, '_>,
    inputs: SchemaContinuityInputs<'_>,
    source: CommitPhaseBacking<mutation_execution::MutatedCommitExecution>,
) -> Result<CommitPhaseBacking<history_binding::HistoryBoundCommitExecution>, TransactionCommitError>
{
    history_binding::bind_commit_history(request, inputs, source)
}
