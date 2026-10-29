use crate::application::{
    scan_shared_read_pin_hot_path_forbidden_patterns,
    scan_shared_read_pin_required_pattern_failures, scan_shared_read_pin_retire_forbidden_patterns,
    shared_read_pinning_operation_inventory, WorthQuerySharedReadPinningOperationKind,
};

use super::*;

#[test]
fn shared_read_inventory_rejects_pin_and_retire_residue() {
    let workspace_root = workspace_root();

    let hot_path_failures = scan_shared_read_pin_hot_path_forbidden_patterns(&workspace_root);
    let retire_failures = scan_shared_read_pin_retire_forbidden_patterns(&workspace_root);
    let missing_required_patterns = scan_shared_read_pin_required_pattern_failures(&workspace_root);

    assert!(
        hot_path_failures.is_empty(),
        "shared-read pin hot path forbidden patterns must stay absent: {hot_path_failures:?}"
    );
    assert!(
        retire_failures.is_empty(),
        "shared-read pin retire forbidden patterns must stay absent: {retire_failures:?}"
    );
    assert!(
        missing_required_patterns.is_empty(),
        "shared-read pinning required patterns must stay present: {missing_required_patterns:?}"
    );
}

#[test]
fn shared_read_inventory_names_required_operations() {
    let operations = shared_read_pinning_operation_inventory()
        .iter()
        .inspect(|row| {
            assert!(!row.path().is_empty());
            assert!(!row.function().is_empty());
        })
        .map(|row| row.kind())
        .collect::<std::collections::BTreeSet<_>>();
    let required = [
        WorthQuerySharedReadPinningOperationKind::PinCurrentGeneration,
        WorthQuerySharedReadPinningOperationKind::ReleaseGeneration,
        WorthQuerySharedReadPinningOperationKind::DrainRetiredGeneration,
        WorthQuerySharedReadPinningOperationKind::CaptureCommittedGeneration,
        WorthQuerySharedReadPinningOperationKind::RetainPublishedArtifactGenerations,
        WorthQuerySharedReadPinningOperationKind::ResolvePublishedArtifactGeneration,
        WorthQuerySharedReadPinningOperationKind::MeasureCommittedReadHotPath,
        WorthQuerySharedReadPinningOperationKind::MintSharedReadContext,
        WorthQuerySharedReadPinningOperationKind::InspectSharedReadBasis,
        WorthQuerySharedReadPinningOperationKind::ConsumePublishedArtifact,
        WorthQuerySharedReadPinningOperationKind::ClassifyPinningBoundaryClosure,
    ];

    for required_operation in required {
        assert!(
            operations.contains(&required_operation),
            "shared-read pinning inventory must name {required_operation:?}"
        );
    }
}
