mod mutation_clone;
use super::ResidentRegion;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind, SlotView};

#[derive(Clone, Copy)]
pub(crate) struct RelationalRegionHeadView<'lease> {
    partition: &'lease PartitionState,
}
pub(crate) struct RelationalRegionFullView<'lease> {
    region: &'lease ResidentRegion,
}

impl<'lease> RelationalRegionHeadView<'lease> {
    pub(crate) fn partition_id(&self) -> crate::identity::data::PartitionId {
        self.partition.partition_id
    }
    pub(crate) fn arena<K: crate::storage::substrate::RecordKind>(
        &self,
    ) -> super::RelationalRecordHeadArenaView<'lease, K> {
        super::RelationalRecordHeadArenaView::new(K::arena(self.partition))
    }
    pub(crate) fn relation_overlay_is_sparse(&self) -> bool {
        self.partition.relation_overlay_is_sparse
    }
    pub(crate) fn entity_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: EntityId,
    ) -> Option<crate::storage::data::EntityReadRecord> {
        crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_entity_record(
            schema, self.partition, id.partition_id, id.slot_index(),
        ).filter(|record| id.generation.is_zero() || record.entity_id.generation == id.generation)
    }
    pub(crate) fn relation_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: RelationId,
    ) -> Option<crate::storage::data::RelationReadRecord> {
        crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_relation_record(
            schema, self.partition, id.partition_id, id.slot_index(),
        ).filter(|record| id.generation.is_zero() || record.relation_id.generation == id.generation)
    }
    pub(super) fn new(region: &'lease ResidentRegion) -> Self {
        Self {
            partition: &region.partition,
        }
    }
    pub(crate) fn from_owned_partition(partition: &'lease PartitionState) -> Self {
        Self { partition }
    }
    pub(crate) fn entity_arena(
        &self,
    ) -> super::RelationalRecordHeadArenaView<'lease, EntityRecordKind> {
        super::RelationalRecordHeadArenaView::new(&self.partition.entity_arena)
    }
    pub(crate) fn relation_arena(
        &self,
    ) -> super::RelationalRecordHeadArenaView<'lease, RelationRecordKind> {
        super::RelationalRecordHeadArenaView::new(&self.partition.relation_arena)
    }
    pub(crate) fn entity(&self, id: EntityId) -> Option<SlotView<'lease, EntityRecordKind>> {
        self.partition.entity_arena.get(&id)
    }
    pub(crate) fn relation(&self, id: RelationId) -> Option<SlotView<'lease, RelationRecordKind>> {
        self.partition.relation_arena.get(&id)
    }
    pub(crate) fn outgoing(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.outgoing_candidates(id)
    }
    pub(crate) fn outgoing_candidates(
        &self,
        id: EntityId,
    ) -> crate::storage::partition::AdjacencyIds<'lease> {
        self.partition
            .adjacency
            .get(id.slot_index())
            .map(|set| set.current_ids())
            .unwrap_or_default()
    }
    pub(crate) fn incoming(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.incoming_candidates(id)
    }
    pub(crate) fn incoming_candidates(
        &self,
        id: EntityId,
    ) -> crate::storage::partition::AdjacencyIds<'lease> {
        self.partition
            .reverse_adjacency
            .get(id.slot_index())
            .map(|set| set.current_ids())
            .unwrap_or_default()
    }
}

impl<'lease> RelationalRegionFullView<'lease> {
    pub(super) fn new(region: &'lease ResidentRegion) -> Self {
        Self { region }
    }
    pub(crate) fn partition(&self) -> &'lease PartitionState {
        &self.region.partition
    }
    #[cfg(test)]
    pub(super) fn commitment(&self) -> &'lease crate::storage::overlay::PartitionContentCommitment {
        self.region
            .commitment
            .as_ref()
            .expect("committed root has a content commitment")
    }
}
