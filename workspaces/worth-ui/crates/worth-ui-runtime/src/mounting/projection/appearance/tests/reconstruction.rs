use super::*;

#[test]
fn reconstruction_rebuilds_from_current_receipt_without_semantic_replay() {
    let (input, ids) = node_input([12, 34, 56, 255], 7, true);
    let (successor, successor_ids) = node_input_for([12, 34, 56, 255], 7, true, Some(&ids));
    let mut sidecar = UiMountedAppearanceSidecar::default();
    sidecar.mount(input).unwrap();

    let work = sidecar
        .reconstruct(successor)
        .expect("current receipt should rebuild retained appearance facts");
    let manifest = work
        .predecessor_manifest()
        .expect("reconstruction keeps the retained predecessor manifest");
    let retained_identities = sidecar
        .current()
        .unwrap()
        .records()
        .iter()
        .map(|record| record.identity().clone())
        .collect::<Vec<_>>();

    assert_eq!(
        work.posture(),
        UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(work.predecessor(), Some(ids.frame));
    assert_eq!(work.successor().frame(), successor_ids.frame);
    assert_eq!(
        manifest.mechanic_identities(),
        retained_identities.as_slice()
    );
    assert_eq!(
        manifest.overlay_order(),
        work.successor().overlay_order().bottom_to_top()
    );
    assert!(work.changes().is_empty());
    assert!(work.damage().is_empty());
    assert!(!work.order_changed());
    assert_eq!(
        sidecar
            .current()
            .unwrap()
            .record(&UiMountedAppearanceMechanicIdentity::Surface(
                successor_ids.instance,
            ))
            .unwrap()
            .node_receipt(),
        Some(successor_ids.receipt)
    );
}
