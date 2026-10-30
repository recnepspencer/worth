pub(crate) use worth_proof::Proof;
pub(crate) use {
    worth_foundational::certify_milestone1_production_test_readiness,
    worth_foundational::milestone1_public_api_inventory,
    worth_foundational::require_milestone1_production_test_readiness,
    worth_foundational::Milestone1ProductionReadinessCertified,
    worth_foundational::Milestone1ProductionTestReadyArtifact,
};

pub(crate) fn accepts_milestone1_readiness_proof(
    _: &Proof<
        Milestone1ProductionReadinessCertified,
        worth_foundational::Milestone1ProductionReadinessAuthority,
    >,
) {
}
