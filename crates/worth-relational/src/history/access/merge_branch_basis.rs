use crate::branch::basis::AdmittedRelationalBranchBasis;
#[cfg(test)]
use crate::history::data::coordinates::BranchId;
use crate::history::data::{RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial};

use super::HistoryAccess;

impl<'runtime> HistoryAccess<'runtime> {
    /// Resolve merge ancestry from two owner-admitted exact observations.
    pub fn merge_branch_basis_for_observations(
        &self,
        source: &crate::mvcc::RelationalBranchObservation,
        target: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
        let source_branch = source.identity().branch_id().clone();
        let target_branch = target.identity().branch_id().clone();
        let source_head = self
            .branch_head_for_observation(source)
            .map_err(RelationalMergeBranchBasisDenial::SourceObservationDenied)?
            .ok_or_else(|| RelationalMergeBranchBasisDenial::MissingSourceHead {
                branch_id: source_branch.clone(),
            })?;
        let target_head = self
            .branch_head_for_observation(target)
            .map_err(RelationalMergeBranchBasisDenial::TargetObservationDenied)?
            .ok_or_else(|| RelationalMergeBranchBasisDenial::MissingTargetHead {
                branch_id: target_branch.clone(),
            })?;
        super::merge_basis_resolution::resolve_from_heads(
            |commit| self.ancestor_set(commit),
            |commit| self.immutable_commit_receipt(commit),
            source_branch,
            target_branch,
            source_head,
            target_head,
        )
    }

    pub(crate) fn resolve_merge_branch_basis_from_bindings(
        &self,
        source_binding: &AdmittedRelationalBranchBasis,
        target_binding: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
        super::merge_basis_resolution::resolve_from_bindings(
            &self.runtime.history,
            |commit| self.ancestor_set(commit),
            |commit| self.immutable_commit_receipt(commit),
            source_binding,
            target_binding,
        )
    }

    #[cfg(test)]
    pub(crate) fn resolve_merge_branch_basis(
        &self,
        source_branch: &BranchId,
        target_branch: &BranchId,
    ) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
        let target_head = self.branch_head(target_branch).ok_or_else(|| {
            RelationalMergeBranchBasisDenial::MissingTargetHead {
                branch_id: target_branch.clone(),
            }
        })?;
        let source_head = self.branch_head(source_branch).ok_or_else(|| {
            RelationalMergeBranchBasisDenial::MissingSourceHead {
                branch_id: source_branch.clone(),
            }
        })?;

        super::merge_basis_resolution::resolve_from_heads(
            |commit| self.ancestor_set(commit),
            |commit| self.immutable_commit_receipt(commit),
            source_branch.clone(),
            target_branch.clone(),
            source_head,
            target_head,
        )
    }

    #[cfg(test)]
    pub(crate) fn lower_merge_branch_basis_to_foundational_current_basis(
        &self,
        basis: &RelationalMergeBranchBasis,
        version: worth_foundational::CanonicalizationRuleVersion,
        profile: worth_foundational::MaterializedFoundationalProfileSet,
    ) -> Result<
        crate::history::data::RelationalFoundationalCurrentMergeBranchBasisArtifact,
        crate::history::data::RelationalMergeBranchBasisFoundationalLoweringDenial,
    > {
        super::merge_branch_basis_foundational::lower_current_basis(
            self.resolve_merge_branch_basis(basis.source_branch(), basis.target_branch()),
            basis,
            version,
            profile,
        )
    }
}
