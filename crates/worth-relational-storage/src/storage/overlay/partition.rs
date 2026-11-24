use std::collections::BTreeSet;
pub mod field_corruption;
pub mod storage_headers;
pub mod touched_slots;
pub use touched_slots::TouchedSlots;

use crate::config::data::AdjacencyPolicy;
use crate::identity::data::PartitionId;

use crate::storage::partition::DenseSlotBitSet;
use crate::storage::substrate::{EntityArena, RelationArena};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionCloneMode {
    Full,
    EntityOnly,
    GraphSparseEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityWorkingSetLayout {
    CanonicalSoA,
    AoSoACandidate { chunk_width: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EntityChunkPlanSummary {
    pub chunk_width: usize,
    pub chunk_count: usize,
    pub slot_count: usize,
}

pub fn summarize_entity_chunk_plan(
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
pub struct PartitionState {
    pub partition_id: PartitionId,
    pub adjacency_policy: AdjacencyPolicy,
    pub relation_overlay_is_sparse: bool,
    pub entity_arena: EntityArena,
    pub relation_arena: RelationArena,
    pub adjacency: crate::storage::partition::SparseAdjacencyTable,
    pub reverse_adjacency: crate::storage::partition::SparseAdjacencyTable,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalPartitionAllocationInventory {
    pub authoritative_bytes: u64,
    pub diagnostic_bytes: u64,
    pub retention_metadata_bytes: u64,
    pub allocator_bookkeeping_bytes: u64,
    pub optional_cache_bytes: u64,
}

impl PartitionState {
    pub fn clear_runtime_pin_counters(&mut self) {
        self.entity_arena.clear_all_pins();
        self.relation_arena.clear_all_pins();
    }

    pub fn allocation_inventory(
        &self,
    ) -> Result<
        RelationalPartitionAllocationInventory,
        crate::storage::substrate::StorageAllocationOverflow,
    > {
        use crate::storage::substrate::StorageAllocationOverflow;
        let authoritative_bytes = self.unique_authoritative_allocation_bytes()?;
        let adjacency_cache_bytes = self
            .adjacency
            .iter()
            .chain(self.reverse_adjacency.iter())
            .try_fold(0_u64, |total, (_, set)| {
                total.checked_add(set.optional_cache_allocation_bytes())
            })
            .ok_or(StorageAllocationOverflow)?;
        let arenas = self
            .entity_arena
            .non_authoritative_allocation_inventory()?
            .checked_add(
                self.relation_arena
                    .non_authoritative_allocation_inventory()?,
            )?;
        Ok(RelationalPartitionAllocationInventory {
            authoritative_bytes,
            diagnostic_bytes: arenas.diagnostic_bytes,
            retention_metadata_bytes: arenas.retention_metadata_bytes,
            allocator_bookkeeping_bytes: arenas.allocator_bookkeeping_bytes,
            optional_cache_bytes: adjacency_cache_bytes,
        })
    }

    pub fn clone_for_overlay(&self, mode: PartitionCloneMode) -> Self {
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

    pub fn clone_entity_slots_for_overlay(&self, touched_entity_slots: &BTreeSet<usize>) -> Self {
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

    pub fn clone_graph_with_sparse_entity_slots(
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

    pub fn clone_graph_with_sparse_entity_slots_and_relation_shape(
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

    pub fn clone_graph_with_sparse_relation_shape(&self) -> Self {
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
    pub fn merge_overlay_from_owned(
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

    pub fn merge_adjacency_from(
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
pub struct PartitionMutationJournal {
    /// The predecessor graph was omitted by the selected clone mode.
    pub adjacency_omitted: bool,
    pub entity_slots: TouchedSlots,
    pub relation_slots: TouchedSlots,
    pub adjacency_slots: TouchedSlots,
    pub reverse_adjacency_slots: TouchedSlots,
}

#[derive(Debug, Clone)]
pub struct SnapshotPartitionPins {
    pub entity_slots: DenseSlotBitSet,
    pub relation_slots: DenseSlotBitSet,
    pub retained_relation_slots: DenseSlotBitSet,
}
