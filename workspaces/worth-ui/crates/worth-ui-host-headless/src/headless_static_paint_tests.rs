use worth_ui_host_contract::{
    UiHostProtocolAgreement, UiHostProtocolContract, UiHostProtocolDenial, UiHostProtocolIdentity,
    UiHostProtocolNegotiation, UiHostProtocolSchemaFamily, UiHostSurfaceIdentity,
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode,
    UiMountedAccessibilityProjection, UiMountedAllocationBasis, UiMountedAllocationProjection,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedDiagnosticProjection, UiMountedFilledRectCompletionInput, UiMountedFilledRectMechanic,
    UiMountedFilledRectReference, UiMountedFilledRectTable, UiMountedFrameConsumptionInput,
    UiMountedFrameIdentity, UiMountedFrameSchemaVersion, UiMountedInstanceIdentity,
    UiMountedMechanicalRole, UiMountedMotionProjection, UiMountedNodeProjectionView,
    UiMountedNodeProjectionViewInput, UiMountedNodeReceiptIssuer, UiMountedOmissionReason,
    UiMountedPaintBatchReference, UiMountedPaintBatchRow, UiMountedPaintBatchTable,
    UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity, UiMountedPaintPrimitiveKind,
    UiMountedPaintProjection, UiMountedParticipation, UiMountedParticipationFact,
    UiMountedParticipationInput, UiMountedParticipationStatus,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationWorkView,
    UiMountedPreviewProjection, UiMountedProjectionView, UiMountedProjectionViewInput,
    UiMountedRealtimeBatchTable, UiMountedResourceTable, UiMountedRgba8,
    UiMountedSpatialBatchTable, UiMountedSurfaceBindingRequirement, UiMountedTransformProjection,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

use super::{UiHeadlessRecorderCapacity, UiHeadlessUnperformedEffect};

#[test]
fn validated_agreement_static_paint_consumes_and_mixed_contract_stops_before_consumer() {
    let projection = complete_projection(ProjectionMutation::None);

    let transcript = translate(&projection, current_protocol()).expect("complete row is valid");

    assert_eq!(transcript.filled_rects().len(), 1);
    let row = transcript.filled_rects()[0];
    assert_eq!(row.frame(), projection.frame());
    assert_eq!(row.surface(), projection.surface());
    assert_eq!(row.binding(), projection.binding());
    assert_eq!(row.bounds(), canonical_bounds());
    assert_eq!(row.color().channels(), [47, 129, 247, 255]);
    assert!(transcript
        .unperformed_effects()
        .contains(&UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 1,
            portal_overlay_count: 0,
            semantic_text_count: 0,
            preview_node_count: 0,
        }));
    assert_eq!(
        mounted_frame_revision_two().negotiate(),
        UiHostProtocolNegotiation::Incompatible(UiHostProtocolDenial::SchemaTooOld(
            UiHostProtocolSchemaFamily::MountedFrame
        ))
    );
}

#[test]
fn foreign_frame_and_allocation_basis_reject_before_a_transcript_exists() {
    for mutation in [
        ProjectionMutation::ForeignFrame,
        ProjectionMutation::ForeignSurface,
        ProjectionMutation::ForeignBinding,
        ProjectionMutation::ForeignAllocationBasis,
        ProjectionMutation::ForeignNodeReceipt,
    ] {
        let projection = complete_projection(mutation);
        assert_eq!(
            translate(&projection, current_protocol()),
            Err(UiHostSurfacePresentationDenial::MalformedProjection)
        );
    }
}

#[test]
fn count_only_paint_remains_recordable_but_non_drawable() {
    let projection = count_only_projection();

    let transcript = translate(&projection, current_protocol()).expect("summary row is valid");

    assert!(transcript.filled_rects().is_empty());
    assert_eq!(
        transcript.nodes()[0].paint(),
        super::UiHeadlessNodePaintMechanic::CountOnlyBatch(0)
    );
    assert!(transcript
        .unperformed_effects()
        .contains(&UiHeadlessUnperformedEffect::NativePaint {
            filled_rect_count: 0,
            portal_overlay_count: 0,
            semantic_text_count: 0,
            preview_node_count: 0,
        }));
}

fn translate(
    projection: &UiMountedProjectionView,
    protocol: UiHostProtocolAgreement,
) -> Result<super::UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
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
        projection,
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
    super::headless_translation::translate_headless_frame(
        &view,
        projection,
        UiHeadlessRecorderCapacity::production_default(),
        initial.order(),
        initial.damage(),
    )
}

fn current_protocol() -> UiHostProtocolAgreement {
    compatible(UiHostProtocolContract::current())
}

fn mounted_frame_revision_two() -> UiHostProtocolContract {
    let current = UiHostProtocolContract::current();
    UiHostProtocolContract::new(
        UiHostProtocolIdentity::worth_ui(),
        current.protocol(),
        UiMountedFrameSchemaVersion::new(2),
        current.mounted_presentation(),
        current.observation(),
        current.measurement(),
        current.solicited_effect(),
    )
}

fn compatible(contract: UiHostProtocolContract) -> UiHostProtocolAgreement {
    match contract.negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(denial) => {
            panic!("fixture protocol must be compatible: {denial:?}")
        }
    }
}

#[derive(Clone, Copy)]
enum ProjectionMutation {
    None,
    ForeignFrame,
    ForeignSurface,
    ForeignBinding,
    ForeignAllocationBasis,
    ForeignNodeReceipt,
}

