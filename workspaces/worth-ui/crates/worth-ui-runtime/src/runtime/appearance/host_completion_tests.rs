#[test]
fn runtime_can_complete_inert_mechanics_without_publication_authority() {
    use worth_ui_host_contract::{
        UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
        UiAppearanceLogicalLength, UiAppearanceNormalizedLogicalRadii, UiAppearanceOutlineGeometry,
        UiHostPointerIdentity, UiMountedAppearanceColor, UiMountedAppearanceOpacity,
        UiMountedBackdropAppearanceAttribution, UiMountedBackdropCompletionInput,
        UiMountedBackdropIdentity, UiMountedBackdropMechanic, UiMountedBackdropScope,
        UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedLayerProjection,
        UiMountedLayerReference, UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIssuer,
        UiMountedOutlineAppearanceCompletionInput, UiMountedOutlineAppearanceMechanic,
        UiMountedOverlayOrderMechanic, UiMountedPointerAffordanceMechanic,
        UiMountedPortalSurfaceAppearanceMechanic, UiMountedPresentationAttemptIdentity,
        UiMountedSurfaceAppearanceCompletionInput, UiMountedSurfaceAppearanceMechanic,
        UiMountedSurfacePaint, UiMountedTextForegroundAppearanceCompletionInput,
        UiMountedTextForegroundAppearanceMechanic, UiMountedTextPaintSpanIdentity,
        UiOverlayParticipantIdentity, UiOverlayPlacementReceipt, UiPointerAffordanceFamily,
        UiSemanticSurfaceIdentity,
    };

    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let node_issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let pointer = UiHostPointerIdentity::new(9);
    let allocation = UiAppearanceAllocationBounds::new(0, 0, 8_000, 8_000).unwrap();
    let backdrop_extent = UiAppearanceBackdropExtent::new(0, 0, 8_000, 8_000).unwrap();
    let clip = UiAppearanceClip::new(0, 0, 8_000, 8_000).unwrap();
    let zero = UiAppearanceLogicalLength::ZERO;
    let radii = UiAppearanceNormalizedLogicalRadii::normalize(allocation, [zero; 4]);
    let color = UiMountedAppearanceColor::from_straight_srgba([12, 34, 56, 255]);
    let projection =
        UiMountedNodeAppearanceAttribution::from_runtime_mounting(node_issuer, 1, 1).unwrap();
    let surface_mechanic = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer: node_issuer,
            node_receipt: node_issuer.receipt_for(portal),
            bounds: allocation,
            clip,
            layer: UiMountedLayerProjection::Layer(UiMountedLayerReference::new(0)),
            radii,
            paint: UiMountedSurfacePaint::Fill(color),
            opacity: UiMountedAppearanceOpacity::ONE,
            projection,
        },
    )
    .unwrap();
    let _portal_surface = UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        portal,
        surface_mechanic,
    )
    .unwrap();
    let _outline = UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedOutlineAppearanceCompletionInput {
            issuer: node_issuer,
            node_receipt: node_issuer.receipt_for(portal),
            clip,
            geometry: UiAppearanceOutlineGeometry::admit(
                allocation,
                radii,
                UiAppearanceLogicalLength::new(1_000).unwrap(),
                zero,
                UiAppearanceLogicalLength::new(500).unwrap(),
            )
            .unwrap(),
            color,
            opacity: UiMountedAppearanceOpacity::ONE,
            projection,
        },
    )
    .unwrap();
    let _text = UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer: node_issuer,
            node_receipt: node_issuer.receipt_for(portal),
            paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([7; 32]),
            foreground: color,
            opacity: UiMountedAppearanceOpacity::ONE,
            projection,
        },
    )
    .unwrap();
    let backdrop_identity = UiMountedBackdropIdentity::from_runtime_mounting(
        "dialog.backdrop",
        UiMountedBackdropScope::PerPortalInstance(portal),
        1,
    )
    .unwrap();
    let placement = UiOverlayPlacementReceipt::from_runtime_overlay_order(4, 1).unwrap();
    let backdrop_attribution =
        UiMountedBackdropAppearanceAttribution::from_runtime_transport(surface, placement, 2, 1)
            .unwrap();
    let _backdrop = UiMountedBackdropMechanic::complete_from_runtime_mounting(
        UiMountedBackdropCompletionInput {
            identity: backdrop_identity.clone(),
            semantic_surface: surface,
            placement,
            extent: backdrop_extent,
            clip,
            background: color,
            opacity: UiMountedAppearanceOpacity::ONE,
            attribution: backdrop_attribution,
        },
    )
    .unwrap();

    let affordance = UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        pointer,
        surface,
        portal,
        UiPointerAffordanceFamily::Default,
    );
    assert_eq!(affordance.pointer(), pointer);
    assert_eq!(affordance.target(), portal);

    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        surface,
        UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        4,
        7,
        [
            UiOverlayParticipantIdentity::Portal(portal),
            UiOverlayParticipantIdentity::Backdrop(backdrop_identity),
        ],
    )
    .unwrap();
    assert_eq!(order.bottom_to_top().len(), 2);
}
