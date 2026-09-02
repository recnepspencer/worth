use super::geometry::{pixel_center, UiNativePhysicalRect, PHYSICAL_MICROS_PER_PIXEL};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeAnalyticCoverage(u16);

impl UiNativeAnalyticCoverage {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const ONE: Self = Self(u16::MAX);

    pub(crate) const fn units(self) -> u16 {
        self.0
    }

    pub(crate) fn subtract(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

pub(crate) fn rounded_signed_distance(
    pixel_x: i64,
    pixel_y: i64,
    rect: UiNativePhysicalRect,
    radii: [i64; 4],
) -> i64 {
    let [point_x, point_y] = pixel_center(pixel_x, pixel_y)
        .expect("qualified physical pixel coordinates fit the analytic sample basis");
    rounded_signed_distance_at([point_x, point_y], rect, radii)
}

pub(crate) fn coverage_from_signed_distance(distance: i64) -> UiNativeAnalyticCoverage {
    let coverage_micros =
        (PHYSICAL_MICROS_PER_PIXEL / 2 - distance).clamp(0, PHYSICAL_MICROS_PER_PIXEL);
    let units = round_ratio_even(
        u128::from(coverage_micros as u64) * u128::from(u16::MAX),
        u128::from(PHYSICAL_MICROS_PER_PIXEL as u64),
    ) as u16;
    UiNativeAnalyticCoverage(units)
}

fn rounded_signed_distance_at(point: [i64; 2], rect: UiNativePhysicalRect, radii: [i64; 4]) -> i64 {
    let point_x = point[0] as f64;
    let point_y = point[1] as f64;
    let center_x = (rect.left as f64 + rect.right as f64) / 2.0;
    let center_y = (rect.top as f64 + rect.bottom as f64) / 2.0;
    let radius = match (point_x < center_x, point_y < center_y) {
        (true, true) => radii[0],
        (false, true) => radii[1],
        (false, false) => radii[2],
        (true, false) => radii[3],
    } as f64;
    let half_width = (rect.right - rect.left) as f64 / 2.0;
    let half_height = (rect.bottom - rect.top) as f64 / 2.0;
    let qx = (point_x - center_x).abs() - half_width + radius;
    let qy = (point_y - center_y).abs() - half_height + radius;
    let outside_x = qx.max(0.0);
    let outside_y = qy.max(0.0);
    let distance =
        (outside_x.mul_add(outside_x, outside_y * outside_y)).sqrt() + qx.max(qy).min(0.0) - radius;
    round_ties_even(distance)
}

fn round_ties_even(value: f64) -> i64 {
    let lower = value.floor();
    let fraction = value - lower;
    let rounded = if fraction < 0.5 {
        lower
    } else if fraction > 0.5 || (lower as i64) % 2 != 0 {
        lower + 1.0
    } else {
        lower
    };
    rounded as i64
}

fn round_ratio_even(numerator: u128, denominator: u128) -> u128 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let twice_remainder = remainder * 2;
    if twice_remainder > denominator || (twice_remainder == denominator && quotient % 2 == 1) {
        quotient + 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_center_rule_has_half_coverage_at_an_exact_edge() {
        let distance = 0;
        assert_eq!(coverage_from_signed_distance(distance).units(), 32_768);
    }

    #[test]
    fn rounded_distance_is_inside_at_the_center_and_outside_beyond_the_edge() {
        let rect = UiNativePhysicalRect {
            left: 0,
            top: 0,
            right: 10 * PHYSICAL_MICROS_PER_PIXEL,
            bottom: 10 * PHYSICAL_MICROS_PER_PIXEL,
        };
        let radii = [2 * PHYSICAL_MICROS_PER_PIXEL; 4];
        assert!(rounded_signed_distance(5, 5, rect, radii) < 0);
        assert!(rounded_signed_distance(12, 5, rect, radii) > 0);
    }
}
