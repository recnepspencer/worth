pub use std::path::Path;
pub use {
    worth_foundational::certify_foundational_profile_milestone3_production_test_readiness,
    worth_foundational::foundational_profile_milestone3_readiness_report,
    worth_foundational::require_foundational_profile_milestone3_production_test_readiness,
    worth_foundational::FoundationalProfileProductionReadinessCertified,
    worth_foundational::FoundationalProfileProductionTestReadyArtifact,
};

pub fn accepts_profile_readiness_artifact(_: &FoundationalProfileProductionTestReadyArtifact) {}
pub fn accepts_profile_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalProfileProductionReadinessCertified,
        worth_foundational::FoundationalProfileProductionReadinessAuthority,
    >,
) {
}

pub fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-foundational"
    ))
    .join(relative)
}
pub fn compile_fail_fixture_root() -> std::path::PathBuf {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-foundational/assets/ui"
    ))
    .canonicalize()
    .expect("shared compile-fail asset root exists")
}
