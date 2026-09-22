use crate::history::data::RecordAllocationClass;
use crate::identity::data::{PartitionId, VersionId};
use crate::runtime::RelationalRuntime;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{
    EntityRecordKind, HistoricalMetadata, RecordKind, RelationRecordKind,
};

type RefreshRetention = fn(&RelationalRuntime, PartitionId, usize, Option<VersionId>, VersionId);

#[derive(Default)]
pub(super) struct RetentionCounts {
    pub(super) branch_pinned: usize,
    pub(super) replay_pinned: usize,
    pub(super) snapshot_pinned: usize,
    pub(super) reclaimable: usize,
    pub(super) branch_replay_overlap: usize,
}

pub(super) struct PartitionRetentionPass {
    pub(super) class: RecordAllocationClass,
    pub(super) partition_id: PartitionId,
    pub(super) chunk_size: usize,
    pub(super) retention_fence: VersionId,
}

#[derive(Default)]
pub(super) struct RetentionPassCounts {
    pub(super) chunks_scanned: usize,
    pub(super) reclaimable: usize,
    pub(super) reclaimed: usize,
}

pub(super) fn refresh_entity_retention_state(
    runtime: &RelationalRuntime,
    partition_id: PartitionId,
    slot: usize,
    retired_at: Option<VersionId>,
    retention_fence: VersionId,
) {
    runtime
        .storage_authority()
        .refresh_retention_state::<EntityRecordKind>(
            partition_id,
            slot,
            retired_at,
            retention_fence,
        );
}

pub(super) fn refresh_relation_retention_state(
    runtime: &RelationalRuntime,
    partition_id: PartitionId,
    slot: usize,
    retired_at: Option<VersionId>,
    retention_fence: VersionId,
) {
    runtime
        .storage_authority()
        .refresh_retention_state::<RelationRecordKind>(
            partition_id,
            slot,
            retired_at,
            retention_fence,
        );
}

pub(super) fn inspect_partition_retention<K: RecordKind>(
    runtime: &RelationalRuntime,
    partition_id: PartitionId,
    retention_fence: VersionId,
    refresh_retention: RefreshRetention,
) -> RetentionCounts {
    let mut counts = RetentionCounts::default();
    for slot in runtime.storage_access().record_slots::<K>(partition_id) {
        let retired_at = runtime
            .storage_access()
            .record_slot_surface::<K>(partition_id, slot)
            .and_then(|surface| surface.retired_at);
        if retired_at.is_some() {
            refresh_retention(runtime, partition_id, slot, retired_at, retention_fence);
        }
        let Some(surface) = runtime
            .storage_access()
            .record_slot_surface::<K>(partition_id, slot)
        else {
            continue;
        };
        counts.branch_pinned += usize::from(surface.branch_pins > 0);
        counts.replay_pinned += usize::from(surface.replay_pins > 0);
        counts.snapshot_pinned += usize::from(surface.snapshot_pins > 0);
        counts.branch_replay_overlap +=
            usize::from(surface.branch_pins > 0 && surface.replay_pins > 0);
        counts.reclaimable += usize::from(surface.lifecycle == RecordLifecycleState::Reclaimable);
    }
    counts
}

