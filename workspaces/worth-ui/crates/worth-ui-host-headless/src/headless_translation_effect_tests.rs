use worth_ui_host_contract::{
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfaceIdentity,
    UiHostSurfacePresentationMode, UiMountedAccessibilityProjection, UiMountedAllocationProjection,
    UiMountedDiagnosticProjection, UiMountedDiagnosticReference, UiMountedFrameConsumptionInput,
    UiMountedFrameIdentity, UiMountedMechanicalRole, UiMountedMotionProjection,
    UiMountedNodeProjectionView, UiMountedNodeProjectionViewInput, UiMountedNodeReceiptIssuer,
    UiMountedOmissionReason, UiMountedPaintBatchTable, UiMountedPaintProjection,
    UiMountedParticipation, UiMountedParticipationFact, UiMountedParticipationInput,
    UiMountedParticipationStatus, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationWorkView, UiMountedPreviewProjection, UiMountedProjectionView,
    UiMountedProjectionViewInput, UiMountedRealtimeBatchTable, UiMountedResourceTable,
    UiMountedSpatialBatchTable, UiMountedSurfaceBindingRequirement, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration, WorthUiHostCapabilityObservationGeneration,
};

use super::{UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect};

#[test]
fn headless_translation_records_motion_metadata_without_synthesizing_a_host_mechanic() {
    let projection = admitted_effect_projection();
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol is compatible"),
    };
    let capability_generation = WorthUiHostCapabilityObservationGeneration::new(7);
    let requirement = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        capability_generation,
        11,
        UiHostSurfacePresentationMode::RecordOnly,
    );
    let presentation_work = worth_ui_test_support::initial_presentation_mechanics_for_certification(
        &projection,
        requirement,
    );
    let view = worth_ui_host_contract::UiMountedFrameConsumptionView::from_inert_mechanics(
        UiMountedFrameConsumptionInput {
            qualified_text: &(),
            text_raster_work: None,
            appearance_work: None,
            authority: std::rc::Rc::new(()),
            host_session_identity: 13,
            protocol,
            capability_generation,
            capability_profile_digest: 11,
            attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            deadline: worth_ui_host_contract::UiPresentationDeadline::at_tick(20),
            requirement,
            presentation_work: UiMountedPresentationWorkView::Initial(&presentation_work),
        },
    );
    let worth_ui_host_contract::UiMountedPresentationWorkView::Initial(initial) =
        UiMountedPresentationWorkView::Initial(&presentation_work)
    else {
        unreachable!("fixture issues initial work")
    };

    let transcript = super::headless_translation::translate_headless_frame(
        &view,
        &projection,
        UiHeadlessRecorderCapacity::production_default(),
        initial.order(),
        initial.damage(),
    )
    .expect("record-only translation accepts admitted external mechanics");
    assert_eq!(transcript.nodes().len(), 1);
    assert_eq!(
        transcript.nodes()[0].motion(),
        UiMountedMotionProjection::Admitted
    );
    assert_eq!(
        transcript.nodes()[0].diagnostic(),
        UiMountedDiagnosticProjection::Reference(UiMountedDiagnosticReference::new(3))
    );
    assert!(transcript
        .unperformed_effects()
        .contains(&UiHeadlessUnperformedEffect::Diagnostic { node_count: 1 }));
}

fn admitted_effect_projection() -> UiMountedProjectionView {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let mounted_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = UiMountedNodeReceiptIssuer::mint_for(frame)
        .unwrap()
        .receipt_for(mounted_instance);
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
                mounted_instance,
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
                    UiMountedDiagnosticReference::new(3),
                ),
                drawables: Vec::new(),
                semantic_text: Vec::new(),
                portal_presentation: None,
            },
        )],
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        filled_rects: worth_ui_host_contract::UiMountedFilledRectTable::empty(),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: worth_ui_host_contract::UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::from_runtime_mounting(Vec::new())
            .unwrap(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}
