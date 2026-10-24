use super::RelationalDiagnosticFields;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticsScope {
    Schema,
    Transaction,
    Snapshot,
    Retention,
    History,
    Replay,
    PatchPublication,
    Lineage,
    QueryPlanning,
    Invariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticsArtifactKind {
    MinimalSummary,
    DetailedTrace,
    Failure,
    Rollback,
    Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticsDeliveryClass {
    MustBeHot,
    CanDefer,
    ReconstructableFromReplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalArtifactPolicy {
    pub delivery_class: DiagnosticsDeliveryClass,
    pub enabled: bool,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DeterminismExpectation {
    Required,
    Measured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticCode {
    StaleHandle,
    InvalidRelationEndpoint,
    DuplicateRelationIdentity,
    RelationEndpointKindViolation,
    RelationCardinalityViolation,
    RelationUniquenessViolation,
    RelationSymmetryViolation,
    RelationEndpointDeletionIntegrityViolation,
    RelationContractAdmissionMismatch,
    ConflictingIntent,
    InvalidSavepoint,
    InvalidMergeParent,
    MissingMergeBase,
    MergeConflictOverlap,
    SavepointRolledBack,
    InvariantViolation,
    PreparationFailure,
    SerialPreparationSelected,
    InvariantProofBoundaryObserved,
    StorageInconsistencyDetected,
    CanonicalOrderingViolation,
    DeterministicMergeViolation,
    SidecarConsistencyFailure,
    InvalidSnapshotHandle,
    SnapshotExpired,
    RetentionPinningConflict,
    EntityCreated,
    EntityUpdated,
    EntityDeleted,
    RelationCreated,
    RelationUpdated,
    RelationDeleted,
    RelationRetainedForAudit,
    ReplayRetentionPinned,
    ReplayRetentionReleased,
    RetentionPlanInspected,
    MergeBaseResolved,
    MergeExecutionPublished,
    MergeCommitPublished,
    CommitPublished,
    // Retained as durable vocabulary for 9.16.1.1 lineage diagnostics. Current
    // publication does not mint these retired diagnostic codes.
    LineageCandidateRecorded,
    LineagePromotionPublished,
    LineagePromotionExecutionFailed,
    DurableAppendFailed,
    CheckpointCreated,
    CheckpointFailed,
    RecoveryCheckpointSelected,
    RecoveryRangeReplayed,
    DurableRecoveryAuthorityContinuityEvaluated,
    SubscriberContractEvaluated,
    SubscriberBoundaryEvaluated,
    DurableCorruptionDetected,
    DiagnosticsPublicationFailure,
    ReplaySchemaVersionMismatch,
    SchemaContinuityViolation,
    SchemaTransitionTraced,
    SchemaTransitionClassified,
    SchemaLineageTraced,
    SchemaBridgeDescriptorConstructed,
    SchemaReconciliationResolved,
    SchemaInterpretationSensitivityClassified,
    SchemaDescriptorVersionSelected,
    SubscriberContractUpgradeDecision,
    SubscriberRenegotiationDecision,
    SnapshotReadPathInspected,
    PublishedSnapshotHandleRead,
    VisibilityCacheHit,
    VisibilityCacheMissReconstructed,
    VisibilityCacheRecentAdmissionCandidate,
    VisibilityCacheProtectedRead,
    VisibilityCacheTransientRead,
    AspectHistoryResolved,
    LineageAspectHistoryResolved,
    AspectDeltaFailure,
    AspectEvaluationTraced,
    AspectEmissionTraced,
    AspectDeclarationTraced,
    AspectLoweringTraced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalDiagnosticsEntry {
    pub code: DiagnosticCode,
    pub message: String,
    pub fields: RelationalDiagnosticFields,
}

impl RelationalDiagnosticsEntry {
    pub fn new(
        code: DiagnosticCode,
        message: impl Into<String>,
        fields: RelationalDiagnosticFields,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            fields,
        }
    }

    pub fn canonicalized(mut self) -> Self {
        self.fields = canonicalize_diagnostic_fields(self.fields);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalDiagnosticArtifact {
    pub scope: DiagnosticsScope,
    pub kind: DiagnosticsArtifactKind,
    pub determinism: DeterminismExpectation,
    pub entries: Vec<RelationalDiagnosticsEntry>,
}

impl RelationalDiagnosticArtifact {
    pub fn new(
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        determinism: DeterminismExpectation,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> Self {
        Self {
            scope,
            kind,
            determinism,
            entries,
        }
        .canonicalized()
    }

    pub fn canonicalized(mut self) -> Self {
        self.entries = self
            .entries
            .into_iter()
            .map(RelationalDiagnosticsEntry::canonicalized)
            .collect();
        self
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        let entries = (self.entries.capacity() as u64)
            .saturating_mul(std::mem::size_of::<RelationalDiagnosticsEntry>() as u64);
        self.entries.iter().fold(entries, |bytes, entry| {
            bytes
                .saturating_add(entry.message.capacity() as u64)
                .saturating_add(entry.fields.owned_allocation_capacity_bytes())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalDiagnosticsFacade {
    pub artifacts: Vec<RelationalDiagnosticArtifact>,
}

impl RelationalDiagnosticsFacade {
    pub fn artifacts(&self) -> &[RelationalDiagnosticArtifact] {
        &self.artifacts
    }

    pub fn by_scope(&self, scope: DiagnosticsScope) -> Vec<&RelationalDiagnosticArtifact> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.scope == scope)
            .collect()
    }

    pub fn minimal_summaries(&self) -> Vec<&RelationalDiagnosticArtifact> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.kind == DiagnosticsArtifactKind::MinimalSummary)
            .collect()
    }
}

pub(crate) fn canonicalize_diagnostic_fields(
    fields: RelationalDiagnosticFields,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(fields.root().clone())
}
