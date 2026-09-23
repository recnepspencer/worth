//! The declared region-kind behind one Scroll owner occurrence.
//!
//! A Scroll owner identity carries the plan index of the region occurrence that
//! declared it, so the region-kind descriptor is an index lookup in the mounted
//! execution plan, never a search of the graph. This file resolves that
//! descriptor and reads the one scalar declaration hanging off it: what a
//! single line of the region's content is worth.

use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning;

impl super::super::WorthUiActiveApplicationSession {
    /// The region-kind descriptor that declared `owner`, or `None` when the
    /// owner is a surface or viewport rather than a declared region occurrence.
    pub(in crate::facade::entry) fn declared_scroll_region_descriptor(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> Option<std::rc::Rc<WorthUiPlanOrdinaryMeaning>> {
        let crate::runtime::scroll::UiScrollOwnerIdentity::Region {
            plan_region_index, ..
        } = owner
        else {
            return None;
        };
        let meaning = self
            .application
            .mounted_geometry_plan()
            .mounted_projection_ordinary_meaning(plan_region_index)?;
        matches!(meaning.as_ref(), WorthUiPlanOrdinaryMeaning::Layout(_)).then_some(meaning)
    }

    /// What one line of `owner`'s content is worth, in logical points. Only a
    /// declared region occurrence answers: a surface or viewport owner declares
    /// no content line, so a coarse wheel has no travel to derive over it.
    pub(in crate::facade::entry) fn declared_scroll_line_extent_logical_points(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> Option<u16> {
        let meaning = self.declared_scroll_region_descriptor(owner)?;
        let WorthUiPlanOrdinaryMeaning::Layout(layout) = meaning.as_ref() else {
            return None;
        };
        layout
            .region_descriptor()?
            .scroll_line_extent()
            .map(crate::capability::UiScrollLineExtent::logical_points_value)
    }
}
