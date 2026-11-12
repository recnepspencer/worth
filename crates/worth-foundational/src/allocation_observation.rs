//! Thread-attributed physical allocation evidence for the value backing owners.
use std::{alloc::System, cell::Cell, sync::Once};
use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker, Allocator,
};

#[global_allocator]
static ALLOCATOR: Allocator<System> = Allocator::system();
static INSTALL: Once = Once::new();
thread_local! {
    static GROUP: Cell<usize> = const { Cell::new(0) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
}

struct Tracker;

pub(crate) fn live_bytes() -> usize {
    LIVE.with(Cell::get)
}
impl AllocationTracker for Tracker {
    fn allocated(&self, _: usize, bytes: usize, _: usize, group: AllocationGroupId) {
        GROUP.with(|active| {
            if active.get() == group.as_usize().get() {
                LIVE.with(|live| {
                    let next = live.get() + bytes;
                    live.set(next);
                    PEAK.with(|peak| peak.set(peak.get().max(next)));
                });
            }
        });
    }
    fn deallocated(
        &self,
        _: usize,
        bytes: usize,
        _: usize,
        source: AllocationGroupId,
        _: AllocationGroupId,
    ) {
        GROUP.with(|active| {
            if active.get() == source.as_usize().get() {
                LIVE.with(|live| live.set(live.get() - bytes));
            }
        });
    }
}

/// Measures Rust allocation layouts, excluding this observer's wrapper metadata.
/// Inputs predate the unique group; their deallocations cannot reduce its peak.
/// These synchronous owners neither send allocations nor spawn worker threads.
pub(crate) fn peak<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    INSTALL.call_once(|| {
        AllocationRegistry::set_global_tracker(Tracker).expect("one unit-binary tracker");
        AllocationRegistry::enable_tracking();
    });
    let mut token = AllocationGroupToken::register().expect("one observation group");
    let identity = token.id().as_usize().get();
    let guard = token.enter();
    GROUP.with(|group| assert_eq!(group.replace(identity), 0));
    LIVE.with(|live| live.set(0));
    PEAK.with(|peak| peak.set(0));
    struct Exit;
    impl Drop for Exit {
        fn drop(&mut self) {
            GROUP.with(|group| group.set(0));
        }
    }
    let exit = Exit;
    let result = operation();
    let peak = PEAK.with(Cell::get);
    drop(exit);
    drop(guard);
    (result, peak)
}
