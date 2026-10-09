use std::{
    alloc::System,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker, Allocator,
};

#[global_allocator]
static ALLOCATOR: Allocator<System> = Allocator::system();

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
/// The one scoped group (0 before any is registered) and its live bytes.
static SCOPED_GROUP: AtomicUsize = AtomicUsize::new(0);
static SCOPED_LIVE: AtomicUsize = AtomicUsize::new(0);

struct ProcessHeapTracker;

fn scoped(group: &AllocationGroupId) -> bool {
    group.as_usize().get() == SCOPED_GROUP.load(Ordering::Acquire)
}

impl AllocationTracker for ProcessHeapTracker {
    fn allocated(
        &self,
        _address: usize,
        _object_size: usize,
        wrapped_size: usize,
        group: AllocationGroupId,
    ) {
        let live = LIVE.fetch_add(wrapped_size, Ordering::AcqRel) + wrapped_size;
        PEAK.fetch_max(live, Ordering::AcqRel);
        if scoped(&group) {
            SCOPED_LIVE.fetch_add(wrapped_size, Ordering::AcqRel);
        }
    }

    fn deallocated(
        &self,
        _address: usize,
        _object_size: usize,
        wrapped_size: usize,
        source_group: AllocationGroupId,
        _current_group: AllocationGroupId,
    ) {
        LIVE.fetch_sub(wrapped_size, Ordering::AcqRel);
        if scoped(&source_group) {
            SCOPED_LIVE.fetch_sub(wrapped_size, Ordering::AcqRel);
        }
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

/// Runs `operation` with its calling thread's allocations attributed to the
/// child's one scoped group. Other threads' allocations are never counted;
/// scoped bytes freed anywhere are. Call inside `peak_live_bytes_during`.
pub(super) fn attribute_allocations<T>(operation: impl FnOnce() -> T) -> T {
    let mut token = AllocationGroupToken::register().expect("one scoped allocation group");
    let previous = SCOPED_GROUP.swap(token.id().as_usize().get(), Ordering::AcqRel);
    assert_eq!(
        previous, 0,
        "one scoped allocation group per C11 child process"
    );
    let guard = token.enter();
    let result = operation();
    // Dropping the guard exits once; this crate's `exit()` would pop twice.
    drop(guard);
    result
}

/// Bytes the scoped operation allocated that are still live now.
pub(super) fn attributed_live_bytes() -> usize {
    SCOPED_LIVE.load(Ordering::Acquire)
}
