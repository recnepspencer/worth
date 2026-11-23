//! Resolve a merge base from exact Branch heads and ordered immutable parentage.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::{
    BranchId, CommitId, MergeBaseSelectionRule, RelationalCommitReceipt,
    RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial, ResolvedMergeBase,
};
use crate::runtime::state::subsystems::history::HistorySubsystem;
use std::collections::BTreeSet;
use worth_foundational::FoundationalBranchTarget;
pub(super) fn resolve_from_heads(
    mut ancestor_set: impl FnMut(CommitId) -> BTreeSet<CommitId>,
    mut receipt: impl FnMut(CommitId) -> Option<RelationalCommitReceipt>,
    source_branch: BranchId,
    target_branch: BranchId,
    source_head: RelationalCommitReceipt,
    target_head: RelationalCommitReceipt,
) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
    let left_ancestors = ancestor_set(target_head.commit_id)
        .into_iter()
        .collect::<Vec<_>>();
    let right_ancestors = ancestor_set(source_head.commit_id)
        .into_iter()
        .collect::<Vec<_>>();
    let merge_base_commit_id = super::ancestry_walk::common_ancestor(
        &ancestor_set(target_head.commit_id),
        &ancestor_set(source_head.commit_id),
    )
    .ok_or_else(|| RelationalMergeBranchBasisDenial::MissingMergeBase {
        source_branch: source_branch.clone(),
        target_branch: target_branch.clone(),
    })?;
    let merge_base = receipt(merge_base_commit_id).ok_or(
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
            supporting_left_ancestors: left_ancestors.into(),
            supporting_right_ancestors: right_ancestors.into(),
        },
    })
}

pub(super) fn resolve_from_bindings(
    history: &HistorySubsystem,
    ancestor_set: impl FnMut(CommitId) -> BTreeSet<CommitId>,
    mut receipt: impl FnMut(CommitId) -> Option<RelationalCommitReceipt>,
    source_binding: &AdmittedRelationalBranchBasis,
    target_binding: &AdmittedRelationalBranchBasis,
) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial> {
    let source_head = commit_receipt_for_binding(history, source_binding, false, &mut receipt)?;
    let target_head = commit_receipt_for_binding(history, target_binding, true, &mut receipt)?;
    resolve_from_heads(
        ancestor_set,
        receipt,
        source_binding.identity().branch_id().clone(),
        target_binding.identity().branch_id().clone(),
        source_head,
        target_head,
    )
}

fn commit_receipt_for_binding(
    history: &HistorySubsystem,
    binding: &AdmittedRelationalBranchBasis,
    target: bool,
    mut receipt: impl FnMut(CommitId) -> Option<RelationalCommitReceipt>,
) -> Result<RelationalCommitReceipt, RelationalMergeBranchBasisDenial> {
    let branch_id = binding.identity().branch_id().clone();
    let missing = |branch_id: BranchId| {
        if target {
            RelationalMergeBranchBasisDenial::MissingTargetHead { branch_id }
        } else {
            RelationalMergeBranchBasisDenial::MissingSourceHead { branch_id }
        }
    };
    let cell = history
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
    receipt(CommitId(target.selected_commit_id())).ok_or_else(|| missing(branch_id))
}
