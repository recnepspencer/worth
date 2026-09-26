use super::WorthUiPreparedApplicationActivation;

impl WorthUiPreparedApplicationActivation {
    pub(in crate::facade::entry) fn candidate_graph(&self) -> crate::graph::UiGraphAuthority<'_> {
        crate::graph::UiGraphAuthority::new(&self.candidate_graph)
    }

    pub(super) fn candidate_scroll_bounds(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        target: crate::graph::UiGraphNodeIdentity,
        catalog: &crate::runtime::UiMountedAllocationProjectionCatalog,
    ) -> Option<crate::runtime::scroll::UiScrollBounds> {
        let graph_node = owner.allocation_graph_node(target);
        let projection = catalog.projection(graph_node).ok()??;
        let worth_ui_host_contract::UiMountedAllocationProjection::Known {
            bounds: content, ..
        } = projection
        else {
            return None;
        };
        let viewport = catalog.viewport_bounds(graph_node).ok()??.mounted_box();
        let declared = self.candidate_declared_scroll_extent(graph_node);
        let inline_content = declared.map_or(content.width(), |value| content.width().max(value));
        let block_content = declared.map_or(content.height(), |value| content.height().max(value));
        crate::runtime::scroll::UiScrollBounds::new(
            crate::units::UiSubpixels::nearest_distance(
                (inline_content - viewport.width()).max(0.0),
            )?
            .count(),
            crate::units::UiSubpixels::nearest_distance(
                (block_content - viewport.height()).max(0.0),
            )?
            .count(),
        )
    }

    fn candidate_declared_scroll_extent(
        &self,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Option<f32> {
        let graph = self.candidate_graph();
        let record = graph.lookup().graph_node(graph_node)?;
        let graph_record = record.value();
        let declaration = graph_record.declaration_identity();
        let trace = self.visual_trace_source();
        let artifact = trace
            .declaration_artifacts()
            .iter()
            .find(|artifact| artifact.identity() == declaration)?;
        let handoff = artifact.graph_handoff().ok()?;
        let sizing = handoff.mosaic_sizing_contract_id()?;
        let measurement = self
            .candidate_application_authority
            .mosaic_sizing_capabilities()
            .get(sizing)?
            .named_measurement()?;
        let crate::capability::MeasurementValue::LogicalPixels(value) = measurement.value() else {
            return None;
        };
        Some(*value as f32)
    }
}
