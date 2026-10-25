use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{PartitionId, VersionBound, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{HistoricalMetadata, RecordKind};

use super::StorageAuthority;

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
        let lifecycle = match self.runtime.config.storage.retention.backend {
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
        };
        arena.set_lifecycle_for_slot(slot, lifecycle);
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
        arena.reset_slot(slot);
        Some(crate::runtime::ReclaimedRecordSlot::new(
            class,
            partition_id,
            slot,
        ))
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
