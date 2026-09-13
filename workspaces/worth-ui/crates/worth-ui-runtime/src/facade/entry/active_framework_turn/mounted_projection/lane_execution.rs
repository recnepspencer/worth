use super::{WorthUiActiveMountedProjectionFrame, WorthUiMountedLaneProjectionDenial};

impl WorthUiActiveMountedProjectionFrame<'_, '_> {
    pub(super) fn execute_requested_lanes(
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
        crate::mounting::UiAssembledMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.assembler.finish()
    }

    pub(crate) fn finish_for_reconciliation(
        self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<
        crate::mounting::UiAssembledMountedFrame,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        self.assembler.finish_for_reconciliation(replacements)
    }
}
