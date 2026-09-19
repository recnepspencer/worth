use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    pub(crate) fn scroll_bounds_for(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        target: crate::graph::UiGraphNodeIdentity,
    ) -> Result<
        crate::runtime::scroll::UiScrollBounds,
        crate::runtime::scroll::UiScrollBoundsResolutionDenial,
    > {
        let graph_node = owner.allocation_graph_node(target);
        let projection = self
            .mounted_allocation_projection_for(graph_node)
            .map_err(|_| {
                crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable
            })?
            .ok_or(crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable)?;
        let content = match projection {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, .. } => bounds,
            worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                ..
            }
            | worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                return Err(
                    crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable,
                );
            }
        };
        let viewport = self
            .mounted_viewport_bounds_for(graph_node)
            .map_err(|_| {
                crate::runtime::scroll::UiScrollBoundsResolutionDenial::ViewportUnavailable
            })?
            .ok_or(crate::runtime::scroll::UiScrollBoundsResolutionDenial::ViewportUnavailable)?
            .mounted_box();
        let declared_extent = self.declared_scroll_content_extent(graph_node);
        let content_inline =
            declared_extent.map_or(content.width(), |extent| content.width().max(extent));
        let content_block =
            declared_extent.map_or(content.height(), |extent| content.height().max(extent));
        let inline = logical_extent_to_subpixels((content_inline - viewport.width()).max(0.0))?;
        let block = logical_extent_to_subpixels((content_block - viewport.height()).max(0.0))?;
        crate::runtime::scroll::UiScrollBounds::new(inline, block)
            .ok_or(crate::runtime::scroll::UiScrollBoundsResolutionDenial::OutOfRange)
    }

    fn declared_scroll_content_extent(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Option<f32> {
        let graph_record = self.app.graph().lookup().graph_node(graph_node)?;
        let graph_node_record = graph_record.value();
        let declaration = graph_node_record.declaration_identity();
        let artifact = self
            .app
            .declaration_artifacts()
            .iter()
            .find(|artifact| artifact.identity() == declaration)?;
        let handoff = artifact.graph_handoff().ok()?;
        let sizing = handoff.mosaic_sizing_contract_id()?;
        let measurement = self
            .app
            .capabilities()
            .mosaic_sizing_contracts()
            .get(sizing)?
            .named_measurement()?;
        let crate::capability::MeasurementValue::LogicalPixels(value) = measurement.value() else {
            return None;
        };
        Some(*value as f32)
    }

    pub(crate) fn install_scroll_ownership(
        &self,
        scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
        identity: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        mounted: &crate::mounting::UiMountedIdentityBasis,
    ) {
        scroll.resolve_and_install_ownership(
            identity,
            incarnation,
            self.app.graph(),
            crate::mounting::UiMountedPlanProjectionSource::Executed(
                self.runtime.active.active_plan_ref(),
            ),
            mounted.graph_node_identity(),
            mounted.semantic_surface_identity(),
            mounted.repeated_instance_basis().identity_digest(),
        );
    }

    pub(crate) fn mounted_allocation_projection_for(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<
        Option<worth_ui_host_contract::UiMountedAllocationProjection>,
        crate::runtime::UiMountedAllocationProjectionDenial,
    > {
        self.runtime
            .allocation_receipt_ledger
            .mounted_projection_source(None)
            .projection(graph_node)
    }

    pub(crate) fn mounted_viewport_bounds_for(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<
        Option<crate::runtime::UiCommittedViewportGeometry>,
        crate::runtime::UiMountedAllocationProjectionDenial,
    > {
        self.runtime
            .allocation_receipt_ledger
            .mounted_projection_source(None)
            .viewport_bounds(graph_node)
    }

    pub(crate) fn measurement_policy_for(
        &self,
        declaration: &crate::declaration::UiDeclarationIdentity,
    ) -> Option<crate::declaration::UiDeclaredMeasurementPolicyPosture> {
        self.app
            .declaration_artifacts()
            .iter()
            .find(|artifact| artifact.identity() == declaration)
            .and_then(|artifact| artifact.graph_handoff().ok())
            .and_then(|handoff| handoff.measurement_policy().admitted().cloned())
    }

    pub(crate) fn activate_initial_mounted_allocation_catalog(
        &mut self,
        graph_successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationGraphSuccessor,
        admitted: crate::graph::UiAdmittedAllocationCatalogBasisSet,
        boundary: crate::runtime::WorthUiFrameBoundary,
    ) -> Result<
        crate::runtime::UiCommittedAllocationReplan,
        crate::runtime::WorthUiInitialMountedAllocationActivationDenial,
    > {
        self.runtime.activate_initial_mounted_allocation_catalog(
            &mut self.app,
            graph_successor,
            admitted,
            boundary,
        )
    }
}

fn logical_extent_to_subpixels(
    value: f32,
) -> Result<i64, crate::runtime::scroll::UiScrollBoundsResolutionDenial> {
    let scaled = f64::from(value)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    if !scaled.is_finite() || scaled < 0.0 || scaled > i64::MAX as f64 {
        return Err(crate::runtime::scroll::UiScrollBoundsResolutionDenial::OutOfRange);
    }
    Ok(scaled.round() as i64)
}
