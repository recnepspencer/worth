use std::path::Path;
use worth_foundational::{
    canonical_milestone2_production_readiness_report,
    canonicalization_api::canonical_public_surface_inventory, CanonicalHarnessExpansionPoint,
};

#[test]
fn readiness_artifact_freezes_public_surface_inventory_and_grouped_lane_evidence() {
    let report = canonical_milestone2_production_readiness_report();

    assert_eq!(
        report.public_surface_inventory(),
        canonical_public_surface_inventory()
    );
    assert!(report
        .harness_expansion_points()
        .contains(&CanonicalHarnessExpansionPoint::GroupedPublicSurfaceLane));
    assert!(
        crate_root_path(report.public_surface_evidence_path()).is_file(),
        "grouped public-surface evidence path must point at a real certification file"
    );
    assert!(
        crate_root_path(report.public_surface_compile_fail_path()).is_file(),
        "grouped public-surface compile-fail path must point at a real fixture"
    );
}

fn crate_root_path(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
