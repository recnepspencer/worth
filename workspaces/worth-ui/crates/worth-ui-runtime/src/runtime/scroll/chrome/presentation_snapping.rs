//! Snapping derived chrome to the device pixel grid at presentation.
//!
//! The accepted offset keeps its subpixel precision and hit testing keeps
//! reading it unsnapped. Presentation is the only place the grid appears: a
//! track edge and a moving thumb edge land on whole device pixels at the
//! current scale, so a hairline gutter and a six-point bar stay sharp while
//! content eases underneath them.
//!
//! Both edges of a rectangle move to the *nearest* grid line rather than
//! outward to enclosing ones. An enclosing snap would grow a moving thumb by up
//! to one device pixel on each side and shrink it again a frame later, which is
//! a visible length flicker during a settle; nearest-edge snapping moves the
//! thumb by whole pixels and keeps its length within one pixel of the derived
//! length for the whole travel.
//!
//! Nothing here feeds back into Scroll or Motion. A snapped rectangle is a
//! presentation derivation and is never the source of an offset.

/// The device scale a frame is presented at, in thousandths, as the host
/// appearance qualification names it. Admission is the only place a zero or
/// zero scale is refused, so the snap itself cannot divide by zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChromeDeviceScale {
    device_scale_milli: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeSnappingDenial {
    /// A zero device scale names no grid at all.
    ZeroDeviceScale,
    /// The snapped rectangle left the range canonical mounted geometry admits.
    RectangleInadmissible,
}

impl UiScrollChromeDeviceScale {
    /// The scale a frame presented at 1x is qualified with.
    pub(crate) const UNSCALED_MILLI: u32 = 1_000;

    pub(crate) const fn admit(
        device_scale_milli: u32,
    ) -> Result<Self, UiScrollChromeSnappingDenial> {
        if device_scale_milli == 0 {
            return Err(UiScrollChromeSnappingDenial::ZeroDeviceScale);
        }
        Ok(Self { device_scale_milli })
    }

    /// Device pixels per logical point.
    fn pixels_per_point(self) -> f64 {
        f64::from(self.device_scale_milli) / f64::from(Self::UNSCALED_MILLI)
    }

    /// The nearest grid line to one logical coordinate, back in logical points.
    fn snap_edge(self, logical_points: f32) -> f64 {
        let pixels_per_point = self.pixels_per_point();
        (f64::from(logical_points) * pixels_per_point).round() / pixels_per_point
    }
}

/// One derived chrome rectangle, placed on the device grid for painting.
///
/// The extent is snapped from the rectangle's own edges rather than from its
/// origin plus its extent, so a track and the thumb inside it stay aligned: two
/// edges that coincided before the snap still coincide after it.
pub(crate) fn snap_to_device_grid(
    rect: worth_ui_host_contract::UiMountedCanonicalBox,
    scale: UiScrollChromeDeviceScale,
) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiScrollChromeSnappingDenial> {
    let left = scale.snap_edge(rect.x());
    let top = scale.snap_edge(rect.y());
    let right = scale.snap_edge(rect.x() + rect.width());
    let bottom = scale.snap_edge(rect.y() + rect.height());
    // A rectangle thinner than one device pixel would snap to nothing. One
    // device pixel is the shortest thing the grid can show, so a present part
    // keeps one rather than disappearing.
    let minimum = 1.0 / scale.pixels_per_point();
    let width = (right - left).max(minimum);
    let height = (bottom - top).max(minimum);
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: left as f32,
            y: top as f32,
            width: width as f32,
            height: height as f32,
            coordinate_space: rect.coordinate_space(),
        },
    )
    .map_err(|_| UiScrollChromeSnappingDenial::RectangleInadmissible)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> worth_ui_host_contract::UiMountedCanonicalBox {
        worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
            worth_ui_host_contract::UiMountedCanonicalBoxInput {
                x,
                y,
                width,
                height,
                coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            },
        )
        .expect("canonical test rectangle")
    }

    #[test]
    fn a_zero_device_scale_names_no_grid() {
        assert_eq!(
            UiScrollChromeDeviceScale::admit(0),
            Err(UiScrollChromeSnappingDenial::ZeroDeviceScale)
        );
    }

    /// At 1x the grid is the logical point grid, so a fractional thumb start
    /// lands on a whole point and keeps its derived length.
    #[test]
    fn unscaled_snapping_moves_edges_to_whole_points() {
        let scale = UiScrollChromeDeviceScale::admit(1_000).expect("1x is admissible");
        let snapped = snap_to_device_grid(rect(388.0, 41.4, 6.0, 107.6), scale).expect("snapped");
        assert_eq!(snapped.x(), 388.0);
        assert_eq!(snapped.y(), 41.0);
        assert_eq!(snapped.width(), 6.0);
        assert_eq!(snapped.height(), 108.0);
    }

    /// At 2x a half-point edge is already on a device pixel and must not move.
    #[test]
    fn a_half_point_edge_is_already_on_the_two_times_grid() {
        let scale = UiScrollChromeDeviceScale::admit(2_000).expect("2x is admissible");
        let snapped = snap_to_device_grid(rect(10.5, 20.5, 6.0, 24.0), scale).expect("snapped");
        assert_eq!(snapped.x(), 10.5);
        assert_eq!(snapped.y(), 20.5);
        assert_eq!(snapped.width(), 6.0);
        assert_eq!(snapped.height(), 24.0);
    }

    /// Two edges that coincided before the snap still coincide after it, which
    /// is what keeps a thumb from drifting off the end of its own track.
    #[test]
    fn coincident_edges_stay_coincident_after_snapping() {
        let scale = UiScrollChromeDeviceScale::admit(1_500).expect("1.5x is admissible");
        let track = snap_to_device_grid(rect(100.3, 0.0, 12.0, 257.0), scale).expect("track");
        let thumb = snap_to_device_grid(rect(103.3, 0.0, 6.0, 24.7), scale).expect("thumb");
        assert_eq!(track.y(), thumb.y());
    }

    /// A part thinner than one device pixel is still a part. Snapping it to
    /// nothing would delete a hairline instead of sharpening it.
    #[test]
    fn a_subpixel_thin_part_keeps_one_device_pixel() {
        let scale = UiScrollChromeDeviceScale::admit(1_000).expect("1x is admissible");
        let snapped = snap_to_device_grid(rect(4.0, 4.0, 0.2, 24.0), scale).expect("snapped");
        assert_eq!(snapped.width(), 1.0);
    }
}
