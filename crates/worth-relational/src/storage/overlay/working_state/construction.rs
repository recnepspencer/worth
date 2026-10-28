use std::collections::{BTreeMap, BTreeSet};

use crate::config::data::AdjacencyPolicy;
use crate::identity::data::PartitionId;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::substrate::{EntityArena, RelationArena};

use super::super::{
    EntityWorkingSetLayout, PartitionCloneMode, PartitionMutationJournal, PartitionState,
};
use super::WorkingState;

impl WorkingState {
    #[cfg(test)]
    pub(crate) fn new(
        partitions: BTreeMap<PartitionId, PartitionState>,
        adjacency_policy: AdjacencyPolicy,
    ) -> Self {
        Self {
            adjacency_policy,
            clone_mode: PartitionCloneMode::Full,
            entity_working_set_layout: EntityWorkingSetLayout::CanonicalSoA,
            partitions,
            mutation_journal: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_touched_partitions(
        base_partitions: &BTreeMap<PartitionId, PartitionState>,
        touched_partitions: impl IntoIterator<Item = PartitionId>,
        adjacency_policy: AdjacencyPolicy,
        clone_mode: PartitionCloneMode,
    ) -> Self {
        Self::from_touched_partitions_with_layout(
            base_partitions,
            touched_partitions,
            adjacency_policy,
            clone_mode,
            EntityWorkingSetLayout::CanonicalSoA,
        )
    }

    #[cfg(test)]
    pub(crate) fn from_touched_partitions_with_layout(
        base_partitions: &BTreeMap<PartitionId, PartitionState>,
        touched_partitions: impl IntoIterator<Item = PartitionId>,
        adjacency_policy: AdjacencyPolicy,
        clone_mode: PartitionCloneMode,
        entity_working_set_layout: EntityWorkingSetLayout,
    ) -> Self {
        Self::from_touched_partitions_with_layout_and_sparse_slots(
            base_partitions,
            touched_partitions,
            adjacency_policy,
            clone_mode,
            entity_working_set_layout,
            None,
            None,
        )
    }

    /// Preparation is the sole production caller. It admits GraphSparseEntities
    /// only with Some(entity selection): an empty map, or a missing partition in
    /// that map, means no existing entity payload is required there. A declined
    /// global selection (None) must use Full instead. Direct test-only callers
    /// may construct partial states to exercise storage and validation behavior.
    pub(crate) fn from_touched_partitions_with_layout_and_sparse_slots(
        base_partitions: &(impl CurrentPartitionAccess + ?Sized),
        touched_partitions: impl IntoIterator<Item = PartitionId>,
        adjacency_policy: AdjacencyPolicy,
        clone_mode: PartitionCloneMode,
        entity_working_set_layout: EntityWorkingSetLayout,
        sparse_entity_slots: Option<&BTreeMap<PartitionId, BTreeSet<usize>>>,
        sparse_relation_overlay_partitions: Option<&BTreeSet<PartitionId>>,
    ) -> Self {
        let mut partitions = BTreeMap::new();
        let mut mutation_journal = BTreeMap::new();
        for partition_id in touched_partitions {
            if mutation_journal.contains_key(&partition_id) {
                continue;
            }
            mutation_journal.insert(partition_id, PartitionMutationJournal::default());
            if let Some(partition) = base_partitions.get_current_partition(partition_id) {
                let partition_state = partition.clone_for_mutation(
                    clone_mode,
                    sparse_entity_slots.and_then(|slots| slots.get(&partition_id)),
                    sparse_relation_overlay_partitions
                        .is_some_and(|partitions| partitions.contains(&partition_id)),
                );
                partitions.insert(partition_id, partition_state);
            }
        }
        Self {
            adjacency_policy,
            clone_mode,
            entity_working_set_layout,
            partitions,
            mutation_journal,
        }
    }

    pub(crate) fn into_partition_commits(
        self,
    ) -> (
        PartitionCloneMode,
        BTreeMap<PartitionId, (PartitionState, PartitionMutationJournal)>,
    ) {
        let WorkingState {
            clone_mode,
            partitions,
            mutation_journal,
            ..
        } = self;
        (
            clone_mode,
            partitions
                .into_iter()
                .map(|(partition_id, partition)| {
                    let journal = mutation_journal
                        .get(&partition_id)
                        .cloned()
                        .unwrap_or_default();
                    (partition_id, (partition, journal))
                })
                .collect(),
        )
    }

    pub(crate) fn get_partition_mut(&mut self, partition_id: PartitionId) -> &mut PartitionState {
        self.mutation_journal.entry(partition_id).or_default();
        self.partitions
            .entry(partition_id)
            .or_insert_with(|| PartitionState {
                partition_id,
                adjacency_policy: self.adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(0),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: Default::default(),
                reverse_adjacency: Default::default(),
            })
    }
}
