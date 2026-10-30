use worth_foundational::{foundational_merge, FoundationalSelectedNodeLocus};

fn requires_scope(_: worth_foundational::FoundationalMergeScope) {}

fn main() {
    requires_scope("gear");

    let source = worth_foundational::foundational_branch_candidate()
        .on_branch(worth_foundational::FoundationalBranchId::new("feature").unwrap())
        .with_candidate_id(worth_foundational::FoundationalBranchCandidateId::new(
            worth_foundational::BoundaryHandle::new(1),
        ))
        .from_fork_basis(worth_foundational::FoundationalBranchCandidateForkBasis::new(
            worth_foundational::FoundationalBranchId::new("main").unwrap(),
            worth_foundational::BoundaryEpoch::new(1),
        ))
        .under_observation_basis(worth_foundational::FoundationalBranchCandidateObservationBasis::new(
            worth_foundational::EquivalenceBasisId::new(1),
            worth_foundational::BoundaryEpoch::new(1),
        ))
        .stage(())
        .unwrap()
        .staged();

    let _ = foundational_merge(source).with_scope(FoundationalSelectedNodeLocus::new("gear").unwrap());
}
