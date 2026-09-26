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
}
