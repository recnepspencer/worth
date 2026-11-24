//! Runtime issuance pairs a reserved proposal sequence with its source.
use super::{RelationalMutationProposalIdentity, TransactionId};
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::{
    bind_mutation_proposal_identity, MutationProposalIdentityParts,
};

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn issue_mutation_proposal_identity(
        &self,
        transaction_id: TransactionId,
        validation_input: &crate::mvcc::RelationalTransactionValidationInput,
    ) -> Result<
        (
            RelationalMutationProposalIdentity,
            std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
        ),
        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    > {
        let (proposed_version_id, ordinal, commit_id) =
            self.reserve_proposal_sequence(validation_input)?;
        let source = crate::validation::candidate_source::NativeManagedSourceInterpretation::issue(
            validation_input,
            proposed_version_id,
            self.publication_binding().source_registration(),
        );
        let identity = bind_mutation_proposal_identity(MutationProposalIdentityParts {
            omitted_publication_appendix: validation_input.appendix_adoption().is_some(),
            runtime_instance_id: self.services.runtime_instance_id(),
            ordinal,
            transaction_id,
            branch_observation: validation_input.basis().reference().clone(),
            branch_version: validation_input.basis().truth_version(),
            proposed_version_id,
            commit_id,
            source_interpretation: source.identity(),
        });
        Ok((identity, source))
    }
}
