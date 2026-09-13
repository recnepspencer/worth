//! Feature-gated, instrumented peak evidence. Ordinary timing never uses this allocator wrapper.
use std::sync::{Mutex, Once};

use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker,
};

static INSTALL: Once = Once::new();
static SESSION: Mutex<()> = Mutex::new(());
static STATE: Mutex<TrackerState> = Mutex::new(TrackerState::idle());

#[derive(Clone)]
struct ActiveGroup {
    id: AllocationGroupId,
    live: usize,
    peak: usize,
}

struct TrackerState {
    active: Option<ActiveGroup>,
}

impl TrackerState {
    const fn idle() -> Self {
        Self { active: None }
    }
}

struct RequestedObjectTracker;

impl AllocationTracker for RequestedObjectTracker {
    fn allocated(
        &self,
        _addr: usize,
        object_size: usize,
        _wrapped_size: usize,
        group_id: AllocationGroupId,
    ) {
        let mut state = state();
        if let Some(active) = &mut state.active {
            if active.id == group_id {
                active.live = active.live.saturating_add(object_size);
                active.peak = active.peak.max(active.live);
            }
        }
    }

    fn deallocated(
        &self,
        _addr: usize,
        object_size: usize,
        _wrapped_size: usize,
        source_group_id: AllocationGroupId,
        _current_group_id: AllocationGroupId,
    ) {
        let mut state = state();
        if let Some(active) = &mut state.active {
            // Ownership follows the allocation's source group. A stale group
            // freeing during a later session cannot subtract the new group.
            if active.id == source_group_id {
                active.live = active.live.saturating_sub(object_size);
            }
        }
    }
}

/// Returns the workload result and requested-object high-water for its group.
/// Pre-existing allocations, wrapper bytes, and allocations on other threads
/// are excluded. This is instrumented evidence, not RSS or ordinary timing.
pub(super) fn measure<T>(run: impl FnOnce() -> T) -> (T, Option<usize>) {
    install();
    let _serial = SESSION.lock().unwrap_or_else(|error| error.into_inner());
    let mut token = AllocationGroupToken::register().expect("allocation group id space exhausted");
    let id = token.id();
    let group = token.enter();
    {
        let mut state = state();
        assert!(
            state.active.is_none(),
            "peak measurement group already active"
        );
        state.active = Some(ActiveGroup {
            id,
            live: 0,
            peak: 0,
        });
    }
    let cleanup = ActiveCleanup;
    let value = run();
    let peak = state().active.as_ref().map(|active| active.peak);
    drop(cleanup);
    drop(group);
    (value, peak)
}

fn install() {
    INSTALL.call_once(|| {
        AllocationRegistry::set_global_tracker(RequestedObjectTracker)
            .expect("peak allocation tracker must be the sole tracker");
        AllocationRegistry::enable_tracking();
    });
}

fn state() -> std::sync::MutexGuard<'static, TrackerState> {
    STATE.lock().unwrap_or_else(|error| error.into_inner())
}

struct ActiveCleanup;

impl Drop for ActiveCleanup {
    fn drop(&mut self) {
        state().active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::measure;

    #[test]
    fn peak_tracks_free_after_high_water() {
        let (_, peak) = measure(|| {
            let first = vec![0_u8; 4_096];
            let second = vec![0_u8; 2_048];
            drop(first);
            drop(second);
        });
        assert_eq!(peak, Some(6_144));
    }

    #[test]
    fn realloc_peak_is_explicitly_instrumented_transient_high_water() {
        let (_, peak) = measure(|| {
            let mut bytes = Vec::with_capacity(1_024);
            bytes.resize(1_024, 1_u8);
            bytes.reserve_exact(2_048);
            std::hint::black_box(bytes);
        });
        assert_eq!(peak, Some(4_096));
    }

    #[test]
    fn preexisting_and_stale_groups_cannot_reduce_a_new_group() {
        let stale = measure(|| vec![0_u8; 8_192]).0;
        let (_, peak) = measure(|| {
            let current = vec![0_u8; 3_072];
            drop(stale);
            std::hint::black_box(current);
        });
        assert_eq!(peak, Some(3_072));
    }

    #[test]
    fn panic_cleans_up_the_active_group() {
        let panic = std::panic::catch_unwind(|| measure(|| panic!("expected")));
        assert!(panic.is_err());
        assert_eq!(
            measure(|| std::hint::black_box(vec![0_u8; 512])).1,
            Some(512)
        );
    }

    #[test]
    fn allocations_on_unrelated_threads_are_excluded() {
        let rendezvous = std::sync::Arc::new(std::sync::Barrier::new(2));
        let worker_rendezvous = rendezvous.clone();
        let worker = std::thread::spawn(move || {
            worker_rendezvous.wait();
            std::hint::black_box(vec![0_u8; 32_768])
        });
        let (_, peak) = measure(|| {
            rendezvous.wait();
            let local = vec![0_u8; 768];
            std::hint::black_box(local);
        });
        worker.join().unwrap();
        assert_eq!(peak, Some(768));
    }
}
