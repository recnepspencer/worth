use std::{
    alloc::System,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracking_allocator::{AllocationGroupId, AllocationRegistry, AllocationTracker, Allocator};

#[global_allocator]
static ALLOCATOR: Allocator<System> = Allocator::system();

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct ProcessHeapTracker;

impl AllocationTracker for ProcessHeapTracker {
    fn allocated(
        &self,
        _address: usize,
        _object_size: usize,
        wrapped_size: usize,
        _group: AllocationGroupId,
    ) {
        let live = LIVE.fetch_add(wrapped_size, Ordering::AcqRel) + wrapped_size;
        PEAK.fetch_max(live, Ordering::AcqRel);
    }

    fn deallocated(
        &self,
        _address: usize,
        _object_size: usize,
        wrapped_size: usize,
        _source_group: AllocationGroupId,
        _current_group: AllocationGroupId,
    ) {
        LIVE.fetch_sub(wrapped_size, Ordering::AcqRel);
    }
}

/// Allocation-event high-water of all threads in this isolated child process.
/// Tracking begins after Store bootstrap and captures even short-lived buffers.
/// Each child invokes this once, so no earlier tracked allocations are omitted.
pub(super) fn peak_live_bytes_during<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    AllocationRegistry::set_global_tracker(ProcessHeapTracker)
        .expect("one allocator probe per C11 child process");
    AllocationRegistry::enable_tracking();
    let result = operation();
    AllocationRegistry::disable_tracking();
    (result, PEAK.load(Ordering::Acquire))
}
