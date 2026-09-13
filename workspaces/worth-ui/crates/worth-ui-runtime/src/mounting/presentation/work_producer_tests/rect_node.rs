use worth_ui_host_contract::{
    UiMountedHitTestProjection, UiMountedMechanicalRole, UiMountedNodeProjectionView,
    UiMountedNodeProjectionViewInput, UiMountedOmissionReason, UiMountedPaintProjection,
    UiMountedParticipation, UiMountedParticipationFact, UiMountedParticipationInput,
    UiMountedParticipationStatus, UiMountedPortalOverlayMechanic, UiMountedPortalOverlayReference,
};

pub(super) fn rect_node(
    index: usize,
    row: &UiMountedPortalOverlayMechanic,
) -> UiMountedNodeProjectionView {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
    let reference = UiMountedPortalOverlayReference::from_runtime_mounting(
        u16::try_from(index).expect("fixture row index"),
    );
    UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
        mounted_instance: row.owner(),
        node_receipt: row.owner_receipt(),
        authored_position: u64::try_from(index).expect("fixture authored position"),
        role: UiMountedMechanicalRole::Control,
        participation: UiMountedParticipation::new(UiMountedParticipationInput {
            paint: admitted,
            clip: admitted,
            input: withheld,
            focus: withheld,
            hit_test: withheld,
            accessibility: withheld,
            motion: withheld,
            diagnostic: withheld,
        }),
        allocation: worth_ui_host_contract::UiMountedAllocationProjection::Known {
            bounds: row.bounds(),
            basis: worth_ui_host_contract::UiMountedAllocationBasis::new(
                1,
                2,
                3,
                worth_ui_host_contract::UiMountedTransformProjection::Identity,
            ),
        },
        preview: worth_ui_host_contract::UiMountedPreviewProjection::Omitted(omitted),
        paint: UiMountedPaintProjection::Omitted(omitted),
        hit_test: UiMountedHitTestProjection::Omitted(omitted),
        accessibility: worth_ui_host_contract::UiMountedAccessibilityProjection::Omitted(omitted),
        motion: worth_ui_host_contract::UiMountedMotionProjection::Omitted(omitted),
        diagnostic: worth_ui_host_contract::UiMountedDiagnosticProjection::Omitted(omitted),
        drawables: vec![
            worth_ui_host_contract::UiMountedDrawableReference::PortalOverlay(reference),
        ],
        semantic_text: Vec::new(),
        portal_presentation: None,
    })
}
