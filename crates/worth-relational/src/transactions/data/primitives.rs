use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::symbols::data::ClientKey;

use super::intents::MutationIntent;
use super::AspectFieldPatch;

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkMutationScope {
    BulkEntityCreate,
    BulkRelationCreate,
    BulkMixedMutation,
    TopologyRegionRewrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationLocalityFootprint {
    pub touched_partitions: Arc<[PartitionId]>,
    pub cross_partition_relation_count: usize,
    pub entity_target_count: usize,
    pub relation_target_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationNamingPlan {
    pub normalized_client_keys: Arc<[ClientKey]>,
    pub naming_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlannedLineageTransition {
    CreateEntity {
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: ClientKey,
    },
    ReplaceEntity {
        entity_id: EntityId,
        replacement_partition_id: PartitionId,
        replacement_kind_id: KindId,
        replacement_client_key: ClientKey,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        partition_id: PartitionId,
        kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
        client_key: ClientKey,
    },
    UpdateRelationEndpoints {
        relation_id: RelationId,
        source: EntityReference,
        target: EntityReference,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationLineagePlan {
    pub transitions: Arc<[PlannedLineageTransition]>,
    pub lineage_scope_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkMutationProvenancePlan {
    pub batch_name: String,
    pub target_branch: Option<BranchId>,
    pub worker_batch_names: Arc<[String]>,
    pub worker_partition_keys: Arc<[Option<String>]>,
    pub worker_local_only_flags: Arc<[bool]>,
    pub provenance_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedBulkMutationBatch {
    pub transaction_id: TransactionId,
    pub scope: BulkMutationScope,
    pub locality: BulkMutationLocalityFootprint,
    pub naming: BulkMutationNamingPlan,
    pub lineage: BulkMutationLineagePlan,
    pub provenance: BulkMutationProvenancePlan,
    pub intents: Arc<[MutationIntent]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamingStableBulkMutationBatch {
    planned: PlannedBulkMutationBatch,
    proof_token: BulkMutationAdmissionToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageSafeBulkMutationBatch {
    naming_stable: NamingStableBulkMutationBatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceCompleteBulkMutationBatch {
    lineage_safe: LineageSafeBulkMutationBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct BulkMutationAdmissionToken;

impl NamingStableBulkMutationBatch {
    pub fn planned(&self) -> &PlannedBulkMutationBatch {
        &self.planned
    }
}

impl LineageSafeBulkMutationBatch {
    pub fn naming_stable(&self) -> &NamingStableBulkMutationBatch {
        &self.naming_stable
    }

    pub fn planned(&self) -> &PlannedBulkMutationBatch {
        self.naming_stable.planned()
    }
}

impl ProvenanceCompleteBulkMutationBatch {
    pub fn lineage_safe(&self) -> &LineageSafeBulkMutationBatch {
        &self.lineage_safe
    }

    pub fn planned(&self) -> &PlannedBulkMutationBatch {
        self.lineage_safe.planned()
    }
}

pub(crate) fn naming_stable_bulk_mutation_batch(
    planned: PlannedBulkMutationBatch,
) -> NamingStableBulkMutationBatch {
    NamingStableBulkMutationBatch {
        planned,
        proof_token: BulkMutationAdmissionToken,
    }
}

pub(crate) fn lineage_safe_bulk_mutation_batch(
    naming_stable: NamingStableBulkMutationBatch,
) -> LineageSafeBulkMutationBatch {
    LineageSafeBulkMutationBatch { naming_stable }
}

pub(crate) fn provenance_complete_bulk_mutation_batch(
    lineage_safe: LineageSafeBulkMutationBatch,
) -> ProvenanceCompleteBulkMutationBatch {
    ProvenanceCompleteBulkMutationBatch { lineage_safe }
}
