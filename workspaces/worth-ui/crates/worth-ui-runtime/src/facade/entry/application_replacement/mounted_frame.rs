use super::WorthUiPreparedApplicationActivation;

impl WorthUiPreparedApplicationActivation {
    fn font_collection(&self) -> &std::sync::Arc<worth_ui_text::UiGlobalFontCollection> {
        &self.font_collection
    }
}

pub(super) struct UiMountedReplacementReuseBasis {
    pub(super) generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(super) host_session: u64,
    pub(super) protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    pub(super) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(super) capability_profile_digest: u64,
}

pub(super) fn prepare_candidate_mounted_frame(
    application: &WorthUiPreparedApplicationActivation,
    state: &crate::mounting::UiMountedGraphReplacementSuccessor,
    graph: crate::graph::UiGraphAuthority<'_>,
    reuse_basis: UiMountedReplacementReuseBasis,
    semantic_content: crate::mounting::UiMountedSemanticContentInput,
    request: crate::mounting::UiMountedFrameRequest,
) -> Result<crate::mounting::UiPreparedMountedFrame, crate::mounting::UiMountedFramePreparationDenial>
{
    let range = request.virtualized_range();
    let visual_overlay = request.visual_overlay();
    let visual_overlay_revision = request.visual_overlay_revision();
    let portal_overlays = request.portal_overlays();
    let plan = application.candidate_plan();
    let lanes = candidate_lanes(plan, range.is_some());
    let allocation_source = crate::runtime::UiMountedAllocationProjectionSource::for_replacement(
        application.candidate_allocation_catalog(),
    );
    let reuse_contract =
        state.seal_frame_reuse_contract(crate::mounting::UiMountedFrameReuseExternalBasis {
            generation: reuse_basis.generation.clone(),
            host_session: reuse_basis.host_session,
            execution: crate::mounting::UiMountedFrameExecutionPosture::ReplacementCandidate,
            plan_digest: application.candidate_plan_digest(),
            allocation_truth_revision: application.candidate_allocation_truth_revision(),
            request: request.reuse_identity(),
            lanes,
            protocol: reuse_basis.protocol,
            capability_generation: reuse_basis.capability_generation,
            capability_profile_digest: reuse_basis.capability_profile_digest,
            visual_overlay_revision,
        });
    let mut assembler =
        state.begin_frame_assembly(crate::mounting::UiMountedFrameAssemblyInput {
            graph,
            generation: reuse_basis.generation,
            trace_source: application.visual_trace_source(),
            plan_digest: application.candidate_plan_digest(),
            plan: crate::mounting::UiMountedPlanProjectionSource::Executed(plan),
            allocation_truth_revision: application.candidate_allocation_truth_revision(),
            allocation_source,
            request,
            lanes,
            preview: None,
            visual_overlay,
            portal_overlays,
            semantic_content,
            theme_values:
                crate::mounting::UiMountedThemeValueSource::replacement_candidate_frozen_plan(),
            appearance_invalidation: None,
            font_collection: std::sync::Arc::clone(application.font_collection()),
            reuse_contract,
        })?;
    execute_candidate_lanes(application, &mut assembler, lanes, range)?;
    assembler.finish()
}

fn candidate_lanes(
    plan: &crate::runtime::WorthUiActiveExecutionPlan,
    virtualized_range_present: bool,
) -> crate::mounting::UiMountedLaneAssembly {
    crate::mounting::UiMountedLaneAssembly {
        ordinary: matches!(
            plan.ordinary_availability(),
            crate::runtime::WorthUiOrdinaryPlanAvailability::Executable
        ),
        virtualized: virtualized_range_present
            && matches!(
                plan.virtualized_availability(),
                crate::runtime::WorthUiVirtualizedPlanAvailability::Executable
            ),
        canvas: matches!(
            plan.canvas_spatial_availability(),
            crate::runtime::WorthUiCanvasSpatialPlanAvailability::Executable
        ),
        realtime: matches!(
            plan.realtime_availability(),
            crate::runtime::WorthUiRealtimePlanAvailability::Executable
        ),
        preview: false,
    }
}

