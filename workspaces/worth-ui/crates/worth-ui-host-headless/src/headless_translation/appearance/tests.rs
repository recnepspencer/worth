fn empty_frame() -> worth_ui_host_contract::UiMountedAppearanceFrame {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let order =
        worth_ui_host_contract::UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            surface,
            presentation,
            1,
            1,
            [],
        )
        .unwrap();
    worth_ui_host_contract::UiMountedAppearanceFrame::from_runtime_mounting(
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface,
        [],
        order,
    )
    .unwrap()
}

fn empty_predecessor_manifest() -> worth_ui_host_contract::UiMountedAppearancePredecessorManifest {
    worth_ui_host_contract::UiMountedAppearancePredecessorManifest::from_runtime_mounting([], [])
        .unwrap()
}

#[test]
fn initial_translation_preserves_an_unpublished_empty_structure() {
    let successor = empty_frame();
    let source = worth_ui_host_contract::UiMountedAppearanceWork::from_runtime_mounting(
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Initial,
        None,
        None,
        successor.clone(),
        [],
        [],
        true,
    )
    .unwrap();

    let transcript = super::super::translate_unpublished_appearance_work(&source).unwrap();

    assert_eq!(
        transcript.posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Initial
    );
    assert_eq!(transcript.predecessor(), None);
    assert!(transcript.order_changed());
    assert_eq!(transcript.successor().frame(), successor.frame());
    assert_eq!(
        transcript.successor().semantic_surface(),
        successor.semantic_surface()
    );
    assert_eq!(
        transcript.successor().overlay_order(),
        successor.overlay_order()
    );
    assert!(transcript.successor().mechanics().is_empty());
    assert!(transcript.changes().is_empty());
    assert!(transcript.damage().is_empty());
}

#[test]
fn unchanged_translation_preserves_predecessor_and_zero_work() {
    let successor = empty_frame();
    let predecessor = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let source = worth_ui_host_contract::UiMountedAppearanceWork::from_runtime_mounting(
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Unchanged,
        Some(predecessor),
        Some(empty_predecessor_manifest()),
        successor,
        [],
        [],
        false,
    )
    .unwrap();

    let transcript = super::super::translate_unpublished_appearance_work(&source).unwrap();

    assert_eq!(transcript.predecessor(), Some(predecessor));
    assert_eq!(
        transcript.posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Unchanged
    );
    assert!(!transcript.order_changed());
    assert!(transcript.changes().is_empty());
    assert!(transcript.damage().is_empty());
}
