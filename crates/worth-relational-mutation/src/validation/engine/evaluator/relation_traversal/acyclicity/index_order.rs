//! In-place index ordering with immediate request stops and no sorting scratch.
use crate::validation::engine::context::InvariantExecutionContext;

/// Heap ordering charges each comparison and swap before performing it.
/// O(N log N) work, O(1) additional storage, including hostile input order.
pub(super) fn order_index<T: Ord>(
    values: &mut [T],
    context: &InvariantExecutionContext<'_, '_>,
) -> bool {
    for root in (0..values.len() / 2).rev() {
        if !sift_down(values, root, values.len(), context) {
            return false;
        }
    }
    for end in (1..values.len()).rev() {
        if !context.checkpoint(1) {
            return false;
        }
        values.swap(0, end);
        if !sift_down(values, 0, end, context) {
            return false;
        }
    }
    context.checkpoint(0)
}

fn sift_down<T: Ord>(
    values: &mut [T],
    mut root: usize,
    end: usize,
    context: &InvariantExecutionContext<'_, '_>,
) -> bool {
    while root < end / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < end {
            if !context.checkpoint(1) {
                return false;
            }
            if values[child] < values[child + 1] {
                child += 1;
            }
        }
        if !context.checkpoint(1) {
            return false;
        }
        if values[root] >= values[child] {
            break;
        }
        if !context.checkpoint(1) {
            return false;
        }
        values.swap(root, child);
        root = child;
    }
    true
}

pub(super) fn lower_bound<T>(
    values: &[T],
    context: &InvariantExecutionContext<'_, '_>,
    before: impl Fn(&T) -> bool,
) -> Option<usize> {
    let (mut low, mut high) = (0, values.len());
    while low < high {
        if !context.checkpoint(1) {
            return None;
        }
        let middle = low + (high - low) / 2;
        if before(&values[middle]) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Some(low)
}
