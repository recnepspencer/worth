//! Exact bounded integer orientation; callers preserve their common unit/frame.
use std::cmp::Ordering;

mod sealed {
    pub trait Coordinate {}
    impl Coordinate for i64 {}
    impl Coordinate for u64 {}
}

/// These two coordinate classes bound every difference by u64::MAX in magnitude.
/// Sealing prevents a wider integer class from invalidating that arithmetic proof.
pub trait PlanarInteger: sealed::Coordinate + Copy + Into<i128> {}
impl PlanarInteger for i64 {}
impl PlanarInteger for u64 {}

/// Sign of (b-a) cross (c-a), including exact zero and full-width cancellation.
/// All points must use the same unit/frame. This creates no geometric identity,
/// topology, access permission or publication evidence.
pub fn integer_orientation<C: PlanarInteger>(a: [C; 2], b: [C; 2], c: [C; 2]) -> Ordering {
    let a = a.map(Into::into);
    let delta = |point: [C; 2]| [point[0].into() - a[0], point[1].into() - a[1]];
    let b = delta(b);
    let c = delta(c);
    let first = signed_product(b[0], c[1]);
    let second = signed_product(b[1], c[0]);
    match (first.0, second.0) {
        (false, true) => Ordering::Greater,
        (true, false) => Ordering::Less,
        (false, false) => first.1.cmp(&second.1),
        (true, true) => second.1.cmp(&first.1),
    }
}

fn signed_product(a: i128, b: i128) -> (bool, u128) {
    // Differences of either supported class have magnitude <= u64::MAX, so
    // their product fits u128. Comparing signed products needs no signed sum.
    let magnitude = a.unsigned_abs() * b.unsigned_abs();
    (
        magnitude != 0 && (a.is_negative() != b.is_negative()),
        magnitude,
    )
}

#[cfg(test)]
mod tests;
