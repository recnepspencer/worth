//! The one crossing between logical points and appearance logical subpixels.
//!
//! Appearance records count subpixels; allocation, paint and hit geometry
//! measure in points. Every crossing goes through here, so a host painting a
//! record and a runtime hit-testing it read one conversion rather than two
//! copies that may round differently.

use super::logical_length::UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT;

/// `points` as the nearest signed subpixel count, halves to even. `None` for
/// points no `i32` count holds, including non-finite points.
pub fn appearance_coordinate_nearest(points: f32) -> Option<i32> {
    let subpixels = nearest(points);
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&subpixels)
        .then_some(subpixels as i32)
}

/// `points` as the nearest subpixel extent, halves to even. `None` for a
/// negative extent or one no `u32` count holds, including non-finite points.
pub fn appearance_extent_nearest(points: f32) -> Option<u32> {
    let subpixels = nearest(points);
    (0.0..=f64::from(u32::MAX))
        .contains(&subpixels)
        .then_some(subpixels as u32)
}

/// `subpixels` in points. Exact before narrowing for every count an
/// appearance record holds, so an edge sum (`x + width`) is one rounding.
pub fn appearance_points(subpixels: i64) -> f64 {
    subpixels as f64 / f64::from(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT)
}

/// `subpixels` in `f32` points: one rounding from the exact quotient.
pub fn appearance_points_f32(subpixels: i64) -> f32 {
    appearance_points(subpixels) as f32
}

/// Widen before multiplying so the source `f32` is not rounded again.
fn nearest(points: f32) -> f64 {
    (f64::from(points) * f64::from(UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT)).round_ties_even()
}

#[cfg(test)]
#[path = "logical_points_tests.rs"]
mod tests;
