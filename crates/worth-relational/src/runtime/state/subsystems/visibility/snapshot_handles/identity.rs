//! The active and prepared paths share one monotonic snapshot identity source.
use crate::snapshots::data::SnapshotId;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub(crate) struct SnapshotIdentityAllocator(AtomicU64);

impl SnapshotIdentityAllocator {
    pub(super) fn new() -> Self {
        Self(AtomicU64::new(1))
    }

    pub(super) fn allocate_active(&self) -> Option<SnapshotId> {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then(|| current.checked_add(1).unwrap_or(0))
            })
            .ok()
            .map(SnapshotId)
    }

    pub(crate) fn reserve_prepared(&self) -> Option<SnapshotId> {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()
            .map(SnapshotId)
    }
}
