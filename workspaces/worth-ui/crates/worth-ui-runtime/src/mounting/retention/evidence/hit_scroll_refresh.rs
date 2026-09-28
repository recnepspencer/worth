//! Carrying a settled scroll pose into the hit rows a retained frame kept.
//!
//! A presented frame answers pointer questions from rows it captured when it
//! was published. Scrolling moves content without republishing anything, so
//! those rows would go on answering for the pose the reader has left behind.
//! Translating them is what keeps the frame's own answers current.
//!
//! The translations are applied to one surface only. A pose is prepared
//! against one surface's geometry and says nothing about any other, so a
//! frame spanning several surfaces moves the rows of the one that scrolled
//! and leaves the rest exactly as they were published.

use worth_ui_host_contract::UiMountedInstanceIdentity;

impl super::UiRetainedPresentedFrame {
    /// Move this frame's presented hit rows by a scroll pose's own
    /// translations, as `standing` says the pose stands, reporting the
    /// spatial work that took.
    pub(in crate::mounting::retention) fn refresh_hit_scroll(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        translations: &[(UiMountedInstanceIdentity, crate::mounting::UiHitScrollMove)],
        standing: crate::mounting::presented_hit_index::UiHitScrollStanding,
    ) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        let Some(binding) = self
            .displayed_for_surface(surface)
            .map(|displayed| displayed.binding())
        else {
            return crate::mounting::hit_test_work::UiHitTestSpatialWork::default();
        };
        self.visual_regions
            .presented_hits
            .move_by_scroll(binding, translations, standing)
    }

    /// Whether this frame presents `surface` and holds a row that leads past
    /// its committed pose. The frame's leads are not told apart by surface:
    /// a frame presenting several surfaces answers for all of its rows.
    pub(in crate::mounting::retention) fn holds_hit_scroll_leads(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.current_presentations()
            .any(|(candidate, _)| candidate == surface)
            && self.visual_regions.presented_hits.holds_scroll_leads()
    }

    /// Take the scroll leads `previous`, the frame the host shows `surface`
    /// in, holds on the rows this successor reuses, because the successor
    /// shows the same accepted sample.
    pub(in crate::mounting::retention) fn inherit_hit_scroll_leads(
        &mut self,
        previous: &Self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) {
        let Some(binding) = self
            .displayed_for_surface(surface)
            .map(|displayed| displayed.binding())
        else {
            return;
        };
        self.visual_regions
            .presented_hits
            .inherit_scroll_leads(&previous.visual_regions.presented_hits, binding);
    }
}
