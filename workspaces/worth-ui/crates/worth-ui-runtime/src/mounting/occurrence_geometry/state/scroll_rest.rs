//! Where a Scroll region's content rests while no region scrolls it.
//!
//! An applied pose translates a region owner's descendants and leaves the
//! owner where it stands, so a region's content box at offset zero is its
//! owner's box. A region nested in another is that other's content, though,
//! and the outer pose moves the inner owner with everything else it carries.
//! A Scroll sample is measured from a rest no enclosing pose moves, so the
//! offset an inner region's sample stands at reads the same whether or not
//! the region carrying it has scrolled since the sample was taken: the
//! owner's box with every enclosing pose taken back out.

use super::*;
use crate::mounting::presentation::{UiPublishedRect, UiScrollPoseShift};

impl UiMountedOccurrenceGeometryState {
    /// The content box of the region at `slot` of `target`'s chain with that
    /// region and every region enclosing it at offset zero, where it is laid
    /// out.
    pub(crate) fn scroll_region_rest(
        &self,
        surface: UiSemanticSurfaceIdentity,
        target: UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<crate::mounting::UiLaidOut<worth_ui_host_contract::UiMountedCanonicalBox>> {
        let (owner, region) = self.scroll_region_geometry(surface, target, slot)?;
        let geometry = self.surfaces.get(&surface)?;
        let mut unscrolled = UiScrollPoseShift::none();
        let mut cursor = geometry.occurrences.get(&owner)?.parent;
        while let Some(ancestor) = cursor {
            if let Some(pose) = geometry.scroll_poses.get(&ancestor) {
                unscrolled = unscrolled.then(UiScrollPoseShift::between(
                    *pose,
                    crate::runtime::scroll::UiScrollOffset::default(),
                ));
            }
            cursor = geometry.occurrences.get(&ancestor)?.parent;
        }
        Some(region.map(|region| UiPublishedRect::box_following_pose(region.content(), unscrolled)))
    }
}
