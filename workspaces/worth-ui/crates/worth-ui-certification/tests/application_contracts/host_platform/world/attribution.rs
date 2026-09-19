use std::collections::HashSet;

use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

pub(super) fn assert_exact_attribution(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    expected_instances: &[UiMountedInstanceIdentity],
    expected_surface: UiSemanticSurfaceIdentity,
) {
    let rows = super::ordered_surfaces(transcript);
    assert_eq!(rows.len(), expected_instances.len());
    let mut instances = HashSet::with_capacity(rows.len());
    let mut receipts = HashSet::with_capacity(rows.len());
    for (row, expected_instance) in rows.iter().zip(expected_instances) {
        let receipt = row.node_receipt();
        assert_eq!(receipt.mounted_instance(), *expected_instance);
        assert!(
            instances.insert(receipt.mounted_instance()),
            "duplicate mounted attribution"
        );
        assert!(receipts.insert(receipt), "duplicate receipt attribution");
    }
    let work = transcript
        .appearance_work()
        .expect("appearance work is required");
    assert!(work.fragments().iter().all(|fragment| {
        fragment.surface_binding().semantic_surface() == expected_surface
            && fragment.surface_binding().binding() == transcript.binding()
            && fragment.work().successor().semantic_surface() == expected_surface
    }));
}
