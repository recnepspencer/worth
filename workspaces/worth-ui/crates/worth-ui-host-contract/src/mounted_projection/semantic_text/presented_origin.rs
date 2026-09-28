//! Where a text command's glyphs are drawn from: its mounted origin on the
//! device pixel grid.
//!
//! Layout places a text command at any fraction of a device pixel, and every
//! glyph's raster key names the fraction its ink starts at. Drawing from the
//! exact origin would give a command's glyphs fresh keys whenever layout moved
//! it by a fraction of a pixel -- during a resize, that is most frames -- and a
//! frame with fresh keys waits for an atlas upload before it can present. So
//! the command moves bodily to the nearest device pixel, the way a moving
//! scroll's content does: each glyph keeps the phase shaping gave it inside the
//! run, and only the command's own fraction is dropped. A command that layout
//! moves by a fraction of a pixel keeps the raster keys it already has.
//!
//! This is a presentation derivation. Hit testing, clipping, and damage read
//! the mounted origin, and nothing here feeds back into layout.

use super::UiMountedSemanticTextMechanic;

impl UiMountedSemanticTextMechanic {
    /// The origin this command's glyphs are drawn from, in millipoints, at
    /// `dpi_milli` thousandths of a device pixel per point. `None` when the
    /// scale is zero or the origin has no millipoint value.
    pub fn presented_origin_millipoints(&self, dpi_milli: u32) -> Option<[i64; 2]> {
        Some([
            device_grid_millipoints(self.origin_x, dpi_milli)?,
            device_grid_millipoints(self.origin_y, dpi_milli)?,
        ])
    }
}

fn device_grid_millipoints(points: f32, dpi_milli: u32) -> Option<i64> {
    if dpi_milli == 0 || !points.is_finite() {
        return None;
    }
    let pixels_per_point = f64::from(dpi_milli) / 1_000.0;
    let pixels = (f64::from(points) * pixels_per_point).round();
    let millipoints = (pixels / pixels_per_point * 1_000.0).round();
    if millipoints < i64::MIN as f64 || millipoints > i64::MAX as f64 {
        return None;
    }
    Some(millipoints as i64)
}

#[cfg(test)]
mod tests {
    use super::device_grid_millipoints;

    #[test]
    fn origins_within_one_device_pixel_present_at_the_same_place() {
        // 1.5 device pixels per point: 10.1 and 10.2 points are 15.15 and
        // 15.3 pixels, both nearest pixel 15, which is 10 points.
        assert_eq!(device_grid_millipoints(10.1, 1_500), Some(10_000));
        assert_eq!(device_grid_millipoints(10.2, 1_500), Some(10_000));
        // 10.4 points is 15.6 pixels, nearest pixel 16.
        assert_eq!(device_grid_millipoints(10.4, 1_500), Some(10_667));
    }

    #[test]
    fn at_one_pixel_per_point_the_grid_is_whole_points() {
        assert_eq!(device_grid_millipoints(236.49, 1_000), Some(236_000));
        assert_eq!(device_grid_millipoints(236.51, 1_000), Some(237_000));
        assert_eq!(device_grid_millipoints(-3.6, 1_000), Some(-4_000));
    }

    #[test]
    fn the_presented_origin_is_within_half_a_device_pixel_of_the_mounted_one() {
        for dpi_milli in [1_000_u32, 1_250, 1_500, 1_750, 2_000] {
            let half_pixel = 500.0 / (f64::from(dpi_milli) / 1_000.0);
            for tenth in 0..400 {
                let points = tenth as f32 * 0.37;
                let presented = device_grid_millipoints(points, dpi_milli).unwrap();
                let mounted = f64::from(points) * 1_000.0;
                assert!(
                    (presented as f64 - mounted).abs() <= half_pixel + 0.5,
                    "{points} at {dpi_milli}"
                );
            }
        }
    }

    #[test]
    fn no_scale_or_no_finite_origin_names_no_grid_position() {
        assert_eq!(device_grid_millipoints(10.0, 0), None);
        assert_eq!(device_grid_millipoints(f32::NAN, 1_500), None);
        assert_eq!(device_grid_millipoints(f32::INFINITY, 1_500), None);
    }
}
