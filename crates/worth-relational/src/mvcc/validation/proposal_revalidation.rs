use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

use super::proposal_invariants::stale_validated_proposal;
use super::validated_proposal::ValidatedRelationalProposal;

impl RelationalPreparationRuntime {
    pub(crate) fn revalidate_proposal_for_publication(
        &self,
        candidate: ValidatedRelationalProposal,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.ensure_validated_proposal_branch_is_current(&candidate)?;
        if candidate.custom_invariant_generation
            != self.schema_contract_runtime.custom_invariant_generation
        {
            return Err(stale_validated_proposal(
                "validated mutation belongs to an obsolete custom-invariant generation",
            ));
        }
        Ok(candidate)
    }

    fn ensure_validated_proposal_branch_is_current(
        &self,
        candidate: &ValidatedRelationalProposal,
    ) -> Result<(), TransactionCommitError> {
        let binding = candidate.validation_input.basis();
        if binding.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: binding.identity().runtime_instance_id(),
                },
            )));
        }
        if !binding.is_current() {
            return Err(stale_validated_proposal(
                "validated mutation branch binding is no longer current",
            ));
        }
        let Some(cell) = self
            .history
            .branch_cell(candidate.validation_input.target_branch())
        else {
            return Err(stale_validated_proposal(
                "validated mutation branch is no longer registered",
            ));
        };
        if cell.identity() != binding.identity()
            || cell.observation() != *binding.reference()
            || cell.truth_version() != candidate.validated_against_branch_version
        {
            return Err(stale_validated_proposal(
                "validated mutation no longer matches the current branch reference",
            ));
        }
        Ok(())
    }
}
