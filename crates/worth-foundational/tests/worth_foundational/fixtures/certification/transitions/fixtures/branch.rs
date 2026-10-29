use worth_foundational::{
    foundational_branch_candidate, BoundaryEpoch, BoundaryHandle, EquivalenceBasisId,
    FoundationalBranchCandidateArtifact, FoundationalBranchCandidateComparisonBasis,
    FoundationalBranchCandidateForkBasis, FoundationalBranchCandidateForkObservationBasis,
    FoundationalBranchCandidateId, FoundationalBranchCandidateObservationBasis,
    FoundationalBranchId, FoundationalStagedBranchArtifact,
};

pub fn branch_id(name: &str) -> FoundationalBranchId {
    FoundationalBranchId::new(name).expect("valid branch id")
}

pub fn authority_first_candidate(
    payload: &'static str,
) -> FoundationalBranchCandidateArtifact<&'static str> {
    foundational_branch_candidate()
        .on_branch(branch_id("feature/geometry"))
        .with_candidate_id(FoundationalBranchCandidateId::new(BoundaryHandle::new(17)))
        .from_fork_basis(FoundationalBranchCandidateForkBasis::new(
            branch_id("main"),
            BoundaryEpoch::new(4),
        ))
        .under_observation_basis(FoundationalBranchCandidateObservationBasis::new(
            EquivalenceBasisId::new(31),
            BoundaryEpoch::new(5),
        ))
        .under_fork_observation_basis(FoundationalBranchCandidateForkObservationBasis::new(
            EquivalenceBasisId::new(37),
            BoundaryEpoch::new(4),
        ))
        .against_comparison_basis(FoundationalBranchCandidateComparisonBasis::new(
            EquivalenceBasisId::new(43),
            branch_id("main"),
        ))
        .stage(payload)
        .expect("candidate")
}

pub fn projection_shaped_candidate(
    payload: &'static str,
) -> FoundationalBranchCandidateArtifact<&'static str> {
    foundational_branch_candidate()
        .against_comparison_basis(FoundationalBranchCandidateComparisonBasis::new(
            EquivalenceBasisId::new(43),
            branch_id("main"),
        ))
        .under_fork_observation_basis(FoundationalBranchCandidateForkObservationBasis::new(
            EquivalenceBasisId::new(37),
            BoundaryEpoch::new(4),
        ))
        .under_observation_basis(FoundationalBranchCandidateObservationBasis::new(
            EquivalenceBasisId::new(31),
            BoundaryEpoch::new(5),
        ))
        .from_fork_basis(FoundationalBranchCandidateForkBasis::new(
            branch_id("main"),
            BoundaryEpoch::new(4),
        ))
        .with_candidate_id(FoundationalBranchCandidateId::new(BoundaryHandle::new(17)))
        .on_branch(branch_id("feature/geometry"))
        .stage(payload)
        .expect("candidate")
}

pub fn staged_candidate(payload: &'static str) -> FoundationalStagedBranchArtifact<&'static str> {
    authority_first_candidate(payload).staged()
}
