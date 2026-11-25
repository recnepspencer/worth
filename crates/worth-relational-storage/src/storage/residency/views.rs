pub mod consequence_materialization;
pub mod mutation_clone;
use super::ResidentRegion;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind, SlotView};

#[derive(Clone, Copy)]
pub struct RelationalRegionHeadView<'lease> {
    partition: &'lease PartitionState,
}
pub struct RelationalRegionFullView<'lease> {
    region: &'lease ResidentRegion,
}

impl<'lease> RelationalRegionHeadView<'lease> {
    pub fn adjacency(
        self,
        direction: crate::storage::partition::AdjacencyDirection,
    ) -> super::RelationalHeadAdjacencyTable<'lease> {
        super::RelationalHeadAdjacencyTable::new(direction.table(self.partition))
    }

    pub fn partition_id(&self) -> crate::identity::data::PartitionId {
        self.partition.partition_id
    }
    pub fn arena<K: crate::storage::substrate::RecordKind>(
        &self,
    ) -> super::RelationalRecordHeadArenaView<'lease, K> {
        super::RelationalRecordHeadArenaView::new(K::arena(self.partition))
    }
    pub fn relation_overlay_is_sparse(&self) -> bool {
        self.partition.relation_overlay_is_sparse
    }
    pub fn entity_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: EntityId,
    ) -> Option<crate::storage::data::EntityReadRecord> {
        crate::storage::partition::current_records::materialize_current_authoritative_entity_record(
            schema,
            self.partition,
            id.partition_id,
            id.slot_index(),
        )
        .filter(|record| id.generation.is_zero() || record.entity_id.generation == id.generation)
    }
    pub fn relation_record(
        &self,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        id: RelationId,
    ) -> Option<crate::storage::data::RelationReadRecord> {
        crate::storage::partition::current_records::materialize_current_authoritative_relation_record(
            schema, self.partition, id.partition_id, id.slot_index(),
        ).filter(|record| id.generation.is_zero() || record.relation_id.generation == id.generation)
    }
    pub fn new(region: &'lease ResidentRegion) -> Self {
        Self {
            partition: &region.partition,
        }
    }
    pub fn from_owned_partition(partition: &'lease PartitionState) -> Self {
        Self { partition }
    }
    /// The stored partition, for an owner restoring rows from this image.
    pub(crate) fn partition_state(&self) -> &'lease PartitionState {
        self.partition
    }
    pub fn same_region_view(&self, other: Self) -> bool {
        std::ptr::eq(self.partition, other.partition)
    }

    pub fn entity_arena(&self) -> super::RelationalRecordHeadArenaView<'lease, EntityRecordKind> {
        super::RelationalRecordHeadArenaView::new(&self.partition.entity_arena)
    }
    pub fn relation_arena(
        &self,
    ) -> super::RelationalRecordHeadArenaView<'lease, RelationRecordKind> {
        super::RelationalRecordHeadArenaView::new(&self.partition.relation_arena)
    }
    pub fn entity(&self, id: EntityId) -> Option<SlotView<'lease, EntityRecordKind>> {
        self.partition.entity_arena.get(&id)
    }
    pub fn relation(&self, id: RelationId) -> Option<SlotView<'lease, RelationRecordKind>> {
        self.partition.relation_arena.get(&id)
    }
    pub fn outgoing(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.outgoing_candidates(id)
    }
    pub fn outgoing_candidates(
        &self,
        id: EntityId,
    ) -> crate::storage::partition::AdjacencyIds<'lease> {
        self.partition
            .adjacency
            .get(id.slot_index())
            .map(|set| set.current_ids())
            .unwrap_or_default()
    }
    pub fn incoming(&self, id: EntityId) -> crate::storage::partition::AdjacencyIds<'lease> {
        if self.entity(id).is_none() {
            return Default::default();
        }
        self.incoming_candidates(id)
    }
    pub fn incoming_candidates(
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
    pub fn partition(&self) -> &'lease PartitionState {
        &self.region.partition
    }
    pub(super) fn commitment(&self) -> &'lease crate::storage::overlay::PartitionContentCommitment {
        self.region
            .commitment
            .as_ref()
            .expect("committed root has a content commitment")
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait RelationalRegionFullViewFixturePort<'lease>: Sized {
    fn full_view_commitment_for_test(
        &self,
    ) -> &'lease crate::storage::overlay::PartitionContentCommitment;
}

impl<'lease> RelationalRegionFullViewFixturePort<'lease> for RelationalRegionFullView<'lease> {
    fn full_view_commitment_for_test(
        &self,
    ) -> &'lease crate::storage::overlay::PartitionContentCommitment {
        self.commitment()
    }
}
