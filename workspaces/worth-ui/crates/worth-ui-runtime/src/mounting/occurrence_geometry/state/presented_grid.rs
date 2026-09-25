//! Where a scrolled occurrence's pixels land, as against where its offset is.
//!
//! An offset keeps its subpixel precision, because that precision is what makes
//! a settle ease rather than step, and it is what hit testing measures against.
//! Pixels have no such freedom. A glyph run, a one-point outline or a border
//! drawn a third of a device pixel off the grid is resampled across two rows of
//! pixels and reads as blur, and during a settle the amount of blur changes
//! every frame, which is worse than blur: it is shimmer.
//!
//! So the grid appears once, on the read that paints. The offset each scrolled
//! occurrence traveled is rounded to whole device pixels and the fraction it
//! was carrying is added back to the box that offset moved, which puts the
//! whole box back on the phase layout gave it. Every command the occurrence
//! lowers to takes that one correction from that one accepted sample, so
//! nothing inside it shifts relative to anything else.
//!
//! The walk collects every scrolled ancestor rather than the nearest one,
//! because a region inside a region moves its content by both offsets and a box
//! corrected for one of them would still be off the grid by the other. Clip
//! rectangles are left alone: a clip is the edge of the region itself, which is
//! at rest, and a region whose edge moved every time the content behind it
//! rounded would open and close a hairline at its own boundary.
//!
//! Nothing here is written down. The correction is derived on the read, so the
//! stored box, the bounds Scroll derives from it and the rows hit testing
//! reads all stay exactly where the accepted offset put them.

use super::*;

impl UiMountedOccurrenceGeometryState {
    /// How far one occurrence's painted box moves to reach the device grid, in
    /// logical points, or nothing when it is already on it.
    ///
    /// Nothing is the common answer and the cheap one: a surface with no
    /// applied pose has nothing displaced, and an offset that is a whole number
    /// of device pixels carries no fraction to give back.
    ///
    /// The walk is over ancestors a completed surface has already been checked
    /// to have, so a missing one is a broken occurrence tree rather than an
    /// occurrence with no scrolled ancestor, and is refused as such instead of
    /// reported as a box that happens to need no correction.
    pub(super) fn presented_scroll_grid_correction(
        &self,
        surface: UiSemanticSurfaceIdentity,
        instance: UiMountedInstanceIdentity,
    ) -> Result<Option<(f32, f32)>, UiMountedOccurrenceGeometryDenial> {
        let geometry = self
            .surfaces
            .get(&surface)
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?;
        if geometry.scroll_poses.is_empty() {
            return Ok(None);
        }
        let scale = geometry.device_scale;
        let subpixels = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
        let mut inline = 0.0;
        let mut block = 0.0;
        let mut cursor = self.parent_of(geometry, instance)?;
        while let Some(ancestor) = cursor {
            if let Some(offset) = geometry.scroll_poses.get(&ancestor) {
                inline += scale.grid_residue(offset.inline_subpixels() as f64 / subpixels);
                block += scale.grid_residue(offset.block_subpixels() as f64 / subpixels);
            }
            cursor = self.parent_of(geometry, ancestor)?;
        }
        Ok((inline != 0.0 || block != 0.0).then_some((inline as f32, block as f32)))
    }

    /// The occurrence one occurrence was laid out inside, on a surface that has
    /// already reported both of them.
    fn parent_of(
        &self,
        geometry: &UiMountedSurfaceGeometry,
        instance: UiMountedInstanceIdentity,
    ) -> Result<Option<UiMountedInstanceIdentity>, UiMountedOccurrenceGeometryDenial> {
        geometry
            .occurrences
            .get(&instance)
            .map(|row| row.parent)
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry)
    }
}
