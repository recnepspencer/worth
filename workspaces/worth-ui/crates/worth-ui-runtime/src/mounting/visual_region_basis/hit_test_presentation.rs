//! The hit row a presented frame hands interaction, before any Motion moves it.

use super::UiMountedHitTestPresentation;
use crate::mounting::UiHitAncestorClip;

impl UiMountedHitTestPresentation {
    pub(in crate::mounting) fn completed(
        mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
        portal: Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
        owns_presented_portal: bool,
        ancestor_clip: UiHitAncestorClip,
    ) -> Self {
        Self {
            mechanic,
            portal,
            owns_presented_portal,
            ancestor_clip,
        }
    }

    pub(crate) const fn mechanic(&self) -> worth_ui_host_contract::UiMountedHitTestMechanic {
        self.mechanic
    }

    pub(crate) const fn portal(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic> {
        self.portal
    }

    pub(crate) const fn owns_presented_portal(&self) -> bool {
        self.owns_presented_portal
    }

    /// The Scroll and Mosaic clips the row sits inside, relative to where its
    /// frame committed it.
    pub(crate) const fn ancestor_clip(&self) -> UiHitAncestorClip {
        self.ancestor_clip
    }

    #[cfg(test)]
    pub(crate) const fn for_test(
        mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
    ) -> Self {
        Self {
            mechanic,
            portal: None,
            owns_presented_portal: false,
            ancestor_clip: UiHitAncestorClip::Unclipped,
        }
    }
}
