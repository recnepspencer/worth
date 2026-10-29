#[allow(unused_imports)] use crate::certification::ordinary_context::boundary::milestone1_readiness::*;
use {worth_foundational::certify_milestone1_production_test_readiness, worth_foundational::milestone1_public_api_inventory, worth_foundational::require_milestone1_production_test_readiness, worth_foundational::Milestone1ProductionReadinessCertified, worth_foundational::Milestone1ProductionTestReadyArtifact};
use {worth_proof::Proof};

#[test]
fn milestone_1_production_test_readiness_is_proof_bearing() {
    let readiness: Milestone1ProductionTestReadyArtifact =
        certify_milestone1_production_test_readiness();
    let report = require_milestone1_production_test_readiness(&readiness);

    accepts_milestone1_readiness_proof(readiness.proofs());
    assert_eq!(report.public_api(), milestone1_public_api_inventory());
}