fn complete_projection(mutation: ProjectionMutation) -> UiMountedProjectionView {
    let projection_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let projection_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let row_frame = if matches!(mutation, ProjectionMutation::ForeignFrame) {
        UiMountedFrameIdentity::mint_unbound().unwrap()
    } else {
        projection_frame
    };
    let row_surface = if matches!(mutation, ProjectionMutation::ForeignSurface) {
        UiSemanticSurfaceIdentity::mint_unbound().unwrap()
    } else {
        projection_surface
    };
    let row_binding = if matches!(mutation, ProjectionMutation::ForeignBinding) {
        UiSurfaceBindingGeneration::mint_unbound().unwrap()
    } else {
        projection_binding
    };
    let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let node_receipt = UiMountedNodeReceiptIssuer::mint_for(row_frame)
        .unwrap()
        .receipt_for(mounted_instance);
    let row_basis = allocation_basis(2);
    let node_basis = if matches!(mutation, ProjectionMutation::ForeignAllocationBasis) {
        allocation_basis(3)
    } else {
        row_basis
    };
    let projected_node_receipt = if matches!(mutation, ProjectionMutation::ForeignNodeReceipt) {
        UiMountedNodeReceiptIssuer::mint_for(row_frame)
            .unwrap()
            .receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap())
    } else {
        node_receipt
    };
    let bounds = canonical_bounds();
    let row = UiMountedFilledRectMechanic::complete_from_runtime_mounting(
        UiMountedFilledRectCompletionInput {
            frame: row_frame,
            surface: row_surface,
            binding: row_binding,
            mounted_instance,
            node_receipt,
            allocation_basis: row_basis,
            bounds,
            color: UiMountedRgba8::new(47, 129, 247, 255),
            layer_semantic_order: 0,
            clip_bounds: bounds,
        },
    )
    .unwrap();
    let command_identity = UiMountedPaintCommandIdentity::filled_rect(&row);
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame: projection_frame,
        surface: projection_surface,
        binding: projection_binding,
        content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
            .unwrap(),
        nodes: vec![complete_node(
            mounted_instance,
            projected_node_receipt,
            bounds,
            node_basis,
        )],
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        filled_rects: UiMountedFilledRectTable::from_runtime_mounting(vec![row]).unwrap(),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: worth_ui_host_contract::UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::from_runtime_mounting(Vec::new())
            .unwrap(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: vec![worth_ui_host_contract::UiMountedPaintCommand::FilledRect {
            identity: command_identity,
            mechanic: row,
        }],
        authored_paint_order: vec![UiMountedPaintOrderIdentity::for_command(command_identity)],
    })
}

fn count_only_projection() -> UiMountedProjectionView {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = UiMountedNodeReceiptIssuer::mint_for(frame)
        .unwrap()
        .receipt_for(mounted_instance);
    let bounds = canonical_bounds();
    let basis = allocation_basis(2);
    let complete = complete_node(mounted_instance, receipt, bounds, basis);
    let node = UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
        mounted_instance: complete.mounted_instance(),
        node_receipt: complete.node_receipt(),
        authored_position: 0,
        role: complete.role(),
        participation: complete.participation(),
        allocation: complete.allocation(),
        preview: complete.preview(),
        paint: UiMountedPaintProjection::CountOnlyBatch(UiMountedPaintBatchReference::new(0)),
        hit_test: worth_ui_host_contract::UiMountedHitTestProjection::Omitted(
            UiMountedOmissionReason::NotDefinedByCurrentRuntime,
        ),
        accessibility: complete.accessibility(),
        motion: complete.motion(),
        diagnostic: complete.diagnostic(),
        drawables: Vec::new(),
        semantic_text: Vec::new(),
        portal_presentation: None,
    });
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame,
        surface,
        binding,
        content_generation: worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
            .unwrap(),
        nodes: vec![node],
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        filled_rects: UiMountedFilledRectTable::empty(),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: worth_ui_host_contract::UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::from_runtime_mounting(Vec::new())
            .unwrap(),
        paint_batches: UiMountedPaintBatchTable::new(vec![UiMountedPaintBatchRow::new(
            1,
            worth_ui_host_contract::UiMountedLayerProjection::Omitted(
                UiMountedOmissionReason::NotDefinedByCurrentRuntime,
            ),
            None,
            UiMountedPaintPrimitiveKind::OrdinaryLaneSummary,
        )]),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}

fn complete_node(
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    bounds: UiMountedCanonicalBox,
    basis: UiMountedAllocationBasis,
) -> UiMountedNodeProjectionView {
    let admitted = UiMountedParticipationFact::new(UiMountedParticipationStatus::Admitted);
    let withheld = UiMountedParticipationFact::new(UiMountedParticipationStatus::Withheld);
    let omitted = UiMountedOmissionReason::NotDefinedByCurrentRuntime;
    UiMountedNodeProjectionView::new(UiMountedNodeProjectionViewInput {
        mounted_instance,
        node_receipt,
        authored_position: 0,
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
        allocation: UiMountedAllocationProjection::Known { bounds, basis },
        preview: UiMountedPreviewProjection::Omitted(omitted),
        paint: UiMountedPaintProjection::FilledRect(
            UiMountedFilledRectReference::from_runtime_mounting(0),
        ),
        hit_test: worth_ui_host_contract::UiMountedHitTestProjection::Omitted(omitted),
        accessibility: UiMountedAccessibilityProjection::Omitted(omitted),
        motion: UiMountedMotionProjection::Omitted(omitted),
        diagnostic: UiMountedDiagnosticProjection::Omitted(omitted),
        drawables: vec![
            worth_ui_host_contract::UiMountedDrawableReference::FilledRect(
                UiMountedFilledRectReference::from_runtime_mounting(0),
            ),
        ],
        semantic_text: Vec::new(),
        portal_presentation: None,
    })
}

fn canonical_bounds() -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 96.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}

fn allocation_basis(generation: u64) -> UiMountedAllocationBasis {
    UiMountedAllocationBasis::new(1, generation, 3, UiMountedTransformProjection::Identity)
}
