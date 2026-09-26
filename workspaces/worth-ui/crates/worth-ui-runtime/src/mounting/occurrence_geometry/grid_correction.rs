//! How far the device grid moves a scrolled occurrence's painted box from the
//! box on record.
//!
//! Paint reads the box moved onto the grid, and hit testing reads the box on
//! record, where the accepted offset put it. The correction only ever moves
//! the box on record, so the box hit testing reads is never derived back out
//! of the painted one.

use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedCanonicalBoxInput};

use super::UiMountedOccurrenceGeometryDenial;

/// The logical points the device grid moves one occurrence's painted box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct UiDeviceGridCorrection {
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
    pub(super) fn onto_grid(
        self,
        bounds: UiMountedCanonicalBox,
    ) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: bounds.x() + self.inline,
            y: bounds.y() + self.block,
            width: bounds.width(),
            height: bounds.height(),
            coordinate_space: bounds.coordinate_space(),
        })
        .map_err(UiMountedOccurrenceGeometryDenial::UnrepresentableBox)
    }
}

#[cfg(test)]
mod tests {
    use worth_ui_host_contract::{UiMountedCoordinateSpace, UiMountedGeometryDenial};

    use super::*;

    #[test]
    fn a_box_moved_past_the_representable_range_is_refused_with_its_cause() {
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: f32::MAX,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap();
        let correction = UiDeviceGridCorrection::from_residues(f64::from(f32::MAX), 0.0)
            .expect("a nonzero residue corrects the box");
        assert!(matches!(
            correction.onto_grid(bounds),
            Err(UiMountedOccurrenceGeometryDenial::UnrepresentableBox(
                UiMountedGeometryDenial::NonFinite
            ))
        ));
    }
}
