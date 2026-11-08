use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId};

use super::InvariantStateView;

impl<'state> InvariantStateView<'state> {
    /// O(log slots + log kinds + degree(kind)) navigation, plus the ordered
    /// answer's insertion work/backing. The kind memberships are carried by
    /// storage's synchronous Maintained owner, never reprojected by this read.
    pub(crate) fn try_kind_relation_candidates<Stop>(
        &self,
        entity: EntityId,
        kind: crate::identity::data::KindId,
        direction: crate::storage::partition::AdjacencyDirection,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<BTreeSet<RelationId>, Stop> {
        let mut relations: BTreeSet<RelationId> = BTreeSet::new();
        for partition in [
            self.state().get_current_partition(entity.partition_id),
            self.state().base_current_partition(entity.partition_id),
        ]
        .into_iter()
        .flatten()
        {
            let table = partition.adjacency(direction);
            prepare(table.navigation_work_bound() as u64, 0)?;
            if let Some(set) = table.get(entity.slot_index()) {
                prepare(set.navigation_work_bound() as u64, 0)?;
                let mut ids = set.kind_ids(kind).try_iter(&mut prepare)?;
                while let Some(&id) = ids.try_next(&mut prepare)? {
                    prepare(
                        1 + u64::from((relations.len() + 1).ilog2()),
                        (std::mem::size_of::<RelationId>() + 4 * std::mem::size_of::<usize>())
                            as u64,
                    )?;
                    relations.insert(id);
                }
            }
        }
        Ok(relations)
    }

    pub(crate) fn outgoing_relations_for_entity(&self, entity_id: EntityId) -> Vec<RelationId> {
        match self.outgoing_relations_for_entity_observed(entity_id, || {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(ids) => ids,
            Err(never) => match never {},
        }
    }

    pub(crate) fn incoming_relations_for_entity(&self, entity_id: EntityId) -> Vec<RelationId> {
        match self.incoming_relations_for_entity_observed(entity_id, || {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(ids) => ids,
            Err(never) => match never {},
        }
    }

    pub(crate) fn outgoing_relations_for_entity_observed<E>(
        &self,
        entity: EntityId,
        acquired: impl FnMut() -> Result<(), E>,
    ) -> Result<Vec<RelationId>, E> {
        self.relations_for_entity_observed(entity, true, acquired)
    }

    pub(crate) fn incoming_relations_for_entity_observed<E>(
        &self,
        entity: EntityId,
        acquired: impl FnMut() -> Result<(), E>,
    ) -> Result<Vec<RelationId>, E> {
        self.relations_for_entity_observed(entity, false, acquired)
    }

    fn relations_for_entity_observed<E>(
        &self,
        entity: EntityId,
        outgoing: bool,
        mut acquired: impl FnMut() -> Result<(), E>,
    ) -> Result<Vec<RelationId>, E> {
        let mut ids = Vec::new();
        for &id in self.relation_candidates(entity, outgoing).iter() {
            let Some(metadata) = self.relation_metadata(id) else {
                continue;
            };
            acquired()?;
            if if outgoing {
                metadata.source == entity
            } else {
                metadata.target == entity
            } {
                ids.push(id);
            }
        }
        Ok(ids)
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

    pub(crate) fn relation_candidates(
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
            self.state().get_current_partition(entity_id.partition_id),
            self.state().base_current_partition(entity_id.partition_id),
        ]
        .into_iter()
        .flatten()
        {
            let candidates = if outgoing {
                partition.outgoing_candidates(entity_id)
            } else {
                partition.incoming_candidates(entity_id)
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
            self.state().get_current_partition(entity_id.partition_id),
            self.state().base_current_partition(entity_id.partition_id),
        ]
        .into_iter()
        .flatten()
        .fold(0usize, |count, partition| {
            let candidates = if outgoing {
                partition.outgoing_candidates(entity_id)
            } else {
                partition.incoming_candidates(entity_id)
            };
            count.saturating_add(candidates.len())
        })
    }
}
