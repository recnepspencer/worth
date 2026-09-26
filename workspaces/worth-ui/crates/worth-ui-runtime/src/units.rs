//! Host subpixels and the logical points they count.
//!
//! Every committed offset, extent, and platform position is counted in whole
//! host subpixels; layout measures in logical points. This module is the only
//! place the two meet, so every crossing rounds one way and refuses the same
//! values. Two readers that convert the same points agree on the subpixel,
//! and a subpixel read back as points converts back to itself -- in `f32`,
//! the precision layout measures in, for every count up to 16_384 points, past
//! which `f32` can no longer tell one thousandth of a point from the next.
//!
//! The host also counts wheel lines and pages at its subpixel scale. Those are
//! counts, not distances, so they cross here too, as thousandths.

use worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as PER_POINT;

/// A signed distance in host subpixels.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct UiSubpixels(i64);

impl UiSubpixels {
    pub(crate) const fn new(count: i64) -> Self {
        Self(count)
    }

    pub(crate) const fn count(self) -> i64 {
        self.0
    }

    /// `points` whole logical points. `None` past the representable range.
    pub(crate) const fn whole_points(points: i64) -> Option<Self> {
        match points.checked_mul(PER_POINT) {
            Some(count) => Some(Self(count)),
            None => None,
        }
    }

    /// The subpixel nearest `points`, halves rounded away from zero. `None`
    /// for a value that is not finite or lies past the representable range.
    pub(crate) fn nearest(points: impl Into<f64>) -> Option<Self> {
        let scaled = (points.into() * PER_POINT as f64).round();
        // `i64::MAX as f64` is 2^63 itself, one past the largest count.
        (scaled.is_finite() && scaled >= i64::MIN as f64 && scaled < i64::MAX as f64)
            .then_some(Self(scaled as i64))
    }

    /// The subpixel nearest a distance that cannot be negative: an extent, or
    /// how far something sits into content. A negative value is refused, never
    /// clamped: a reader that measures a difference between two distances
    /// would otherwise be handed one smaller than the distance traveled.
    pub(crate) fn nearest_distance(points: impl Into<f64>) -> Option<Self> {
        Self::nearest(points).filter(|distance| distance.0 >= 0)
    }

    /// This distance in logical points.
    pub(crate) fn to_points(self) -> f64 {
        self.0 as f64 / PER_POINT as f64
    }

    /// This distance in logical points, at the precision layout measures in.
    /// Past 16_384 points that precision is coarser than one subpixel, so the
    /// point read back may name a neighboring count.
    pub(crate) fn to_points_f32(self) -> f32 {
        self.to_points() as f32
    }

    /// `thousandths` thousandths of this distance, halves rounded away from
    /// zero. `None` past the representable range.
    pub(crate) fn thousandths(self, thousandths: i64) -> Option<Self> {
        i64::try_from(rounded_quotient(
            i128::from(self.0) * i128::from(thousandths),
            1_000,
        ))
        .ok()
        .map(Self)
    }
}

/// A host count of lines or pages, which the host encodes at its subpixel
/// scale, in thousandths of a line or page.
pub(crate) fn host_count_thousandths(host_units: i64) -> Option<i64> {
    i64::try_from(rounded_quotient(
        i128::from(host_units) * 1_000,
        i128::from(PER_POINT),
    ))
    .ok()
}

/// `count` whole host units -- points of a position or pixel delta, or lines
/// or pages -- as the host encodes them.
#[cfg(test)]
pub(crate) const fn host_count_of(count: i64) -> i64 {
    count * PER_POINT
}

/// A viewport position `x`, `y` logical points from the origin, encoded the
/// way a host reports it.
#[cfg(test)]
pub(crate) fn viewport_position_for_test(
    x: impl Into<f64>,
    y: impl Into<f64>,
) -> worth_ui_host_contract::UiHostSurfacePosition {
    let at = |points: f64| {
        UiSubpixels::nearest(points)
            .expect("a representable position")
            .count()
    };
    worth_ui_host_contract::UiHostSurfacePosition::viewport_logical(at(x.into()), at(y.into()))
}

/// `dividend / divisor`, halves rounded away from zero. `divisor` is positive.
const fn rounded_quotient(dividend: i128, divisor: i128) -> i128 {
    let half = divisor / 2;
    if dividend < 0 {
        (dividend - half) / divisor
    } else {
        (dividend + half) / divisor
    }
}

#[cfg(test)]
#[path = "units_tests.rs"]
mod tests;
