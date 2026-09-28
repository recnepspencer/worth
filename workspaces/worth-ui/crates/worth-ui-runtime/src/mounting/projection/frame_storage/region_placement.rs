//! Where a frame presents a Scroll region occurrence.
//!
//! Scroll geometry is derived where a region is laid out: its content and
//! viewport boxes, and the chrome drawn from them. A region in Portal content
//! is not presented there. The frame moves it by the step its Portal moves
//! the source anchor and shows it only within what the Portal covers, as it
//! presents the region's own node. Each reader places scroll geometry through
//! the frame that presents it: the frame being prepared for paint and Motion,
//! the frame on screen for the pointer.

use super::{UiMountedProjectionFrame, UiMountedSemanticProjection};
use crate::mounting::projection::appearance::UiMountedAppearanceGeometryDenial;
use crate::mounting::{
    UiLaidOut, UiMountedAppearanceScrollChromeInput, UiMountedPlacement, UiPresented,
};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPortalOverlayMechanic};

impl UiMountedSemanticProjection {
    /// Where this projection presents the occurrence `instance`. An
    /// occurrence it does not project is no Portal content, so it stays where
    /// it is laid out; Portal content is shown only through a Portal that
    /// covers some of its surface.
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

impl UiMountedProjectionFrame {
    /// Whether scrolling `region` moves where this frame presents
    /// `instance`, laid out in the region's content: the occurrence itself,
    /// or with `opened` the overlay of the Portal it opened. Content
    /// presented where the region is moves with it. Content moved through a
    /// Portal the region is not presented through, and a Portal's overlay,
    /// move with it only while that Portal stands below or above an anchor
    /// the region moves: a centered modal opened over the region, or any
    /// Portal the region's own owner opened, stays where it is while the
    /// region scrolls.
    pub(in crate::mounting) fn scrolls_with_region(
        &self,
        region: UiMountedInstanceIdentity,
        instance: UiMountedInstanceIdentity,
        opened: Option<UiMountedPortalOverlayMechanic>,
    ) -> bool {
        if opened.is_some_and(|portal| !self.overlay_follows_anchor(portal)) {
            return false;
        }
        let region_portal = self
            .semantic
            .region_placement(region)
            .portal()
            .map(|portal| (portal.owner(), portal.portal_identity()));
        let mut at = instance;
        // Each Portal crossed leads to its owner, laid out outside the
        // Portal's content, and Portal content nests as a tree, so the walk
        // ends at content presented in place.
        while let Some(presentation) = self.semantic.region_placement(at).presentation() {
            let portal = presentation.portal();
            if region_portal == Some((portal.owner(), portal.portal_identity())) {
                return true;
            }
            // A region carries its content, never its own owner's box.
            if portal.owner() == region || !presentation.follows_anchor() {
                return false;
            }
            at = portal.owner();
        }
        true
    }

    /// Whether the overlay `portal` paints stands where it does because of
    /// where its anchor is. This frame paints an overlay only for a Portal it
    /// has an input for; one it has none for is placed by no anchor it
    /// knows, so it stays where it is shown.
    fn overlay_follows_anchor(&self, portal: UiMountedPortalOverlayMechanic) -> bool {
        self.portal_overlays.iter().any(|input| {
            input.surface() == portal.surface()
                && input.owner() == portal.owner()
                && input.portal_identity() == portal.portal_identity()
                && input.placement().follows_anchor()
        })
    }
}
