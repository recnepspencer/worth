use super::*;

#[test]
fn ordered_portal_and_backdrop_rows_damage_extent_without_creating_input() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let portal = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = issuer.receipt_for(portal);
    let bounds = UiAppearanceAllocationBounds::new(0, 0, 120, 90).unwrap();
    let zero = UiAppearanceLogicalLength::ZERO;
    let radii = UiAppearanceNormalizedLogicalRadii::normalize(bounds, [zero; 4]);
    let projection =
        UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 21, 4).unwrap();
    let node = UiMountedAppearanceNodeInput {
        geometry_input: None,
        issuer,
        semantic_surface: surface,
        node_receipt: receipt,
        projection,
        bounds,
        clip: crate::mounting::projection::appearance::UiMountedAppearanceClip::Ancestor(
            UiAppearanceClip::new(0, 0, 100, 70).unwrap(),
        ),
        surface_paint_order: Some(0),
        radii,
        surface_border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL,
        surface_border_omissions: Box::new([]),
        surface_paint: Some(UiMountedSurfacePaint::Fill(
            UiMountedAppearanceColor::from_straight_srgba([0, 0, 0, 255]),
        )),
        outline: None,
        text_foregrounds: Box::new([]),
        appearance_opacity: UiMountedAppearanceOpacity::ONE,
        motion_opacity: None,
        semantic_digest: 13,
        portal_instance: Some(portal),
    };
    let placement = UiOverlayPlacementReceipt::from_runtime_overlay_order(7, 0).unwrap();
    let identity = UiMountedBackdropIdentity::from_runtime_mounting(
        "dialog.backdrop",
        UiMountedBackdropScope::PerPortalInstance(portal),
        1,
    )
    .unwrap();
    let backdrop = UiMountedAppearanceBackdropInput {
        identity: identity.clone(),
        semantic_surface: surface,
        placement,
        extent: worth_ui_host_contract::UiAppearanceBackdropExtent::new(0, 0, 200, 160).unwrap(),
        clip: UiAppearanceClip::new(0, 0, 100, 100).unwrap(),
        background: UiMountedAppearanceColor::from_straight_srgba([0, 0, 0, 128]),
        appearance_opacity: UiMountedAppearanceOpacity::ONE,
        motion_opacity: None,
        motion_target: None,
        attribution: UiMountedBackdropAppearanceAttribution::from_runtime_transport(
            surface, placement, 31, 2,
        )
        .unwrap(),
        semantic_digest: 31,
    };
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        surface,
        presentation,
        7,
        9,
        [
            UiOverlayParticipantIdentity::Backdrop(identity.clone()),
            UiOverlayParticipantIdentity::Portal(portal),
        ],
    )
    .unwrap();
    let mut sidecar = UiMountedAppearanceSidecar::default();
    let work = sidecar
        .mount(UiMountedAppearanceLoweringInput {
            frame,
            semantic_surface: surface,
            presentation,
            nodes: vec![node],
            backdrops: vec![backdrop],
            overlay: UiMountedAppearanceOverlayInput {
                semantic_surface: surface,
                presentation,
                portal_revision: 7,
                backdrop_revision: 9,
                bottom_to_top: Box::new([
                    UiOverlayParticipantIdentity::Backdrop(identity.clone()),
                    UiOverlayParticipantIdentity::Portal(portal),
                ]),
            },
        })
        .unwrap();

    assert_eq!(work.posture(), UiMountedAppearanceWorkPosture::Initial);
    assert_eq!(
        work.successor().overlay_order().bottom_to_top(),
        order.bottom_to_top()
    );
    assert_eq!(work.damage().len(), 1);
    assert_eq!(work.damage()[0].x(), 0);
    assert_eq!(work.damage()[0].y(), 0);
    assert_eq!(work.damage()[0].width(), 120);
    assert_eq!(work.damage()[0].height(), 100);
    assert!(work
        .successor()
        .mechanics()
        .iter()
        .any(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::PortalSurface(_))));
    assert!(work
        .successor()
        .mechanics()
        .iter()
        .any(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(_))));
    assert!(sidecar.current().unwrap().records().iter().any(|record| {
        matches!(
            record.mechanic(),
            UiMountedAppearanceMechanic::Backdrop(mechanic)
                if mechanic.extent().width() == 200 && !mechanic.participates_in_hit_testing()
        )
    }));
}
