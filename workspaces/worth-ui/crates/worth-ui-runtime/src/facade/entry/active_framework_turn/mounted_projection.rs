use super::WorthUiActiveFrameworkTurnExecution;

mod lane_participation;
mod reuse;

pub(crate) struct WorthUiActiveMountedProjectionFrame<'frame, 'session> {
    execution: &'frame crate::runtime::WorthUiFrameworkTurnExecution<'session>,
    assembler: crate::mounting::UiMountedFrameAssembler<'frame>,
}

#[derive(Debug, PartialEq)]
pub enum WorthUiMountedLaneProjectionDenial {
    Ordinary(crate::runtime::WorthUiOrdinaryLaneFrameDenial),
    Virtualized(crate::runtime::WorthUiVirtualizedDataFrameDenial),
    Canvas(crate::runtime::WorthUiCanvasSpatialFrameDenial),
    Realtime(crate::runtime::WorthUiRealtimeFrameDenial),
    Projection(crate::mounting::UiMountedProjectionDenial),
}

impl<'session> WorthUiActiveFrameworkTurnExecution<'session> {
    pub(crate) fn classify_mounted_frame_reuse_internal(
        &self,
        request: &crate::mounting::UiMountedFrameRequest,
    ) -> crate::mounting::UiMountedFrameReuse {
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let allocation_truth_revision = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .truth_revision()
            .revision();
        self.mounted.classify_frame_reuse(self.reuse_contract(
            request,
            lanes,
            allocation_truth_revision,
        ))
    }

    pub(crate) fn prepare_mounted_frame_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.prepare_mounted_frame_with_content_internal(
            request,
            crate::mounting::UiMountedSemanticContentInput::empty(),
            self.presentation.theme_values_source(),
        )
    }

    pub(crate) fn prepare_mounted_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        theme_values: crate::mounting::UiMountedThemeValueSource,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection =
            self.begin_mounted_projection(request, lanes, semantic_content, theme_values, None)?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let mut frame = projection.finish()?;
        self.finish_appearance_projection(&mut frame)?;
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_superseding_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        theme_values: crate::mounting::UiMountedThemeValueSource,
        predecessor: &crate::mounting::UiPreparedMountedFrame,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection = self.begin_mounted_projection(
            request,
            lanes,
            semantic_content,
            theme_values,
            Some(predecessor),
        )?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let mut frame = projection.finish()?;
        self.finish_appearance_projection(&mut frame)?;
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        Ok(frame)
    }

    pub(crate) fn prepare_mounted_reconciliation_frame_with_content_internal(
        &mut self,
        request: crate::mounting::UiMountedFrameRequest,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        theme_values: crate::mounting::UiMountedThemeValueSource,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let virtualized_range = request.virtualized_range();
        let plan = self.execution.runtime.active.active_plan_ref();
        let lanes = lane_participation::mounted_lanes(plan, request.virtualized_range().is_some());
        let mut projection =
            self.begin_mounted_projection(request, lanes, semantic_content, theme_values, None)?;
        projection.execute_requested_lanes(lanes, virtualized_range)?;
        let mut frame = projection.finish_for_reconciliation(replacements)?;
        self.finish_appearance_projection(&mut frame)?;
        frame.stage_pointer_affordance(self.pointer_affordance_snapshot.as_ref(), self.mounted)?;
        Ok(frame)
    }

    fn begin_mounted_projection<'frame>(
        &'frame self,
        request: crate::mounting::UiMountedFrameRequest,
        lanes: crate::mounting::UiMountedLaneAssembly,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        theme_values: crate::mounting::UiMountedThemeValueSource,
        predecessor: Option<&'frame crate::mounting::UiPreparedMountedFrame>,
    ) -> Result<
        WorthUiActiveMountedProjectionFrame<'frame, 'session>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        let visual_overlay = request.visual_overlay();
        let portal_overlays = request.portal_overlays();
        let plan = self.execution.runtime.active.active_plan_ref();
        let allocation_truth_revision = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .truth_revision()
            .revision();
        let allocation_source = self
            .execution
            .runtime
            .allocation_receipt_ledger
            .mounted_projection_source(self.mounted.current_allocation_truth_revision());
        let reuse_contract = self.reuse_contract(&request, lanes, allocation_truth_revision);
        let appearance_invalidation =
            self.appearance_invalidation_for_content(&semantic_content, predecessor);
        let input = crate::mounting::UiMountedFrameAssemblyInput {
            graph: self.graph,
            generation: self.generation_identity.clone(),
            trace_source: self.visual_trace_source.clone(),
            plan_digest: plan.digest().as_u64(),
            plan: crate::mounting::UiMountedPlanProjectionSource::Executed(plan),
            allocation_source,
            allocation_truth_revision,
            request,
            lanes,
            preview: None,
            visual_overlay,
            portal_overlays,
            semantic_content,
            theme_values,
            appearance_invalidation: Some(
                crate::runtime::appearance::UiAppearanceInvalidationInput {
                    index: self.consumed_facts,
                    pending: appearance_invalidation,
                },
            ),
            font_collection: std::sync::Arc::clone(&self.font_collection),
            reuse_contract,
        };
        let assembler = match predecessor {
            Some(predecessor) => self
                .mounted
                .begin_superseding_frame_assembly(predecessor, input)?,
            None => self.mounted.begin_frame_assembly(input)?,
        };
        Ok(WorthUiActiveMountedProjectionFrame {
            execution: &self.execution,
            assembler,
        })
    }
}

