//! Storage-owned mutation cloning preserves residency without lending history.
use super::RelationalRegionHeadView;
use crate::storage::overlay::{PartitionCloneMode, PartitionState};
use std::collections::BTreeSet;

impl RelationalRegionHeadView<'_> {
    pub(in crate::storage) fn clone_for_mutation(
        &self,
        mode: PartitionCloneMode,
        entity_slots: Option<&BTreeSet<usize>>,
        sparse_relations: bool,
    ) -> PartitionState {
        match (mode, entity_slots, sparse_relations) {
            (PartitionCloneMode::GraphSparseEntities, Some(slots), true) if !slots.is_empty() => {
                self.partition
                    .clone_graph_with_sparse_entity_slots_and_relation_shape(slots)
            }
            (PartitionCloneMode::EntityOnly, Some(slots), _) if !slots.is_empty() => {
                self.partition.clone_entity_slots_for_overlay(slots)
            }
            (PartitionCloneMode::GraphSparseEntities, Some(slots), _) if !slots.is_empty() => {
                self.partition.clone_graph_with_sparse_entity_slots(slots)
            }
            (PartitionCloneMode::GraphSparseEntities, None, true) => {
                self.partition.clone_graph_with_sparse_relation_shape()
            }
            _ => self.partition.clone_for_overlay(mode),
        }
    }
}
