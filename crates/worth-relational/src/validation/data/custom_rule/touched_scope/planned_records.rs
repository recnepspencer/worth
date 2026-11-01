use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::EntityReference;

use super::TouchedStructuralSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEntityCreate {
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: ClientKey,
}

impl PlannedEntityCreate {
    pub(crate) fn new(partition_id: PartitionId, kind_id: KindId, client_key: ClientKey) -> Self {
        Self {
            partition_id,
            kind_id,
            client_key,
        }
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn kind_id(&self) -> KindId {
        self.kind_id
    }

    pub fn client_key(&self) -> &ClientKey {
        &self.client_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRelationCreate {
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: ClientKey,
    source: EntityReference,
    target: EntityReference,
}

impl PlannedRelationCreate {
    pub(crate) fn new(
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: ClientKey,
        source: EntityReference,
        target: EntityReference,
    ) -> Self {
        Self {
            partition_id,
            kind_id,
            client_key,
            source,
            target,
        }
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn kind_id(&self) -> KindId {
        self.kind_id
    }

    pub fn client_key(&self) -> &ClientKey {
        &self.client_key
    }

    pub fn source(&self) -> &EntityReference {
        &self.source
    }

    pub fn target(&self) -> &EntityReference {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRelationEndpointUpdate {
    relation_id: RelationId,
    kind_id: KindId,
    source: EntityReference,
    target: EntityReference,
}

impl PlannedRelationEndpointUpdate {
    pub(crate) fn new(
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
    ) -> Self {
        Self {
            relation_id,
            kind_id,
            source,
            target,
        }
    }

    pub fn relation_id(&self) -> RelationId {
        self.relation_id
    }

    pub fn kind_id(&self) -> KindId {
        self.kind_id
    }

    pub fn source(&self) -> &EntityReference {
        &self.source
    }

    pub fn target(&self) -> &EntityReference {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomInvariantTouchedSummary {
    pub visible_entity_ids: super::InvariantEntityIds,
    pub visible_relation_ids: super::InvariantRelationIds,
    pub touched_partition_ids: super::InvariantPartitionIds,
    pub planned_entity_delete_count: usize,
    pub planned_entity_create_count: usize,
    pub planned_relation_create_count: usize,
    pub planned_relation_delete_count: usize,
    pub planned_relation_endpoint_update_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructuralCountView {
    visible_entity_count: usize,
    visible_relation_count: usize,
    planned_entity_delete_count: usize,
    planned_entity_create_count: usize,
    planned_relation_create_count: usize,
    planned_relation_delete_count: usize,
    planned_relation_endpoint_update_count: usize,
    touched_partition_count: usize,
}

impl StructuralCountView {
    pub(crate) fn from_touched_scope(touched: &TouchedStructuralSet) -> Self {
        Self {
            visible_entity_count: touched.visible_entity_ids().len(),
            visible_relation_count: touched.visible_relation_ids().len(),
            planned_entity_delete_count: touched.planned_entity_deletes().len(),
            planned_entity_create_count: touched.planned_entity_creates().len(),
            planned_relation_create_count: touched.planned_relation_creates().len(),
            planned_relation_delete_count: touched.planned_relation_deletes().len(),
            planned_relation_endpoint_update_count: touched
                .planned_relation_endpoint_updates()
                .len(),
            touched_partition_count: touched.touched_partitions().len(),
        }
    }

    pub fn visible_entity_count(&self) -> usize {
        self.visible_entity_count
    }

    pub fn visible_relation_count(&self) -> usize {
        self.visible_relation_count
    }

    pub fn planned_entity_create_count(&self) -> usize {
        self.planned_entity_create_count
    }

    pub fn planned_entity_delete_count(&self) -> usize {
        self.planned_entity_delete_count
    }

    pub fn planned_relation_create_count(&self) -> usize {
        self.planned_relation_create_count
    }

    pub fn planned_relation_delete_count(&self) -> usize {
        self.planned_relation_delete_count
    }

    pub fn planned_relation_endpoint_update_count(&self) -> usize {
        self.planned_relation_endpoint_update_count
    }

    pub fn touched_partition_count(&self) -> usize {
        self.touched_partition_count
    }
}
