use worth_foundational::{
    foundational_branch_candidate, foundational_merge, BoundaryEpoch, BoundaryHandle,
    CanonicalDigestId, EquivalenceBasisId, FoundationalBranchCandidateId,
    FoundationalBranchCandidateForkBasis, FoundationalBranchId, FoundationalBranchCandidateObservationBasis,
    FoundationalMergeBaseSelectionBasis, FoundationalMergeBasis, FoundationalMergeIntent,
    FoundationalMergeStructuralSummary, FoundationalStrategyBasis,
    FoundationalTransitionBasisFamily, FoundationalTransitionBasisIdentity,
    FoundationalTransitionBasisVersion, FoundationalTransitionStrategyContractBasis,
    FoundationalTransitionStrategyDescriptorDigest, FoundationalTransitionStrategyFamily,
    FoundationalTransitionStrategyId, FoundationalTransitionStrategyIdentity,
    FoundationalTransitionStrategyOwnershipClass, FoundationalTransitionStrategySemanticName,
    FoundationalTransitionStrategyVersion,
};
use worth_proof::TransitionOutcome;

fn main() {
    let source = foundational_branch_candidate()
        .on_branch(FoundationalBranchId::new("feature/geometry").unwrap())
        .with_candidate_id(FoundationalBranchCandidateId::new(BoundaryHandle::new(7)))
        .from_fork_basis(FoundationalBranchCandidateForkBasis::new(
            FoundationalBranchId::new("main").unwrap(),
            BoundaryEpoch::new(4),
        ))
        .under_observation_basis(FoundationalBranchCandidateObservationBasis::new(
            EquivalenceBasisId::new(31),
            BoundaryEpoch::new(5),
        ))
        .stage("mesh-update")
        .unwrap()
        .staged();

    let verdict = match foundational_merge(source)
        .into_target_branch(FoundationalBranchId::new("main").unwrap())
        .with_intent(FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(FoundationalMergeStructuralSummary::new(4, 5, 3, 2))
        .with_merge_basis(FoundationalMergeBasis::new(
            FoundationalTransitionBasisIdentity::new(EquivalenceBasisId::new(73)),
            FoundationalTransitionBasisFamily::new("geometry-kernel").unwrap(),
            FoundationalTransitionBasisVersion::new("2026-05").unwrap(),
            FoundationalBranchId::new("feature/geometry").unwrap(),
            FoundationalBranchId::new("main").unwrap(),
        ))
        .with_merge_base_selection_basis(FoundationalMergeBaseSelectionBasis::new(
            EquivalenceBasisId::new(57),
        ))
        .under_strategy(FoundationalTransitionStrategyIdentity::new(
            FoundationalTransitionStrategyId::new(BoundaryHandle::new(91)),
            FoundationalTransitionStrategyFamily::new("relational-merge").unwrap(),
            FoundationalTransitionStrategySemanticName::new("geometry-aware-reconcile").unwrap(),
            FoundationalTransitionStrategyVersion::new("v1").unwrap(),
            FoundationalTransitionStrategyOwnershipClass::CustomRegistered,
        ))
        .with_strategy_descriptor_digest(FoundationalTransitionStrategyDescriptorDigest::new(
            CanonicalDigestId::new([77; 32]),
        ))
        .with_strategy_contract_basis(FoundationalTransitionStrategyContractBasis::new(
            EquivalenceBasisId::new(61),
        ))
        .with_strategy_basis(FoundationalStrategyBasis::new(EquivalenceBasisId::new(59)))
        .plan()
        .unwrap()
        .admit_as_accepted()
    {
        TransitionOutcome::Success(verdict) => verdict,
        _ => unreachable!(),
    };

    let _ = verdict.transition_class();
}
