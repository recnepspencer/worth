//! Apply only the prepared physical overlay and preserve the base's live pins.
use super::{summarize_entity_chunk_plan, EntityWorkingSetLayout};
use crate::storage::mirror_plan::PlannedExistingPartitionPublication;

pub(in crate::storage) fn merge_entity_slots(
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

pub(in crate::storage) fn publish_hybrid_partition(
    mut publication: PlannedExistingPartitionPublication,
    mut base: crate::storage::overlay::PartitionState,
    entity_layout: Option<EntityWorkingSetLayout>,
) -> (crate::storage::overlay::PartitionState, Option<usize>) {
    base.merge_adjacency_from(&publication.partition_state, &publication.journal);
    let published_chunks =
        entity_layout.and_then(|layout| merge_entity_slots(&mut base, &mut publication, layout));
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
    (publication.partition_state, published_chunks)
}

pub(in crate::storage) fn publish_whole_partition(
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
