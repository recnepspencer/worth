pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::certify_foundational_transition_milestone5_production_test_readiness, worth_foundational::certify_foundational_transition_milestone9_scoped_merge_production_test_readiness, worth_foundational::foundational_transition_milestone5_readiness_report, worth_foundational::foundational_transition_milestone9_scoped_merge_readiness_report, worth_foundational::require_foundational_transition_milestone5_production_test_readiness, worth_foundational::require_foundational_transition_milestone9_scoped_merge_production_test_readiness, worth_foundational::FoundationalTransitionCertifiedSurface, worth_foundational::FoundationalTransitionProductionReadinessCertified, worth_foundational::FoundationalTransitionProductionTestReadyArtifact};

pub(crate) fn accepts_transition_readiness_artifact(_: &FoundationalTransitionProductionTestReadyArtifact) {}
pub(crate) fn accepts_transition_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalTransitionProductionReadinessCertified,
        worth_foundational::FoundationalTransitionProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}