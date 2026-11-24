//! Retain the exact observed version of the still-registered branch.
use super::RelationalPreparationHistory;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;

impl RelationalPreparationHistory {
    pub(crate) fn admitted_branch_basis_version(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Option<worth_relational_contracts::internal_port::identity::data::primitives::VersionId>
    {
        let cell = self.branch_cell(basis.identity().branch_id())?;
        if cell.identity() != basis.identity()
            || cell.observation() != *basis.reference()
            || cell.truth_version() != basis.truth_version()
        {
            return None;
        }
        match basis.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => Some(
                worth_relational_contracts::internal_port::identity::data::primitives::VersionId(0),
            ),
            worth_foundational::FoundationalBranchTarget::Basis(target) => cell
                .root()
                .and_then(|root| root.commit_receipt().cloned())
                .filter(|receipt| receipt.commit_id.0 == target.selected_commit_id())
                .map(|receipt| receipt.version_id),
        }
    }
}
