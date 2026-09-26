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
use crate::mounting::{
    UiLaidOut, UiMountedAppearanceScrollChromeInput, UiMountedPlacement, UiPresented,
};
use worth_ui_host_contract::UiMountedInstanceIdentity;

impl UiMountedSemanticProjection {
    /// Where this projection presents the Scroll region occurrence
    /// `instance`. An occurrence it does not project is no Portal content, so
    /// it stays where it is laid out; Portal content is shown only through a
    /// Portal that covers some of its surface.
    pub(in crate::mounting) fn region_placement(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> UiMountedPlacement {
        let Some(node) = self.node(instance) else {
            return UiMountedPlacement::InPlace;
        };
        if node.portal_child_owner.is_none() {
            return UiMountedPlacement::InPlace;
        }
        match node.appearance_geometry.placement() {
            placement @ UiMountedPlacement::ThroughPortal(_) if placement.coverage().is_some() => {
                placement
            }
            UiMountedPlacement::ThroughPortal(_)
            | UiMountedPlacement::InPlace
            | UiMountedPlacement::Hidden => UiMountedPlacement::Hidden,
        }
    }

    /// `chrome`, derived where each region is laid out, placed where this
    /// projection presents that region. A region it presents nowhere paints
    /// no chrome.
    pub(in crate::mounting) fn place_scroll_chrome(
        &self,
        chrome: &[UiLaidOut<UiMountedAppearanceScrollChromeInput>],
    ) -> Result<
        Vec<UiPresented<UiMountedAppearanceScrollChromeInput>>,
        UiMountedAppearanceGeometryDenial,
    > {
        let mut placed = Vec::with_capacity(chrome.len());
        for input in chrome {
            placed.extend(
                self.region_placement(input.in_layout_space().owner_instance())
                    .present(*input)?,
            );
        }
        Ok(placed)
    }
}
