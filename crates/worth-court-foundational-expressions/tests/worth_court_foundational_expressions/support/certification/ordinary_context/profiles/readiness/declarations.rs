pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::certify_foundational_profile_milestone3_production_test_readiness, worth_foundational::foundational_profile_milestone3_readiness_report, worth_foundational::require_foundational_profile_milestone3_production_test_readiness, worth_foundational::FoundationalProfileProductionReadinessCertified, worth_foundational::FoundationalProfileProductionTestReadyArtifact};

pub(crate) fn accepts_profile_readiness_artifact(_: &FoundationalProfileProductionTestReadyArtifact) {}
pub(crate) fn accepts_profile_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalProfileProductionReadinessCertified,
        worth_foundational::FoundationalProfileProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}