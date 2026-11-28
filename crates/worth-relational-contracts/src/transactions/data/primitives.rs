use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;

use super::intents::MutationIntent;
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SavepointId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerIntentBatch {
    pub name: String,
    pub partition_key: Option<String>,
    pub worker_local_only: bool,
    pub intents: Vec<MutationIntent>,
}

impl WorkerIntentBatch {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            partition_key: None,
            worker_local_only: true,
            intents: Vec::new(),
        }
    }

    pub fn with_partition_key(mut self, partition_key: impl Into<String>) -> Self {
        self.partition_key = Some(partition_key.into());
        self
    }

    pub fn push(mut self, intent: MutationIntent) -> Self {
        self.intents.push(intent);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// A typed physical record identity. Its domain stays distinct; the value
/// alone confers no read, allocation or mutation authority.
pub enum RecordRef {
    Entity(EntityId),
    Relation(RelationId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExistingRecordTarget {
    Entity(EntityId),
    Relation(RelationId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelationIdentity {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub source: EntityReference,
    pub target: EntityReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationScope {
    SamePartition,
    CrossPartition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossContextEndpointClass {
    SamePartitionEndpoints,
    CrossPartitionEndpoints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySpec {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
    pub fields: AspectFieldPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CreatedEntityRef {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
}

/// Exact owner-issued correspondence key for one relation creation intent.
///
/// Relation identity includes its declared endpoints because a relation
/// client key is only meaningful within its kind and endpoint intent. The
/// commit owner resolves this key to the allocated `RelationId`; callers must
/// not reconstruct that identity from allocator slots or read order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CreatedRelationRef {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
    pub source: EntityReference,
    pub target: EntityReference,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EntityReference {
    Existing(EntityId),
    Created(CreatedEntityRef),
}

impl EntityReference {
    pub fn partition_id(&self) -> PartitionId {
        match self {
            Self::Existing(entity_id) => entity_id.partition_id,
            Self::Created(created) => created.partition_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSpec {
    pub partition_id: PartitionId,
    pub kind_id: KindId,
    pub client_key: ClientKey,
    pub source: EntityReference,
    pub target: EntityReference,
    pub fields: AspectFieldPatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationLocalityFootprint {
    pub touched_partitions: Arc<[PartitionId]>,
    pub cross_partition_relation_count: usize,
    pub entity_target_count: usize,
    pub relation_target_count: usize,
}
