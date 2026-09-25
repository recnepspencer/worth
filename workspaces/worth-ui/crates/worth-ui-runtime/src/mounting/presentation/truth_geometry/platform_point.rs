use worth_ui_host_contract::{
    UiHostSurfaceCoordinateSpace, UiHostSurfaceCoordinateUnit, UiHostSurfacePosition,
    UiHostSurfacePositionBasis, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// Where a platform event puts the pointer, in viewport logical points: the
/// platform-event edge. Only a host position mints one, so hit testing,
/// pointer presence, Scroll chrome, and Portal dismissal all read the same
/// point from the same report.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct UiPlatformPoint(UiHostSurfacePosition);

impl UiPlatformPoint {
    /// The point a host position reports. A position on any basis other than
    /// viewport logical points names no point here, and is refused with its
    /// basis.
    pub(crate) fn from_host_position(
        position: UiHostSurfacePosition,
    ) -> Result<Self, UiHostSurfacePositionBasis> {
        let basis = position.basis();
        if basis.coordinate_space() != UiHostSurfaceCoordinateSpace::Viewport
            || basis.coordinate_unit() != UiHostSurfaceCoordinateUnit::LogicalPoint
        {
            return Err(basis);
        }
        Ok(Self(position))
    }

    /// The inline coordinate, in logical points.
    pub(crate) fn x(self) -> f32 {
        logical_points(self.0.x_subpixels())
    }

    /// The block coordinate, in logical points.
    pub(crate) fn y(self) -> f32 {
        logical_points(self.0.y_subpixels())
    }

    /// The point the spatial index is queried at: the acceleration edge,
    /// whose candidates are each still tested against this point.
    pub(crate) fn index_point(self) -> [f64; 2] {
        [f64::from(self.x()), f64::from(self.y())]
    }
}

fn logical_points(subpixels: i64) -> f32 {
    (subpixels as f64 / UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64) as f32
}

/// A platform point at `x`, `y` logical points, minted from the host position
/// a platform event would report there.
#[cfg(test)]
pub(crate) fn platform_point_for_test(x: f32, y: f32) -> UiPlatformPoint {
    let subpixels = |value: f32| {
        (f64::from(value) * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64).round()
    };
    UiPlatformPoint::from_host_position(UiHostSurfacePosition::viewport_logical(
        subpixels(x) as i64,
        subpixels(y) as i64,
    ))
    .expect("a viewport logical position names a platform point")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_viewport_logical_position_names_a_platform_point() {
        let physical = UiHostSurfacePositionBasis::new(
            UiHostSurfaceCoordinateSpace::Viewport,
            UiHostSurfaceCoordinateUnit::PhysicalPixel,
        );
        assert_eq!(
            UiPlatformPoint::from_host_position(UiHostSurfacePosition::new(physical, 1, 1)),
            Err(physical)
        );
        let point = UiPlatformPoint::from_host_position(UiHostSurfacePosition::viewport_logical(
            760_500, 41_250,
        ))
        .unwrap();
        assert_eq!([point.x(), point.y()], [760.5, 41.25]);
        assert_eq!(point.index_point(), [760.5, 41.25]);
    }
}
