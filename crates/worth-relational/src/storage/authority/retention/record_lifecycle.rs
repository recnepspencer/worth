use crate::identity::data::{VersionBound, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::RecordKind;

use super::super::StorageAuthority;

impl StorageAuthority<'_> {
    pub(crate) fn refresh_retention_state<K: RecordKind>(
        &self,
        partition: &mut crate::storage::overlay::PartitionState,
        slot: usize,
        retired_at: Option<VersionId>,
        retention_fence: VersionId,
    ) {
        let Some(_retired_at) = retired_at else {
            return;
        };
        let arena = K::arena_mut(partition);
        let lifecycle = self.retention_lifecycle::<K>(arena, slot, retired_at, retention_fence);
        arena.set_lifecycle_for_slot(slot, lifecycle);
    }

    pub(in crate::storage::authority) fn retention_lifecycle<K: RecordKind>(
        &self,
        arena: &crate::storage::substrate::RecordArena<K>,
        slot: usize,
        retired_at: Option<VersionId>,
        retention_fence: VersionId,
    ) -> RecordLifecycleState {
        match self.runtime.config.storage.retention.backend {
            crate::config::data::RetentionBackend::PinTrackedRetention => {
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
            crate::config::data::RetentionBackend::EpochChunkRetention => {
                if arena.branch_pin_count(slot).unwrap_or(0) > 0 {
                    RecordLifecycleState::PinnedByBranch
                } else if arena.replay_pin_count(slot).unwrap_or(0) > 0 {
                    RecordLifecycleState::PinnedByReplayRetention
                } else if retired_at.is_some_and(|retired| {
                    !VersionBound::new(retention_fence).retains_retired(retired)
                }) {
                    RecordLifecycleState::Reclaimable
                } else {
                    RecordLifecycleState::PinnedBySnapshot
                }
            }
        }
    }

    pub(crate) fn reclaim_record_if_reclaimable<K: RecordKind>(
        &self,
        class: crate::history::data::RecordAllocationClass,
        partition: &mut crate::storage::overlay::PartitionState,
        slot: usize,
    ) -> Option<crate::runtime::ReclaimedRecordSlot> {
        let partition_id = partition.partition_id;
        let arena = K::arena_mut(partition);
        if arena.get_slot(slot)?.lifecycle() != RecordLifecycleState::Reclaimable {
            return None;
        }
        arena.set_lifecycle_for_slot(slot, RecordLifecycleState::Reusable);
        arena.reset_slot(slot)?;
        Some(crate::runtime::ReclaimedRecordSlot::new(
            class,
            partition_id,
            slot,
        ))
    }
}
