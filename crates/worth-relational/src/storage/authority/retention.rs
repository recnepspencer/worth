use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{PartitionId, VersionBound, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{HistoricalMetadata, RecordKind};

use super::StorageAuthority;

impl StorageAuthority<'_> {
    pub(crate) fn refresh_retention_state<K: RecordKind>(
        &self,
        partition_id: PartitionId,
        slot: usize,
        retired_at: Option<VersionId>,
        retention_fence: VersionId,
    ) {
        let Some(_retired_at) = retired_at else {
            return;
        };
        let mut writer = self.runtime.edit_partitions();
        let partition = writer
            .partition_mut(partition_id)
            .expect("retention partition present");
        let arena = K::arena_mut(partition);
        let lifecycle = self.retention_lifecycle::<K>(arena, slot, retired_at, retention_fence);
        if arena
            .get_slot(slot)
            .is_some_and(|old| old.lifecycle() != lifecycle)
        {
            arena.set_lifecycle_for_slot(slot, lifecycle);
        }
    }

    pub(super) fn retention_lifecycle<K: RecordKind>(
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
        partition_id: PartitionId,
        slot: usize,
    ) -> bool {
        let fence = self
            .runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id());
        self.retention_step::<K>(class, partition_id, slot, fence, true, None)
            .expect("legacy retention has no request admission or observed-spine refusal")
            .reclaimed
    }

    pub(crate) fn trim_live_history<K: RecordKind>(
        &self,
        slots_by_partition: BTreeMap<PartitionId, BTreeSet<usize>>,
        oldest_pinned_version: VersionId,
    ) -> usize
    where
        K::Meta: HistoricalMetadata,
    {
        let mut total_trimmed = 0usize;
        let mut writer = self.runtime.edit_partitions();
        for (partition_id, slots) in slots_by_partition {
            let Some(partition) = writer.partition_mut(partition_id) else {
                continue;
            };
            let arena = K::arena_mut(partition);
            for slot in slots {
                if arena
                    .get_slot(slot)
                    .is_none_or(|slot_view| slot_view.lifecycle() != RecordLifecycleState::Live)
                {
                    continue;
                }
                let bound = VersionBound::new(oldest_pinned_version);
                let original_len = match arena.metadata_history_at(slot) {
                    Some(metadata_history) => metadata_history.len(),
                    None => continue,
                };
                let trimmed_len = {
                    let Some(metadata_history) = arena.metadata_history_at_mut(slot) else {
                        continue;
                    };
                    metadata_history.retain(|entry| {
                        entry
                            .retired_at()
                            .is_none_or(|retired| bound.retains_retired(retired))
                    });
                    metadata_history.len()
                };
                total_trimmed += original_len.saturating_sub(trimmed_len);
            }
        }
        total_trimmed
    }
}
