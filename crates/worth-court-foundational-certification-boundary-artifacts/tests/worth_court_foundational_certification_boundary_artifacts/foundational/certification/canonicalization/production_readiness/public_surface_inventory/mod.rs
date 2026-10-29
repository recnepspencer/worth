#[allow(unused_imports)] use crate::certification::ordinary_context::canonicalization::production_readiness::public_surface_inventory::*;
use {std::path::Path};
use {worth_foundational::canonical_milestone2_production_readiness_report, worth_foundational::canonicalization_api::canonical_public_surface_inventory, worth_foundational::CanonicalHarnessExpansionPoint};

#[test]
fn readiness_artifact_freezes_public_surface_inventory_and_grouped_lane_evidence() {
    let report = canonical_milestone2_production_readiness_report();

    assert_eq!(
        report.public_surface_inventory(),
        canonical_public_surface_inventory()
    );
    assert!(report
        .harness_expansion_points()
        .contains(&worth_foundational::CanonicalHarnessExpansionPoint::GroupedPublicSurfaceLane));
    assert!(
        crate_root_path(report.public_surface_evidence_path()).is_file(),
        "grouped public-surface evidence path must point at a real certification file"
    );
    assert!(
        crate_root_path(report.public_surface_compile_fail_path()).is_file(),
        "grouped public-surface compile-fail path must point at a real fixture"
    );
}
