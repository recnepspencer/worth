//! Exact centroid Morton order groups both axes before raw-bound/occurrence ties.
use super::Box2;
use std::cmp::Ordering;
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

pub(super) fn key<Source: Copy>(bounds: Box2, source: Source) -> (Morton, [i64; 4], Source) {
    let raw = bounds.coordinates();
    let center = std::array::from_fn(|axis| {
        // An i64 endpoint sum is in [-2^64, 2^64-2]. The biased exact sum
        // fits 65 unsigned bits; neither averaging nor rounding is performed.
        let biased = i128::from(raw.lower[axis]) + i128::from(raw.upper[axis]) + (1_i128 << 64);
        u128::try_from(biased).expect("biased i64 endpoint sum fits unsigned65 bits")
    });
    (
        Morton(center),
        [raw.lower[0], raw.lower[1], raw.upper[0], raw.upper[1]],
        source,
    )
}
