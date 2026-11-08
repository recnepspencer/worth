//! Preserve each already-applied writer's slots when adopting a later writer image.
use super::WorkingState;
use crate::storage::overlay::PartitionCloneMode;

impl WorkingState {
    pub(crate) fn adopt_sequential_apply(&mut self, next: Self) {
        // Each journal is an overlay against the same unpublished proposal.
        // Retaining sparse posture prevents a later whole-arena replacement
        // from erasing untouched records in the original branch region.
        self.clone_mode = PartitionCloneMode::GraphSparseEntities;
        self.content = next.content;
        for (id, mut overlay) in next.partitions {
            let mut journal = next.mutation_journal.get(&id).cloned().unwrap_or_default();
            if let Some(prior) = self.partitions.get_mut(&id) {
                // An entity-only suffix contributes no relation image. Keep
                // the prefix's actual materialization posture so the current-
                // fact writer can still promote an entity-only shell later.
                let relations_are_sparse =
                    prior.relation_overlay_is_sparse || !journal.relation_slots.is_empty();
                prior.merge_overlay_from_owned(&mut overlay, &journal);
                prior.relation_overlay_is_sparse = relations_are_sparse;
            } else {
                self.partitions.insert(id, overlay);
            }
            let combined = self.mutation_journal.entry(id).or_default();
            combined.entity_slots.append(&mut journal.entity_slots);
            combined.relation_slots.append(&mut journal.relation_slots);
            combined
                .adjacency_slots
                .append(&mut journal.adjacency_slots);
            combined
                .reverse_adjacency_slots
                .append(&mut journal.reverse_adjacency_slots);
        }
    }
}