impl WorthUiActiveMountedProjectionFrame<'_, '_> {
    fn execute_requested_lanes(
        &mut self,
        lanes: crate::mounting::UiMountedLaneAssembly,
        virtualized_range: Option<crate::runtime::WorthUiVisibleRange>,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        if lanes.ordinary {
            self.execute_ordinary(crate::runtime::WorthUiOrdinaryFrameTarget::root_shell())
                .map_err(crate::mounting::UiMountedFramePreparationDenial::Lane)?;
        }
        if let Some(range) = virtualized_range.filter(|_| lanes.virtualized) {
            self.execute_requested_virtualized(range)?;
        }
        if lanes.canvas {
            self.execute_requested_canvas()?;
        }
        if lanes.realtime {
            self.execute_requested_realtime()?;
        }
        Ok(())
    }

    fn execute_requested_virtualized(
        &mut self,
        range: crate::runtime::WorthUiVisibleRange,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let target = self
            .execution
            .runtime
            .active
            .active_plan_ref()
            .virtualized_summary(
                &self.execution.runtime.query_binding,
                crate::runtime::WorthUiVirtualizedPlanSummaryRequest::first_view(),
            )
            .map_err(|_| {
                crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
                    worth_ui_host_contract::UiMountedLaneParticipation::Virtualized,
                )
            })?
            .target(range);
        self.execute_virtualized(target)
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Lane)?;
        Ok(())
    }

    fn execute_requested_canvas(
        &mut self,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let handle = self
            .execution
            .runtime
            .active
            .active_plan_ref()
            .first_canvas_spatial_handle()
            .ok_or(
                crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
                    worth_ui_host_contract::UiMountedLaneParticipation::CanvasSpatial,
                ),
            )?;
        self.execute_canvas(crate::runtime::WorthUiCanvasSpatialFrameTarget::draw(
            handle,
        ))
        .map_err(crate::mounting::UiMountedFramePreparationDenial::Lane)?;
        Ok(())
    }

    fn execute_requested_realtime(
        &mut self,
    ) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
        let handle = self
            .execution
            .runtime
            .active
            .active_plan_ref()
            .first_realtime_handle()
            .ok_or(
                crate::mounting::UiMountedFramePreparationDenial::LaneWorkUnavailable(
                    worth_ui_host_contract::UiMountedLaneParticipation::Realtime,
                ),
            )?;
        self.execute_realtime(crate::runtime::WorthUiRealtimeFrameTarget::renderer_surface(handle))
            .map_err(crate::mounting::UiMountedFramePreparationDenial::Lane)?;
        Ok(())
    }
}

impl WorthUiActiveMountedProjectionFrame<'_, '_> {
    pub(crate) fn execute_ordinary(
        &mut self,
        target: crate::runtime::WorthUiOrdinaryFrameTarget,
    ) -> Result<crate::runtime::WorthUiOrdinaryLaneFrameReceipt, WorthUiMountedLaneProjectionDenial>
    {
        let receipt = self
            .execution
            .execute_active_ordinary_frame(target)
            .map_err(WorthUiMountedLaneProjectionDenial::Ordinary)?;
        self.assembler
            .record_ordinary(&receipt)
            .map_err(WorthUiMountedLaneProjectionDenial::Projection)?;
        Ok(receipt)
    }

    pub(crate) fn execute_virtualized(
        &mut self,
        target: crate::runtime::WorthUiVirtualizedDataFrameTarget,
    ) -> Result<
        crate::runtime::WorthUiVirtualizedDataFrameReceipt,
        WorthUiMountedLaneProjectionDenial,
    > {
        let receipt = self
            .execution
            .execute_active_virtualized_data_frame(target)
            .map_err(WorthUiMountedLaneProjectionDenial::Virtualized)?;
        self.assembler
            .record_virtualized(&receipt)
            .map_err(WorthUiMountedLaneProjectionDenial::Projection)?;
        Ok(receipt)
    }

    pub(crate) fn execute_canvas(
        &mut self,
        target: crate::runtime::WorthUiCanvasSpatialFrameTarget,
    ) -> Result<crate::runtime::WorthUiCanvasSpatialFrameReceipt, WorthUiMountedLaneProjectionDenial>
    {
        let receipt = self
            .execution
            .execute_active_canvas_spatial_frame(target)
            .map_err(WorthUiMountedLaneProjectionDenial::Canvas)?;
        let runtime_handle = receipt
            .touched_runtime_handles()
            .first()
            .copied()
            .expect("sealed canvas receipts name one runtime handle");
        let lane_handle = crate::runtime::WorthUiLaneHandle::from_locator(runtime_handle.locator());
        let resource_content_identity = self
            .execution
            .runtime
            .active
            .active_plan_ref()
            .canvas_spatial_summary(lane_handle)
            .expect("sealed canvas receipt resolves in its active plan")
            .plan_basis_digest();
        self.assembler
            .record_canvas(&receipt, resource_content_identity)
            .map_err(WorthUiMountedLaneProjectionDenial::Projection)?;
        Ok(receipt)
    }

    pub(crate) fn execute_realtime(
        &mut self,
        target: crate::runtime::WorthUiRealtimeFrameTarget,
    ) -> Result<crate::runtime::WorthUiRealtimeFrameReceipt, WorthUiMountedLaneProjectionDenial>
    {
        let receipt = self
            .execution
            .execute_active_realtime_frame(target)
            .map_err(WorthUiMountedLaneProjectionDenial::Realtime)?;
        self.assembler
            .record_realtime(&receipt)
            .map_err(WorthUiMountedLaneProjectionDenial::Projection)?;
        Ok(receipt)
    }

    pub(crate) fn finish(
        self,
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.assembler.finish()
    }

    pub(crate) fn finish_for_reconciliation(
        self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::mounting::UiPreparedMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.assembler.finish_for_reconciliation(replacements)
    }
}
