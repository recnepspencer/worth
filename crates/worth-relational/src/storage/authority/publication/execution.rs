//! Effect-only execution of a complete partition-publication plan.

use crate::storage::overlay::{summarize_entity_chunk_plan, EntityWorkingSetLayout};

use super::super::StorageAuthority;
use super::plan::{
    PartitionPublicationStrategy, PlannedExistingPartitionPublication, PlannedPartitionPublication,
    PreparedMirrorPublication,
};

pub(super) fn execute_partition_publication(
    authority: &StorageAuthority<'_>,
    plan: PreparedMirrorPublication,
) {
    for partition in plan.partitions {
        execute_partition(authority, partition);
    }
}

fn execute_partition(authority: &StorageAuthority<'_>, publication: PlannedPartitionPublication) {
    let mut writer = authority.runtime.edit_partitions();
    match publication {
        PlannedPartitionPublication::New {
            target,
            entity_layout,
            entity_slots,
        } => {
            if let Some(layout) = entity_layout {
                record_entity_replacement(authority.runtime, entity_slots, layout);
            }
            writer.install_target(target);
        }
        PlannedPartitionPublication::Existing(mut publication) => {
            let target = publication.target.clone();
            let mut entity_chunks = None;
            let readmission = publication.readmission.clone();
            writer.transform_target(&target, readmission, |mut base| {
                match publication.strategy {
                    PartitionPublicationStrategy::EntityOnly(layout) => {
                        entity_chunks = merge_entity_slots(&mut base, &mut publication, layout);
                        base
                    }
                    PartitionPublicationStrategy::HybridGraph { entity_layout } => {
                        publish_hybrid_partition(
                            authority.runtime,
                            publication,
                            base,
                            entity_layout,
                        )
                    }
                    PartitionPublicationStrategy::WholePartition => {
                        publish_whole_partition(publication, base)
                    }
                }
            });
            drop(writer);
            count_published_chunks(authority.runtime, entity_chunks);
        }
    }
}

fn merge_entity_slots(
    base: &mut crate::storage::overlay::PartitionState,
    publication: &mut PlannedExistingPartitionPublication,
    layout: EntityWorkingSetLayout,
) -> Option<usize> {
    let chunk_plan = summarize_entity_chunk_plan(publication.journal.entity_slots.len(), layout);
    match layout {
        EntityWorkingSetLayout::AoSoACandidate { chunk_width } => {
            let published = base.entity_arena.merge_slot_chunks_from_owned(
                &mut publication.partition_state.entity_arena,
                &publication.journal.entity_slots,
                chunk_width,
            );
            Some(published.max(chunk_plan.chunk_count))
        }
        EntityWorkingSetLayout::CanonicalSoA => {
            base.entity_arena.merge_slots_from_owned(
                &mut publication.partition_state.entity_arena,
                &publication.journal.entity_slots,
            );
            None
        }
    }
}

fn count_published_chunks(
    runtime: &crate::runtime::RelationalRuntime,
    published_chunks: Option<usize>,
) {
    if let Some(published_chunks) = published_chunks {
        runtime
            .performance_access()
            .count_aosoa_publish_chunks(published_chunks);
    }
}

fn record_entity_replacement(
    runtime: &crate::runtime::RelationalRuntime,
    entity_slots: usize,
    layout: EntityWorkingSetLayout,
) {
    if matches!(layout, EntityWorkingSetLayout::AoSoACandidate { .. }) {
        let summary = summarize_entity_chunk_plan(entity_slots, layout);
        runtime
            .performance_access()
            .count_aosoa_publish_soa_merge(summary.chunk_count.max(1), entity_slots);
    }
}

fn publish_hybrid_partition(
    runtime: &crate::runtime::RelationalRuntime,
    mut publication: PlannedExistingPartitionPublication,
    mut base: crate::storage::overlay::PartitionState,
    entity_layout: Option<EntityWorkingSetLayout>,
) -> crate::storage::overlay::PartitionState {
    base.merge_adjacency_from(&publication.partition_state, &publication.journal);
    if let Some(layout) = entity_layout {
        let published_chunks = merge_entity_slots(&mut base, &mut publication, layout);
        count_published_chunks(runtime, published_chunks);
    }
    publication.partition_state.entity_arena = base.entity_arena;
    if publication.partition_state.relation_overlay_is_sparse
        && !publication.journal.relation_slots.is_empty()
    {
        base.relation_arena.merge_slots_from_owned(
            &mut publication.partition_state.relation_arena,
            &publication.journal.relation_slots,
        );
        publication.partition_state.relation_arena = base.relation_arena;
    } else if publication.journal.relation_slots.is_empty() {
        publication.partition_state.relation_arena = base.relation_arena;
    } else {
        publication
            .partition_state
            .relation_arena
            .preserve_runtime_pins_from(&base.relation_arena);
    }
    publication.partition_state.adjacency = base.adjacency;
    publication.partition_state.reverse_adjacency = base.reverse_adjacency;
    publication.partition_state
}

fn publish_whole_partition(
    mut publication: PlannedExistingPartitionPublication,
    mut base: crate::storage::overlay::PartitionState,
) -> crate::storage::overlay::PartitionState {
    base.merge_adjacency_from(&publication.partition_state, &publication.journal);
    if publication.journal.entity_slots.is_empty() {
        publication.partition_state.entity_arena = base.entity_arena;
    } else {
        publication
            .partition_state
            .entity_arena
            .preserve_runtime_pins_from(&base.entity_arena);
    }
    if publication.journal.relation_slots.is_empty() {
        publication.partition_state.relation_arena = base.relation_arena;
    } else {
        publication
            .partition_state
            .relation_arena
            .preserve_runtime_pins_from(&base.relation_arena);
    }
    publication.partition_state.adjacency = base.adjacency;
    publication.partition_state.reverse_adjacency = base.reverse_adjacency;
    publication.partition_state
}
