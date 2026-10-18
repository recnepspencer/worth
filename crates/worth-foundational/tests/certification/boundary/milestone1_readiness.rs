use worth_foundational::{
    certify_milestone1_production_test_readiness, milestone1_public_api_inventory,
    require_milestone1_production_test_readiness, Milestone1ProductionReadinessCertified,
    Milestone1ProductionTestReadyArtifact,
};
use worth_proof::Proof;

fn accepts_milestone1_readiness_proof(
    _: &Proof<
        Milestone1ProductionReadinessCertified,
        worth_foundational::Milestone1ProductionReadinessAuthority,
    >,
) {
}

#[test]
fn milestone_1_production_test_readiness_is_proof_bearing() {
    let readiness: Milestone1ProductionTestReadyArtifact =
        certify_milestone1_production_test_readiness();
    let report = require_milestone1_production_test_readiness(&readiness);

    accepts_milestone1_readiness_proof(readiness.proofs());
    assert_eq!(report.public_api(), milestone1_public_api_inventory());
}
