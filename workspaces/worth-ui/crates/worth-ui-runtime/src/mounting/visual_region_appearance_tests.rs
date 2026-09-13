use super::*;
use worth_ui_host_contract::*;

#[test]
fn rounded_and_border_coverage_cannot_claim_uniform_rectangular_opacity() {
    let opaque = UiMountedAppearanceColor::from_straight_srgba([30, 60, 90, 255]);
    let clear = UiMountedAppearanceColor::from_straight_srgba([0; 4]);
    // A 32-point square with radius 8 leaves (0,0) transparent. A one-point
    // inward border paints the edge but leaves (16,16) transparent. Neither
    // shape permits a single opaque rectangle to occlude the lower surface.
    for (paint, radius) in [
        (UiMountedSurfacePaint::Fill(opaque), 8),
        (
            UiMountedSurfacePaint::Border {
                color: opaque,
                inward_width: length(1),
            },
            0,
        ),
        (
            UiMountedSurfacePaint::FillAndBorder {
                fill: clear,
                border: opaque,
                inward_width: length(1),
            },
            8,
        ),
    ] {
        let basis =
            paint_basis(paint, radius).expect("visible border and rounded fill retain attribution");
        assert_eq!(basis.bounds().width(), 32.0);
        assert_eq!(basis.bounds().height(), 32.0);
        assert_eq!(
            basis.alpha(),
            None,
            "exact coverage is unsupported, not opaque"
        );
    }
    assert_eq!(
        paint_basis(UiMountedSurfacePaint::Fill(opaque), 0)
            .unwrap()
            .alpha(),
        Some(255)
    );
    assert!(paint_basis(UiMountedSurfacePaint::Fill(clear), 0).is_none());
}

fn length(points: i32) -> UiAppearanceLogicalLength {
    UiAppearanceLogicalLength::new(points * UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as i32)
        .unwrap()
}

fn paint_basis(paint: UiMountedSurfacePaint, radius: i32) -> Option<UiMountedAppearancePaintBasis> {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let extent = 32 * UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT;
    let bounds = UiAppearanceAllocationBounds::new(0, 0, extent, extent).unwrap();
    let surface = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: UiAppearanceClip::new(0, 0, extent, extent).unwrap(),
            surface_paint_order: 2,
            portal_group: None,
            radii: UiAppearanceNormalizedLogicalRadii::normalize(bounds, [length(radius); 4]),
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint,
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap();
    UiMountedRetainedAppearanceVisualMechanic::new(
        UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedAppearanceMechanic::Surface(surface),
        1,
    )
    .into_paint_basis(UiSurfaceBindingGeneration::mint_unbound().unwrap())
}
