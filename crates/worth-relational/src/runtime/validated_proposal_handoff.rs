//! Hand the accepted mutation's exact operands to the Publication owner.
use crate::authority::commit::pipeline::{AuthoritativeCommitContext, SelectedCommitMutation};
use crate::mvcc::{ValidatedProposalState, ValidatedRelationalProposal};

impl From<ValidatedRelationalProposal>
    for AuthoritativeCommitContext<crate::mvcc::RelationalTransactionValidationInput>
{
    fn from(candidate: ValidatedRelationalProposal) -> Self {
        let ValidatedProposalState {
            caller_association,
            required_output_publication,
            required_output_materialization,
            mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_mutation,
            invariants,
            proposal_identity,
            validated_against_commit,
            validated_against_version,
            validated_against_branch_version,
            batch_count,
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
            commit_work_budget,
            ..
        } = candidate.into_state();
        Self::from_selected_mutation(SelectedCommitMutation {
            caller_association,
            required_output_publication,
            required_output_materialization,
            mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_mutation,
            invariants,
            proposal_identity,
            validated_against_commit,
            validated_against_version,
            validated_against_branch_version,
            batch_count,
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
            commit_work_budget,
        })
    }
}
