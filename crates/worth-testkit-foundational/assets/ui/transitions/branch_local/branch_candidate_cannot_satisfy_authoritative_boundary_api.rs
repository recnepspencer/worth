use worth_foundational::{
    admit_authoritative_current_boundary_surface, foundational_boundary_authority_admission,
    foundational_branch_candidate, BoundaryEpoch, BoundaryHandle, EquivalenceBasisId,
    FoundationalBranchCandidateId, FoundationalBranchCandidateForkBasis, FoundationalBranchId,
    FoundationalBranchCandidateObservationBasis,
};

fn main() {
    let candidate = foundational_branch_candidate()
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
        .unwrap();

    let authority = foundational_boundary_authority_admission();
    let _ = admit_authoritative_current_boundary_surface(candidate, authority);
}
