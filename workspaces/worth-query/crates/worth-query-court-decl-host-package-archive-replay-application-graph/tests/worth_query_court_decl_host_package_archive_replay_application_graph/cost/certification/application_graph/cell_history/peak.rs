//! Instrumented requested-object peaks; no runtime admission or storage authority.
use std::sync::{Mutex, Once};
use tracking_allocator::{
    AllocationGroupId, AllocationGroupToken, AllocationRegistry, AllocationTracker,
};

#[global_allocator]
static ALLOCATOR: tracking_allocator::Allocator<std::alloc::System> =
    tracking_allocator::Allocator::system();
static INSTALL: Once = Once::new();
static SESSION: Mutex<()> = Mutex::new(());
static STATE: Mutex<Option<ActiveGroup>> = Mutex::new(None);

struct ActiveGroup {
    id: AllocationGroupId,
    live: usize,
    peak: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct RequestedObjects {
    pub peak_bytes: usize,
    pub live_at_return_bytes: usize,
}

struct RequestedObjectTracker;
impl AllocationTracker for RequestedObjectTracker {
    fn allocated(&self, _: usize, object_size: usize, _: usize, group: AllocationGroupId) {
        let mut state = state();
        if let Some(active) = state.as_mut()
            && active.id == group
        {
            active.live = active.live.saturating_add(object_size);
            active.peak = active.peak.max(active.live);
        }
    }
    fn deallocated(
        &self,
        _: usize,
        object_size: usize,
        _: usize,
        source: AllocationGroupId,
        _: AllocationGroupId,
    ) {
        let mut state = state();
        if let Some(active) = state.as_mut()
            && active.id == source
        {
            active.live = active.live.saturating_sub(object_size);
        }
    }
}

/// Always uses the binary's tracking System wrapper, including between groups.
/// Counts only objects allocated by this group on the executing thread, with
/// source-group frees honored on any thread. Pre-existing objects, wrapper and
/// allocator metadata, and unrelated thread allocations are excluded.
pub fn measure<T>(run: impl FnOnce() -> T) -> (T, RequestedObjects) {
    INSTALL.call_once(|| {
        AllocationRegistry::set_global_tracker(RequestedObjectTracker)
            .expect("the binary has one allocator observer");
        AllocationRegistry::enable_tracking();
    });
    let _session = SESSION.lock().unwrap_or_else(|error| error.into_inner());
    let mut token = AllocationGroupToken::register().expect("allocation group identity capacity");
    let id = token.id();
    let group = token.enter();
    *state() = Some(ActiveGroup {
        id,
        live: 0,
        peak: 0,
    });
    let cleanup = ActiveCleanup;
    let value = run();
    let observation = {
        let state = state();
        let active = state
            .as_ref()
            .expect("the measurement owns its active group");
        RequestedObjects {
            peak_bytes: active.peak,
            live_at_return_bytes: active.live,
        }
    };
    drop(cleanup);
    drop(group);
    (value, observation)
}

pub fn report(lane: &str, observation: RequestedObjects) {
    println!("cell allocation lane={lane} allocator=instrumented-tracking-system requested_object_peak_bytes={} requested_object_live_at_return_bytes={} peak_minus_return_bytes={} scope=group-tagged-executing-thread-allocations;source-group-frees-any-thread;excludes-preexisting-objects-wrapper-headers-allocator-metadata-untagged-other-threads;not-RSS-or-total-heap",
        observation.peak_bytes, observation.live_at_return_bytes,
        observation.peak_bytes.saturating_sub(observation.live_at_return_bytes));
}

fn state() -> std::sync::MutexGuard<'static, Option<ActiveGroup>> {
    STATE.lock().unwrap_or_else(|error| error.into_inner())
}
struct ActiveCleanup;
impl Drop for ActiveCleanup {
    fn drop(&mut self) {
        *state() = None;
    }
}

#[cfg(test)]
#[path = "peak/tests.rs"]
mod tests;
