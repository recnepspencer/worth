use worth_foundational::{
    foundational_merge, BoundaryHandle, CanonicalDigestId, EquivalenceBasisId,
    FoundationalBranchBasisDrift, FoundationalBranchBasisDriftKind,
    FoundationalMergeBaseSelectionBasis, FoundationalMergeBasis, FoundationalMergeCandidate,
    FoundationalMergeConflictLocus, FoundationalMergeIntent, FoundationalMergeStructuralSummary,
    FoundationalStrategyBasis, FoundationalTransitionBasisFamily,
    FoundationalTransitionBasisIdentity, FoundationalTransitionBasisVersion,
    FoundationalTransitionCorrespondenceBasis, FoundationalTransitionRemapBasis,
    FoundationalTransitionStrategyContractBasis, FoundationalTransitionStrategyDescriptorDigest,
    FoundationalTransitionStrategyFamily, FoundationalTransitionStrategyId,
    FoundationalTransitionStrategyIdentity, FoundationalTransitionStrategyOwnershipClass,
    FoundationalTransitionStrategySemanticName, FoundationalTransitionStrategyVersion,
};

use super::branch::{branch_id, staged_candidate};

pub fn strategy_identity() -> FoundationalTransitionStrategyIdentity {
    FoundationalTransitionStrategyIdentity::new(
        FoundationalTransitionStrategyId::new(BoundaryHandle::new(91)),
        FoundationalTransitionStrategyFamily::new("relational-merge").expect("family"),
        FoundationalTransitionStrategySemanticName::new("geometry-aware-reconcile").expect("name"),
        FoundationalTransitionStrategyVersion::new("v1").expect("version"),
        FoundationalTransitionStrategyOwnershipClass::CustomRegistered,
    )
}

pub fn merge_basis(source_branch: &str, target_branch: &str) -> FoundationalMergeBasis {
    FoundationalMergeBasis::new(
        FoundationalTransitionBasisIdentity::new(EquivalenceBasisId::new(73)),
        FoundationalTransitionBasisFamily::new("geometry-kernel").expect("family"),
        FoundationalTransitionBasisVersion::new("2026-05").expect("version"),
        branch_id(source_branch),
        branch_id(target_branch),
    )
}

pub const fn merge_summary() -> FoundationalMergeStructuralSummary {
    FoundationalMergeStructuralSummary::new(4, 5, 3, 2)
}

pub fn authority_first_merge_candidate(
    payload: &'static str,
) -> FoundationalMergeCandidate<&'static str> {
    foundational_merge(staged_candidate(payload))
        .into_target_branch(branch_id("main"))
        .with_intent(FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_merge_basis(merge_basis("feature/geometry", "main"))
        .with_merge_base_selection_basis(FoundationalMergeBaseSelectionBasis::new(
            EquivalenceBasisId::new(57),
        ))
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(FoundationalTransitionStrategyDescriptorDigest::new(
            CanonicalDigestId::new([77; 32]),
        ))
        .with_strategy_contract_basis(FoundationalTransitionStrategyContractBasis::new(
            EquivalenceBasisId::new(61),
        ))
        .with_strategy_basis(FoundationalStrategyBasis::new(EquivalenceBasisId::new(59)))
        .under_correspondence_basis(FoundationalTransitionCorrespondenceBasis::new(
            EquivalenceBasisId::new(67),
        ))
        .under_remap_basis(FoundationalTransitionRemapBasis::new(
            EquivalenceBasisId::new(71),
        ))
        .plan()
        .expect("merge candidate")
}

pub fn projection_shaped_merge_candidate(
    payload: &'static str,
) -> FoundationalMergeCandidate<&'static str> {
    foundational_merge(staged_candidate(payload))
        .under_remap_basis(FoundationalTransitionRemapBasis::new(
            EquivalenceBasisId::new(71),
        ))
        .under_correspondence_basis(FoundationalTransitionCorrespondenceBasis::new(
            EquivalenceBasisId::new(67),
        ))
        .with_strategy_basis(FoundationalStrategyBasis::new(EquivalenceBasisId::new(59)))
        .with_strategy_contract_basis(FoundationalTransitionStrategyContractBasis::new(
            EquivalenceBasisId::new(61),
        ))
        .with_strategy_descriptor_digest(FoundationalTransitionStrategyDescriptorDigest::new(
            CanonicalDigestId::new([77; 32]),
        ))
        .under_strategy(strategy_identity())
        .with_merge_base_selection_basis(FoundationalMergeBaseSelectionBasis::new(
            EquivalenceBasisId::new(57),
        ))
        .with_merge_basis(merge_basis("feature/geometry", "main"))
        .with_structural_summary(merge_summary())
        .with_intent(FoundationalMergeIntent::ReconcileIntoTarget)
        .into_target_branch(branch_id("main"))
        .plan()
        .expect("merge candidate")
}

pub fn conflict_locus() -> FoundationalMergeConflictLocus {
    FoundationalMergeConflictLocus::new("geometry-face", "source:face-7", "target:face-7")
}

pub const fn stale_target_advanced() -> FoundationalBranchBasisDrift {
    FoundationalBranchBasisDrift::new(
        FoundationalBranchBasisDriftKind::TargetAdvanced,
        "target branch advanced after merge planning",
    )
}
