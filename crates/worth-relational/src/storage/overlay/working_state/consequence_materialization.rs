//! Late, exact payload demand for Native-owned commit consequences.
use super::WorkingState;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::overlay::{PartitionAccess, PartitionCloneMode};
use std::collections::BTreeSet;

impl WorkingState {
    /// The consequence owner may demand only records protected from ordinary
    /// authors. A slot already changed by the business apply wins over base
    /// materialization (for example, a dangling membership retained on delete).
    /// This never clones a whole partition's dynamic record payload.
    pub(crate) fn materialize_consequence_records<E>(
        &mut self,
        base: &dyn PartitionAccess,
        partitions: &BTreeSet<PartitionId>,
        entities: &BTreeSet<EntityId>,
        relations: &BTreeSet<RelationId>,
        mut admit: impl FnMut(u64, Option<u64>) -> Result<(), E>,
    ) -> Result<u64, E> {
        let entity_only = self.clone_mode == PartitionCloneMode::EntityOnly;
        if entity_only {
            self.clone_mode = PartitionCloneMode::GraphSparseEntities;
        }
        let mut copied = 0u64;
        for &id in partitions {
            admit(
                1,
                tree_bound::<PartitionId, crate::storage::overlay::PartitionMutationJournal>(
                    self.mutation_journal.len(),
                ),
            )?;
            let journal = self.mutation_journal.entry(id).or_default();
            let mut entity_slots = BTreeSet::new();
            for entity in
                entities.range(EntityId::new(id, 0, 0)..=EntityId::new(id, u64::MAX, u32::MAX))
            {
                admit(1, tree_bound::<usize, ()>(entity_slots.len()))?;
                if !journal.entity_slots.contains(&entity.slot_index()) {
                    entity_slots.insert(entity.slot_index());
                }
            }
            let mut relation_slots = BTreeSet::new();
            for relation in
                relations.range(RelationId::new(id, 0, 0)..=RelationId::new(id, u64::MAX, u32::MAX))
            {
                admit(1, tree_bound::<usize, ()>(relation_slots.len()))?;
                if !journal.relation_slots.contains(&relation.slot_index()) {
                    relation_slots.insert(relation.slot_index());
                }
            }
            let Some(prior) = base.get_partition(id) else {
                continue;
            };
            admit(
                1,
                tree_bound::<PartitionId, crate::storage::overlay::PartitionState>(
                    self.partitions.len(),
                ),
            )?;
            if !self.partitions.contains_key(&id) {
                admit(
                    1,
                    prior
                        .entity_arena
                        .sparse_shell_allocation_bound()
                        .and_then(|entities| {
                            entities
                                .checked_add(prior.relation_arena.sparse_shell_allocation_bound()?)
                        }),
                )?;
            }
            let staged = self.partitions.entry(id).or_insert_with(|| {
                prior.clone_graph_with_sparse_entity_slots_and_relation_shape(&BTreeSet::new())
            });
            if entity_only && staged.relation_arena.slot_count() == 0 {
                admit(1, prior.relation_arena.sparse_shell_allocation_bound())?;
                admit(
                    journal.adjacency_slots.len() as u64,
                    prior
                        .adjacency
                        .adoption_allocation_bound(journal.adjacency_slots.len()),
                )?;
                admit(
                    journal.reverse_adjacency_slots.len() as u64,
                    prior
                        .reverse_adjacency
                        .adoption_allocation_bound(journal.reverse_adjacency_slots.len()),
                )?;
                staged.relation_arena = prior.relation_arena.sparse_shape_clone_for_overlay();
                staged.relation_overlay_is_sparse = true;
                let mut outgoing = prior.adjacency.clone();
                let mut incoming = prior.reverse_adjacency.clone();
                copied = copied.saturating_add(
                    outgoing.merge_slots_from(&staged.adjacency, &journal.adjacency_slots),
                );
                copied =
                    copied.saturating_add(incoming.merge_slots_from(
                        &staged.reverse_adjacency,
                        &journal.reverse_adjacency_slots,
                    ));
                staged.adjacency = outgoing;
                staged.reverse_adjacency = incoming;
            }
            if !entity_slots.is_empty() {
                admit(1, prior.entity_arena.sparse_shell_allocation_bound())?;
                for &slot in &entity_slots {
                    let quote = prior
                        .entity_arena
                        .sparse_slot_adoption_bound(slot)
                        .and_then(|source| {
                            source
                                .checked_add(staged.entity_arena.sparse_slot_adoption_bound(slot)?)
                        });
                    admit(1, quote)?;
                }
                let mut selected = prior
                    .entity_arena
                    .sparse_clone_slots_for_overlay(&entity_slots);
                copied = copied.saturating_add(
                    staged
                        .entity_arena
                        .merge_slots_from_owned(&mut selected, &entity_slots),
                );
            }
            if !relation_slots.is_empty() && staged.relation_overlay_is_sparse {
                admit(1, prior.relation_arena.sparse_shell_allocation_bound())?;
                for &slot in &relation_slots {
                    let quote = prior
                        .relation_arena
                        .sparse_slot_adoption_bound(slot)
                        .and_then(|source| {
                            source.checked_add(
                                staged.relation_arena.sparse_slot_adoption_bound(slot)?,
                            )
                        });
                    admit(1, quote)?;
                }
                let mut selected = prior
                    .relation_arena
                    .sparse_clone_slots_for_overlay(&relation_slots);
                copied = copied.saturating_add(
                    staged
                        .relation_arena
                        .merge_slots_from_owned(&mut selected, &relation_slots),
                );
            }
        }
        Ok(copied)
    }
}

// Rust 1.94 B-tree node: eleven entries, twelve child links, parent and length.
// Allow a split at every ancestor plus a new root before growing the owner.
fn tree_bound<K, V>(count: usize) -> Option<u64> {
    let levels = usize::BITS - count.saturating_add(1).max(1).leading_zeros();
    (11 * std::mem::size_of::<(K, V)>() + 14 * std::mem::size_of::<usize>())
        .checked_mul(levels as usize + 1)
        .and_then(|bytes| u64::try_from(bytes).ok())
}
