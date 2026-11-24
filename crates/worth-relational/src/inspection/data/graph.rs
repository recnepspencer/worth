use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};

use super::taxonomy::{
    InspectionAccessPath, InspectionAvailability, InspectionDegradation, InspectionOrigin,
    InspectionRecordClass, InspectionResolutionContext, InspectionScope,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphInspectionRequest {
    pub scope: InspectionScope,
    pub partition_scope: Option<Vec<PartitionId>>,
    pub relation_kind_scope: Option<Vec<KindId>>,
    pub summary_only: bool,
    pub budget: GraphInspectionBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphInspectionBudget {
    pub max_entities: u64,
    pub max_relations: u64,
    pub max_work_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindInspectionRequest {
    pub scope: InspectionScope,
    pub partition_scope: Option<Vec<PartitionId>>,
    pub kind_id: KindId,
    pub record_class: InspectionRecordClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityInspectionRequest {
    pub scope: InspectionScope,
    pub partition_scope: Option<Vec<PartitionId>>,
    pub relation_kind_scope: Option<Vec<KindId>>,
    pub include_members: bool,
    pub budget: ConnectivityInspectionBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityInspectionBudget {
    pub max_entities: u64,
    pub max_relations: u64,
    pub max_frontier: u64,
    pub max_components: u64,
    pub max_work_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct GraphInspectionSummary {
    pub scope: InspectionScope,
    pub version_id: VersionId,
    pub partition_count: u64,
    pub entity_count: u64,
    pub relation_count: u64,
    pub entity_kinds: Vec<(KindId, u64)>,
    pub relation_kinds: Vec<(KindId, u64)>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
    pub degradations: Vec<InspectionDegradation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct KindInspectionSummary {
    pub scope: InspectionScope,
    pub version_id: VersionId,
    pub kind_id: KindId,
    pub record_class: InspectionRecordClass,
    pub count: u64,
    pub touched_partitions: Vec<PartitionId>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub availability: InspectionAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct ConnectivityComponentSummary {
    pub member_count: u64,
    pub members: Option<Vec<EntityId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct ConnectivityInspectionSummary {
    pub scope: InspectionScope,
    pub version_id: VersionId,
    pub component_count: u64,
    pub largest_component_size: u64,
    pub enumerated_entity_count: u64,
    pub components: Vec<ConnectivityComponentSummary>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub resolution_context: InspectionResolutionContext,
    pub availability: InspectionAvailability,
    pub degradations: Vec<InspectionDegradation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct NeighborInspectionResult {
    pub entity_id: EntityId,
    pub version_id: VersionId,
    pub outgoing_relation_ids: Vec<RelationId>,
    pub incoming_relation_ids: Vec<RelationId>,
    pub origin: InspectionOrigin,
    pub access_path: InspectionAccessPath,
    pub resolution_context: InspectionResolutionContext,
    pub availability: InspectionAvailability,
}
