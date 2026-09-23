//! Choosing the item a Scroll region holds still when its layout is replaced.
//!
//! A layout replacement rebuilds a surface from scratch. If the offset alone
//! survived it, content inserted above what the reader is looking at would
//! push that content out from under them by exactly the height of the
//! insertion, and content removed above it would pull it up the same way. The
//! offset is a distance from rest, and rest is what just moved.
//!
//! So an owner remembers one of its content occurrences and where that
//! occurrence sits inside its content. When the same occurrence is still there
//! after the replacement, the distance it moved is the distance the offset has
//! to move with it, which is what `UiScrollAnchorPolicy::Rebase` computes.
//! When it is gone there is nothing to measure against, and the existing
//! offset is clamped into the new extent instead of guessed at.
//!
//! Positions are taken in the content's own space -- from the owner's box,
//! which is the content box at rest -- so they do not depend on which offset
//! happened to be applied when they were read. Every reading here is taken
//! before the replacement applies its poses, so the rows are at rest.

use super::*;

/// One occurrence of a region's content: which it is, and how far its top-left
/// corner sits from the content's own origin.
type ContentPosition = (UiMountedInstanceIdentity, i64, i64);

impl UiMountedOccurrenceGeometryState {
    /// The anchor a replaced layout should reconcile one region owner against,
    /// and the policy that says what to do with it.
    ///
    /// `previous` is the anchor the owner was carrying, `offset` the distance
    /// it had travelled, and `bounds` the extent the new layout leaves it.
    pub(crate) fn scroll_rebind_anchor(
        &self,
        surface: UiSemanticSurfaceIdentity,
        owner: UiMountedInstanceIdentity,
        binding: UiSurfaceBindingGeneration,
        previous: Option<crate::runtime::scroll::UiScrollAnchor>,
        offset: crate::runtime::scroll::UiScrollOffset,
        bounds: crate::runtime::scroll::UiScrollBounds,
    ) -> (
        Option<crate::runtime::scroll::UiScrollAnchor>,
        crate::runtime::scroll::UiScrollAnchorPolicy,
    ) {
        let positions = self.scroll_content_positions(surface, owner);
        if let Some(item) =
            previous.and_then(crate::runtime::scroll::UiScrollAnchor::mounted_identity)
        {
            if let Some(surviving) = positions.iter().find(|(instance, _, _)| *instance == item) {
                return (
                    anchor_at(*surviving, binding),
                    crate::runtime::scroll::UiScrollAnchorPolicy::Rebase,
                );
            }
        }
        // Nothing to measure against, so the offset is clamped rather than
        // moved. A fresh anchor is chosen from the new layout in the same
        // breath, because leaving the old one behind would leave the owner
        // matching an occurrence that can never be mounted again.
        let settled = bounds.clamp_subpixels(offset.inline_subpixels(), offset.block_subpixels());
        (
            self.chosen_scroll_anchor(&positions, settled, binding),
            crate::runtime::scroll::UiScrollAnchorPolicy::Clamp,
        )
    }

    /// The content occurrence nearest the corner the reader is looking at: the
    /// first one that begins at or after the offset, or the last one when the
    /// offset is past all of them.
    fn chosen_scroll_anchor(
        &self,
        positions: &[ContentPosition],
        offset: crate::runtime::scroll::UiScrollOffset,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<crate::runtime::scroll::UiScrollAnchor> {
        let corner = (offset.block_subpixels(), offset.inline_subpixels());
        let chosen = positions
            .iter()
            .find(|(_, inline, block)| (*block, *inline) >= corner)
            .or_else(|| positions.last())?;
        anchor_at(*chosen, binding)
    }

    /// Every occurrence that travels with `owner`, ordered down the content and
    /// then across it, so the same layout always names the same anchor.
    fn scroll_content_positions(
        &self,
        surface: UiSemanticSurfaceIdentity,
        owner: UiMountedInstanceIdentity,
    ) -> Vec<ContentPosition> {
        let Some(geometry) = self.surfaces.get(&surface) else {
            return Vec::new();
        };
        let Some(origin) = geometry.occurrences.get(&owner).map(|row| row.bounds) else {
            return Vec::new();
        };
        let mut positions = geometry
            .occurrences
            .iter()
            .filter(|(instance, _)| self.scrolled_content_owner(surface, **instance) == Some(owner))
            .filter_map(|(instance, row)| {
                Some((
                    *instance,
                    content_subpixels(row.bounds.x() - origin.x())?,
                    content_subpixels(row.bounds.y() - origin.y())?,
                ))
            })
            .collect::<Vec<_>>();
        positions.sort_unstable_by_key(|(instance, inline, block)| (*block, *inline, *instance));
        positions
    }
}

fn anchor_at(
    (instance, inline, block): ContentPosition,
    binding: UiSurfaceBindingGeneration,
) -> Option<crate::runtime::scroll::UiScrollAnchor> {
    crate::runtime::scroll::UiScrollAnchor::new(
        crate::runtime::scroll::UiScrollAnchorIdentity::mounted(instance),
        binding,
        inline,
        block,
    )
}

/// A distance inside the content, in the subpixels an anchor is measured in.
/// Content laid out above or left of its own origin has no distance into the
/// content to report, so it reports none.
///
/// Reporting zero for it instead would be worse than reporting nothing. An
/// anchor is remembered as a distance and later re-measured, and the offset
/// moves by the difference; content clamped to zero on the way in would report
/// a difference smaller than the distance it travelled, and the reader would
/// land that much away from what they were looking at. Refusing the row costs
/// only its candidacy: the region anchors to another occurrence, or to none
/// and clamps, which is the answer it already has for content it cannot
/// measure.
fn content_subpixels(logical_points: f32) -> Option<i64> {
    let scaled = f64::from(logical_points)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    (scaled.is_finite() && scaled >= 0.0 && scaled <= i64::MAX as f64)
        .then(|| scaled.round() as i64)
}
