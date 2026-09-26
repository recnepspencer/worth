//! A Scroll region's content and viewport boxes, which a Portal presents
//! together.

use super::{UiPortalMove, UiPortalPresentable};
use worth_ui_host_contract::UiMountedCanonicalBox;

/// One Scroll region occurrence's geometry: its owner's box, where its
/// content lays out at offset zero, and the viewport that clips it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMountedScrollRegionBoxes {
    content: UiMountedCanonicalBox,
    viewport: UiMountedCanonicalBox,
}

impl UiMountedScrollRegionBoxes {
    pub(crate) const fn new(
        content: UiMountedCanonicalBox,
        viewport: UiMountedCanonicalBox,
    ) -> Self {
        Self { content, viewport }
    }

    pub(crate) const fn content(&self) -> UiMountedCanonicalBox {
        self.content
    }

    pub(crate) const fn viewport(&self) -> UiMountedCanonicalBox {
        self.viewport
    }

    /// How far the region scrolls. A placement moves both boxes by one step,
    /// so the region scrolls as far wherever a frame presents it.
    pub(crate) fn bounds(&self) -> Option<crate::runtime::scroll::UiScrollBounds> {
        crate::runtime::scroll::UiScrollBounds::from_mounted_region(self.content, self.viewport)
    }
}

impl UiPortalPresentable for UiMountedScrollRegionBoxes {
    type Denial = <UiMountedCanonicalBox as UiPortalPresentable>::Denial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Self::Denial> {
        let (Some(content), Some(viewport)) = (self.content.moved(by)?, self.viewport.moved(by)?)
        else {
            return Ok(None);
        };
        Ok(Some(Self { content, viewport }))
    }
}
