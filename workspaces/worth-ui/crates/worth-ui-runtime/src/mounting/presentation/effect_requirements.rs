use worth_ui_host_contract::{
    UiHostSurfacePresentationMode, UiMountedEffectFamily, UiMountedProjectionView,
};

pub(super) fn required_effects(
    mode: UiHostSurfacePresentationMode,
    projection: &UiMountedProjectionView,
) -> Vec<UiMountedEffectFamily> {
    if mode == UiHostSurfacePresentationMode::RecordOnly {
        return vec![UiMountedEffectFamily::RecordedProjection];
    }

    projection.authored_native_effects().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiMountedAccessibilityProjection, UiMountedAllocationProjection,
        UiMountedDiagnosticProjection, UiMountedDiagnosticReference, UiMountedFrameIdentity,
        UiMountedMechanicalRole, UiMountedMotionProjection, UiMountedNodeProjectionView,
        UiMountedNodeProjectionViewInput, UiMountedNodeReceiptIssuer, UiMountedOmissionReason,
        UiMountedPaintBatchTable, UiMountedPaintProjection, UiMountedParticipation,
        UiMountedParticipationFact, UiMountedParticipationInput, UiMountedParticipationStatus,
        UiMountedPreviewProjection, UiMountedProjectionViewInput, UiMountedRealtimeBatchTable,
        UiMountedResourceTable, UiMountedSpatialBatchTable, UiSemanticSurfaceIdentity,
        UiSurfaceBindingGeneration,
    };

    #[test]
    fn motion_participation_remains_projection_metadata_not_a_host_effect() {
        let projection = admitted_effect_projection();
        assert_eq!(
            required_effects(UiHostSurfacePresentationMode::NativeDisplay, &projection),
            vec![UiMountedEffectFamily::Diagnostic]
        );
        assert_eq!(
            required_effects(UiHostSurfacePresentationMode::RecordOnly, &projection),
            vec![UiMountedEffectFamily::RecordedProjection]
        );
    }

    fn admitted_effect_projection() -> UiMountedProjectionView {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let receipt = UiMountedNodeReceiptIssuer::mint_for(frame)
            .unwrap()
            .receipt_for(instance);
        let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
        let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
        let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
        UiMountedProjectionView::new(UiMountedProjectionViewInput {
            frame,
            surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
                .unwrap(),
            nodes: vec![UiMountedNodeProjectionView::new(
                UiMountedNodeProjectionViewInput {
                    mounted_instance: instance,
                    node_receipt: receipt,
                    authored_position: 0,
                    role: UiMountedMechanicalRole::Diagnostic,
                    participation: UiMountedParticipation::new(UiMountedParticipationInput {
                        paint: withheld,
                        clip: withheld,
                        input: withheld,
                        focus: withheld,
                        hit_test: withheld,
                        accessibility: withheld,
                        motion: admitted,
                        diagnostic: admitted,
                    }),
                    allocation: UiMountedAllocationProjection::Omitted(omitted),
                    preview: UiMountedPreviewProjection::Omitted(omitted),
                    paint: UiMountedPaintProjection::Omitted(omitted),
                    hit_test: worth_ui_host_contract::UiMountedHitTestProjection::Omitted(omitted),
                    accessibility: UiMountedAccessibilityProjection::Omitted(omitted),
                    motion: UiMountedMotionProjection::Admitted,
                    diagnostic: UiMountedDiagnosticProjection::Reference(
                        UiMountedDiagnosticReference::new(8),
                    ),
                    drawables: Vec::new(),
                    semantic_text: Vec::new(),
                    portal_presentation: None,
                },
            )],
            clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
            layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
            portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
            semantic_text: worth_ui_host_contract::UiMountedSemanticTextTable::empty(),
            hit_tests: worth_ui_host_contract::UiMountedHitTestTable::from_runtime_mounting(
                Vec::new(),
            )
            .unwrap(),
            paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
            spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
            realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
            resources: UiMountedResourceTable::new(Vec::new()),
            authored_paint_commands: Vec::new(),
            authored_paint_order: Vec::new(),
        })
    }
}
