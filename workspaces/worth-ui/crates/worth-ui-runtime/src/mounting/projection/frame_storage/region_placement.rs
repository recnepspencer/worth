//! Where a frame presents a Scroll region occurrence.
//!
//! Scroll geometry is derived where a region is laid out: its content and
//! viewport boxes, and the chrome drawn from them. A region in Portal content
//! is not presented there. The frame moves it by the step its Portal moves
//! the source anchor and shows it only within what the Portal covers, as it
//! presents the region's own node. Each reader places scroll geometry through
//! the frame that presents it: the frame being prepared for paint and Motion,
//! the frame on screen for the pointer.

use super::UiMountedSemanticProjection;
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryDenial;
use crate::mounting::UiMountedAppearanceScrollChromeInput;
use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiMountedPortalOverlayMechanic,
};

/// Where one frame presents a Scroll region occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiMountedRegionPlacement {
    /// Where it is laid out.
    InPlace,
    /// Moved through an open Portal, and shown only within `coverage`.
    ThroughPortal {
        portal: UiMountedPortalOverlayMechanic,
        source_anchor: UiMountedCanonicalBox,
        coverage: UiMountedCanonicalBox,
    },
    /// Portal content the frame presents nowhere.
    Hidden,
}

impl UiMountedRegionPlacement {
    /// Laid-out `bounds` where this placement presents them: `None` when it
    /// presents nothing, or the move leaves canonical geometry.
    pub(crate) fn place(self, bounds: UiMountedCanonicalBox) -> Option<UiMountedCanonicalBox> {
        match self {
            Self::InPlace => Some(bounds),
            Self::ThroughPortal {
                portal,
                source_anchor,
                ..
            } => crate::mounting::projection::appearance::translate_box(
                bounds,
                portal,
                source_anchor,
            )
            .ok(),
            Self::Hidden => None,
        }
    }

    /// What of the surface a region placed here shows through beyond its own
    /// viewport: `None` when nothing else confines it.
    pub(crate) const fn coverage(self) -> Option<UiMountedCanonicalBox> {
        match self {
            Self::ThroughPortal { coverage, .. } => Some(coverage),
            Self::InPlace | Self::Hidden => None,
        }
    }
}

impl UiMountedSemanticProjection {
    /// Where this projection presents `instance`. An occurrence it does not
    /// project is no Portal content, so it stays where it is laid out.
    pub(in crate::mounting) fn region_placement(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> UiMountedRegionPlacement {
        let Some(node) = self.node(instance) else {
            return UiMountedRegionPlacement::InPlace;
        };
        if node.portal_child_owner.is_none() {
            return UiMountedRegionPlacement::InPlace;
        }
        let Some((portal, source_anchor)) = node.appearance_geometry.portal_presentation else {
            return UiMountedRegionPlacement::Hidden;
        };
        crate::mounting::projection::appearance::portal_coverage_box(portal).map_or(
            UiMountedRegionPlacement::Hidden,
            |coverage| UiMountedRegionPlacement::ThroughPortal {
                portal,
                source_anchor,
                coverage,
            },
        )
    }

    /// `chrome`, derived where each region is laid out, placed where this
    /// projection presents that region. A region it presents nowhere paints
    /// no chrome.
    pub(in crate::mounting) fn place_scroll_chrome(
        &self,
        chrome: &[UiMountedAppearanceScrollChromeInput],
    ) -> Result<Vec<UiMountedAppearanceScrollChromeInput>, UiMountedAppearanceGeometryDenial> {
        let mut placed = Vec::with_capacity(chrome.len());
        for input in chrome {
            match self.region_placement(input.owner_instance()) {
                UiMountedRegionPlacement::InPlace => placed.push(*input),
                UiMountedRegionPlacement::ThroughPortal {
                    portal,
                    source_anchor,
                    ..
                } => placed.extend(input.through_portal(portal, source_anchor)?),
                UiMountedRegionPlacement::Hidden => {}
            }
        }
        Ok(placed)
    }
}
