//! How far the device grid moves a scrolled occurrence's painted box from the
//! box on record.
//!
//! Paint reads the box moved onto the grid, and hit testing reads the box on
//! record, where the accepted offset put it. The correction is the one value
//! that relates the two, so it owns both moves and neither reader handles its
//! components.

use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedCanonicalBoxInput};

use super::UiMountedOccurrenceGeometryDenial;

/// The logical points the device grid moves one occurrence's painted box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiDeviceGridCorrection {
    inline: f32,
    block: f32,
}

impl UiDeviceGridCorrection {
    /// The correction the grid residues of every scrolled ancestor add up
    /// to, or nothing when the box is already on the grid.
    pub(super) fn from_residues(inline: f64, block: f64) -> Option<Self> {
        (inline != 0.0 || block != 0.0).then_some(Self {
            inline: inline as f32,
            block: block as f32,
        })
    }

    /// The box on record, moved onto the grid where it is painted.
    pub(crate) fn onto_grid(
        self,
        bounds: UiMountedCanonicalBox,
    ) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
        shift(bounds, self.inline, self.block)
    }

    /// The box on record under a box painted on the grid.
    pub(crate) fn off_grid(
        self,
        painted: UiMountedCanonicalBox,
    ) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
        shift(painted, -self.inline, -self.block)
    }
}

fn shift(
    bounds: UiMountedCanonicalBox,
    inline: f32,
    block: f32,
) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x() + inline,
        y: bounds.y() + block,
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| UiMountedOccurrenceGeometryDenial::ParentCoordinateSpaceMismatch)
}
