use worth_foundational::{
    foundational_branch_candidate, BoundaryEpoch, BoundaryHandle, EquivalenceBasisId,
    FoundationalBranchCandidateId, FoundationalBranchCandidateForkBasis, FoundationalBranchId,
    FoundationalBranchCandidateObservationBasis,
};

fn main() {
    let discard = foundational_branch_candidate()
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
        .discard_with_zero_residue_proof()
        .unwrap();

    let _ = discard.commit_id();
}
