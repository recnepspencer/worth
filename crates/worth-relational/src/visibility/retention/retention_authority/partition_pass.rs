use crate::history::data::RecordAllocationClass;
use crate::identity::data::{PartitionId, VersionId};
use crate::runtime::RelationalRuntime;
use crate::runtime::ResidentPartitionEntry;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{
    EntityRecordKind, HistoricalMetadata, RecordKind, RelationRecordKind,
};

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
    let mut writer = runtime.edit_partitions();
    if let Some(partition) = writer.partition_mut(partition_id) {
        runtime
            .storage_authority()
            .refresh_retention_state::<EntityRecordKind>(
                partition,
                slot,
                retired_at,
                retention_fence,
            );
    }
}

pub(super) fn refresh_relation_retention_state(
    runtime: &RelationalRuntime,
    partition_id: PartitionId,
    slot: usize,
    retired_at: Option<VersionId>,
    retention_fence: VersionId,
) {
    let mut writer = runtime.edit_partitions();
    if let Some(partition) = writer.partition_mut(partition_id) {
        runtime
            .storage_authority()
            .refresh_retention_state::<RelationRecordKind>(
                partition,
                slot,
                retired_at,
                retention_fence,
            );
    }
}

pub(super) fn inspect_partition_retention<K: RecordKind>(
    runtime: &RelationalRuntime,
    partition: &mut ResidentPartitionEntry<'_>,
    retention_fence: VersionId,
) -> Result<RetentionCounts, crate::execution::RelationalExecutionDenialCause> {
    let mut counts = RetentionCounts::default();
    for slot in K::arena(partition.read()).occupied_slots() {
        let retired_at = K::arena(partition.read()).retired_at_for_slot(slot);
        if retired_at.is_some() {
            runtime.storage_authority().refresh_retention_state::<K>(
                partition.edit()?,
                slot,
                retired_at,
                retention_fence,
            );
        }
        let arena = K::arena(partition.read());
        counts.branch_pinned += usize::from(arena.branch_pin_count(slot).unwrap_or(0) > 0);
        counts.replay_pinned += usize::from(arena.replay_pin_count(slot).unwrap_or(0) > 0);
        counts.snapshot_pinned += usize::from(arena.snapshot_pin_count(slot).unwrap_or(0) > 0);
        counts.branch_replay_overlap += usize::from(
            arena.branch_pin_count(slot).unwrap_or(0) > 0
                && arena.replay_pin_count(slot).unwrap_or(0) > 0,
        );
        counts.reclaimable += usize::from(
            arena
                .get_slot(slot)
                .is_some_and(|view| view.lifecycle() == RecordLifecycleState::Reclaimable),
        );
    }
    Ok(counts)
}

pub(super) fn run_partition_retention_pass<K: RecordKind>(
    runtime: &RelationalRuntime,
    partition: &mut ResidentPartitionEntry<'_>,
    pass: PartitionRetentionPass,
    count_scan: impl Fn(&RelationalRuntime),
    reclaimed: &mut Vec<crate::runtime::ReclaimedRecordSlot>,
) -> Result<RetentionPassCounts, crate::execution::RelationalExecutionDenialCause> {
    let mut counts = RetentionPassCounts::default();
    let mut current_chunk = None;
    for slot in K::arena(partition.read()).occupied_slots() {
        let chunk = slot / pass.chunk_size.max(1);
        if current_chunk != Some(chunk) {
            current_chunk = Some(chunk);
            counts.chunks_scanned += 1;
        }
        count_scan(runtime);
        let retired_at = K::arena(partition.read()).retired_at_for_slot(slot);
        let Some(version) = retired_at else { continue };
        runtime.storage_authority().refresh_retention_state::<K>(
            partition.edit()?,
            slot,
            Some(version),
            pass.retention_fence,
        );
        if !K::arena(partition.read())
            .get_slot(slot)
            .is_some_and(|view| view.lifecycle() == RecordLifecycleState::Reclaimable)
        {
            continue;
        }
        counts.reclaimable += 1;
        if runtime.config.storage.mvcc.auto_reclaim_deleted_records
            && counts.reclaimed < runtime.config.storage.mvcc.reclaim_batch_size
            && let Some(slot) = runtime
                .storage_authority()
                .reclaim_record_if_reclaimable::<K>(pass.class, partition.edit()?, slot)
        {
            reclaimed.push(slot);
            counts.reclaimed += 1;
        }
    }
    Ok(counts)
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
        inspect_partition_retention, run_partition_retention_pass, PartitionRetentionPass,
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

        let retained = runtime.acquire_partition_edition();
        runtime.performance_access().reset_counters();
        let request = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&request);
        let mut writer = runtime
            .partitions
            .demand_resident_edit(&execution, &runtime.services.instrumentation)
            .unwrap();
        let scans = Cell::new(0);
        writer
            .visit_partitions(|mut partition| {
                let inspected = inspect_partition_retention::<EntityRecordKind>(
                    &runtime,
                    &mut partition,
                    VersionId(1),
                )?;
                assert_eq!(inspected.branch_pinned, 1);
                let outcome = run_partition_retention_pass::<EntityRecordKind>(
                    &runtime,
                    &mut partition,
                    PartitionRetentionPass {
                        class: RecordAllocationClass::Entity,
                        chunk_size: 128,
                        retention_fence: VersionId(1),
                    },
                    |_| scans.set(scans.get() + 1),
                    &mut Vec::new(),
                )?;
                assert_eq!(outcome.chunks_scanned, 1);
                Ok(())
            })
            .unwrap();
        assert_eq!(scans.get(), 1);
        drop(writer);
        let counts = runtime.performance_access().counters();
        assert_eq!(counts.ordinary_partitions_copied_on_write, 0);
        assert_eq!(counts.ordinary_partition_slots_copied_on_write, 0);
        assert!(retained.partition(partition_id).is_some());
    }
}
