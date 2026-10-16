//! Stack lifetimes of the owned commit preparation phases.
//!
//! Keep these semantic steps out of line: their mutable phase values and fallible
//! results must not all occupy the candidate-registration coordinator's frame.

use super::authority_context::AuthoritativeCommitContext;
use super::mutation_execution::MutatedCommitExecution;
use super::publication_execution::PreparedCommitPublicationExecution;
use super::snapshot_validation::SnapshotValidatedCommitExecution;
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::InvariantPreparationControl;

#[inline(never)]
pub(super) fn prepare_validated_mutation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
    control: &InvariantPreparationControl<'_, '_>,
) -> Result<MutatedCommitExecution, TransactionCommitError> {
    let admitted = super::execution_admission::admit_commit_execution(runtime, context)?;
    let prepared = super::draft_execution::prepare_commit_execution(execution, runtime, admitted)?;
    let validated = super::boundary_validation::validate_commit_boundary(
        execution, runtime, prepared, control,
    )?;
    super::mutation_execution::mutate_commit_execution(execution, runtime, validated, control)
}

#[inline(never)]
pub(super) fn prepare_validated_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mutated: MutatedCommitExecution,
    control: &InvariantPreparationControl<'_, '_>,
) -> Result<SnapshotValidatedCommitExecution, TransactionCommitError> {
    let history_bound = super::history_binding::bind_commit_history(execution, runtime, mutated)?;
    super::snapshot_validation::validate_snapshot_publication(
        execution,
        runtime,
        history_bound,
        control,
    )
}

#[inline(never)]
pub(super) fn prepare_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    snapshot_validated: SnapshotValidatedCommitExecution,
) -> Result<PreparedCommitPublicationExecution, TransactionCommitError> {
    let assembled = super::artifact_execution::assemble_commit_artifacts(
        execution,
        runtime,
        snapshot_validated,
    )?;
    super::publication_execution::prepare_commit_publication_execution(
        execution, runtime, assembled,
    )
}
