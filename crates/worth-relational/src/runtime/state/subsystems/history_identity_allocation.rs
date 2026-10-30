//! Checked commit/version identity issuance for the history owner.
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn reserve_identity(allocator: &AtomicU64, legacy_floor: u64) -> Option<u64> {
    allocator.fetch_max(legacy_floor, Ordering::Relaxed);
    {
        let update_counter = allocator;
        let mut next = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = next.checked_add(1) else {
                break Err(next);
            };
            match update_counter.compare_exchange_weak(
                next,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => next = reloaded_value,
            }
        }
    }
    .ok()
}
