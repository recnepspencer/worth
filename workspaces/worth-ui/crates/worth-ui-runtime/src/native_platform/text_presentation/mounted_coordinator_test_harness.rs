use std::{rc::Rc, sync::Arc};

use super::super::UiNativeMountedSurfaceTextObservation;
use crate::native_platform::text_presentation::{
    derive_text_presentation_request_bases, UiMountedEventTimeDpiAuthority,
    UiNativeMountedTextCoordinator, UiNativeTextPresentationPreparation,
    UiNativeTextPresentationPrepared,
};
use worth_ui_host_contract::{
    UiGlyphRasterBatchSink, UiGlyphRasterBatchSubmissionDenial, UiGlyphRasterMissSelectionView,
    UiHostPresentationCostReport, UiHostPresentationEpoch, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfaceIdentity, UiHostSurfacePresentationMode,
    UiHostSurfacePresentationOutcome, UiMountedCompletedEffects, UiMountedFrameConsumptionInput,
    UiMountedFrameConsumptionView, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationDelta, UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationWorkView, UiMountedRgba8, UiMountedSemanticTextCompletionInput,
    UiMountedSemanticTextMechanic, UiMountedSurfaceBindingRequirement, UiMountedTextForegroundSpan,
    UiPresentationDeadline, UiTextOriginalRange, WorthUiHostCapabilityObservationGeneration,
};

pub(super) fn requirement(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    )
}

pub(super) fn prepare<'work>(
    coordinator: &UiNativeMountedTextCoordinator,
    work: UiMountedPresentationWorkView<'work>,
    requirement: UiMountedSurfaceBindingRequirement,
    layout: &'work worth_ui_text::UiQualifiedTextLayout,
) -> UiNativeTextPresentationPrepared {
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
    match coordinator
        .prepare_mounted_semantic_text(
            work,
            requirement,
            dpi,
            Some(lineage(work, requirement)),
            |identity| (identity == layout.identity()).then_some(layout),
        )
        .expect("mounted semantic text reaches preparation")
    {
        UiNativeTextPresentationPreparation::Prepared(prepared) => prepared,
        UiNativeTextPresentationPreparation::Denied(_) => panic!("text preparation denied"),
    }
}

pub(super) fn present<'work>(
    coordinator: &mut UiNativeMountedTextCoordinator,
    work: UiMountedPresentationWorkView<'work>,
    requirement: UiMountedSurfaceBindingRequirement,
    prepared: &'work UiNativeTextPresentationPrepared,
    layout: &'work worth_ui_text::UiQualifiedTextLayout,
) -> UiNativeMountedSurfaceTextObservation {
    let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let host_lineage = lineage(work, requirement);
    coordinator
        .present_with_mounted_work(
            requirement.binding(),
            work,
            requirement,
            Some(host_lineage),
            prepared,
            |identity| (identity == layout.identity()).then_some(layout),
            |text_raster_work| {
                let view = consumption_view(work, requirement, attempt, Some(text_raster_work));
                let bases = derive_text_presentation_request_bases(
                    &view,
                    prepared,
                    text_raster_work.pins(),
                    text_raster_work.binding_pins(),
                )
                .expect("mounted text work has a qualified request basis");
                if bases.len() == 1 && !bases[0].pin_additions().is_empty() {
                    admit_all_raster_misses(text_raster_work);
                }
                (presented_outcome(), bases, Vec::new().into_boxed_slice())
            },
        )
        .expect("mounted text transaction is prepared")
}

fn admit_all_raster_misses(work: &worth_ui_host_contract::UiMountedTextRasterWork<'_>) {
    for demand in work.demands().iter().copied() {
        if demand.records().is_empty() {
            continue;
        }
        let misses = UiGlyphRasterMissSelectionView::from_text_mechanics(
            demand.identity(),
            demand.layout_identity(),
            demand.lane(),
            demand.records(),
        );
        work.rasterize(misses, &mut AcceptingRasterSink)
            .expect("host-admitted raster misses are accepted");
    }
}

struct AcceptingRasterSink;

impl UiGlyphRasterBatchSink for AcceptingRasterSink {
    fn submit_alpha(
        &mut self,
        _batch: worth_ui_host_contract::UiAlphaRasterBatchView<'_, '_>,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        Ok(())
    }

    fn submit_color(
        &mut self,
        _batch: worth_ui_host_contract::UiColorRasterBatchView<'_, '_>,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        Ok(())
    }
}

fn presented_outcome() -> UiHostSurfacePresentationOutcome {
    UiHostSurfacePresentationOutcome::Presented(
        worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(1),
            UiMountedCompletedEffects::new(Vec::new()),
            UiHostPresentationCostReport::default(),
        ),
    )
}

