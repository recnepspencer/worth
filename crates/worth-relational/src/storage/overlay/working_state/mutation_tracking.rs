use std::collections::BTreeMap;

use crate::identity::data::PartitionId;

use super::super::access::PartitionAccess;
use super::WorkingState;

impl WorkingState {
    pub(crate) fn mark_entity_slot_touched(&mut self, partition_id: PartitionId, slot: usize) {
        self.mutation_journal
            .entry(partition_id)
            .or_default()
            .entity_slots
            .insert(slot);
    }

    pub(crate) fn mark_relation_slot_touched(&mut self, partition_id: PartitionId, slot: usize) {
        self.mutation_journal
            .entry(partition_id)
            .or_default()
            .relation_slots
            .insert(slot);
    }

    pub(crate) fn mark_adjacency_slot_touched(&mut self, partition_id: PartitionId, slot: usize) {
        self.mutation_journal
            .entry(partition_id)
            .or_default()
            .adjacency_slots
            .insert(slot);
    }

    pub(crate) fn mark_reverse_adjacency_slot_touched(
        &mut self,
        partition_id: PartitionId,
        slot: usize,
    ) {
        self.mutation_journal
            .entry(partition_id)
            .or_default()
            .reverse_adjacency_slots
            .insert(slot);
    }

    pub(crate) fn mutation_journal(
        &self,
    ) -> &BTreeMap<PartitionId, super::super::PartitionMutationJournal> {
        &self.mutation_journal
    }

    pub(crate) fn touched_partition_count(&self) -> usize {
        self.mutation_journal.len()
    }

    pub(crate) fn entity_working_set_layout(&self) -> super::super::EntityWorkingSetLayout {
        self.entity_working_set_layout
    }

    pub(crate) fn clone_mode(&self) -> super::super::PartitionCloneMode {
        self.clone_mode
    }
}

impl PartitionAccess for WorkingState {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&super::super::PartitionState> {
        self.partitions.get(&partition_id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partitions.keys().copied().collect()
    }

    fn touched_partition_ids(&self) -> Option<Vec<PartitionId>> {
        (!self.mutation_journal.is_empty()).then(|| self.mutation_journal.keys().copied().collect())
    }

    fn touched_entity_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.mutation_journal
            .get(&partition_id)
            .map(|journal| journal.entity_slots.iter().copied().collect())
    }

    fn touched_relation_slots(&self, partition_id: PartitionId) -> Option<Vec<usize>> {
        self.mutation_journal
            .get(&partition_id)
            .map(|journal| journal.relation_slots.iter().copied().collect())
    }

    fn has_touched_entity_slots(&self, partition_id: PartitionId) -> bool {
        self.mutation_journal.contains_key(&partition_id)
    }

    fn has_touched_relation_slots(&self, partition_id: PartitionId) -> bool {
        self.mutation_journal.contains_key(&partition_id)
    }

    fn entity_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.mutation_journal
            .get(&partition_id)
            .is_some_and(|journal| journal.entity_slots.contains(&slot))
    }

    fn relation_slot_is_touched(&self, partition_id: PartitionId, slot: usize) -> bool {
        self.mutation_journal
            .get(&partition_id)
            .is_some_and(|journal| journal.relation_slots.contains(&slot))
    }
}
