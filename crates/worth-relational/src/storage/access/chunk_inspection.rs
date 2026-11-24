//! Observe physical chunk summaries through the selected runtime edition.
use crate::identity::data::VersionId;
use crate::query::data::{PlannedQueryPacket, ReadPacketPlan};
use crate::runtime::RelationalRuntime;
use crate::storage::data::{ChunkDiagnostics, ChunkVisibilitySummary, ChunkedStorageSummary};
use crate::storage::partition::chunks::{slot_chunk_index, summarize_arena_chunks};
use crate::transactions::data::primitives::RecordRef;

pub(crate) fn chunked_storage_summary(
    runtime: &RelationalRuntime,
    edition: &crate::runtime::ResidentPartitionEdition,
    version_id: VersionId,
) -> ChunkedStorageSummary {
    ChunkedStorageSummary {
        entity_chunks: summarize_entity_chunks(runtime, edition, version_id),
        relation_chunks: summarize_relation_chunks(runtime, edition, version_id),
    }
}

pub(crate) fn chunk_diagnostics(
    runtime: &RelationalRuntime,
    edition: &crate::runtime::ResidentPartitionEdition,
    version_id: VersionId,
) -> ChunkDiagnostics {
    let summary = chunked_storage_summary(runtime, edition, version_id);
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
    edition: &crate::runtime::ResidentPartitionEdition,
    version_id: VersionId,
) -> Vec<ChunkVisibilitySummary> {
    edition
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
    edition: &crate::runtime::ResidentPartitionEdition,
    version_id: VersionId,
) -> Vec<ChunkVisibilitySummary> {
    edition
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
