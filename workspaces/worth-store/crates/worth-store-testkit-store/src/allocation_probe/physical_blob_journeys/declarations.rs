pub use {std::alloc::System, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};

pub use {tracking_allocator::AllocationGroupId, tracking_allocator::AllocationGroupToken, tracking_allocator::AllocationRegistry, tracking_allocator::AllocationTracker, tracking_allocator::Allocator};

#[global_allocator]
pub static ALLOCATOR: Allocator<System> = tracking_allocator::Allocator::system();

pub static LIVE: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static PEAK: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// The one scoped group (0 before any is registered) and its live bytes.
pub static SCOPED_GROUP: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static SCOPED_LIVE: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct ProcessHeapTracker;

pub fn scoped(group: &AllocationGroupId) -> bool {
    group.as_usize().get() == SCOPED_GROUP.load(std::sync::atomic::Ordering::Acquire)
}

impl AllocationTracker for ProcessHeapTracker {
    fn allocated(
        &self,
        _address: usize,
        _object_size: usize,
        wrapped_size: usize,
        group: AllocationGroupId,
    ) {
        let live = LIVE.fetch_add(wrapped_size, std::sync::atomic::Ordering::AcqRel) + wrapped_size;
        PEAK.fetch_max(live, std::sync::atomic::Ordering::AcqRel);
        if scoped(&group) {
            SCOPED_LIVE.fetch_add(wrapped_size, std::sync::atomic::Ordering::AcqRel);
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
        LIVE.fetch_sub(wrapped_size, std::sync::atomic::Ordering::AcqRel);
        if scoped(&source_group) {
            SCOPED_LIVE.fetch_sub(wrapped_size, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

/// Allocation-event high-water of all threads in this isolated child process.
/// Tracking begins after Store bootstrap and captures even short-lived buffers.
/// Each child invokes this once, so no earlier tracked allocations are omitted.
pub fn peak_live_bytes_during<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    tracking_allocator::AllocationRegistry::set_global_tracker(ProcessHeapTracker)
        .expect("one allocator probe per C11 child process");
    tracking_allocator::AllocationRegistry::enable_tracking();
    let result = operation();
    tracking_allocator::AllocationRegistry::disable_tracking();
    (result, PEAK.load(std::sync::atomic::Ordering::Acquire))
}

/// Runs `operation` with its calling thread's allocations attributed to the
/// child's one scoped group. Other threads' allocations are never counted;
/// scoped bytes freed anywhere are. Call inside `peak_live_bytes_during`.
pub fn attribute_allocations<T>(operation: impl FnOnce() -> T) -> T {
    let mut token = tracking_allocator::AllocationGroupToken::register().expect("one scoped allocation group");
    let previous = SCOPED_GROUP.swap(token.id().as_usize().get(), std::sync::atomic::Ordering::AcqRel);
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
pub fn attributed_live_bytes() -> usize {
    SCOPED_LIVE.load(std::sync::atomic::Ordering::Acquire)
}