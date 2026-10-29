pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::performance_api::stronger_lane::readiness, worth_foundational::performance_api::FoundationalPerformancePublicLane, worth_foundational::performance_api::FoundationalPerformancePublicSurfaceEntry, worth_foundational::FoundationalPerformanceProductionReadinessCertified, worth_foundational::FoundationalPerformanceProductionTestReadyArtifact, worth_foundational::FoundationalPerformancePublicSurfaceDocumentationCoverage};

pub(crate) fn accepts_performance_readiness_artifact(_: &FoundationalPerformanceProductionTestReadyArtifact) {}

pub(crate) fn accepts_performance_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalPerformanceProductionReadinessCertified,
        worth_foundational::FoundationalPerformanceProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}