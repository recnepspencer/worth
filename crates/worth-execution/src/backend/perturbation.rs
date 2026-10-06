use crate::authority::ExecutionResourceLease;

use super::native;

pub(super) fn run(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
    seed: u64,
    execute: &(impl Fn(usize) + Sync),
) -> usize {
    let mut order: Vec<_> = (0..count).collect();
    let mut state = seed;
    for end in (1..count).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let other = (state as usize) % (end + 1);
        order.swap(end, other);
    }
    native::run_order(lease, &order, execute)
}
