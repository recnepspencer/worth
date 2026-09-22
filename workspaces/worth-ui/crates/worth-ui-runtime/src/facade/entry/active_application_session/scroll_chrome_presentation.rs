//! Lowering every region's derived chrome on one surface into painted parts.
//!
//! This is the whole presentation side of chrome in one call: the regions that
//! present chrome, the offset each was derived at, the grid the surface is
//! bound to, and the interaction posture each part is in. What comes back is
//! the set of rectangles a paint mechanic would publish, in paint order, with
//! the declared role and the Hover/Pressed classes already resolved.
//!
//! The pointer position it takes is the one the surface last resolved. Hover is
//! re-answered from the accepted displayed offset every time this is called, so
//! a bar that moved under a stationary pointer reports the part that is now
//! under it rather than the part that was.

use crate::mounting::{
    lower_scroll_chrome, UiMountedScrollChromeNode, UiScrollChromeLoweringDenial,
    UiScrollChromeLoweringInput,
};

impl super::super::WorthUiActiveApplicationSession {
    /// Every painted chrome rectangle on `surface`, snapped to that surface's
    /// device grid and clipped to the region that reserved its gutter.
    ///
    /// A surface with no bound device grid paints nothing rather than guessing
    /// one, and a region without travel contributes nothing because its chrome
    /// was never derived.
    pub(in crate::facade::entry) fn lowered_scroll_chrome(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        pointer: Option<[f32; 2]>,
    ) -> Result<Vec<UiMountedScrollChromeNode>, UiScrollChromeLoweringDenial> {
        let Some(device_scale) = self.mounted.scroll_chrome_device_scale(surface) else {
            return Ok(Vec::new());
        };
        let hovered =
            pointer.and_then(|point| self.prepared_scroll_chrome_under_pointer(surface, point));
        let drag = self.interaction.scroll_chrome_latch();
        let mut nodes = Vec::new();
        for region in self.scroll_chrome_facts(surface) {
            nodes.extend(lower_scroll_chrome(UiScrollChromeLoweringInput {
                owner_instance: region.owner_instance(),
                facts: region.facts(),
                track_role: region.track_role(),
                thumb_role: region.thumb_role(),
                clip: region.viewport(),
                device_scale,
                hovered: hovered
                    .as_ref()
                    .filter(|answer| answer.owner() == region.owner())
                    .and_then(|answer| answer.part())
                    .map(|part| (part.axis(), part.part())),
                drag: drag
                    .filter(|held| held.owner() == region.owner())
                    .map(|held| (held.axis(), held.posture())),
            })?);
        }
        Ok(nodes)
    }
}
