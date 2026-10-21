//! Complete conservative bounds for each immutable call-sequence view.
use super::*;
pub(super) fn shape_bound(calls: usize) -> Option<u64> {
    (size_of::<CallShape>() as u64)
        .checked_add(2 * size_of::<usize>() as u64)?
        .checked_add((calls as u64).checked_mul(
            (11 * size_of::<(ComputationRead, usize)>() + 16 * size_of::<usize>()) as u64,
        )?)
}
