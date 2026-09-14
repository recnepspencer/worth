use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::{DescriptorSemanticsVersion, SchemaVersionId};
use crate::transactions::data::{AspectDeltaFailureFields, ExistingRecordTarget, SavepointId};
use crate::validation::data::InvariantViolationFields;
use serde::{Deserialize, Serialize};

mod aspect_field_target_rejection;
mod authoritative_aspect_source_locator;
mod entity_authoritative_aspect_state_denial;
mod entity_authoritative_aspect_state_presentation;
mod mutation_state_inconsistency;
mod record_allocation_denial;
mod record_aspect_patch_denial;
pub use aspect_field_target_rejection::AspectFieldTargetRejectionReason;
pub use entity_authoritative_aspect_state_denial::EntityAuthoritativeAspectStateDenial;
pub use mutation_state_inconsistency::{
    BulkMutationAdmissionDenial, EntityCascadeDeleteMissingState,
    MutationStateInconsistencyEvidence,
};
pub use record_allocation_denial::RecordAllocationDenial;
pub use record_aspect_patch_denial::{RecordAspectPatchDenial, RecordAspectPatchTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityFieldUpdateMissingState {
    Partition,
    Slot,
    KindId,
    AuthoritativeAspectState,
}

impl EntityFieldUpdateMissingState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Partition => "partition",
            Self::Slot => "slot",
            Self::KindId => "kind_id",
            Self::AuthoritativeAspectState => "authoritative aspect state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityUpdateMissingState {
    Partition,
    Slot,
    KindId,
    AuthoritativeAspectState,
}

impl EntityUpdateMissingState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Partition => "partition",
            Self::Slot => "slot",
            Self::KindId => "kind_id",
            Self::AuthoritativeAspectState => "authoritative aspect state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationEndpointUpdateMissingState {
    Partition,
    Slot,
    KindId,
    Endpoints,
}

impl RelationEndpointUpdateMissingState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Partition => "partition",
            Self::Slot => "slot",
            Self::KindId => "kind_id",
            Self::Endpoints => "endpoints",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkImportRowDomain {
    Entity,
    Relation,
}

impl BulkImportRowDomain {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relation => "relation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkImportStage {
    EntityCreate,
    RelationCreate,
}

impl BulkImportStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EntityCreate => "bulk_entity_stage_import",
            Self::RelationCreate => "bulk_relation_stage_import",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictClass {
    StaleTarget {
        target: ExistingRecordTarget,
        context: String,
    },
    InvalidRelationEndpoint {
        detail: String,
    },
    DuplicateRelationIdentity {
        detail: String,
    },
    InvariantViolation {
        code: DiagnosticCode,
        detail: String,
        fields: InvariantViolationFields,
    },
    KindSchemaMismatch {
        detail: String,
    },
    MutationStateInconsistency {
        detail: String,
        evidence: MutationStateInconsistencyEvidence,
    },
    RecordAllocationDenied {
        denial: RecordAllocationDenial,
    },
    EntityUpdateStateInconsistency {
        entity_id: crate::identity::data::EntityId,
        missing: EntityUpdateMissingState,
    },
    EntityFieldUpdateStateInconsistency {
        entity_id: crate::identity::data::EntityId,
        missing: EntityFieldUpdateMissingState,
    },
    RelationEndpointUpdateStateInconsistency {
        relation_id: crate::identity::data::RelationId,
        missing: RelationEndpointUpdateMissingState,
    },
    RelationEndpointUpdateKindMismatch {
        relation_id: crate::identity::data::RelationId,
        intent_kind_id: crate::identity::data::KindId,
        authoritative_kind_id: crate::identity::data::KindId,
    },
    BulkImportDomainMismatch {
        expected: BulkImportRowDomain,
        actual: BulkImportRowDomain,
        stage: BulkImportStage,
    },
    EntityAuthoritativeAspectStateDenied {
        kind_id: crate::identity::data::KindId,
        denial: EntityAuthoritativeAspectStateDenial,
    },
    RecordAspectPatchDenied {
        target: RecordAspectPatchTarget,
        denial: RecordAspectPatchDenial,
    },
    AspectDeltaFailure {
        detail: String,
        fields: AspectDeltaFailureFields,
    },
    ConflictingIntent {
        target: ExistingRecordTarget,
    },
    InvalidSavepoint {
        savepoint_id: SavepointId,
    },
    InvalidMergeParent {
        detail: String,
    },
    StaleValidationBasis {
        detail: String,
    },
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    TransactionFootprintBudgetExceeded {
        maximum_loci: usize,
        required_loci: usize,
    },
    TransactionOverlayBudgetExceeded {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    TransactionSavepointBudgetExceeded {
        maximum_savepoints: usize,
    },
    TransactionSavepointFootprintBudgetExceeded {
        maximum_loci: usize,
        required_loci: usize,
    },
    TransactionSavepointIdentityExhausted,
    MaterializationAuthorityRequired,
    MaterializationModeMismatch,
    MergeConflictOverlap {
        detail: String,
    },
    MissingMergeBase {
        detail: String,
    },
    UndeclaredSchemaTransition {
        previous_schema_version: SchemaVersionId,
        current_schema_version: SchemaVersionId,
        previous_descriptor_semantics_version: DescriptorSemanticsVersion,
        current_descriptor_semantics_version: DescriptorSemanticsVersion,
    },
    DescriptorSemanticsVersionUnsupported {
        previous_descriptor_semantics_version: DescriptorSemanticsVersion,
        current_descriptor_semantics_version: DescriptorSemanticsVersion,
    },
    InvalidSchemaTransitionSourceBasis {
        declared_schema_id: crate::schema::data::SchemaId,
        declared_schema_version: SchemaVersionId,
        expected_schema_id: crate::schema::data::SchemaId,
        expected_schema_version: SchemaVersionId,
    },
    InvalidSchemaTransitionTargetBasis {
        declared_schema_id: crate::schema::data::SchemaId,
        declared_schema_version: SchemaVersionId,
        expected_schema_id: crate::schema::data::SchemaId,
        expected_schema_version: SchemaVersionId,
    },
    MissingSchemaBasisForTransition {
        role: String,
    },
    UnsupportedBridgeDescriptor {
        detail: String,
    },
    HistoricalReinterpretationViolation {
        detail: String,
    },
    TypeContinuityDeniedSchemaTransition {
        detail: String,
    },
    StructuralContinuityDeniedSchemaTransition {
        detail: String,
    },
    DirectionalityMismatchUnderCanonicalReconciliation {
        detail: String,
    },
    InvalidSchemaTransitionShape {
        detail: String,
    },
}

mod presentation;

#[cfg(test)]
mod tests;
