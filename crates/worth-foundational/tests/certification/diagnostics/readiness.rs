use std::path::Path;
use worth_foundational::{
    certify_foundational_diagnostic_milestone6_production_test_readiness,
    foundational_diagnostic_milestone6_readiness_report,
    require_foundational_diagnostic_milestone6_production_test_readiness,
    FoundationalDiagnosticProductionReadinessCertified,
    FoundationalDiagnosticProductionTestReadyArtifact,
};

fn accepts_diagnostic_readiness_artifact(_: &FoundationalDiagnosticProductionTestReadyArtifact) {}
fn accepts_diagnostic_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalDiagnosticProductionReadinessCertified,
        worth_foundational::FoundationalDiagnosticProductionReadinessAuthority,
    >,
) {
}

#[test]
fn production_readiness_artifact_carries_complete_machine_checkable_inventory() {
    let readiness = certify_foundational_diagnostic_milestone6_production_test_readiness();
    let report = require_foundational_diagnostic_milestone6_production_test_readiness(&readiness);

    accepts_diagnostic_readiness_artifact(&readiness);
    accepts_diagnostic_readiness_proof(readiness.proofs());
    assert!(report.passes_readiness_checklist());
    assert_eq!(
        readiness.strong_basis().value().milestone(),
        "worth-foundational.milestone-6"
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
    let report = foundational_diagnostic_milestone6_readiness_report();

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
fn production_readiness_hostile_pressures_and_compile_fail_boundaries_have_real_evidence_paths() {
    let report = foundational_diagnostic_milestone6_readiness_report();

    for evidence in report.synthetic_pressure_evidence() {
        assert!(report.synthetic_pressures().contains(&evidence.pressure()));
        assert!(crate_root_path(evidence.owning_test_path()).is_file());
    }

    for evidence in report.canonical_golden_artifact_evidence() {
        assert!(report
            .canonical_golden_artifacts()
            .contains(&evidence.artifact()));
        assert!(crate_root_path(evidence.evidence_path()).is_file());
    }

    for evidence in report.property_seed_evidence() {
        assert!(report.property_seed_inventory().contains(&evidence.seed()));
        assert!(crate_root_path(evidence.owning_test_path()).is_file());
        assert!(!evidence.hostile_dimension().trim().is_empty());
    }

    for evidence in report.harness_expansion_evidence() {
        assert!(report
            .harness_expansion_points()
            .contains(&evidence.point()));
        assert!(crate_root_path(evidence.owning_test_path()).is_file());
    }

    for evidence in report.compile_fail_evidence() {
        assert!(report
            .compile_fail_boundaries()
            .contains(&evidence.boundary()));
        assert!(crate_root_path(evidence.evidence_path()).is_file());
        assert!(
            evidence.evidence_path().starts_with("tests/ui/"),
            "compile-fail evidence must point at a trybuild fixture"
        );
    }
}

#[test]
fn production_readiness_worth_proof_appendix_is_bound_to_real_diagnostics_source_lanes() {
    let report = foundational_diagnostic_milestone6_readiness_report();

    for evidence in report.worth_proof_api_evidence() {
        assert!(report.worth_proof_api_appendix().contains(&evidence.api()));
        let source = std::fs::read_to_string(crate_root_path(evidence.source_path()))
            .expect("worth-proof api evidence source must be readable");
        assert!(
            source.contains(evidence.source_snippet()),
            "worth-proof api evidence for {:?} drifted from the named source lane",
            evidence.api()
        );
    }
}

#[test]
fn production_readiness_phase_gates_are_evidence_backed() {
    let report = foundational_diagnostic_milestone6_readiness_report();

    assert!(report
        .phase_gates()
        .iter()
        .all(|evidence| crate_root_path(evidence.evidence_path()).exists()));
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
