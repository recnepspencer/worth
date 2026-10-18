use std::collections::BTreeSet;
use std::path::Path;
use worth_foundational::{
    boundary_evidence_api::{stronger_lane::readiness, BoundaryEvidencePublicLane},
    FoundationalBoundaryEvidenceProductionReadinessCertified,
    FoundationalBoundaryEvidenceProductionTestReadyArtifact,
};

fn accepts_boundary_evidence_readiness_artifact(
    _: &FoundationalBoundaryEvidenceProductionTestReadyArtifact,
) {
}

fn accepts_boundary_evidence_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryEvidenceProductionReadinessCertified,
        worth_foundational::FoundationalBoundaryEvidenceProductionReadinessAuthority,
    >,
) {
}

#[test]
fn production_readiness_artifact_carries_complete_machine_checkable_inventory() {
    let readiness_artifact =
        readiness::certify_foundational_boundary_evidence_milestone7_production_test_readiness();
    let report =
        readiness::require_foundational_boundary_evidence_milestone7_production_test_readiness(
            &readiness_artifact,
        );

    accepts_boundary_evidence_readiness_artifact(&readiness_artifact);
    accepts_boundary_evidence_readiness_proof(readiness_artifact.proofs());
    assert!(report.passes_readiness_checklist());
    assert_eq!(
        readiness_artifact.strong_basis().value().milestone(),
        "worth-foundational.milestone-7"
    );

    for path in report.documentation_surface_inventory() {
        assert!(crate_root_path(path).is_file());
    }
}

#[test]
fn production_readiness_binds_surfaces_to_real_tests_and_exact_public_surface_inventory() {
    let report = readiness::foundational_boundary_evidence_milestone7_readiness_report();

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
    }

    let paths: BTreeSet<_> = report
        .public_surface_inventory()
        .iter()
        .map(|entry| entry.path())
        .collect();
    let stronger_lane_count = report
        .public_surface_inventory()
        .iter()
        .filter(|entry| entry.lane() == BoundaryEvidencePublicLane::StrongerLane)
        .count();

    assert_eq!(report.public_surface_inventory().len(), 10);
    assert_eq!(stronger_lane_count, 3);
    assert!(paths.contains("worth_foundational::boundary_evidence_api::common_path"));
    assert!(paths.contains("worth_foundational::boundary_evidence_api::stronger_lane::readmission"));
    assert!(paths.contains("worth_foundational::boundary_evidence_api::stronger_lane::readiness"));
    assert!(crate_root_path(report.public_surface_evidence_path()).is_file());
    assert!(crate_root_path(report.public_surface_compile_fail_path()).is_file());
    let crate_root_docs =
        std::fs::read_to_string(crate_root_path("src/lib.rs")).expect("crate root docs");
    assert!(crate_root_docs.contains("docs/README.md"));
    assert!(
        crate_root_docs.contains("docs/lineage-provenance-receipts-and-support-truth/README.md")
    );
}

#[test]
fn production_readiness_phase_gates_are_evidence_backed() {
    let report = readiness::foundational_boundary_evidence_milestone7_readiness_report();

    for evidence in report.phase_gates() {
        assert!(crate_root_path(evidence.evidence_path()).exists());
    }
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
