use crate::authority::commit::phases::prepare::PreparedWorkingStateScope;
use crate::identity::data::VersionId;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};
use crate::validation::engine::InvariantExecutionResult;

pub(super) fn stale_validated_proposal(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::StaleValidationBasis {
        detail: detail.to_owned(),
    }))
}

pub(super) fn validate_proposed_state(
    runtime: &RelationalPreparationRuntime,
    prepared: &PreparedWorkingStateScope,
    proposed: &crate::runtime::WorkingState,
    proposed_version: VersionId,
    proposal_identity: Option<&super::proposal_identity::RelationalMutationProposalIdentity>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<(InvariantExecutionResult, InvariantExecutionResult), TransactionCommitError> {
    let mutation_sensitive = runtime
        .invariant_authority()
        .enforce_mutation_sensitive_for_working_state(
            &prepared.selected_branch_state,
            proposed,
            proposed_version,
            &prepared.merged_plan,
            proposal_identity,
            lease,
        )?;
    let publication = runtime
        .invariant_authority()
        .enforce_snapshot_publication_for_working_state(
            &prepared.selected_branch_state,
            proposed,
            proposed_version,
            &prepared.merged_plan,
            proposal_identity,
            lease,
        )?;
    Ok((mutation_sensitive, publication))
}
