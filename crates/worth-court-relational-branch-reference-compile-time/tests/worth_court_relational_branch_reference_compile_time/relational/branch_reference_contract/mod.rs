use worth_relational::facade::branch::relational_branch_observation;
use worth_relational::facade::history::BranchId;

#[test]
fn empty_relational_targets_remain_runtime_affine_through_branch_identity() {
    let first = relational_branch_observation(
        11,
        "storm",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid branch");
    let second = relational_branch_observation(
        12,
        "storm",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("valid branch");

    assert_ne!(first, second);
    assert!(relational_branch_observation(
        11,
        "   ",
        worth_foundational::FoundationalBranchTarget::empty(),
        worth_foundational::FoundationalBranchReferenceGeneration::initial(),
    )
    .is_err());
}

#[test]
fn owner_branch_identity_is_runtime_affine_without_a_public_target_constructor() {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
    let identity = runtime.main_branch_identity();
    assert_eq!(identity.branch_id(), &BranchId("main".to_owned()));
}
