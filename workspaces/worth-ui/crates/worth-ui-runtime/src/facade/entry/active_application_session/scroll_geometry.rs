impl super::super::WorthUiActiveApplicationSession {
    pub(super) fn scroll_bounds_for_mounted_owner(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        slot: usize,
    ) -> Result<
        crate::runtime::scroll::UiScrollBounds,
        crate::runtime::scroll::UiScrollBoundsResolutionDenial,
    > {
        if let Some(scroll) = self.scroll.as_ref() {
            if scroll.has_unpresented_layout(owner.semantic_surface()) {
                if let Some(incarnation) = self.mounted.scroll_region_incarnation(target, slot) {
                    if let Ok(bounds) = scroll.accepted_owner_bounds(owner, incarnation) {
                        return Ok(bounds);
                    }
                }
            }
        }
        if let Some((_, region)) = self.mounted.scroll_region_geometry(target, slot) {
            return region
                .in_layout_space()
                .bounds()
                .ok_or(crate::runtime::scroll::UiScrollBoundsResolutionDenial::OutOfRange);
        }
        if matches!(
            owner,
            crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. }
        ) {
            return Err(
                crate::runtime::scroll::UiScrollBoundsResolutionDenial::AllocationUnavailable,
            );
        }
        self.application.scroll_bounds_for(owner, graph_node)
    }

    /// The viewport `owner` shows, as laid out: a Scroll region's own, or the
    /// viewport a surface or viewport owner is allocated.
    pub(super) fn scroll_viewport_for_mounted_owner(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        slot: usize,
    ) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        if let Some((_, region)) = self.mounted.scroll_region_geometry(target, slot) {
            return Some(region.in_layout_space().viewport());
        }
        if matches!(
            owner,
            crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. }
        ) {
            return None;
        }
        self.application
            .mounted_viewport_bounds_for(owner.allocation_graph_node(graph_node))
            .ok()?
            .map(|viewport| viewport.mounted_box())
    }
}
