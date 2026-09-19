//! SUPPORT AUTHORITY for host-adapter identity-overlay translation tests.

use worth_ui_host_contract::{
    UiHostClientAreaObservation, UiHostCoordinateOrientation, UiHostCoordinatePosture,
    UiHostCoordinateRounding, UiHostCoordinateTransform, UiHostViewportTransformObservation,
    UiMountedAccessibilityProjection, UiMountedAllocationProjection,
    UiMountedClientCoordinateBasis, UiMountedClientPhysicalRect, UiMountedDiagnosticProjection,
    UiMountedFrameIdentity, UiMountedIdentityOverlayMechanic,
    UiMountedIdentityOverlayMechanicInput, UiMountedInstanceIdentity, UiMountedMechanicalRole,
    UiMountedMotionProjection, UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput,
    UiMountedNodeReceiptIdentity, UiMountedNodeReceiptIssuer, UiMountedOmissionReason,
    UiMountedPaintBatchTable, UiMountedPaintProjection, UiMountedParticipation,
    UiMountedParticipationFact, UiMountedParticipationInput, UiMountedParticipationStatus,
    UiMountedPreviewProjection, UiMountedProjectionView, UiMountedProjectionViewInput,
    UiMountedRealtimeBatchTable, UiMountedResourceTable, UiMountedSpatialBatchTable,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIdentityOverlayProjectionCertificationMutation {
    Exact,
    ForeignSurface,
    OffscreenBounds,
}

struct UiIdentityOverlayProjectionCertificationBasis {
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    instance: UiMountedInstanceIdentity,
    receipt: UiMountedNodeReceiptIdentity,
    mechanic: UiMountedIdentityOverlayMechanic,
}

pub fn identity_overlay_projection_for_certification(
    mutation: UiIdentityOverlayProjectionCertificationMutation,
) -> UiMountedProjectionView {
    let base_frame = UiMountedFrameIdentity::mint_unbound().expect("base frame identity");
    let successor_frame = UiMountedFrameIdentity::mint_unbound().expect("successor frame identity");
    let instance = UiMountedInstanceIdentity::mint_unbound().expect("mounted instance identity");
    let base_receipt = node_receipt(base_frame, instance);
    let successor_receipt = node_receipt(successor_frame, instance);
    let surface = UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity");
    let mechanic_surface = match mutation {
        UiIdentityOverlayProjectionCertificationMutation::ForeignSurface => {
            UiSemanticSurfaceIdentity::mint_unbound().expect("foreign surface identity")
        }
        UiIdentityOverlayProjectionCertificationMutation::Exact
        | UiIdentityOverlayProjectionCertificationMutation::OffscreenBounds => surface,
    };
    let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding generation");
    let mechanic = UiMountedIdentityOverlayMechanic::from_runtime_mounting(
        UiMountedIdentityOverlayMechanicInput {
            overlay_identity: 7,
            base_snapshot: 11,
            base_frame,
            target_receipt: base_receipt,
            successor_frame,
            surface: mechanic_surface,
            binding,
            coordinate_basis: coordinate_basis(),
            target_region: target_region(mutation),
        },
    )
    .expect("certification mechanic is structurally valid");
    projection(UiIdentityOverlayProjectionCertificationBasis {
        frame: successor_frame,
        surface,
        binding,
        instance,
        receipt: successor_receipt,
        mechanic,
    })
}

fn node_receipt(
    frame: UiMountedFrameIdentity,
    instance: UiMountedInstanceIdentity,
) -> UiMountedNodeReceiptIdentity {
    UiMountedNodeReceiptIssuer::mint_for(frame)
        .expect("receipt issuer")
        .receipt_for(instance)
}

fn target_region(
    mutation: UiIdentityOverlayProjectionCertificationMutation,
) -> UiMountedClientPhysicalRect {
    let right = match mutation {
        UiIdentityOverlayProjectionCertificationMutation::OffscreenBounds => 300,
        UiIdentityOverlayProjectionCertificationMutation::Exact
        | UiIdentityOverlayProjectionCertificationMutation::ForeignSurface => 128,
    };
    UiMountedClientPhysicalRect::from_runtime_mounting(32, 20, right, 76)
        .expect("certification target region")
}

fn coordinate_basis() -> UiMountedClientCoordinateBasis {
    UiMountedClientCoordinateBasis::from_runtime_mounting(
        UiHostCoordinateTransform::observed_by_host(
            UiHostClientAreaObservation::observed_by_host([40, 24], [160, 96]),
            UiHostViewportTransformObservation::observed_by_host(
                [160.0, 96.0],
                [1.0, 1.0],
                [0.0, 0.0],
            ),
            UiHostCoordinatePosture::observed_by_host(
                UiHostCoordinateOrientation::TopLeftOrigin,
                UiHostCoordinateRounding::PixelCenterNearest,
            ),
        ),
    )
    .expect("certification coordinate basis")
}

fn projection(basis: UiIdentityOverlayProjectionCertificationBasis) -> UiMountedProjectionView {
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame: basis.frame,
        surface: basis.surface,
        binding: basis.binding,
        content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
            .expect("certification content generation"),
        nodes: vec![UiMountedNodeProjectionView::new(
            UiMountedNodeProjectionViewInput {
                mounted_instance: basis.instance,
                node_receipt: basis.receipt,
                authored_position: 0,
                role: UiMountedMechanicalRole::Diagnostic,
                participation: UiMountedParticipation::new(UiMountedParticipationInput {
                    paint: withheld,
                    clip: withheld,
                    input: withheld,
                    focus: withheld,
                    hit_test: withheld,
                    accessibility: withheld,
                    motion: withheld,
                    diagnostic: withheld,
                }),
                allocation: UiMountedAllocationProjection::Omitted(omitted),
                preview: UiMountedPreviewProjection::Omitted(omitted),
                paint: UiMountedPaintProjection::Omitted(omitted),
                hit_test: worth_ui_host_contract::UiMountedHitTestProjection::Omitted(omitted),
                accessibility: UiMountedAccessibilityProjection::Omitted(omitted),
                motion: UiMountedMotionProjection::Omitted(omitted),
                diagnostic: UiMountedDiagnosticProjection::IdentityOverlay(basis.mechanic),
                drawables: Vec::new(),
                semantic_text: Vec::new(),
                portal_presentation: None,
            },
        )],
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: worth_ui_host_contract::UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::empty(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}
