use crate::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
    UiMountedAllocationBasis, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedFilledRectCompletionInput, UiMountedFilledRectMechanic,
    UiMountedFrameIdentity, UiMountedHitTestCompletionInput, UiMountedHitTestMechanic,
    UiMountedHitTestOrder, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer,
    UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionInput,
    UiMountedPortalOverlayLifecyclePosture, UiMountedPortalOverlayMechanic, UiMountedRgba8,
    UiMountedTransformProjection, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[test]
fn portal_children_translate_clip_raise_and_redigest_from_the_portal_surface() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child_receipt = UiMountedNodeReceiptIssuer::mint_for(frame)
        .unwrap()
        .receipt_for(child);
    let portal = portal(frame, surface, binding);
    let occurrence = canonical_box(
        100.0,
        178.0,
        80.0,
        32.0,
        UiMountedCoordinateSpace::HostSurface,
    );

    let paint = UiMountedFilledRectMechanic::complete_from_runtime_mounting(
        UiMountedFilledRectCompletionInput {
            frame,
            surface,
            binding,
            mounted_instance: child,
            node_receipt: child_receipt,
            allocation_basis: allocation_basis(),
            bounds: occurrence,
            color: UiMountedRgba8::new(26, 31, 44, 255),
            layer_semantic_order: 7,
            clip_bounds: occurrence,
        },
    )
    .unwrap();
    let hit =
        UiMountedHitTestMechanic::complete_from_runtime_mounting(UiMountedHitTestCompletionInput {
            frame,
            surface,
            binding,
            mounted_instance: child,
            node_receipt: child_receipt,
            bounds: occurrence,
            clip_bounds: occurrence,
            order: UiMountedHitTestOrder::from_runtime_plan(9),
        })
        .unwrap();

    let presented_paint = paint.presented_within_portal(portal).unwrap().unwrap();
    let presented_hit = hit.presented_within_portal(portal).unwrap().unwrap();
    let expected = canonical_box(112.0, 218.0, 80.0, 32.0, UiMountedCoordinateSpace::Viewport);

    assert_eq!(presented_paint.bounds(), expected);
    assert_eq!(presented_hit.bounds(), expected);
    assert_eq!(presented_paint.clip_bounds(), expected);
    assert_eq!(presented_hit.clip_bounds(), expected);
    assert_eq!(presented_paint.layer_semantic_order(), 2_008);
    assert_eq!(presented_hit.order().rank(), 9);
    assert!(presented_hit.order() < hit.order());
    assert_ne!(presented_paint.semantic_digest(), paint.semantic_digest());
    assert_ne!(presented_hit.semantic_digest(), hit.semantic_digest());
}

fn portal(
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiMountedPortalOverlayMechanic {
    let bounds = canonical_box(
        100.0,
        200.0,
        280.0,
        320.0,
        UiMountedCoordinateSpace::Viewport,
    );
    portal_region(frame, surface, binding, bounds, bounds)
}

fn portal_region(
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    bounds: UiMountedCanonicalBox,
    clip_bounds: UiMountedCanonicalBox,
) -> UiMountedPortalOverlayMechanic {
    UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(portal_input(
        frame,
        surface,
        binding,
        bounds,
        clip_bounds,
    ))
    .unwrap()
}

fn portal_input(
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    bounds: UiMountedCanonicalBox,
    clip_bounds: UiMountedCanonicalBox,
) -> UiMountedPortalOverlayCompletionInput {
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    UiMountedPortalOverlayCompletionInput {
        frame,
        surface,
        binding,
        owner,
        owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
            .unwrap()
            .receipt_for(owner),
        portal_identity: 41,
        anchor_presentation: UiHostObservationPresentationBasis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            frame,
            binding,
            UiHostPresentationEpoch::issued_by_host(1),
        ),
        anchor_bounds: canonical_box(88.0, 160.0, 96.0, 40.0, UiMountedCoordinateSpace::Viewport),
        bounds,
        clip_bounds,
        color: UiMountedRgba8::new(246, 247, 249, 255),
        layer_semantic_order: 2_000,
        layer_depth: 1,
        lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
        shielding: UiMountedPortalInputShielding::ContentBounds,
    }
}

#[test]
fn retained_portal_equivalence_excludes_lineage_but_rejects_physical_changes() {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let bounds = canonical_box(
        100.0,
        200.0,
        280.0,
        320.0,
        UiMountedCoordinateSpace::Viewport,
    );
    let baseline = portal_input(
        frame,
        UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        bounds,
        bounds,
    );
    let row = UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(baseline).unwrap();
    let mut advanced = baseline;
    advanced.frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    advanced.owner_receipt = UiMountedNodeReceiptIssuer::mint_for(advanced.frame)
        .unwrap()
        .receipt_for(advanced.owner);
    advanced.anchor_presentation = UiHostObservationPresentationBasis::new(
        baseline.anchor_presentation.host_surface(),
        advanced.frame,
        baseline.binding,
        UiHostPresentationEpoch::issued_by_host(2),
    );
    assert!(row.same_retained_paint_meaning(
        UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(advanced).unwrap()
    ));

    let mut variants = [baseline; 6];
    variants[0].color = UiMountedRgba8::new(245, 247, 249, 255);
    variants[1].bounds = canonical_box(
        101.0,
        200.0,
        279.0,
        320.0,
        UiMountedCoordinateSpace::Viewport,
    );
    variants[2].anchor_bounds =
        canonical_box(89.0, 160.0, 95.0, 40.0, UiMountedCoordinateSpace::Viewport);
    variants[3].layer_semantic_order += 1;
    variants[4].lifecycle = UiMountedPortalOverlayLifecyclePosture::Closing;
    variants[5].shielding = UiMountedPortalInputShielding::ModalSurface;
    for variant in variants {
        assert!(!row.same_retained_paint_meaning(
            UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(variant).unwrap()
        ));
    }
}

mod clipping;

fn allocation_basis() -> UiMountedAllocationBasis {
    UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity)
}

fn canonical_box(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}
