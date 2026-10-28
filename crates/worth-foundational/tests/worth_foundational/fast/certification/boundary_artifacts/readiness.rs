use std::path::Path;
use worth_foundational::{
    certify_foundational_boundary_artifact_milestone4_production_test_readiness,
    foundational_boundary_artifact_milestone4_readiness_report,
    require_foundational_boundary_artifact_milestone4_production_test_readiness,
    FoundationalBoundaryArtifactProductionReadinessCertified,
    FoundationalBoundaryArtifactProductionTestReadyArtifact,
};

fn accepts_boundary_artifact_readiness_artifact(
    _: &FoundationalBoundaryArtifactProductionTestReadyArtifact,
) {
}

fn accepts_boundary_artifact_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryArtifactProductionReadinessCertified,
        worth_foundational::FoundationalBoundaryArtifactProductionReadinessAuthority,
    >,
) {
}

#[test]
fn production_readiness_artifact_carries_complete_machine_checkable_inventory() {
    let readiness = certify_foundational_boundary_artifact_milestone4_production_test_readiness();
    let report =
        require_foundational_boundary_artifact_milestone4_production_test_readiness(&readiness);

    accepts_boundary_artifact_readiness_artifact(&readiness);
    accepts_boundary_artifact_readiness_proof(readiness.proofs());
    assert!(report.passes_readiness_checklist());
    assert_eq!(
        readiness.strong_basis().value().milestone(),
        "worth-foundational.milestone-4"
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
fn production_readiness_surface_evidence_binds_surfaces_to_real_hostile_compile_fail_and_blind_consumer_tests(
) {
    let report = foundational_boundary_artifact_milestone4_readiness_report();

    for evidence in report.certified_surface_evidence() {
        assert!(report.certified_surfaces().contains(&evidence.surface()));
        assert!(report
            .synthetic_pressures()
            .contains(&evidence.hostile_pressure()));
        assert!(report
            .compile_fail_boundaries()
            .contains(&evidence.compile_fail_boundary()));
        assert!(crate_root_path(evidence.owning_test_path()).is_file());
        assert!(crate_root_path(evidence.compile_fail_evidence_path()).is_file());
        assert!(crate_root_path(evidence.blind_consumer_evidence_path()).is_file());
        assert!(
            evidence
                .compile_fail_evidence_path()
                .starts_with("tests/ui/"),
            "compile-fail evidence must point at a trybuild fixture"
        );
    }
}

#[test]
fn production_readiness_phase_gates_are_evidence_backed() {
    let report = foundational_boundary_artifact_milestone4_readiness_report();

    assert!(report
        .phase_gates()
        .iter()
        .all(|evidence| crate_root_path(evidence.evidence_path()).exists()));
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
