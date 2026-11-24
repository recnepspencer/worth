use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId};

use super::StructuralStateView;

impl StructuralStateView<'_> {
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

    pub(crate) fn gather_relation_candidates(
        &self,
        entity_id: EntityId,
        outgoing: bool,
    ) -> Vec<RelationId> {
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
