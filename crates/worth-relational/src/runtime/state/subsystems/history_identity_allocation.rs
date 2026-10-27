//! Checked commit/version identity issuance for the history owner.
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn reserve_identity(allocator: &AtomicU64, legacy_floor: u64) -> Option<u64> {
    allocator.fetch_max(legacy_floor, Ordering::Relaxed);
    allocator
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .ok()
}
