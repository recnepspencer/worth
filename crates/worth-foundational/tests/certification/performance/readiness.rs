use std::collections::BTreeSet;
use std::path::Path;
use worth_foundational::{
    performance_api::{
        stronger_lane::readiness, FoundationalPerformancePublicLane,
        FoundationalPerformancePublicSurfaceEntry,
    },
    FoundationalPerformanceProductionReadinessCertified,
    FoundationalPerformanceProductionTestReadyArtifact,
    FoundationalPerformancePublicSurfaceDocumentationCoverage,
};

fn accepts_performance_readiness_artifact(_: &FoundationalPerformanceProductionTestReadyArtifact) {}

fn accepts_performance_readiness_proof(
    _: &worth_proof::Proof<
        FoundationalPerformanceProductionReadinessCertified,
        worth_foundational::FoundationalPerformanceProductionReadinessAuthority,
    >,
) {
}

#[test]
fn production_readiness_artifact_carries_complete_machine_checkable_inventory() {
    let readiness_artifact =
        readiness::certify_foundational_performance_milestone8_production_test_readiness();
    let report = readiness::require_foundational_performance_milestone8_production_test_readiness(
        &readiness_artifact,
    );

    accepts_performance_readiness_artifact(&readiness_artifact);
    accepts_performance_readiness_proof(readiness_artifact.proofs());
    assert!(report.passes_readiness_checklist());
    assert_eq!(
        readiness_artifact.strong_basis().value().milestone(),
        "worth-foundational.milestone-8"
    );
}

#[test]
fn production_readiness_binds_surfaces_to_real_tests_and_exact_public_surface_inventory() {
    let report = readiness::foundational_performance_milestone8_readiness_report();

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
        .map(FoundationalPerformancePublicSurfaceEntry::path)
        .collect();
    let doc_paths: BTreeSet<_> = report
        .documentation_surface_inventory()
        .iter()
        .copied()
        .collect();
    let coverage_surface_paths: BTreeSet<_> = report
        .public_surface_documentation_coverage()
        .iter()
        .map(FoundationalPerformancePublicSurfaceDocumentationCoverage::public_surface_path)
        .collect();
    let stronger_lane_count = report
        .public_surface_inventory()
        .iter()
        .filter(|entry| entry.lane() == FoundationalPerformancePublicLane::StrongerLane)
        .count();

    assert_eq!(report.public_surface_inventory().len(), 9);
    assert_eq!(stronger_lane_count, 3);
    assert!(paths.contains("worth_foundational::performance_api::common_path"));
    assert!(paths.contains("worth_foundational::performance_api::stronger_lane::certified"));
    assert!(paths.contains("worth_foundational::performance_api::stronger_lane::readiness"));
    assert!(doc_paths.contains("docs/performance/README.md"));
    assert!(doc_paths.contains("docs/performance/canonical-bundles-and-comparison.md"));
    assert!(doc_paths.contains("docs/performance/counter-backed-performance-receipts.md"));
    assert!(
        doc_paths.contains("docs/performance/performance-report-planning-and-materialization.md")
    );
    assert!(doc_paths.contains("docs/performance/certified-and-readmitted-performance-bundles.md"));
    assert_eq!(paths, coverage_surface_paths);
    assert!(crate_root_path(report.public_surface_evidence_path()).is_file());
    assert!(crate_root_path(report.public_surface_compile_fail_path()).is_file());
    for path in report.documentation_surface_inventory() {
        assert!(crate_root_path(path).is_file());
    }
    for row in report.public_surface_documentation_coverage() {
        assert!(crate_root_path(row.primary_documentation_path()).is_file());
    }
}

#[test]
fn production_readiness_phase_gates_are_evidence_backed() {
    let report = readiness::foundational_performance_milestone8_readiness_report();

    for evidence in report.phase_gates() {
        assert!(crate_root_path(evidence.evidence_path()).exists());
    }
}

#[test]
fn production_readiness_runtime_adoption_proof_paths_exist() {
    let report = readiness::foundational_performance_milestone8_readiness_report();

    for row in report.runtime_adoption_pressure_evidence() {
        assert!(crate_root_path(row.evidence_path()).is_file());
    }
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
