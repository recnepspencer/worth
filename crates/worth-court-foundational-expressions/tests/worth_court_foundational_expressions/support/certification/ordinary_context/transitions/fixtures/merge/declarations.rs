pub(crate) use {worth_foundational::foundational_merge, worth_foundational::BoundaryHandle, worth_foundational::CanonicalDigestId, worth_foundational::EquivalenceBasisId, worth_foundational::FoundationalBranchBasisDrift, worth_foundational::FoundationalBranchBasisDriftKind, worth_foundational::FoundationalMergeBaseSelectionBasis, worth_foundational::FoundationalMergeBasis, worth_foundational::FoundationalMergeCandidate, worth_foundational::FoundationalMergeConflictLocus, worth_foundational::FoundationalMergeIntent, worth_foundational::FoundationalMergeStructuralSummary, worth_foundational::FoundationalStrategyBasis, worth_foundational::FoundationalTransitionBasisFamily, worth_foundational::FoundationalTransitionBasisIdentity, worth_foundational::FoundationalTransitionBasisVersion, worth_foundational::FoundationalTransitionCorrespondenceBasis, worth_foundational::FoundationalTransitionRemapBasis, worth_foundational::FoundationalTransitionStrategyContractBasis, worth_foundational::FoundationalTransitionStrategyDescriptorDigest, worth_foundational::FoundationalTransitionStrategyFamily, worth_foundational::FoundationalTransitionStrategyId, worth_foundational::FoundationalTransitionStrategyIdentity, worth_foundational::FoundationalTransitionStrategyOwnershipClass, worth_foundational::FoundationalTransitionStrategySemanticName, worth_foundational::FoundationalTransitionStrategyVersion};

pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::branch::branch_id, crate::certification::ordinary_context::transitions::fixtures::branch::staged_candidate};

pub(crate) fn strategy_identity() -> FoundationalTransitionStrategyIdentity {
    worth_foundational::FoundationalTransitionStrategyIdentity::new(
        worth_foundational::FoundationalTransitionStrategyId::new(worth_foundational::BoundaryHandle::new(91)),
        worth_foundational::FoundationalTransitionStrategyFamily::new("relational-merge").expect("family"),
        worth_foundational::FoundationalTransitionStrategySemanticName::new("geometry-aware-reconcile").expect("name"),
        worth_foundational::FoundationalTransitionStrategyVersion::new("v1").expect("version"),
        worth_foundational::FoundationalTransitionStrategyOwnershipClass::CustomRegistered,
    )
}

pub(crate) fn merge_basis(source_branch: &str, target_branch: &str) -> FoundationalMergeBasis {
    worth_foundational::FoundationalMergeBasis::new(
        worth_foundational::FoundationalTransitionBasisIdentity::new(worth_foundational::EquivalenceBasisId::new(73)),
        worth_foundational::FoundationalTransitionBasisFamily::new("geometry-kernel").expect("family"),
        worth_foundational::FoundationalTransitionBasisVersion::new("2026-05").expect("version"),
        branch_id(source_branch),
        branch_id(target_branch),
    )
}

pub(crate) const fn merge_summary() -> FoundationalMergeStructuralSummary {
    worth_foundational::FoundationalMergeStructuralSummary::new(4, 5, 3, 2)
}

pub(crate) fn authority_first_merge_candidate(
    payload: &'static str,
) -> FoundationalMergeCandidate<&'static str> {
    foundational_merge(staged_candidate(payload))
        .into_target_branch(branch_id("main"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_merge_basis(merge_basis("feature/geometry", "main"))
        .with_merge_base_selection_basis(worth_foundational::FoundationalMergeBaseSelectionBasis::new(
            worth_foundational::EquivalenceBasisId::new(57),
        ))
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(worth_foundational::FoundationalTransitionStrategyDescriptorDigest::new(
            worth_foundational::CanonicalDigestId::new([77; 32]),
        ))
        .with_strategy_contract_basis(worth_foundational::FoundationalTransitionStrategyContractBasis::new(
            worth_foundational::EquivalenceBasisId::new(61),
        ))
        .with_strategy_basis(worth_foundational::FoundationalStrategyBasis::new(worth_foundational::EquivalenceBasisId::new(59)))
        .under_correspondence_basis(worth_foundational::FoundationalTransitionCorrespondenceBasis::new(
            worth_foundational::EquivalenceBasisId::new(67),
        ))
        .under_remap_basis(worth_foundational::FoundationalTransitionRemapBasis::new(
            worth_foundational::EquivalenceBasisId::new(71),
        ))
        .plan()
        .expect("merge candidate")
}

pub(crate) fn projection_shaped_merge_candidate(
    payload: &'static str,
) -> FoundationalMergeCandidate<&'static str> {
    foundational_merge(staged_candidate(payload))
        .under_remap_basis(worth_foundational::FoundationalTransitionRemapBasis::new(
            worth_foundational::EquivalenceBasisId::new(71),
        ))
        .under_correspondence_basis(worth_foundational::FoundationalTransitionCorrespondenceBasis::new(
            worth_foundational::EquivalenceBasisId::new(67),
        ))
        .with_strategy_basis(worth_foundational::FoundationalStrategyBasis::new(worth_foundational::EquivalenceBasisId::new(59)))
        .with_strategy_contract_basis(worth_foundational::FoundationalTransitionStrategyContractBasis::new(
            worth_foundational::EquivalenceBasisId::new(61),
        ))
        .with_strategy_descriptor_digest(worth_foundational::FoundationalTransitionStrategyDescriptorDigest::new(
            worth_foundational::CanonicalDigestId::new([77; 32]),
        ))
        .under_strategy(strategy_identity())
        .with_merge_base_selection_basis(worth_foundational::FoundationalMergeBaseSelectionBasis::new(
            worth_foundational::EquivalenceBasisId::new(57),
        ))
        .with_merge_basis(merge_basis("feature/geometry", "main"))
        .with_structural_summary(merge_summary())
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .into_target_branch(branch_id("main"))
        .plan()
        .expect("merge candidate")
}

pub(crate) fn conflict_locus() -> FoundationalMergeConflictLocus {
    worth_foundational::FoundationalMergeConflictLocus::new("geometry-face", "source:face-7", "target:face-7")
}

pub(crate) const fn stale_target_advanced() -> FoundationalBranchBasisDrift {
    worth_foundational::FoundationalBranchBasisDrift::new(
        worth_foundational::FoundationalBranchBasisDriftKind::TargetAdvanced,
        "target branch advanced after merge planning",
    )
}