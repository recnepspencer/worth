pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::fmt::Debug};
pub(crate) use {std::path::Path};
pub(crate) use {worth_foundational::canonical_milestone2_production_readiness_report, worth_foundational::certify_canonical_milestone2_production_readiness, worth_foundational::require_canonical_production_test_readiness, worth_foundational::CanonicalProductionReadinessCertified, worth_foundational::CanonicalProductionTestReadyArtifact};

pub(crate) fn accepts_production_readiness_artifact(_: &CanonicalProductionTestReadyArtifact) {}
pub(crate) fn accepts_production_readiness_proof(
    _: &worth_proof::Proof<
        CanonicalProductionReadinessCertified,
        worth_foundational::CanonicalProductionReadinessAuthority,
    >,
) {
}

pub(crate) fn crate_root_path(relative: &str) -> std::path::PathBuf {
    std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-foundational")).join(relative)
}

pub(crate) fn canonicalization_test_path(relative: &str) -> std::path::PathBuf {
    crate_root_path("tests/worth_foundational/fast/certification/canonicalization").join(relative)
}

pub(crate) fn assert_exact_inventory<T>(name: &str, actual: &[T], expected: &[T])
where
    T: Copy + Debug + Ord,
{
    let actual_set: BTreeSet<_> = actual.iter().copied().collect();
    let expected_set: BTreeSet<_> = expected.iter().copied().collect();

    assert_eq!(
        actual.len(),
        expected.len(),
        "{name} contains duplicate rows"
    );
    assert_eq!(
        actual_set, expected_set,
        "{name} changed without updating readiness certification"
    );
}