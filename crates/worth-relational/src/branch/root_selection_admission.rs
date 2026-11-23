//! Runtime admission of a branch basis before selecting its exact root.
use super::{root_selection::SelectedRelationalBranchState, AdmittedRelationalBranchBasis};

impl crate::runtime::RelationalRuntime {
    pub(crate) fn selected_branch_state(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<SelectedRelationalBranchState, crate::transactions::data::CommitPreparationError>
    {
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(
                crate::transactions::data::CommitPreparationError::selected_branch_root_reference_mismatch(
                    basis.identity().branch_id().clone(),
                    basis.observation().commit_id().map(|id| id.0),
                    basis.observation().version_id(),
                ),
            );
        }
        Ok(SelectedRelationalBranchState::from_admitted_basis(basis))
    }
}
