//! Exact centroid Morton order groups both axes before raw-bound/occurrence ties.
use super::{BoxSpace, IntegerBox2};
#[cfg(test)]
use crate::Box2;
use crate::PlanarInteger;
use std::cmp::Ordering;
#[cfg(test)]
mod model_local_tests;
#[cfg(test)]
mod tests;

#[derive(Eq, PartialEq)]
pub(super) struct Morton([u128; 2]);
impl Ord for Morton {
    fn cmp(&self, other: &Self) -> Ordering {
        let x = (self.0[0] ^ other.0[0]).leading_zeros();
        let y = (self.0[1] ^ other.0[1]).leading_zeros();
        // The highest differing interleaved bit decides; x precedes y on ties.
        let axis = usize::from(y < x);
        self.0[axis].cmp(&other.0[axis])
    }
}
impl PartialOrd for Morton {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(super) fn key<Source: Copy, Space: BoxSpace>(
    bounds: IntegerBox2<Space>,
    source: Source,
) -> (Morton, [Space::Coordinate; 4], Source) {
    let raw = bounds.coordinates();
    let center = std::array::from_fn(|axis| {
        // Normalize to the sealed coordinate minimum before exact endpoint sum.
        // Both domains occupy [0, 2^65-2]; signed reproduces the original bias.
        let normalized =
            raw.lower[axis].into() + raw.upper[axis].into() - 2 * Space::Coordinate::MIN.into();
        u128::try_from(normalized).expect("normalized endpoint sum fits unsigned65 bits")
    });
    (
        Morton(center),
        [raw.lower[0], raw.lower[1], raw.upper[0], raw.upper[1]],
        source,
    )
}

/// The same exact spatial order for immutable probes, before framework slot ties.
pub fn compare_box_bounds<Space: BoxSpace>(
    a: IntegerBox2<Space>,
    b: IntegerBox2<Space>,
) -> Ordering {
    key(a, ()).cmp(&key(b, ()))
}
