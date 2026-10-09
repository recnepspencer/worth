//! Actual entries to the ordinary unretained region owner.
use std::sync::atomic::{AtomicUsize, Ordering};
static CALLS: [AtomicUsize; 4] = [const { AtomicUsize::new(0) }; 4];
pub(super) fn record(kind: usize) {
    CALLS[kind].fetch_add(1, Ordering::Relaxed);
}
pub(in super::super) fn take() -> [usize; 4] {
    std::array::from_fn(|kind| CALLS[kind].swap(0, Ordering::Relaxed))
}
