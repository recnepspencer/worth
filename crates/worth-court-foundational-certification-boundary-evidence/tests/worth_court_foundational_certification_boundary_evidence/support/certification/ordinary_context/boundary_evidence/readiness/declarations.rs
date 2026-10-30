pub(crate) use std::collections::BTreeSet;
pub(crate) use std::path::Path;
pub(crate) use {
    worth_foundational::boundary_evidence_api::stronger_lane::readiness,
    worth_foundational::boundary_evidence_api::BoundaryEvidencePublicLane,
    worth_foundational::FoundationalBoundaryEvidenceProductionReadinessCertified,
    worth_foundational::FoundationalBoundaryEvidenceProductionTestReadyArtifact,
};

pub(crate) fn accepts_boundary_evidence_readiness_artifact(
    _: &FoundationalBoundaryEvidenceProductionTestReadyArtifact,
) {
}

pub(crate) fn accepts_boundary_evidence_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryEvidenceProductionReadinessCertified,
        worth_foundational::FoundationalBoundaryEvidenceProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-foundational"
    ))
    .join(relative)
}
