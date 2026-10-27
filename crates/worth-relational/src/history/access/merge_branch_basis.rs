use std::sync::Arc;

use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::{
    BranchId, MergeBaseSelectionRule, RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial,
    ResolvedMergeBase,
};
use worth_foundational::FoundationalBranchTarget;

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
        self.resolve_merge_branch_basis_from_heads(
            source_branch,
            target_branch,
            source_head,
            target_head,
        )
    }

    fn resolve_merge_branch_basis_from_heads(
        &self,
        source_branch: BranchId,
        target_branch: BranchId,
        source_head: crate::history::data::RelationalCommitReceipt,
        target_head: crate::history::data::RelationalCommitReceipt,
    ) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
        let left_ancestors = self.ancestor_closure_by_commit_id_order(target_head.commit_id);
        let right_ancestors = self.ancestor_closure_by_commit_id_order(source_head.commit_id);
        let merge_base_commit_id = self
            .max_commit_id_common_ancestor(target_head.commit_id, source_head.commit_id)
            .ok_or_else(|| RelationalMergeBranchBasisDenial::MissingMergeBase {
                source_branch: source_branch.clone(),
                target_branch: target_branch.clone(),
            })?;
        let merge_base = self.immutable_commit_receipt(merge_base_commit_id).ok_or(
            RelationalMergeBranchBasisDenial::MissingMergeBaseCommit {
                commit_id: merge_base_commit_id,
            },
        )?;
        Ok(RelationalMergeBranchBasis {
            source_head,
            target_head,
            merge_base: ResolvedMergeBase {
                rule: MergeBaseSelectionRule::MaxCommitIdCommonAncestor,
                commit: merge_base,
                supporting_left_ancestors: Arc::from(left_ancestors),
                supporting_right_ancestors: Arc::from(right_ancestors),
            },
        })
    }

    pub(crate) fn resolve_merge_branch_basis_from_bindings(
        &self,
        source_binding: &AdmittedRelationalBranchBasis,
        target_binding: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
        let source_head = self.commit_receipt_for_binding(source_binding, false)?;
        let target_head = self.commit_receipt_for_binding(target_binding, true)?;
        self.resolve_merge_branch_basis_from_heads(
            source_binding.identity().branch_id().clone(),
            target_binding.identity().branch_id().clone(),
            source_head,
            target_head,
        )
    }

    fn commit_receipt_for_binding(
        &self,
        binding: &AdmittedRelationalBranchBasis,
        target: bool,
    ) -> Result<crate::history::data::RelationalCommitReceipt, RelationalMergeBranchBasisDenial>
    {
        let branch_id = binding.identity().branch_id().clone();
        let missing = |branch_id: BranchId| {
            if target {
                RelationalMergeBranchBasisDenial::MissingTargetHead { branch_id }
            } else {
                RelationalMergeBranchBasisDenial::MissingSourceHead { branch_id }
            }
        };
        let cell = self
            .runtime
            .history
            .branch_cell(&branch_id)
            .ok_or_else(|| missing(branch_id.clone()))?;
        if cell.identity() != binding.identity()
            || cell.observation() != *binding.reference()
            || cell.truth_version() != binding.truth_version()
        {
            return Err(missing(branch_id));
        }
        let FoundationalBranchTarget::Basis(target) = binding.reference().target() else {
            return Err(missing(branch_id));
        };
        self.immutable_commit_receipt(crate::history::data::CommitId(target.selected_commit_id()))
            .ok_or_else(|| missing(branch_id))
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

        self.resolve_merge_branch_basis_from_heads(
            source_branch.clone(),
            target_branch.clone(),
            source_head,
            target_head,
        )
    }
}