pub(super) fn run_partition_retention_pass<K: RecordKind>(
    runtime: &RelationalRuntime,
    pass: PartitionRetentionPass,
    count_scan: impl Fn(&RelationalRuntime),
    refresh_retention: RefreshRetention,
) -> RetentionPassCounts {
    let mut counts = RetentionPassCounts::default();
    let chunk_size = pass.chunk_size.max(1);
    let mut current_chunk = None;
    for slot in runtime
        .storage_access()
        .record_slots::<K>(pass.partition_id)
    {
        let chunk = slot / chunk_size;
        if current_chunk != Some(chunk) {
            current_chunk = Some(chunk);
            counts.chunks_scanned += 1;
        }
        count_scan(runtime);
        let retired_at = runtime
            .storage_access()
            .record_slot_surface::<K>(pass.partition_id, slot)
            .and_then(|surface| surface.retired_at);
        let Some(version) = retired_at else {
            continue;
        };
        refresh_retention(
            runtime,
            pass.partition_id,
            slot,
            Some(version),
            pass.retention_fence,
        );
        let reclaimable = runtime
            .storage_access()
            .record_slot_surface::<K>(pass.partition_id, slot)
            .is_some_and(|surface| surface.lifecycle == RecordLifecycleState::Reclaimable);
        if !reclaimable {
            continue;
        }
        counts.reclaimable += 1;
        if runtime.config.storage.mvcc.auto_reclaim_deleted_records
            && counts.reclaimed < runtime.config.storage.mvcc.reclaim_batch_size
            && runtime
                .storage_authority()
                .reclaim_record_if_reclaimable::<K>(pass.class, pass.partition_id, slot)
        {
            counts.reclaimed += 1;
        }
    }
    counts
}

pub(super) fn trim_live_history<K: RecordKind>(
    runtime: &RelationalRuntime,
    slots_by_partition: std::collections::BTreeMap<PartitionId, std::collections::BTreeSet<usize>>,
    oldest_pinned_version: VersionId,
    count_trimmed: impl Fn(&RelationalRuntime, usize),
) where
    K::Meta: HistoricalMetadata,
{
    let total_trimmed = runtime
        .storage_authority()
        .trim_live_history::<K>(slots_by_partition, oldest_pinned_version);
    count_trimmed(runtime, total_trimmed);
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::history::data::RecordAllocationClass;
    use crate::identity::data::{KindId, PartitionId, VersionId};
    use crate::runtime::{RelationalRuntime, RelationalRuntimeConfig};
    use crate::storage::overlay::PartitionState;
    use crate::storage::substrate::{
        EntityArena, EntityExtra, EntityRecordKind, PinClass, RelationArena, SlotInit,
    };

    use super::{
        inspect_partition_retention, refresh_entity_retention_state, run_partition_retention_pass,
        PartitionRetentionPass,
    };

    #[test]
    fn sparse_high_slot_retention_scans_one_materialized_chunk() {
        let runtime = RelationalRuntime::new(RelationalRuntimeConfig::default());
        let partition_id = PartitionId(7);
        let high_slot = 50_000;
        let mut entity_arena = EntityArena::with_capacity(0);
        entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id,
                    kind_id: KindId(11),
                    version_id: VersionId(1),
                    extra: EntityExtra::default(),
                },
                high_slot,
                1,
            )
            .unwrap();
        entity_arena
            .adjust_named_pin(high_slot, PinClass::Branch, 1)
            .unwrap();
        let adjacency_policy = runtime.config.storage.adjacency_policy.clone();
        runtime.edit_partitions().insert(
            partition_id,
            PartitionState {
                partition_id,
                adjacency_policy,
                relation_overlay_is_sparse: false,
                entity_arena,
                relation_arena: RelationArena::with_capacity(0),
                adjacency: Default::default(),
                reverse_adjacency: Default::default(),
            },
        );

        let inspected = inspect_partition_retention::<EntityRecordKind>(
            &runtime,
            partition_id,
            VersionId(1),
            refresh_entity_retention_state,
        );
        assert_eq!(inspected.branch_pinned, 1);

        let scans = Cell::new(0);
        let outcome = run_partition_retention_pass::<EntityRecordKind>(
            &runtime,
            PartitionRetentionPass {
                class: RecordAllocationClass::Entity,
                partition_id,
                chunk_size: 128,
                retention_fence: VersionId(1),
            },
            |_| scans.set(scans.get() + 1),
            refresh_entity_retention_state,
        );
        assert_eq!(scans.get(), 1);
        assert_eq!(outcome.chunks_scanned, 1);
    }
}
