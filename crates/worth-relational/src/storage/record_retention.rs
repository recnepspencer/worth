//! Lifecycle selection for exact retired record metadata in the installed mirror.
use crate::config::data::RetentionBackend;
use crate::identity::data::{VersionBound, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::RecordKind;

pub(crate) fn refresh_slot<K: RecordKind>(
    backend: RetentionBackend,
    partition: &mut PartitionState,
    slot: usize,
    retired_at: Option<VersionId>,
    retention_fence: VersionId,
) {
    if retired_at.is_none() {
        return;
    }
    let arena = K::arena_mut(partition);
    let lifecycle = retention_lifecycle::<K>(backend, arena, slot, retired_at, retention_fence);
    arena.set_lifecycle_for_slot(slot, lifecycle);
}

pub(crate) fn retention_lifecycle<K: RecordKind>(
    backend: RetentionBackend,
    arena: &crate::storage::substrate::RecordArena<K>,
    slot: usize,
    retired_at: Option<VersionId>,
    retention_fence: VersionId,
) -> RecordLifecycleState {
    match backend {
        RetentionBackend::PinTrackedRetention => {
            if arena.snapshot_pin_count(slot).unwrap_or(0) > 0 {
                RecordLifecycleState::PinnedBySnapshot
            } else if arena.branch_pin_count(slot).unwrap_or(0) > 0 {
                RecordLifecycleState::PinnedByBranch
            } else if arena.replay_pin_count(slot).unwrap_or(0) > 0 {
                RecordLifecycleState::PinnedByReplayRetention
            } else {
                RecordLifecycleState::Reclaimable
            }
        }
        RetentionBackend::EpochChunkRetention => {
            if arena.branch_pin_count(slot).unwrap_or(0) > 0 {
                RecordLifecycleState::PinnedByBranch
            } else if arena.replay_pin_count(slot).unwrap_or(0) > 0 {
                RecordLifecycleState::PinnedByReplayRetention
            } else if retired_at
                .is_some_and(|retired| !VersionBound::new(retention_fence).retains_retired(retired))
            {
                RecordLifecycleState::Reclaimable
            } else {
                RecordLifecycleState::PinnedBySnapshot
            }
        }
    }
}
