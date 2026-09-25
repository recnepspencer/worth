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
    /// Move this frame's presented hit rows by a settled scroll pose's own
    /// translations, reporting the spatial work that took.
    pub(in crate::mounting::retention) fn refresh_hit_scroll(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        translations: &[(
            UiMountedInstanceIdentity,
            crate::mounting::presentation::UiScrollPoseShift,
        )],
    ) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        let mut work = crate::mounting::hit_test_work::UiHitTestSpatialWork::default();
        for (candidate, presentation) in self.current_presentations().collect::<Vec<_>>() {
            if candidate != surface {
                continue;
            }
            work.merge(
                self.visual_regions
                    .presented_hits
                    .apply_scroll_translations(presentation.binding(), translations),
            );
        }
        work
    }
}
