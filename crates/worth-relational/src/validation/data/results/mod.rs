mod custom_invariant_failure;
mod storage_inconsistency;
#[cfg(test)]
mod tests;
mod witness_keys;

use serde::{Deserialize, Serialize};

use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::schema::data::{
    ContractId, EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope,
};
use crate::transactions::data::EntityReference;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};

use super::execution::InvariantClass;
use super::execution::InvariantWitnessKey;
pub use custom_invariant_failure::{
    CustomInvariantFailureIdentity, CustomInvariantFailureKind, CustomInvariantFailurePhase,
};
pub use storage_inconsistency::{
    StorageInconsistencyFailure, StorageInconsistencyLookup, StorageInconsistencyScan,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantViolationFields {
    None,
    MergedIntentLimit {
        merged_intent_count: usize,
        limit: usize,
    },
    SnapshotEntityLimit {
        version_id: VersionId,
        visible_entities: usize,
        limit: usize,
    },
    UniqueEntityField {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        field_locator: AspectFieldLocator,
        value: AspectValue,
    },
    SidecarConsistency {
        partition_id: PartitionId,
        slot: usize,
        missing_label: String,
    },
    RelationEndpointKindMismatch {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
        source_kind_id: KindId,
        target_kind_id: KindId,
        boundary: RelationEndpointBoundary,
    },
    RelationEndpointKindSelfEdge {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
        self_edge: bool,
    },
    RelationEndpointKindCrossContext {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source_partition_id: PartitionId,
        target_partition_id: PartitionId,
    },
    RelationCardinalityEndpoint {
        contract_id: ContractId,
        relation_kind_id: KindId,
        entity_id: EntityReference,
        boundary: RelationCardinalityBoundary,
        count: usize,
        limit: u64,
    },
    RelationCardinalityPair {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
        count: usize,
        limit: u64,
    },
    RelationUniqueness {
        contract_id: ContractId,
        relation_kind_id: KindId,
        scope: UniquenessScope,
        source: EntityReference,
        target: EntityReference,
        count: usize,
    },
    RelationSymmetry {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
        mode: SymmetryMode,
    },
    RelationEndpointDeletionIntegrity {
        contract_id: ContractId,
        relation_kind_id: KindId,
        entity_id: EntityId,
        remaining_relation_endpoint_count: usize,
        mode: EndpointDeletionIntegrityMode,
        cascade_delete_policy: Option<CascadeDeletePolicy>,
    },
    StorageInconsistency {
        entity_id: Option<EntityId>,
        partition_id: Option<PartitionId>,
        slot: Option<usize>,
        field: Option<FieldKey>,
        missing_label: Option<String>,
        scan: Option<StorageInconsistencyScan>,
        lookup: Option<StorageInconsistencyLookup>,
        failure: Option<StorageInconsistencyFailure>,
    },
    RelationIntegrityScopeBudgetExceeded {
        limit_name: String,
        limit: usize,
        observed: usize,
        relation_kind_count: usize,
        touched_entity_count: usize,
        deleted_entity_count: usize,
        scanned_relation_count: usize,
        planned_edge_count: usize,
    },
    CustomInvariantFailure {
        identity: CustomInvariantFailureIdentity,
        phase: CustomInvariantFailurePhase,
        failure: CustomInvariantFailureKind,
        detail: String,
    },
    CustomInvariantViolation {
        identity: crate::validation::data::CustomInvariantSemanticIdentity,
    },
    PartitionIsolation {
        contract_id: ContractId,
        relation_kind_id: KindId,
        relation_id: Option<crate::identity::data::RelationId>,
        source_partition_id: PartitionId,
        target_partition_id: PartitionId,
    },
    Acyclicity {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        target: EntityReference,
    },
    ConnectivityMinimum {
        contract_id: ContractId,
        relation_kind_id: KindId,
        source: EntityReference,
        reachable_target_count: usize,
        minimum_reachable_targets: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationEndpointBoundary {
    Source,
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationCardinalityBoundary {
    Source,
    Target,
    Pair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub class: InvariantClass,
    pub code: crate::diagnostics::data::DiagnosticCode,
    pub detail: String,
    pub fields: InvariantViolationFields,
}

impl InvariantViolation {
    pub fn witness_key(&self) -> InvariantWitnessKey {
        witness_keys::invariant_violation_witness_key(self.code, &self.fields)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantAdvisory {
    AuditOnly,
}
