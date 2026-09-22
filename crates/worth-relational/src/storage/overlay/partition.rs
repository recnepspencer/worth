use std::collections::BTreeSet;

use crate::config::data::AdjacencyPolicy;
use crate::identity::data::PartitionId;

use crate::storage::partition::DenseSlotBitSet;
use crate::storage::substrate::{EntityArena, RelationArena};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PartitionCloneMode {
    Full,
    EntityOnly,
    GraphSparseEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntityWorkingSetLayout {
    CanonicalSoA,
    AoSoACandidate { chunk_width: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct EntityChunkPlanSummary {
    pub(crate) chunk_width: usize,
    pub(crate) chunk_count: usize,
    pub(crate) slot_count: usize,
}

pub(crate) fn summarize_entity_chunk_plan(
    slot_count: usize,
    layout: EntityWorkingSetLayout,
) -> EntityChunkPlanSummary {
    match layout {
        EntityWorkingSetLayout::CanonicalSoA => EntityChunkPlanSummary::default(),
        EntityWorkingSetLayout::AoSoACandidate { chunk_width } => EntityChunkPlanSummary {
            chunk_width,
            chunk_count: if slot_count == 0 {
                0
            } else {
                slot_count.div_ceil(chunk_width)
            },
            slot_count,
        },
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PartitionState {
    pub(crate) partition_id: PartitionId,
    pub(crate) adjacency_policy: AdjacencyPolicy,
    pub(crate) relation_overlay_is_sparse: bool,
    pub(crate) entity_arena: EntityArena,
    pub(crate) relation_arena: RelationArena,
    pub(crate) adjacency: crate::storage::partition::SparseAdjacencyTable,
    pub(crate) reverse_adjacency: crate::storage::partition::SparseAdjacencyTable,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RelationalPartitionAllocationInventory {
    pub(crate) authoritative_bytes: u64,
    pub(crate) diagnostic_bytes: u64,
    pub(crate) retention_metadata_bytes: u64,
    pub(crate) allocator_bookkeeping_bytes: u64,
    pub(crate) optional_cache_bytes: u64,
}

impl PartitionState {
    pub(crate) fn clear_runtime_pin_counters(&mut self) {
        self.entity_arena.clear_all_pins();
        self.relation_arena.clear_all_pins();
    }

    pub(crate) fn allocation_inventory(&self) -> RelationalPartitionAllocationInventory {
        let adjacency_bytes = self
            .adjacency
            .iter()
            .chain(self.reverse_adjacency.iter())
            .map(|(_, set)| set.authoritative_allocation_bytes())
            .sum::<u64>();
        let adjacency_cache_bytes = self
            .adjacency
            .iter()
            .chain(self.reverse_adjacency.iter())
            .map(|(_, set)| set.optional_cache_allocation_bytes())
            .sum::<u64>();
        let arenas = self
            .entity_arena
            .allocation_inventory()
            .saturating_add(self.relation_arena.allocation_inventory());
        let authoritative_bytes = arenas
            .authoritative_bytes
            .saturating_add(self.adjacency.allocation_bytes())
            .saturating_add(self.reverse_adjacency.allocation_bytes())
            .saturating_add(adjacency_bytes);
        RelationalPartitionAllocationInventory {
            authoritative_bytes,
            diagnostic_bytes: arenas.diagnostic_bytes,
            retention_metadata_bytes: arenas.retention_metadata_bytes,
            allocator_bookkeeping_bytes: arenas.allocator_bookkeeping_bytes,
            optional_cache_bytes: adjacency_cache_bytes,
        }
    }

    pub(crate) fn clone_for_overlay(&self, mode: PartitionCloneMode) -> Self {
        match mode {
            PartitionCloneMode::Full => self.clone(),
            PartitionCloneMode::EntityOnly => Self {
                partition_id: self.partition_id,
                adjacency_policy: self.adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: self.entity_arena.clone(),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: Default::default(),
                reverse_adjacency: Default::default(),
            },
            PartitionCloneMode::GraphSparseEntities => Self {
                partition_id: self.partition_id,
                adjacency_policy: self.adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: self.entity_arena.sparse_shape_clone_for_overlay(),
                relation_arena: self.relation_arena.clone(),
                adjacency: self.adjacency.clone(),
                reverse_adjacency: self.reverse_adjacency.clone(),
            },
        }
    }

    pub(crate) fn clone_entity_slots_for_overlay(
        &self,
        touched_entity_slots: &BTreeSet<usize>,
    ) -> Self {
        Self {
            partition_id: self.partition_id,
            adjacency_policy: self.adjacency_policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: self
                .entity_arena
                .sparse_clone_slots_for_overlay(touched_entity_slots),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        }
    }

    pub(crate) fn clone_graph_with_sparse_entity_slots(
        &self,
        touched_entity_slots: &BTreeSet<usize>,
    ) -> Self {
        Self {
            partition_id: self.partition_id,
            adjacency_policy: self.adjacency_policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: self
                .entity_arena
                .sparse_clone_slots_for_overlay(touched_entity_slots),
            relation_arena: self.relation_arena.clone(),
            adjacency: self.adjacency.clone(),
            reverse_adjacency: self.reverse_adjacency.clone(),
        }
    }

    pub(crate) fn clone_graph_with_sparse_entity_slots_and_relation_shape(
        &self,
        touched_entity_slots: &BTreeSet<usize>,
    ) -> Self {
        Self {
            partition_id: self.partition_id,
            adjacency_policy: self.adjacency_policy.clone(),
            relation_overlay_is_sparse: true,
            entity_arena: self
                .entity_arena
                .sparse_clone_slots_for_overlay(touched_entity_slots),
            relation_arena: self.relation_arena.sparse_shape_clone_for_overlay(),
            adjacency: self.adjacency.clone(),
            reverse_adjacency: self.reverse_adjacency.clone(),
        }
    }

    pub(crate) fn clone_graph_with_sparse_relation_shape(&self) -> Self {
        Self {
            partition_id: self.partition_id,
            adjacency_policy: self.adjacency_policy.clone(),
            relation_overlay_is_sparse: true,
            entity_arena: self.entity_arena.sparse_shape_clone_for_overlay(),
            relation_arena: self.relation_arena.sparse_shape_clone_for_overlay(),
            adjacency: self.adjacency.clone(),
            reverse_adjacency: self.reverse_adjacency.clone(),
        }
    }

    /// Apply one storage-owner overlay to an immutable branch-region copy.
    ///
    /// Branch roots cannot consult the runtime-wide partition map: a sibling
    /// branch may have published a different current partition there.  The
    /// publication journal therefore travels with the overlay and this
    /// operation reconstructs the exact branch-local partition from the prior
    /// region.  Empty journal axes deliberately retain the prior arena/graph.
    pub(crate) fn merge_overlay_from_owned(
        &mut self,
        overlay: &mut Self,
        journal: &PartitionMutationJournal,
    ) -> u64 {
        let mut copied_bytes = 0_u64;
        if !journal.entity_slots.is_empty() {
            copied_bytes = self
                .entity_arena
                .merge_slots_from_owned(&mut overlay.entity_arena, &journal.entity_slots);
        }
        if !journal.relation_slots.is_empty() {
            if overlay.relation_overlay_is_sparse {
                copied_bytes =
                    copied_bytes.saturating_add(self.relation_arena.merge_slots_from_owned(
                        &mut overlay.relation_arena,
                        &journal.relation_slots,
                    ));
            } else {
                self.relation_arena = overlay.relation_arena.clone();
            }
        }
        copied_bytes = copied_bytes.saturating_add(self.merge_adjacency_from(overlay, journal));
        self.relation_overlay_is_sparse = false;
        copied_bytes
    }

    pub(crate) fn merge_adjacency_from(
        &mut self,
        overlay: &Self,
        journal: &PartitionMutationJournal,
    ) -> u64 {
        self.adjacency
            .merge_slots_from(&overlay.adjacency, &journal.adjacency_slots)
            .saturating_add(
                self.reverse_adjacency
                    .merge_slots_from(&overlay.reverse_adjacency, &journal.reverse_adjacency_slots),
            )
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PartitionMutationJournal {
    pub(crate) entity_slots: BTreeSet<usize>,
    pub(crate) relation_slots: BTreeSet<usize>,
    pub(crate) adjacency_slots: BTreeSet<usize>,
    pub(crate) reverse_adjacency_slots: BTreeSet<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct SnapshotPartitionPins {
    pub(crate) entity_slots: DenseSlotBitSet,
    pub(crate) relation_slots: DenseSlotBitSet,
    pub(crate) retained_relation_slots: DenseSlotBitSet,
}
