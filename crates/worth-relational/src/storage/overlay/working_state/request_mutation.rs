//! Original-request admission at the real working-state growth doors.
use super::WorkingState;
use crate::config::data::AdjacencyPolicy;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionMutationJournal, PartitionState, TouchedSlots};
use crate::storage::substrate::{EntityArena, RelationArena};
use worth_execution::ExecutionRequest;

impl WorkingState {
    pub(crate) fn retained_entity_extra(
        &self,
        id: PartitionId,
        slot: usize,
    ) -> Option<crate::storage::substrate::StorageAllocation<crate::storage::substrate::EntityExtra>>
    {
        self.partitions.get(&id)?.entity_arena.retained_extra(slot)
    }
    pub(crate) fn retained_relation_extra(
        &self,
        id: PartitionId,
        slot: usize,
    ) -> Option<
        crate::storage::substrate::StorageAllocation<crate::storage::substrate::RelationExtra>,
    > {
        self.partitions
            .get(&id)?
            .relation_arena
            .retained_extra(slot)
    }

    /// The original root's immutable numeric map is shared in O(1). Every
    /// controlled write detaches its actual selected path under the request.
    pub(crate) fn from_original_root_partitions(
        partitions: &crate::storage::substrate::SharedMap<PartitionId, PartitionState>,
        adjacency_policy: AdjacencyPolicy,
    ) -> Self {
        Self {
            adjacency_policy,
            clone_mode: crate::storage::overlay::PartitionCloneMode::Full,
            entity_working_set_layout:
                crate::storage::overlay::EntityWorkingSetLayout::CanonicalSoA,
            partitions: partitions.clone(),
            mutation_journal: Default::default(),
            // Original row replay reconstructs content separately at root admission.
            content: Default::default(),
        }
    }

    pub(crate) fn into_original_root_partitions(
        self,
    ) -> crate::storage::substrate::SharedMap<PartitionId, PartitionState> {
        self.partitions
    }

    pub(crate) fn get_partition_mut_for_request(
        &mut self,
        id: PartitionId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<&mut PartitionState, Cause> {
        self.prepare_journal(id, request)?;
        if !self.partitions.contains_key(&id) {
            self.partitions.insert_for_request(
                id,
                empty_partition(id, self.adjacency_policy.clone()),
                request,
            )?;
        }
        Ok(self
            .partitions
            .get_mut_for_request(&id, request)?
            .expect("the same exclusively borrowed map just installed this partition"))
    }

    fn prepare_journal(
        &mut self,
        id: PartitionId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<&mut PartitionMutationJournal, Cause> {
        if !self.mutation_journal.contains_key(&id) {
            self.mutation_journal
                .insert_for_request(id, Default::default(), request)?;
        }
        Ok(self
            .mutation_journal
            .get_mut_for_request(&id, request)?
            .expect("the same exclusively borrowed map just installed this journal"))
    }

    fn mark_for_request(
        &mut self,
        id: PartitionId,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
        select: fn(&mut PartitionMutationJournal) -> &mut TouchedSlots,
    ) -> Result<(), Cause> {
        select(self.prepare_journal(id, request)?).insert_for_request(slot, request)?;
        Ok(())
    }

    pub(crate) fn mark_entity_slot_touched_for_request(
        &mut self,
        id: PartitionId,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        self.mark_for_request(id, slot, request, |journal| &mut journal.entity_slots)
    }
    pub(crate) fn mark_relation_slot_touched_for_request(
        &mut self,
        id: PartitionId,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        self.mark_for_request(id, slot, request, |journal| &mut journal.relation_slots)
    }
    pub(crate) fn mark_adjacency_slot_touched_for_request(
        &mut self,
        id: PartitionId,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        self.mark_for_request(id, slot, request, |journal| &mut journal.adjacency_slots)
    }
    pub(crate) fn mark_reverse_adjacency_slot_touched_for_request(
        &mut self,
        id: PartitionId,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        self.mark_for_request(id, slot, request, |journal| {
            &mut journal.reverse_adjacency_slots
        })
    }
}

fn empty_partition(id: PartitionId, adjacency_policy: AdjacencyPolicy) -> PartitionState {
    PartitionState {
        partition_id: id,
        adjacency_policy,
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}
