use std::collections::BTreeMap;

use crate::identity::data::{VersionBound, VersionId};
use crate::query::data::{PlannedQueryPacket, ReadPacketPlan};
use crate::runtime::RelationalRuntime;
use crate::storage::data::{
    ChunkDiagnostics, ChunkVisibilitySummary, ChunkedStorageSummary, RecordLifecycleState,
};
use crate::storage::substrate::{HistoricalMetadata, RecordArena, RecordKind};
use crate::transactions::data::RecordRef;

pub(crate) fn chunked_storage_summary(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> ChunkedStorageSummary {
    ChunkedStorageSummary {
        entity_chunks: summarize_entity_chunks(runtime, version_id),
        relation_chunks: summarize_relation_chunks(runtime, version_id),
    }
}

pub(crate) fn chunk_diagnostics(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> ChunkDiagnostics {
    let summary = chunked_storage_summary(runtime, version_id);
    ChunkDiagnostics {
        version_id,
        entity_chunks_total: summary.entity_chunks.len(),
        entity_chunks_with_visible_records: summary
            .entity_chunks
            .iter()
            .filter(|chunk| chunk.visible_records > 0)
            .count(),
        entity_chunks_with_retained_records: summary
            .entity_chunks
            .iter()
            .filter(|chunk| chunk.retained_records > 0)
            .count(),
        relation_chunks_total: summary.relation_chunks.len(),
        relation_chunks_with_visible_records: summary
            .relation_chunks
            .iter()
            .filter(|chunk| chunk.visible_records > 0)
            .count(),
        relation_chunks_with_retained_records: summary
            .relation_chunks
            .iter()
            .filter(|chunk| chunk.retained_records > 0)
            .count(),
    }
}

pub(crate) fn plan_read_explicit_query_packet(
    runtime: &RelationalRuntime,
    handle: &crate::snapshots::data::SnapshotHandle,
    packet: &PlannedQueryPacket,
) -> Option<ReadPacketPlan> {
    if !runtime.visibility.is_known_snapshot(handle.snapshot_id) {
        return None;
    }
    let targets = packet.explicit_target_refs()?;
    let mut entity_chunk_indexes = Vec::new();
    let mut relation_chunk_indexes = Vec::new();

    for target in targets {
        match target {
            RecordRef::Entity(entity_id) => push_unique_chunk(
                &mut entity_chunk_indexes,
                entity_id.slot_index(),
                runtime.config.storage.layout.entity_chunk_size,
            ),
            RecordRef::Relation(relation_id) => push_unique_chunk(
                &mut relation_chunk_indexes,
                relation_id.slot_index(),
                runtime.config.storage.layout.relation_chunk_size,
            ),
        }
    }

    Some(ReadPacketPlan {
        label: packet.label.clone(),
        entity_chunk_indexes,
        relation_chunk_indexes,
        target_count: targets.len(),
    })
}

fn push_unique_chunk(chunks: &mut Vec<usize>, slot: usize, chunk_size: usize) {
    let chunk = slot_chunk_index(slot, chunk_size);
    if !chunks.contains(&chunk) {
        chunks.push(chunk);
    }
}

fn summarize_entity_chunks(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> Vec<ChunkVisibilitySummary> {
    runtime
        .acquire_partition_edition()
        .partitions()
        .flat_map(|partition| {
            summarize_arena_chunks(
                &partition.entity_arena,
                version_id,
                runtime.current_version_id(),
                runtime.config.storage.layout.entity_chunk_size,
            )
        })
        .collect()
}

fn summarize_relation_chunks(
    runtime: &RelationalRuntime,
    version_id: VersionId,
) -> Vec<ChunkVisibilitySummary> {
    runtime
        .acquire_partition_edition()
        .partitions()
        .flat_map(|partition| {
            summarize_arena_chunks(
                &partition.relation_arena,
                version_id,
                runtime.current_version_id(),
                runtime.config.storage.layout.relation_chunk_size,
            )
        })
        .collect()
}

fn summarize_arena_chunks<K: RecordKind>(
    arena: &RecordArena<K>,
    version_id: VersionId,
    current_version: VersionId,
    chunk_size: usize,
) -> Vec<ChunkVisibilitySummary>
where
    K::Meta: HistoricalMetadata,
{
    if chunk_size == 0 {
        return Vec::new();
    }
    let occupied = arena.occupied_slots();
    let Some(logical_end) = occupied.last().and_then(|slot| slot.checked_add(1)) else {
        return Vec::new();
    };
    let mut chunks = BTreeMap::<usize, ChunkAccumulator>::new();
    for slot in occupied {
        let visible = if version_id == current_version {
            arena.live_bitset.count_ones_in_range(slot, slot + 1) == 1
        } else {
            arena
                .metadata_history_at(slot)
                .is_some_and(|history| visible_at_version(history, version_id))
        };
        chunks
            .entry(slot_chunk_index(slot, chunk_size))
            .or_default()
            .observe(
                visible,
                arena.created_at_for_slot(slot),
                arena.retired_at_for_slot(slot),
                arena.get_slot(slot).map(|view| view.lifecycle()),
            );
    }
    chunks
        .into_iter()
        .map(|(chunk_index, values)| {
            let slot_start = chunk_index.saturating_mul(chunk_size);
            ChunkVisibilitySummary {
                chunk_index,
                slot_start,
                slot_len: logical_end.saturating_sub(slot_start).min(chunk_size),
                visible_records: values.visible_records,
                retained_records: values.retained_records,
                reclaimable_records: values.reclaimable_records,
                earliest_created_at: values.earliest_created_at,
                latest_retired_at: values.latest_retired_at,
            }
        })
        .collect()
}

fn visible_at_version<M: HistoricalMetadata + Clone>(
    history: &crate::storage::substrate::SharedColumn<M>,
    version_id: VersionId,
) -> bool {
    let bound = VersionBound::new(version_id);
    let end = history.partition_point(|entry| bound.includes_created(entry.effective_at()));
    (0..end).rev().map(|index| &history[index]).any(|entry| {
        entry
            .retired_at()
            .is_none_or(|retired| bound.retains_retired(retired))
    })
}

#[derive(Default)]
struct ChunkAccumulator {
    visible_records: usize,
    retained_records: usize,
    reclaimable_records: usize,
    earliest_created_at: Option<VersionId>,
    latest_retired_at: Option<VersionId>,
}

impl ChunkAccumulator {
    fn observe(
        &mut self,
        visible: bool,
        created_at: Option<VersionId>,
        retired_at: Option<VersionId>,
        lifecycle: Option<RecordLifecycleState>,
    ) {
        self.visible_records += usize::from(visible);
        if let Some(created_at) = created_at {
            self.earliest_created_at = Some(
                self.earliest_created_at
                    .map_or(created_at, |current| current.min(created_at)),
            );
        }
        if let Some(retired_at) = retired_at {
            self.retained_records += 1;
            self.latest_retired_at = Some(
                self.latest_retired_at
                    .map_or(retired_at, |current| current.max(retired_at)),
            );
        }
        self.reclaimable_records +=
            usize::from(lifecycle == Some(RecordLifecycleState::Reclaimable));
    }
}

fn slot_chunk_index(slot: usize, chunk_size: usize) -> usize {
    if chunk_size == 0 {
        0
    } else {
        slot / chunk_size
    }
}
