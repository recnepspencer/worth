pub(crate) use std::path::Path;
pub(crate) use {
    worth_foundational::certify_foundational_diagnostic_milestone6_production_test_readiness,
    worth_foundational::foundational_diagnostic_milestone6_readiness_report,
    worth_foundational::require_foundational_diagnostic_milestone6_production_test_readiness,
    worth_foundational::FoundationalDiagnosticProductionReadinessCertified,
    worth_foundational::FoundationalDiagnosticProductionTestReadyArtifact,
};

pub(crate) fn accepts_diagnostic_readiness_artifact(
    _: &FoundationalDiagnosticProductionTestReadyArtifact,
) {
}
pub(crate) fn accepts_diagnostic_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalDiagnosticProductionReadinessCertified,
        worth_foundational::FoundationalDiagnosticProductionReadinessAuthority,
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
pub(crate) fn compile_fail_fixture_root() -> std::path::PathBuf {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-foundational/assets/ui"
    ))
    .canonicalize()
    .expect("shared compile-fail asset root exists")
}