fn execute_candidate_lanes(
    application: &WorthUiPreparedApplicationActivation,
    assembler: &mut crate::mounting::UiMountedFrameAssembler<'_>,
    lanes: crate::mounting::UiMountedLaneAssembly,
    range: Option<crate::runtime::WorthUiVisibleRange>,
) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
    let plan = application.candidate_plan();
    if lanes.ordinary {
        let receipt = plan
            .execute_ordinary(crate::runtime::WorthUiOrdinaryFrameTarget::root_shell())
            .map_err(candidate_lane_denial_ordinary)?;
        assembler
            .record_ordinary(&receipt)
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)?;
    }
    if lanes.virtualized {
        execute_virtualized(
            application,
            assembler,
            range.expect("admitted lane has a range"),
        )?;
    }
    if lanes.canvas {
        execute_canvas(plan, assembler)?;
    }
    if lanes.realtime {
        let handle = plan.first_realtime_handle().ok_or(
            crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
                worth_ui_host_contract::UiMountedLaneParticipation::Realtime,
            ),
        )?;
        let receipt = plan
            .execute_realtime(crate::runtime::WorthUiRealtimeFrameTarget::renderer_surface(handle))
            .map_err(candidate_lane_denial_realtime)?;
        assembler
            .record_realtime(&receipt)
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)?;
    }
    Ok(())
}

fn execute_virtualized(
    application: &WorthUiPreparedApplicationActivation,
    assembler: &mut crate::mounting::UiMountedFrameAssembler<'_>,
    range: crate::runtime::WorthUiVisibleRange,
) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
    let plan = application.candidate_plan();
    let target = plan
        .virtualized_summary(
            application.candidate_query_binding(),
            crate::runtime::WorthUiVirtualizedPlanSummaryRequest::first_view(),
        )
        .map_err(|_| {
            crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
                worth_ui_host_contract::UiMountedLaneParticipation::Virtualized,
            )
        })?
        .target(range);
    let receipt = plan
        .execute_virtualized(application.candidate_query_binding(), target)
        .map_err(candidate_lane_denial_virtualized)?;
    assembler
        .record_virtualized(&receipt)
        .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)
}

fn execute_canvas(
    plan: &crate::runtime::WorthUiActiveExecutionPlan,
    assembler: &mut crate::mounting::UiMountedFrameAssembler<'_>,
) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
    let handle = plan.first_canvas_spatial_handle().ok_or(
        crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
            worth_ui_host_contract::UiMountedLaneParticipation::CanvasSpatial,
        ),
    )?;
    let resource = plan
        .canvas_spatial_summary(handle)
        .expect("candidate lane handle resolves in its own plan")
        .plan_basis_digest();
    let receipt = plan
        .execute_canvas_spatial(crate::runtime::WorthUiCanvasSpatialFrameTarget::draw(
            handle,
        ))
        .map_err(candidate_lane_denial_canvas)?;
    assembler
        .record_canvas(&receipt, resource)
        .map_err(crate::mounting::UiMountedFramePreparationDenial::Projection)
}

fn candidate_lane_denial_ordinary(
    denial: crate::runtime::WorthUiOrdinaryLaneFrameDenial,
) -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Lane(
        crate::facade::WorthUiMountedLaneProjectionDenial::Ordinary(denial),
    )
}

fn candidate_lane_denial_virtualized(
    denial: crate::runtime::WorthUiVirtualizedDataFrameDenial,
) -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Lane(
        crate::facade::WorthUiMountedLaneProjectionDenial::Virtualized(denial),
    )
}

fn candidate_lane_denial_canvas(
    denial: crate::runtime::WorthUiCanvasSpatialFrameDenial,
) -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Lane(
        crate::facade::WorthUiMountedLaneProjectionDenial::Canvas(denial),
    )
}

fn candidate_lane_denial_realtime(
    denial: crate::runtime::WorthUiRealtimeFrameDenial,
) -> crate::mounting::UiMountedFramePreparationDenial {
    crate::mounting::UiMountedFramePreparationDenial::Lane(
        crate::facade::WorthUiMountedLaneProjectionDenial::Realtime(denial),
    )
}
