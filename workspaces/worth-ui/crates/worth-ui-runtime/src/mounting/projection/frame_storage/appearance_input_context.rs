#[derive(Clone)]
pub(crate) struct UiMountedAppearanceNodeInputContext {
    pub(crate) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(crate) semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(crate) incarnation: worth_ui_host_contract::UiMountIncarnation,
    pub(crate) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(super) issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    pub(super) plan_digest: u64,
    pub(super) allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    pub(super) appearance_clip: super::super::appearance::UiMountedAppearanceClip,
    pub(super) surface_paint_order: Option<u32>,
    pub(super) portal_group: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    pub(crate) geometry_input: Option<crate::mounting::UiMountedAppearanceGeometryInput>,
    pub(super) text_foreground_spans: Box<[crate::mounting::UiMountedAppearanceTextSpanInput]>,
}

impl UiMountedAppearanceNodeInputContext {
    #[cfg(test)]
    pub(crate) fn with_clip_for_test(
        mut self,
        clip: super::super::appearance::UiMountedAppearanceClip,
    ) -> Self {
        self.appearance_clip = clip;
        self
    }
    pub(crate) const fn issuer(&self) -> worth_ui_host_contract::UiMountedNodeReceiptIssuer {
        self.issuer
    }

    pub(crate) fn lower_resolved_projection(
        &self,
        projection: &crate::runtime::appearance::UiAppearanceProjection,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        outline_fringe: Result<
            worth_ui_host_contract::UiAppearanceLogicalLength,
            crate::mounting::UiMountedAppearanceLoweringDenial,
        >,
    ) -> Result<
        super::super::UiMountedAppearanceLoweringInput,
        super::super::UiMountedAppearanceLoweringDenial,
    > {
        let input = super::super::UiMountedAppearanceNodeInput::from_resolved_projection(
            super::super::UiResolvedAppearanceNodeSource {
                issuer: self.issuer,
                semantic_surface: self.semantic_surface,
                node_receipt: self.node_receipt,
                graph_node: self.graph_node,
                plan_digest: self.plan_digest,
                allocation: self.allocation,
                clip: self.appearance_clip,
                surface_paint_order: self.surface_paint_order,
                portal_group: self.portal_group,
                geometry_input: self.geometry_input.clone(),
                text_foreground_spans: &self.text_foreground_spans,
                projection,
                outline_fringe,
            },
        )?;
        Ok(super::super::UiMountedAppearanceLoweringInput::for_node(
            self.frame,
            self.semantic_surface,
            presentation,
            input,
        ))
    }
}