fn consumption_view<'work>(
    work: UiMountedPresentationWorkView<'work>,
    requirement: UiMountedSurfaceBindingRequirement,
    attempt: UiMountedPresentationAttemptIdentity,
    text_raster_work: Option<&'work worth_ui_host_contract::UiMountedTextRasterWork<'work>>,
) -> UiMountedFrameConsumptionView<'work> {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current host protocol must negotiate");
    };
    UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
        authority: Rc::new(()),
        host_session_identity: 41,
        protocol,
        capability_generation: requirement.capability_generation(),
        capability_profile_digest: requirement.capability_profile_digest(),
        attempt,
        deadline: UiPresentationDeadline::at_tick(100),
        requirement,
        presentation_work: work,
        appearance_work: None,
        qualified_text: &(),
        text_raster_work,
    })
}

fn lineage(
    work: UiMountedPresentationWorkView<'_>,
    requirement: UiMountedSurfaceBindingRequirement,
) -> worth_ui_host_contract::UiHostPresentationLineageIdentity {
    let attempt = UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    consumption_view(work, requirement, attempt, None)
        .host_presentation_lineage()
        .expect("test consumption has host lineage")
}

pub(super) fn initial_text(
    initial: &UiMountedPresentationInitial,
) -> &UiMountedSemanticTextMechanic {
    match &initial.commands()[0] {
        UiMountedPaintCommand::SemanticText { mechanic, .. } => mechanic,
        _ => panic!("certification initial work contains semantic text"),
    }
}

pub(super) fn color_mechanic(delta: &UiMountedPresentationDelta) -> &UiMountedSemanticTextMechanic {
    match &delta.changes()[0] {
        UiMountedPaintCommandChange::Replace { successor, .. } => match successor {
            UiMountedPaintCommand::SemanticText { mechanic, .. } => mechanic,
            _ => panic!("color twin contains semantic text"),
        },
        _ => panic!("color twin is a replacement"),
    }
}

pub(super) fn text_command(
    predecessor: &UiMountedSemanticTextMechanic,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    text: Arc<str>,
    layout: &worth_ui_text::UiQualifiedTextLayout,
    color: [u8; 4],
    reuse_layout: bool,
) -> UiMountedPaintCommand {
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let length = u32::try_from(text.len()).unwrap();
    let foregrounds = Arc::from([UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::new(0, length).unwrap(),
        UiMountedRgba8::new(color[0], color[1], color[2], color[3]),
        predecessor.foregrounds()[0].identity(),
    )]);
    let input = UiMountedSemanticTextCompletionInput {
        content_generation: predecessor.content_generation(),
        frame,
        surface: predecessor.surface(),
        binding: predecessor.binding(),
        mounted_instance: predecessor.mounted_instance(),
        node_receipt: issuer.receipt_for(predecessor.mounted_instance()),
        allocation_basis: predecessor.allocation_basis(),
        bounds: predecessor.bounds(),
        clip_bounds: predecessor.clip_bounds(),
        origin_x: predecessor.origin_x(),
        origin_y: predecessor.origin_y(),
        text,
        layout: layout.view(),
        slot: predecessor.slot(),
        collection_row: predecessor.collection_row().cloned(),
        foregrounds,
        profile: predecessor.profile(),
        layer_semantic_order: predecessor.layer_semantic_order(),
        capability_generation: predecessor.capability_generation(),
        capability_profile_digest: predecessor.capability_profile_digest(),
    };
    let mechanic = if reuse_layout {
        UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(input)
    } else {
        UiMountedSemanticTextMechanic::complete_from_runtime_mounting(input)
    }
    .unwrap();
    let identity = UiMountedPaintCommandIdentity::semantic_text(&mechanic);
    UiMountedPaintCommand::SemanticText { identity, mechanic }
}

pub(super) fn replacement_delta(
    affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
    successor: UiMountedPaintCommand,
) -> UiMountedPresentationDelta {
    let predecessor = successor.identity();
    let (successor_frame, bounds) = match &successor {
        UiMountedPaintCommand::SemanticText { mechanic, .. } => {
            (mechanic.frame(), mechanic.bounds())
        }
        _ => unreachable!(),
    };
    UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: affinity.successor(),
        successor: successor_frame,
        surface: affinity.surface(),
        binding: affinity.binding(),
        content: affinity.content(),
        baseline: affinity.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            predecessor,
            successor,
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity::for_order(&[]),
        damage: vec![worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(bounds)],
        auxiliary: None,
        production_cost: worth_ui_host_contract::UiMountedPresentationProductionCost::default(),
    })
}

pub(super) fn glyph_geometry(
    prepared: &UiNativeTextPresentationPrepared,
) -> Vec<(i64, i64, u32, u32, worth_ui_host_contract::UiGlyphRasterKey)> {
    prepared
        .glyph_runs()
        .iter()
        .map(|run| {
            (
                run.origin_x_millipoints(),
                run.origin_y_millipoints(),
                run.line_index(),
                run.visual_run_index(),
                run.raster_key(),
            )
        })
        .collect()
}
