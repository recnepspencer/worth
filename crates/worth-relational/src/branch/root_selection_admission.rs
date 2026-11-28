//! Runtime admission of a branch basis before selecting its exact root.
#[cfg(test)]
use super::{root_selection::SelectedRelationalBranchState, AdmittedRelationalBranchBasis};

impl crate::runtime::RelationalRuntime {
    #[cfg(test)]
    pub(crate) fn selected_branch_state(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<SelectedRelationalBranchState, crate::transactions::data::CommitPreparationError>
    {
        SelectedRelationalBranchState::from_owner_basis(self.runtime_instance_id(), basis)
    }
}
