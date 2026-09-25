//! Chrome of a region the frame presents through a Portal.
//!
//! Chrome is derived where its region is laid out. When the frame presents
//! that region through a Portal, the chrome moves by the step the Portal
//! moves its source anchor and shows only within what the Portal covers, so
//! the bars paint over the content they scroll.

use super::portal_geometry::{portal_coverage, portal_step, step_coordinate};
use super::{UiMountedAppearanceGeometryDenial as Denial, UiMountedAppearanceScrollChromeInput};
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiMountedCanonicalBox,
    UiMountedPortalOverlayMechanic,
};

impl UiMountedAppearanceScrollChromeInput {
    /// This chrome where `portal` presents its region: `None` when none of it
    /// shows within what the Portal covers.
    pub(in crate::mounting::projection) fn through_portal(
        mut self,
        portal: UiMountedPortalOverlayMechanic,
        source_anchor: UiMountedCanonicalBox,
    ) -> Result<Option<Self>, Denial> {
        let [x, y] = portal_step(portal, source_anchor)?;
        self.rect = UiAppearanceAllocationBounds::new(
            step_coordinate(self.rect.x(), x)?,
            step_coordinate(self.rect.y(), y)?,
            self.rect.width(),
            self.rect.height(),
        )
        .map_err(|_| Denial::EmptyAtCanonicalPrecision)?;
        let clip = UiAppearanceClip::new(
            step_coordinate(self.clip.x(), x)?,
            step_coordinate(self.clip.y(), y)?,
            self.clip.width(),
            self.clip.height(),
        )
        .map_err(|_| Denial::EmptyAtCanonicalPrecision)?;
        let Some(clip) = portal_coverage(portal)?
            .and_then(|coverage| super::clip::intersect_clips(coverage, clip))
        else {
            return Ok(None);
        };
        self.clip = clip;
        Ok(Some(self))
    }
}
