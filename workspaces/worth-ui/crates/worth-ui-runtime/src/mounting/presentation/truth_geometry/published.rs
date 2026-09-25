use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedCoordinateSpace};

use super::logical_rect::{UiLogicalRect, UiTruthGeometryDenial};

/// A rectangle the runtime committed: mounted layout, an allocation, or the
/// target an owner declares for Motion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPublishedRect {
    rect: UiLogicalRect,
}

impl UiPublishedRect {
    /// The rectangle a committed mounted box stands for.
    pub(crate) fn from_committed_box(bounds: UiMountedCanonicalBox) -> Self {
        Self {
            rect: UiLogicalRect::from_box(bounds),
        }
    }

    /// A rectangle an owner computed for its committed target.
    pub(crate) fn from_committed_components(
        components: [f32; 4],
        coordinate_space: UiMountedCoordinateSpace,
    ) -> Result<Self, UiTruthGeometryDenial> {
        UiLogicalRect::new(components, coordinate_space).map(|rect| Self { rect })
    }

    pub(super) const fn from_rect(rect: UiLogicalRect) -> Self {
        Self { rect }
    }

    pub(super) const fn rect(self) -> UiLogicalRect {
        self.rect
    }

    /// This committed rect moved a committed, finite distance: arithmetic
    /// that stays within published truth.
    pub(crate) fn translated(self, by: [f32; 2]) -> Self {
        Self::from_rect(self.rect.shifted(by))
    }

    /// The raw components: for the serialization and GPU-upload edges, for
    /// the damage regions handed to the host, and for owners doing committed
    /// arithmetic on published values they own. Motion reads them against an
    /// accepted sample only inside `UiTrackCurveSpan`.
    pub(crate) fn components(self) -> [f32; 4] {
        self.rect.components()
    }

    /// Whether the point a platform event reports lands in this committed
    /// rect: the platform-event edge.
    pub(crate) fn admits_platform_point(self, point: [f32; 2]) -> bool {
        self.rect.admits(point)
    }

    pub(crate) const fn coordinate_space(self) -> UiMountedCoordinateSpace {
        self.rect.coordinate_space()
    }

    /// Whether this rect encloses area, rather than a line or a point.
    pub(crate) fn has_area(self) -> bool {
        self.rect.canonical_box().posture()
            == worth_ui_host_contract::UiMountedGeometryPosture::Area
    }

    pub(crate) fn canonical_box(self) -> UiMountedCanonicalBox {
        self.rect.canonical_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_rect_is_minted_only_after_finite_non_negative_validation() {
        assert_eq!(
            UiPublishedRect::from_committed_components(
                [0.0, 0.0, -1.0, 1.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiTruthGeometryDenial::NegativeExtent)
        );
        assert_eq!(
            UiPublishedRect::from_committed_components(
                [f32::NAN, 0.0, 1.0, 1.0],
                UiMountedCoordinateSpace::HostSurface,
            ),
            Err(UiTruthGeometryDenial::NonFinite)
        );
        let rect = UiPublishedRect::from_committed_components(
            [1.0, 2.0, 3.0, 4.0],
            UiMountedCoordinateSpace::Viewport,
        )
        .unwrap();
        assert_eq!(rect.components(), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(rect.coordinate_space(), UiMountedCoordinateSpace::Viewport);
    }

    #[test]
    fn a_zero_reached_from_below_equals_zero() {
        let rect = |x| {
            UiPublishedRect::from_committed_components(
                [x, 0.0, 1.0, 1.0],
                UiMountedCoordinateSpace::Viewport,
            )
            .unwrap()
        };
        assert_eq!(rect(-0.0), rect(0.0));
        assert_eq!(rect(-0.0).components()[0].to_bits(), 0.0f32.to_bits());
    }
}
