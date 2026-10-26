use super::ResidentRegion;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind, SlotView};

pub(crate) struct RelationalRegionHeadView<'lease> {
    region: &'lease ResidentRegion,
}
pub(crate) struct RelationalRegionFullView<'lease> {
    region: &'lease ResidentRegion,
}

impl<'lease> RelationalRegionHeadView<'lease> {
    pub(crate) fn entity_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: EntityId,
    ) -> Option<crate::storage::data::EntityReadRecord> {
        crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_entity_record(
            schema, &self.region.partition, id.partition_id, id.slot_index(),
        ).filter(|record| id.generation.is_zero() || record.entity_id.generation == id.generation)
    }
    pub(crate) fn relation_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: RelationId,
    ) -> Option<crate::storage::data::RelationReadRecord> {
        crate::visibility::materialization::read_records::materialization::materialize_current_authoritative_relation_record(
            schema, &self.region.partition, id.partition_id, id.slot_index(),
        ).filter(|record| id.generation.is_zero() || record.relation_id.generation == id.generation)
    }
    pub(super) fn new(region: &'lease ResidentRegion) -> Self {
        Self { region }
    }
    pub(crate) fn entity(&self, id: EntityId) -> Option<SlotView<'lease, EntityRecordKind>> {
        self.region.partition.entity_arena.get(&id)
    }
    pub(crate) fn relation(&self, id: RelationId) -> Option<SlotView<'lease, RelationRecordKind>> {
        self.region.partition.relation_arena.get(&id)
    }
    pub(crate) fn outgoing(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.region
            .partition
            .adjacency
            .get(id.slot_index())
            .map(|set| set.current_ids())
            .unwrap_or_default()
    }
    pub(crate) fn incoming(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.region
            .partition
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
    pub(crate) fn commitment(&self) -> &'lease PartitionContentCommitment {
        self.region
            .commitment
            .as_ref()
            .expect("committed root has a content commitment")
    }
}
