//! Thread-attributed allocation counts for complexity courts, which compare the
//! same operation at two sizes by allocations and bytes rather than by time.
//!
//! A court binary declares the allocator once with `worth_allocation_probe!()`.
//! [`AllocationProbe::measure`] then counts every allocation the measuring
//! thread makes inside its closure, by requested object size. Allocations made
//! before or after the closure, or on any other thread, are not counted.
//! Measurements in one process run one at a time.
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, Once, PoisonError};
use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker,
};

/// The allocator `worth_allocation_probe!()` installs.
pub use tracking_allocator::Allocator;

static INSTALL: Once = Once::new();
static SESSION: Mutex<()> = Mutex::new(());
/// The measuring group's id, or 0 when no measurement runs.
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

/// What one measured closure allocated.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AllocationCount {
    pub allocations: u64,
    pub bytes: u64,
}

pub struct AllocationProbe;

impl AllocationProbe {
    /// Run `operation` and count what this thread allocates while it runs.
    ///
    /// Panics if the binary did not declare `worth_allocation_probe!()`: an
    /// allocation the probe itself makes must be seen, so a probe that would
    /// read zero for everything refuses instead.
    pub fn measure<T>(operation: impl FnOnce() -> T) -> (T, AllocationCount) {
        INSTALL.call_once(|| {
            AllocationRegistry::set_global_tracker(Counter)
                .expect("worth_allocation_probe!() owns this binary's tracker");
            AllocationRegistry::enable_tracking();
        });
        let _session = SESSION.lock().unwrap_or_else(PoisonError::into_inner);
        let (_, canary) = Self::group(|| std::hint::black_box(Box::new(0_u64)));
        assert_eq!(
            canary.allocations, 1,
            "declare worth_allocation_probe!() in this court binary"
        );
        Self::group(operation)
    }

    fn group<T>(operation: impl FnOnce() -> T) -> (T, AllocationCount) {
        struct Leave;
        impl Drop for Leave {
            fn drop(&mut self) {
                ACTIVE.store(0, Ordering::Release);
            }
        }
        let mut token = AllocationGroupToken::register().expect("an allocation group");
        ALLOCATIONS.store(0, Ordering::Release);
        BYTES.store(0, Ordering::Release);
        ACTIVE.store(token.id().as_usize().get(), Ordering::Release);
        let leave = Leave;
        let guard = token.enter();
        let result = operation();
        drop(guard);
        drop(leave);
        let count = AllocationCount {
            allocations: ALLOCATIONS.load(Ordering::Acquire),
            bytes: BYTES.load(Ordering::Acquire),
        };
        (result, count)
    }
}

struct Counter;

impl AllocationTracker for Counter {
    fn allocated(&self, _: usize, object_size: usize, _: usize, group: AllocationGroupId) {
        if group.as_usize().get() == ACTIVE.load(Ordering::Acquire) {
            ALLOCATIONS.fetch_add(1, Ordering::AcqRel);
            BYTES.fetch_add(object_size as u64, Ordering::AcqRel);
        }
    }

    fn deallocated(
        &self,
        _: usize,
        _: usize,
        _: usize,
        _: AllocationGroupId,
        _: AllocationGroupId,
    ) {
    }
}

/// Install the probe's allocator as this binary's global allocator.
#[macro_export]
macro_rules! worth_allocation_probe {
    () => {
        #[global_allocator]
        static WORTH_ALLOCATION_PROBE: $crate::allocation_probe::Allocator<::std::alloc::System> =
            $crate::allocation_probe::Allocator::system();
    };
}
