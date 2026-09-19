//! Physical placement of an already rasterized glyph image.

use super::{UiGlyphRasterBearing, UiGlyphRasterExtent};

impl UiGlyphRasterBearing {
    /// Returns the image's physical `[left, top, width, height]` before clipping.
    ///
    /// The actual raster bearing and extent must belong to the image rasterized
    /// for this origin and scale. Rasterization already includes the signed
    /// fractional phase; only the integral physical origin is added here.
    /// Truncation toward zero matches the raster key's signed remainder basis.
    /// This rectangle describes the image, not its nonzero-alpha coverage.
    pub fn positioned_bounds(
        self,
        extent: UiGlyphRasterExtent,
        origin_millipoints: [i64; 2],
        dpi_milli: u32,
    ) -> Option<[f32; 4]> {
        if dpi_milli == 0 {
            return None;
        }
        let integral = |origin: i64| {
            i128::from(origin)
                .checked_mul(i128::from(dpi_milli))?
                .checked_div(1_000_000)?
                .checked_mul(64)
        };
        let left = integral(origin_millipoints[0])?.checked_add(i128::from(self.x_over_64()))?;
        let top = integral(origin_millipoints[1])?.checked_sub(i128::from(self.y_over_64()))?;
        let bounds = [
            (left as f64 / 64.0) as f32,
            (top as f64 / 64.0) as f32,
            extent.width() as f32,
            extent.height() as f32,
        ];
        bounds
            .iter()
            .all(|value| value.is_finite())
            .then_some(bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_image_placement_uses_signed_integral_origins_at_each_scale() {
        let bearing = UiGlyphRasterBearing::from_sixty_fourths(-64, 192);
        let extent = UiGlyphRasterExtent::new(8, 12).unwrap();
        for (origin, dpi, expected) in [
            ([1_999, 2_001], 1_000, [0.0, -1.0, 8.0, 12.0]),
            ([-1_999, -2_001], 1_000, [-2.0, -5.0, 8.0, 12.0]),
            ([1_999, -2_001], 1_250, [1.0, -5.0, 8.0, 12.0]),
            ([-1_999, 2_001], 1_500, [-3.0, 0.0, 8.0, 12.0]),
            ([1_999, -2_001], 2_000, [2.0, -7.0, 8.0, 12.0]),
            ([999, -999], 1_000, [-1.0, -3.0, 8.0, 12.0]),
        ] {
            assert_eq!(
                bearing.positioned_bounds(extent, origin, dpi),
                Some(expected)
            );
        }
        assert_eq!(bearing.positioned_bounds(extent, [0, 0], 0), None);
    }
}
