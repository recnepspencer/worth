use crate::runtime::RelationalRuntime;
use crate::storage::data::{PartitionStorageStats, StorageStats};
use crate::storage::substrate::LifecycleCounts;

pub(crate) fn partition_ids(
    runtime: &RelationalRuntime,
) -> Vec<crate::identity::data::PartitionId> {
    runtime.partitions.partition_ids()
}

pub(crate) fn partition_storage_stats(
    runtime: &RelationalRuntime,
    edition: &crate::runtime::ResidentPartitionEdition,
) -> Vec<PartitionStorageStats> {
    edition
        .entries()
        .map(|(partition_id, partition)| {
            let entity_counts = partition.entity_arena.lifecycle_counts();
            let relation_counts = partition.relation_arena.lifecycle_counts();
            PartitionStorageStats {
                partition_id,
                entity_slots: partition.entity_arena.slot_count(),
                entity_chunks: partition
                    .entity_arena
                    .slot_count()
                    .div_ceil(runtime.storage_access().entity_chunk_size()),
                live_entities: entity_counts.live,
                deleted_entities: entity_counts.deleted,
                reusable_entity_slots: entity_counts.reusable,
                relation_slots: partition.relation_arena.slot_count(),
                relation_chunks: partition
                    .relation_arena
                    .slot_count()
                    .div_ceil(runtime.storage_access().relation_chunk_size()),
                live_relations: relation_counts.live,
                deleted_relations: relation_counts.deleted,
                reusable_relation_slots: relation_counts.reusable,
            }
        })
        .collect()
}

pub(crate) fn storage_stats(
    runtime: &RelationalRuntime,
    edition: &crate::runtime::ResidentPartitionEdition,
) -> StorageStats {
    let chunked_summary = crate::storage::partition::chunks::chunked_storage_summary(
        runtime,
        edition,
        runtime.current_version_id(),
    );
    let mut entity_counts = LifecycleCounts::default();
    let mut relation_counts = LifecycleCounts::default();
    let mut entity_slots = 0;
    let mut relation_slots = 0;
    for partition in edition.partitions() {
        entity_slots += partition.entity_arena.slot_count();
        relation_slots += partition.relation_arena.slot_count();
        let counts = partition.entity_arena.lifecycle_counts();
        entity_counts.live += counts.live;
        entity_counts.deleted += counts.deleted;
        entity_counts.reusable += counts.reusable;
        let counts = partition.relation_arena.lifecycle_counts();
        relation_counts.live += counts.live;
        relation_counts.deleted += counts.deleted;
        relation_counts.reusable += counts.reusable;
    }
    StorageStats {
        entity_slots,
        entity_chunks: chunked_summary.entity_chunks.len(),
        live_entities: entity_counts.live,
        deleted_entities: entity_counts.deleted,
        reusable_entity_slots: entity_counts.reusable,
        relation_slots,
        relation_chunks: chunked_summary.relation_chunks.len(),
        live_relations: relation_counts.live,
        deleted_relations: relation_counts.deleted,
        reusable_relation_slots: relation_counts.reusable,
        snapshot_count: runtime.visibility.active_snapshot_count(),
        published_snapshot_handle_count: runtime.visibility.published_snapshot_handle_count(),
        cached_visibility_version_count: runtime.visibility.cached_visibility_version_count(),
        protected_visibility_version_count: runtime.visibility.protected_visibility_version_count(
            runtime
                .config
                .visibility
                .cache_policy
                .protect_active_snapshots,
        ),
        recent_visibility_cache_count: runtime.visibility.recent_visibility_cache_count(),
    }
}
