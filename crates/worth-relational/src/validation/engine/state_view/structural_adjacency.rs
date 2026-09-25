use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId};
use crate::storage::partition::adjacency_queries;

use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    pub(crate) fn outgoing_relations_for_entity(&self, entity_id: EntityId) -> Vec<RelationId> {
        self.relation_candidates(entity_id, true)
            .iter()
            .copied()
            .filter(|relation_id| {
                self.relation_metadata(*relation_id)
                    .is_some_and(|metadata| metadata.source == entity_id)
            })
            .collect()
    }

    pub(crate) fn incoming_relations_for_entity(&self, entity_id: EntityId) -> Vec<RelationId> {
        self.relation_candidates(entity_id, false)
            .iter()
            .copied()
            .filter(|relation_id| {
                self.relation_metadata(*relation_id)
                    .is_some_and(|metadata| metadata.target == entity_id)
            })
            .collect()
    }

    pub(crate) fn all_relations_for_entity(&self, entity_id: EntityId) -> Vec<RelationId> {
        let mut relation_ids = BTreeSet::new();
        for relation_id in self
            .relation_candidates(entity_id, true)
            .iter()
            .copied()
            .chain(self.relation_candidates(entity_id, false).iter().copied())
        {
            let Some(metadata) = self.relation_metadata(relation_id) else {
                continue;
            };
            if metadata.source == entity_id || metadata.target == entity_id {
                relation_ids.insert(relation_id);
            }
        }
        relation_ids.into_iter().collect()
    }

    fn relation_candidates(
        &self,
        entity_id: EntityId,
        outgoing: bool,
    ) -> std::sync::Arc<[RelationId]> {
        if let Some((inputs, basis)) = self.candidate_inputs() {
            return inputs.adjacency(*basis, self.version_id(), entity_id, outgoing, || {
                self.gather_relation_candidates(entity_id, outgoing)
            });
        }
        self.gather_relation_candidates(entity_id, outgoing).into()
    }

    fn gather_relation_candidates(&self, entity_id: EntityId, outgoing: bool) -> Vec<RelationId> {
        let mut relation_ids = BTreeSet::new();
        for partition in [
            self.state().get_partition(entity_id.partition_id),
            self.state().base_partition(entity_id.partition_id),
        ]
        .into_iter()
        .flatten()
        {
            let candidates = if outgoing {
                adjacency_queries::outgoing_relation_candidates_from_state(
                    &SinglePartitionAccess::new(partition),
                    entity_id,
                )
            } else {
                adjacency_queries::incoming_relation_candidates_from_state(
                    &SinglePartitionAccess::new(partition),
                    entity_id,
                )
            };
            relation_ids.extend(candidates);
        }
        relation_ids.into_iter().collect()
    }

    pub(crate) fn relation_candidate_count(&self, entity_id: EntityId, outgoing: bool) -> usize {
        if let Some((inputs, basis)) = self.candidate_inputs() {
            return inputs.adjacency_count(*basis, self.version_id(), entity_id, outgoing, || {
                self.read_relation_candidate_count(entity_id, outgoing)
            });
        }
        self.read_relation_candidate_count(entity_id, outgoing)
    }

    fn read_relation_candidate_count(&self, entity_id: EntityId, outgoing: bool) -> usize {
        [
            self.state().get_partition(entity_id.partition_id),
            self.state().base_partition(entity_id.partition_id),
        ]
        .into_iter()
        .flatten()
        .fold(0usize, |count, partition| {
            let table = if outgoing {
                &partition.adjacency
            } else {
                &partition.reverse_adjacency
            };
            count.saturating_add(
                table
                    .get(entity_id.slot_index())
                    .map_or(0, |relations| relations.current_ids().len()),
            )
        })
    }
}

struct SinglePartitionAccess<'partition> {
    partition: &'partition crate::storage::overlay::PartitionState,
}

impl<'partition> SinglePartitionAccess<'partition> {
    fn new(partition: &'partition crate::storage::overlay::PartitionState) -> Self {
        Self { partition }
    }
}

impl crate::storage::overlay::PartitionAccess for SinglePartitionAccess<'_> {
    fn get_partition(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<&crate::storage::overlay::PartitionState> {
        (self.partition.partition_id == partition_id).then_some(self.partition)
    }

    fn partition_ids(&self) -> Vec<crate::identity::data::PartitionId> {
        vec![self.partition.partition_id]
    }
}
