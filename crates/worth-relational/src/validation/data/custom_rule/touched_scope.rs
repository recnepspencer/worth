mod planned_records;

pub use planned_records::{
    CustomInvariantTouchedSummary, PlannedRelationEndpointUpdate, StructuralCountView,
};
pub(crate) use planned_records::{PlannedEntityCreate, PlannedRelationCreate};

use std::sync::Arc;

use crate::identity::data::{EntityId, PartitionId, RelationId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedStructuralSet {
    direct_visible_entity_ids: Arc<[EntityId]>,
    visible_entity_ids: Arc<[EntityId]>,
    visible_relation_ids: Arc<[RelationId]>,
    touched_partitions: Arc<[PartitionId]>,
    planned_entity_deletes: Arc<[EntityId]>,
    planned_entity_creates: Arc<[PlannedEntityCreate]>,
    planned_relation_creates: Arc<[PlannedRelationCreate]>,
    planned_relation_deletes: Arc<[RelationId]>,
    planned_relation_endpoint_updates: Arc<[PlannedRelationEndpointUpdate]>,
}

impl TouchedStructuralSet {
    pub(crate) fn new(
        direct_visible_entity_ids: Arc<[EntityId]>,
        visible_entity_ids: Arc<[EntityId]>,
        visible_relation_ids: Arc<[RelationId]>,
        touched_partitions: Arc<[PartitionId]>,
        planned_entity_deletes: Arc<[EntityId]>,
        planned_entity_creates: Arc<[PlannedEntityCreate]>,
        planned_relation_creates: Arc<[PlannedRelationCreate]>,
        planned_relation_deletes: Arc<[RelationId]>,
        planned_relation_endpoint_updates: Arc<[PlannedRelationEndpointUpdate]>,
    ) -> Self {
        Self {
            direct_visible_entity_ids,
            visible_entity_ids,
            visible_relation_ids,
            touched_partitions,
            planned_entity_deletes,
            planned_entity_creates,
            planned_relation_creates,
            planned_relation_deletes,
            planned_relation_endpoint_updates,
        }
    }

    /// Entities directly named by the mutation, including both old and new
    /// endpoints of changed relations. Adjacency-only traversal neighbors are
    /// excluded.
    pub fn direct_visible_entity_ids(&self) -> &[EntityId] {
        &self.direct_visible_entity_ids
    }

    pub fn visible_entity_ids(&self) -> &[EntityId] {
        &self.visible_entity_ids
    }

    pub fn visible_relation_ids(&self) -> &[RelationId] {
        &self.visible_relation_ids
    }

    pub fn touched_partitions(&self) -> &[PartitionId] {
        &self.touched_partitions
    }

    pub fn planned_entity_deletes(&self) -> &[EntityId] {
        &self.planned_entity_deletes
    }

    pub fn planned_entity_creates(&self) -> &[PlannedEntityCreate] {
        &self.planned_entity_creates
    }

    pub fn planned_relation_creates(&self) -> &[PlannedRelationCreate] {
        &self.planned_relation_creates
    }

    pub fn planned_relation_deletes(&self) -> &[RelationId] {
        &self.planned_relation_deletes
    }

    pub fn planned_relation_endpoint_updates(&self) -> &[PlannedRelationEndpointUpdate] {
        &self.planned_relation_endpoint_updates
    }

    pub(crate) fn provenance_summary(&self) -> CustomInvariantTouchedSummary {
        CustomInvariantTouchedSummary {
            visible_entity_ids: self.visible_entity_ids.clone(),
            visible_relation_ids: self.visible_relation_ids.clone(),
            touched_partition_ids: self.touched_partitions.clone(),
            planned_entity_delete_count: self.planned_entity_deletes.len(),
            planned_entity_create_count: self.planned_entity_creates.len(),
            planned_relation_create_count: self.planned_relation_creates.len(),
            planned_relation_delete_count: self.planned_relation_deletes.len(),
            planned_relation_endpoint_update_count: self.planned_relation_endpoint_updates.len(),
        }
    }
}
