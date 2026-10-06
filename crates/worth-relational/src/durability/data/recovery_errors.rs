use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};
use crate::history::data::HistoryDriftClass;
use crate::identity::data::KindId;
use crate::schema::data::{
    ContractId, DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion,
    SchemaBoundaryFingerprint, SchemaVersionId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryFailureClass {
    SchemaMismatch,
    ProfileMismatch,
    RuntimeNameMismatch,
    CorruptCheckpoint,
    /// The checkpoint is an intact image of a format this build does not read.
    /// It is refused outright: recovery never falls back past it.
    UnsupportedCheckpointFormat,
    CorruptSegment,
    UnsupportedLegacySemantics,
    MissingAuthoritativeParentClosure,
    ReplayFailure,
    DurableIoFailure,
    CheckpointPublicationInFlight,
    PerformedPublicationRequiresSettlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationIntegrityContractFamily {
    EndpointKind,
    Cardinality,
    Uniqueness,
    Symmetry,
    EndpointDeletionIntegrity,
    Aggregate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAuthorityContinuityMismatch {
    SchemaRegistryShape {
        expected_primary_schema_version: SchemaVersionId,
        found_primary_schema_version: SchemaVersionId,
        expected_entity_kind_count: usize,
        found_entity_kind_count: usize,
        expected_relation_kind_count: usize,
        found_relation_kind_count: usize,
    },
    EntityAspectPlanRevision {
        kind_id: KindId,
        kind_name: String,
        expected_revision: u128,
        found_revision: u128,
    },
    RelationAspectPlanRevision {
        kind_id: KindId,
        kind_name: String,
        expected_revision: u128,
        found_revision: u128,
    },
    RelationIntegrityPlanRevision {
        kind_id: KindId,
        kind_name: String,
        contract_family: RelationIntegrityContractFamily,
        expected_revision: u128,
        found_revision: u128,
        expected_contract_ids: Vec<ContractId>,
        found_contract_ids: Vec<ContractId>,
    },
    RuntimeProfile {
        expected: String,
        found: String,
    },
    RuntimeName {
        expected: String,
        found: String,
    },
    DescriptorSemanticsVersion {
        expected: DescriptorSemanticsVersion,
        found: DescriptorSemanticsVersion,
    },
    DescriptorCanonicalBasisVersion {
        expected: DescriptorCanonicalBasisVersion,
        found: DescriptorCanonicalBasisVersion,
    },
    SchemaTransitionArtifact {
        commit_id: u64,
        detail: String,
    },
    ContinuationDescriptor {
        commit_id: u64,
        boundary_fingerprint: Option<SchemaBoundaryFingerprint>,
        detail: String,
    },
    ReconciliationDescriptor {
        commit_id: u64,
        detail: String,
    },
    SchemaLineage {
        commit_id: u64,
        detail: String,
    },
}

impl RecoveryAuthorityContinuityMismatch {
    pub fn summary(&self) -> String {
        match self {
            Self::SchemaRegistryShape {
                expected_primary_schema_version,
                found_primary_schema_version,
                expected_entity_kind_count,
                found_entity_kind_count,
                expected_relation_kind_count,
                found_relation_kind_count,
            } => format!(
                "schema basis mismatch: expected primary schema version {}, found {}; expected entity kinds {}, found {}; expected relation kinds {}, found {}",
                expected_primary_schema_version.0,
                found_primary_schema_version.0,
                expected_entity_kind_count,
                found_entity_kind_count,
                expected_relation_kind_count,
                found_relation_kind_count
            ),
            Self::EntityAspectPlanRevision {
                kind_id,
                kind_name,
                expected_revision,
                found_revision,
            } => format!(
                "entity aspect plan revision mismatch for {} ({}) expected {} found {}",
                kind_name, kind_id.0, expected_revision, found_revision
            ),
            Self::RelationAspectPlanRevision {
                kind_id,
                kind_name,
                expected_revision,
                found_revision,
            } => format!(
                "relation aspect plan revision mismatch for {} ({}) expected {} found {}",
                kind_name, kind_id.0, expected_revision, found_revision
            ),
            Self::RelationIntegrityPlanRevision {
                kind_id,
                kind_name,
                contract_family,
                expected_revision,
                found_revision,
                expected_contract_ids,
                found_contract_ids,
            } => format!(
                "relation integrity plan revision mismatch for {} ({}) family {:?} expected {} found {} expected contracts {:?} found {:?}",
                kind_name,
                kind_id.0,
                contract_family,
                expected_revision,
                found_revision,
                expected_contract_ids,
                found_contract_ids
            ),
            Self::RuntimeProfile { expected, found } => {
                format!("runtime profile mismatch expected {expected} found {found}")
            }
            Self::RuntimeName { expected, found } => {
                format!("runtime name mismatch expected {expected} found {found}")
            }
            Self::DescriptorSemanticsVersion { expected, found } => format!(
                "descriptor semantics version mismatch expected {} found {}",
                expected.0, found.0
            ),
            Self::DescriptorCanonicalBasisVersion { expected, found } => format!(
                "descriptor canonical basis version mismatch expected {} found {}",
                expected.0, found.0
            ),
            Self::SchemaTransitionArtifact { commit_id, detail } => {
                format!("schema transition artifact mismatch at commit {commit_id}: {detail}")
            }
            Self::ContinuationDescriptor {
                commit_id,
                boundary_fingerprint,
                detail,
            } => format!(
                "schema continuation descriptor mismatch at commit {commit_id} boundary {:?}: {detail}",
                boundary_fingerprint
            ),
            Self::ReconciliationDescriptor { commit_id, detail } => {
                format!("schema reconciliation descriptor mismatch at commit {commit_id}: {detail}")
            }
            Self::SchemaLineage { commit_id, detail } => {
                format!("schema lineage mismatch at commit {commit_id}: {detail}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurabilityError {
    pub class: RecoveryFailureClass,
    pub detail: String,
    pub history_drift_class: Option<HistoryDriftClass>,
    pub authority_continuity_mismatch: Option<RecoveryAuthorityContinuityMismatch>,
    pub context: ErrorContext,
}

impl DurabilityError {
    pub fn new(class: RecoveryFailureClass, detail: impl Into<String>) -> Self {
        let operation = match class {
            RecoveryFailureClass::DurableIoFailure => ErrorOperation::ReadDurableStore,
            RecoveryFailureClass::CheckpointPublicationInFlight
            | RecoveryFailureClass::PerformedPublicationRequiresSettlement => {
                ErrorOperation::WriteDurableStore
            }
            RecoveryFailureClass::CorruptCheckpoint
            | RecoveryFailureClass::UnsupportedCheckpointFormat
            | RecoveryFailureClass::CorruptSegment
            | RecoveryFailureClass::UnsupportedLegacySemantics
            | RecoveryFailureClass::MissingAuthoritativeParentClosure
            | RecoveryFailureClass::ReplayFailure
            | RecoveryFailureClass::SchemaMismatch
            | RecoveryFailureClass::ProfileMismatch
            | RecoveryFailureClass::RuntimeNameMismatch => ErrorOperation::Recover,
        };
        Self {
            class,
            detail: detail.into(),
            history_drift_class: None,
            authority_continuity_mismatch: None,
            context: ErrorContext::new(RelationalSubsystem::Durability, operation)
                .with_fix(SuggestedFix::RepairDurableStore),
        }
    }

    pub fn with_history_drift_class(mut self, drift_class: HistoryDriftClass) -> Self {
        self.history_drift_class = Some(drift_class);
        self
    }

    pub fn with_authority_continuity_mismatch(
        mut self,
        mismatch: RecoveryAuthorityContinuityMismatch,
    ) -> Self {
        self.detail = format!("{}: {}", self.detail, mismatch.summary());
        self.authority_continuity_mismatch = Some(mismatch);
        self
    }
}
