pub(crate) use {
    worth_foundational::foundational_branch_candidate, worth_foundational::BoundaryEpoch,
    worth_foundational::BoundaryHandle, worth_foundational::EquivalenceBasisId,
    worth_foundational::FoundationalBranchCandidateArtifact,
    worth_foundational::FoundationalBranchCandidateComparisonBasis,
    worth_foundational::FoundationalBranchCandidateForkBasis,
    worth_foundational::FoundationalBranchCandidateForkObservationBasis,
    worth_foundational::FoundationalBranchCandidateId,
    worth_foundational::FoundationalBranchCandidateObservationBasis,
    worth_foundational::FoundationalBranchId, worth_foundational::FoundationalStagedBranchArtifact,
};

pub(crate) fn branch_id(name: &str) -> FoundationalBranchId {
    worth_foundational::FoundationalBranchId::new(name).expect("valid branch id")
}

pub(crate) fn authority_first_candidate(
    payload: &'static str,
) -> FoundationalBranchCandidateArtifact<&'static str> {
    foundational_branch_candidate()
        .on_branch(branch_id("feature/geometry"))
        .with_candidate_id(worth_foundational::FoundationalBranchCandidateId::new(
            worth_foundational::BoundaryHandle::new(17),
        ))
        .from_fork_basis(
            worth_foundational::FoundationalBranchCandidateForkBasis::new(
                branch_id("main"),
                worth_foundational::BoundaryEpoch::new(4),
            ),
        )
        .under_observation_basis(
            worth_foundational::FoundationalBranchCandidateObservationBasis::new(
                worth_foundational::EquivalenceBasisId::new(31),
                worth_foundational::BoundaryEpoch::new(5),
            ),
        )
        .under_fork_observation_basis(
            worth_foundational::FoundationalBranchCandidateForkObservationBasis::new(
                worth_foundational::EquivalenceBasisId::new(37),
                worth_foundational::BoundaryEpoch::new(4),
            ),
        )
        .against_comparison_basis(
            worth_foundational::FoundationalBranchCandidateComparisonBasis::new(
                worth_foundational::EquivalenceBasisId::new(43),
                branch_id("main"),
            ),
        )
        .stage(payload)
        .expect("candidate")
}

pub(crate) fn projection_shaped_candidate(
    payload: &'static str,
) -> FoundationalBranchCandidateArtifact<&'static str> {
    foundational_branch_candidate()
        .against_comparison_basis(
            worth_foundational::FoundationalBranchCandidateComparisonBasis::new(
                worth_foundational::EquivalenceBasisId::new(43),
                branch_id("main"),
            ),
        )
        .under_fork_observation_basis(
            worth_foundational::FoundationalBranchCandidateForkObservationBasis::new(
                worth_foundational::EquivalenceBasisId::new(37),
                worth_foundational::BoundaryEpoch::new(4),
            ),
        )
        .under_observation_basis(
            worth_foundational::FoundationalBranchCandidateObservationBasis::new(
                worth_foundational::EquivalenceBasisId::new(31),
                worth_foundational::BoundaryEpoch::new(5),
            ),
        )
        .from_fork_basis(
            worth_foundational::FoundationalBranchCandidateForkBasis::new(
                branch_id("main"),
                worth_foundational::BoundaryEpoch::new(4),
            ),
        )
        .with_candidate_id(worth_foundational::FoundationalBranchCandidateId::new(
            worth_foundational::BoundaryHandle::new(17),
        ))
        .on_branch(branch_id("feature/geometry"))
        .stage(payload)
        .expect("candidate")
}

pub(crate) fn staged_candidate(
    payload: &'static str,
) -> FoundationalStagedBranchArtifact<&'static str> {
    authority_first_candidate(payload).staged()
}
