use std::collections::BTreeSet;
use std::fmt::Debug;
use std::path::Path;
use worth_foundational::{
    canonical_milestone2_production_readiness_report,
    certify_canonical_milestone2_production_readiness, require_canonical_production_test_readiness,
    CanonicalProductionReadinessCertified, CanonicalProductionTestReadyArtifact,
};

fn accepts_production_readiness_artifact(_: &CanonicalProductionTestReadyArtifact) {}
fn accepts_production_readiness_proof(
    _: &worth_proof::Proof<
        CanonicalProductionReadinessCertified,
        worth_foundational::CanonicalProductionReadinessAuthority,
    >,
) {
}

#[test]
fn production_readiness_artifact_carries_complete_machine_checkable_inventory() {
    let readiness = certify_canonical_milestone2_production_readiness();
    let report = require_canonical_production_test_readiness(&readiness);

    accepts_production_readiness_artifact(&readiness);
    accepts_production_readiness_proof(readiness.proofs());
    assert!(report.passes_readiness_checklist());
    assert_eq!(
        readiness.strong_basis().value().milestone(),
        "worth-foundational.milestone-2"
    );

    for surface in report.certified_surfaces() {
        assert_eq!(
            report
                .certified_surface_evidence()
                .iter()
                .filter(|evidence| evidence.surface() == *surface)
                .count(),
            1,
            "each certified surface must have exactly one evidence row"
        );
    }
}

#[test]
fn production_readiness_surface_evidence_binds_surfaces_to_real_hostile_and_blind_tests() {
    let report = canonical_milestone2_production_readiness_report();

    for evidence in report.certified_surface_evidence() {
        assert!(report.certified_surfaces().contains(&evidence.surface()));
        assert!(report
            .synthetic_pressures()
            .contains(&evidence.hostile_pressure()));
        assert!(report
            .compile_fail_boundaries()
            .contains(&evidence.compile_fail_boundary()));
        assert!(report
            .cost_counter_evidence()
            .contains(&evidence.cost_counter_evidence()));
        assert!(crate_root_path(evidence.blind_consumer_evidence_path()).is_file());
        assert!(crate_root_path(evidence.compile_fail_evidence_path()).is_file());
        assert!(
            evidence
                .compile_fail_evidence_path()
                .starts_with("tests/ui/"),
            "compile-fail evidence must be a trybuild fixture"
        );
        if let Some(artifact) = evidence.golden_artifact() {
            assert!(report.golden_artifacts().contains(&artifact));
            assert!(report
                .fixture_manifest()
                .iter()
                .any(|fixture| fixture.artifact() == artifact));
        }
    }
}

#[test]
fn production_readiness_property_seed_evidence_covers_every_seed() {
    let report = canonical_milestone2_production_readiness_report();

    let evidenced_seeds: Vec<_> = report
        .property_seed_evidence()
        .iter()
        .map(|evidence| evidence.seed())
        .collect();
    assert_exact_inventory(
        "property seed evidence",
        &evidenced_seeds,
        report.property_seeds(),
    );
    assert!(report.property_seed_evidence().iter().all(|evidence| {
        !evidence.hostile_dimension().trim().is_empty()
            && canonicalization_test_path(evidence.owning_test()).is_file()
            && report
                .harness_expansion_points()
                .contains(&evidence.harness_lane())
    }));
}

#[test]
fn production_readiness_fixture_manifest_covers_every_golden_artifact() {
    let report = canonical_milestone2_production_readiness_report();

    let manifest_artifacts: Vec<_> = report
        .fixture_manifest()
        .iter()
        .map(|fixture| fixture.artifact())
        .collect();
    assert_exact_inventory(
        "fixture manifest artifacts",
        &manifest_artifacts,
        report.golden_artifacts(),
    );
}

#[test]
fn production_readiness_phase_gates_are_evidence_backed() {
    let report = canonical_milestone2_production_readiness_report();

    assert!(report
        .phase_gates()
        .iter()
        .all(|evidence| evidence.evidence_path().starts_with("tests/certification/")));
    assert!(report
        .phase_gates()
        .iter()
        .all(|evidence| crate_root_path(evidence.evidence_path()).exists()));
}

#[test]
fn production_readiness_fixture_manifest_points_at_real_owning_tests() {
    let report = canonical_milestone2_production_readiness_report();

    assert!(report
        .fixture_manifest()
        .iter()
        .all(|fixture| { canonicalization_test_path(fixture.owning_test()).is_file() }));
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn canonicalization_test_path(relative: &str) -> std::path::PathBuf {
    crate_root_path("tests/certification/canonicalization").join(relative)
}

fn assert_exact_inventory<T>(name: &str, actual: &[T], expected: &[T])
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
