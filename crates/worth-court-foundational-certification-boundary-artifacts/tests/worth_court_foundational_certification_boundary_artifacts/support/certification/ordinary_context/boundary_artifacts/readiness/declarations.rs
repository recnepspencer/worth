pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::certify_foundational_boundary_artifact_milestone4_production_test_readiness, worth_foundational::foundational_boundary_artifact_milestone4_readiness_report, worth_foundational::require_foundational_boundary_artifact_milestone4_production_test_readiness, worth_foundational::FoundationalBoundaryArtifactProductionReadinessCertified, worth_foundational::FoundationalBoundaryArtifactProductionTestReadyArtifact};

pub(crate) fn accepts_boundary_artifact_readiness_artifact(
    _: &FoundationalBoundaryArtifactProductionTestReadyArtifact,
) {
}

pub(crate) fn accepts_boundary_artifact_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryArtifactProductionReadinessCertified,
        worth_foundational::FoundationalBoundaryArtifactProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}