//! Public API boundary for `worth-relational`.
pub mod content {
    pub use crate::content::read_closure::RelationalContentReadClosure;
    pub use crate::content::{
        RelationalContentAttachments, RelationalContentCodec, RelationalContentControl,
        RelationalContentDenial, RelationalContentFieldBinding, RelationalContentFormat,
        RelationalContentFormatDeclaration, RelationalContentFormatRegistry,
        RelationalContentFrame, RelationalContentRejection, RelationalContentSchemaSnapshot,
        RelationalContentSink, MAXIMUM_CONTENT_FRAME_BYTES, MAXIMUM_STAGED_CONTENT_BYTES,
    };
}
#[path = "facade/authorization.rs"]
pub mod authorization;
#[path = "facade/branch.rs"]
pub mod branch;
mod runtime_validation_exports;
pub mod config {
    pub use crate::config::data::{
        AdjacencyBackend, AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy,
        ConfigProvenance, ConfigProvenanceEntry, ConfigValueSource, CrossContextPolicy,
        DiagnosticsBoundary, DurabilityPolicy, DurableLogPolicy, DurableLogRetentionMode,
        MvccConfig, PublicationConfig, RelationalConfigOverride, RelationalRuntimeProfile,
        ResidencyBudget, RetentionBackend, RetentionPolicy, RuntimeExecutionLane,
        RuntimeProfileBoundaryPolicy, SnapshotReleasePolicy, StorageLayoutConfig,
        VisibilityCachePolicy,
    };
}
#[path = "facade/change_source.rs"]
pub mod change_source;
pub mod diagnostics {
    pub use crate::diagnostics::data::RelationalDiagnosticsFacade;
    pub use crate::diagnostics::data::{
        DeterminismExpectation, DiagnosticCode, DiagnosticObjectFields, DiagnosticsArtifactKind,
        DiagnosticsDeliveryClass, DiagnosticsScope, RelationalArtifactPolicy,
        RelationalDiagnosticArtifact, RelationalDiagnosticValue, RelationalDiagnosticsEntry,
        RelationalDiagnosticsProfile,
    };
}

#[path = "facade/durability.rs"]
pub mod durability;

pub mod errors {
    pub use crate::errors::{
        ErrorContext, ErrorOperation, RelationalError, RelationalSubsystem, SuggestedFix,
    };
}

pub mod history {
    pub use crate::history::data::{
        AspectHistoryCommitSpan, AspectHistoryDigest, AspectHistoryEntry,
        AspectHistoryLineageEventSpan, AspectHistoryOrigin, AspectHistoryQueryResult,
        AspectHistoryResolutionTrace, AspectResolutionContext, BranchCreateError,
        BranchCreateErrorClass, BranchId, CanonicalCommitEnvelope,
        CanonicalRetainedEntitySelection, CommitId, CommittedVersionSummary,
        HistoryAspectQueryTarget, HistoryDriftClass, HistoryRetentionClass,
        HistoryShapeClassification, LineageAspectHistory, LineageAspectHistoryQueryResult,
        LineageAspectResolutionDigest, OrderedParentList, RelationalCommitReceipt,
        RelationalDescriptiveTouch, RelationalDescriptiveTouchGraph,
        RelationalDescriptiveTouchPrecision, RelationalIntroducedSchemaObservation,
        RelationalOriginalInputObservation, RelationalReplayRetentionDenial,
        RelationalTouchAdjacencyDirection, SharedCanonicalCommitEnvelope, VersionGraphPolicy,
    };
    pub use crate::history::{
        BoundedCanonicalCommitPatchDenial, HistoryAccess, HistoryAuthority,
        OutputOriginHeaderCorruptionForTest,
    };
    pub use crate::history::{
        RelationalAuthoredEntityIndexDefinition, RelationalAuthoredManagedSourceImpact,
        RelationalAuthoredManagedSourceMaterial, RelationalAuthoredManagedSourcePayloads,
        RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteCursor,
        RelationalAuthoredWriteDenial, RelationalAuthoredWriteOccurrence,
        RelationalAuthoredWritePage, RelationalAuthoredWritePageLimits,
        RelationalAuthoredWriteScope, RelationalAuthoredWriteSelector, RelationalAuthoredWriteSpan,
        RelationalPreparedAuthoredWritePage,
    };
    pub use crate::history::{
        RelationalCanonicalRecordHistoryCursor, RelationalPreparedRecordHistoryPage,
        RelationalRecordHistoryChange, RelationalRecordHistoryDenial, RelationalRecordHistoryPage,
        RelationalRecordHistoryPageLimits, RelationalRecordHistorySelection,
        RelationalRecordHistorySelectionLimits, RelationalRecordHistorySubject,
    };
    pub use crate::history::{
        RelationalCommitArtifactDenial, RelationalCommitCatalogAppendDenial,
        RelationalCommitCatalogEntry, RelationalCommitIdentity, RelationalCommitParentage,
        RelationalCommitParentageDenial, RelationalCommitRootDescriptor,
        RelationalRetainedSelectionArtifactDenial,
    };
}

#[path = "facade/aggregate_identity.rs"]
pub mod aggregate_identity;

#[path = "facade/identity.rs"]
pub mod identity;

pub mod identity_authority {
    pub use crate::identity_authority::*;
}

#[path = "facade/indexes.rs"]
pub mod indexes;
#[path = "facade/inspection.rs"]
pub mod inspection;

#[path = "facade/read.rs"]
pub mod read;

#[path = "facade/lineage.rs"]
pub mod lineage;

pub mod runtime {
    pub use super::runtime_validation_exports::*;
    pub use crate::execution::request_work::observation::RelationalWorkAccountRecorderForTest;
    pub use crate::presentation::facade::runtime::{
        ImmutableReadContract, RelationalBoundaryContract, RelationalRuntimeApi,
    };
    #[doc(hidden)]
    pub use crate::publication::ownership::{
        CurrentOwnershipContactsForTest, CurrentOwnershipLocalityForTest,
    };
    pub use crate::runtime::builder::{RelationalRuntimeBuilder, RelationalRuntimeDefinition};
    #[cfg(test)]
    pub use crate::runtime::HarnessAuditMode;
    pub use crate::runtime::RelationalColdSymbolPlacementForTest;
    pub use crate::runtime::{
        custom_invariant_inventory_digest, home_program_transition_body_digest,
        AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial, BoundedFrontierReadStop,
        ComplexityContract, ComplexityStatus, EntityProjectionRecord, EntityRecordProjection,
        FreshRelationalInitialSchemaInstallation, HomeReadmissionMetadataKinds,
        HomeReadmissionMetadataReadDenial, HomeReadmissionMetadataView, InvariantAccess,
        PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind,
        PinnedStructuralWalkReads, PinnedStructuralWalkScope, PinnedStructuralWalkVisit,
        PositionedRelationalSnapshot, PreparedRelationalHomeProgramContracts,
        QueryLeasedReadOutcome, QueryReadExecutionStop, QueryReadPacketDenial,
        RecoveredCheckpointProgramActivationUpdate, RelationProjectionRecord,
        RelationRecordProjection, RelationalAdjacencyDirection, RelationalAdjacencySupportDenial,
        RelationalAdjacencyVisit, RelationalAppliedCandidateReadIdentity,
        RelationalAppliedCandidateReadQualification, RelationalAspectPresenceSupportDenial,
        RelationalBorrowedRecordReadDenial, RelationalCandidateInputCounters,
        RelationalEntityMetadata, RelationalEntityMetadataPinDenial, RelationalEntityRetirement,
        RelationalFieldSupportComparisonDenial, RelationalHomeProgramBodyDenial,
        RelationalHomeProgramCatalogReadmissionSource, RelationalHomeProgramCatalogReceipt,
        RelationalHomeProgramContractDenial, RelationalHomeProgramPlanningSource,
        RelationalHomeProgramPromotionRefused, RelationalHomeProgramStepContracts,
        RelationalInitialSchemaInstallation, RelationalInitialSchemaInstallationDenial,
        RelationalInitialSchemaInstallationDenialKind, RelationalInitialSchemaInstallationReceipt,
        RelationalPatchPositionReservationCounters, RelationalPhase4ReferenceCostCounters,
        RelationalPinnedEntityMetadata, RelationalPinnedRead, RelationalPinnedReadAdmissionDenial,
        RelationalProjectedFieldObservation, RelationalRelationMetadata,
        RelationalRetentionPassProgress, RelationalRetentionPassStop, RelationalRuntime,
        RelationalRuntimeConfig, RelationalRuntimeConstructionDenial, RelationalRuntimeForkDenial,
        RelationalSchemaTransitionAdmissionDenial, RelationalSchemaTransitionAdmissionDenialKind,
        RelationalSnapshotPositionAdmissionStop, RelationalStoppedRetentionPass,
        ReplaySchemaVersion, RuntimeComplexityCounters, SnapshotGuard, SnapshotPositionDenial,
        VisibilityProjectionView, VisibilityReadContext, VisibilityRetentionAuthority,
    };
    pub use crate::runtime::{
        PreparedRelationalRuntimeClose, PreparedRelationalSourceClose,
        RelationalPublicationCompanionCustody, RelationalPublicationCustodyHolders,
        RelationalPublicationRegistryCustody, RelationalRuntimeCloseDenial,
        RelationalRuntimeCloseRefused, RelationalSourceClose, RelationalSourceCloseDenial,
        RelationalSourceCloseProgress, RetiredSourceRelationalAllocation,
    };
    pub use crate::storage::data::{
        ChunkVisibilitySummary, ChunkedStorageSummary, EntityReadRecord, PartitionStorageStats,
        RelationReadRecord, RelationalFieldPresence, RelationalFieldRevision, StorageStats,
    };
    pub use crate::visibility::authority::VisibilityAuthority as SnapshotAuthority;
    pub use crate::visibility::exact_commit_snapshot::{
        RelationalRetainedCommitEntityProjection, RelationalRetainedCommitProjectionWork,
        RelationalRetainedCommitSnapshot, RelationalRetainedCommitSnapshotDenial,
        RelationalRetainedCommitSnapshotDenialKind,
    };
    pub use crate::visibility::materialization::read_records::{
        AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead, BoundedEntityKindTruthRead,
        BoundedFrontierAdjacencyTruthRead, BoundedFrontierFieldEqualityTruthRead,
        BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
        FrontierAdjacencyTruthReadLimitExceeded, FrontierFieldEqualityTruthReadLimitExceeded,
        ProjectionAspectFilter, ProjectionAspectFilterMode, ProjectionAspectRequirement,
        ProjectionAspectScope, RelationKindTruthReadDenial, RelationKindTruthReadLimitExceeded,
    };
    pub use crate::visibility::materialization::read_records::{
        ProjectionContractRefusal, ProjectionRecordDomain, RelationalProjectionContractDenial,
    };
    pub use crate::visibility::snapshot_read_view::RelationalReadView;
    pub use crate::visibility::snapshot_states::HistoricalVisibilityDenial;
}

#[cfg(test)]
pub mod harness {
    pub use crate::presentation::facade::harness::{
        default_harness_expectations, FixtureEntity, FixtureRelation, RelationalFixture,
        RelationalHarnessAdapter, RelationalHarnessError, RelationalHarnessExpectations,
        RelationalHarnessPlan,
    };
}

pub mod publication {
    pub use crate::publication::bundle::{PublicationStage, PublicationStatus};
    pub use crate::publication::cdc::facade::{
        SubscriberBoundaryAssessment, SubscriberCheckpoint, SubscriberContinuationAssessment,
        SubscriberRecoveryDecision, SubscriberRecoveryDisposition, SubscriberRecoverySource,
        SubscriberResumeRequest, SubscriberStreamBatch, SubscriberStreamFailure,
        SubscriberStreamFailureClass,
    };
    pub use crate::publication::data::{
        AuthoredRetirementRefusal, CoupledFreshWriter, CoupledProvenancePostponement,
        PublicationDiagnosticsSnapshot, PublicationError,
        PublicationObservationSnapshot, PublicationSettlementRepairError,
    };
    pub use crate::publication::patch::data::{
        PatchDetail, PatchFragmentBudget, PatchOrdering, PatchPublicationMode, PatchStreamBatch,
        PatchStreamPosition, PatchStreamReadError, PatchStreamReadErrorClass, PatchStreamRequest,
        PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
        PublishedAuthoritativeFieldEffect, PublishedAuthoritativeFieldSet,
        PublishedAuthoritativeFieldValueLocation, PublishedAuthoritativePatch,
        PublishedAuthoritativePatchEnvelope, PublishedAuthoritativeRecordPatch,
        PublishedRelationBirth, RecordStructuralChange,
    };
    pub use crate::publication::{
        PublicationArtifactsAccess, PublicationDiagnosticsAccess, PublicationPatchStreamAccess,
        PublicationSubscriberStreamAccess, PublicationSurface,
    };
}

pub mod query {
    pub use crate::query::data::index_parity_outcome::IndexParityVerifiedQueryOutcome;
    pub use crate::query::data::query_outcome::QueryExecutionOutcome;
    pub use crate::query::data::snapshot_pinned_plan::SnapshotPinnedQueryPlan;
    pub use crate::query::data::{
        CanonicalQueryResult, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
        IndexParityMode, IndexQueryRejectionClass, PartitionHint, PlannedQueryPacket,
        QueryAccessContract, QueryAccessPath, QueryComplexitySummary, QueryExecutionShape,
        QueryFragmentCounters, QueryLocalityClass, QueryOrderingContract, QueryParallelLegality,
        QueryParallelProfitability, QueryPlanContextId, QueryPlanEvidenceBasis, QueryScope,
        QuerySerialReason, QueryWorkerFragment, ReductionDiscipline,
    };
}

pub mod replay {
    pub use crate::history::data::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
    pub use crate::replay::data::{
        CertifiedLineageSurfaceComparisonBasis, CertifiedLineageSurfaceDigest,
        LineageCertifiedSurfaceKind, RelationalReplayOutcome, RelationalReplayRequest,
        ReplayAuthorityBasisKind, ReplayError, ReplayExecutionMode, ReplayFailureClass,
        ReplayLineageAuthorityBasis, ReplayLineageDigestMode, ReplayMismatch, ReplayMismatchClass,
        ReplayObservableSurface, ReplaySnapshotSurface, ReplayVerificationLayer,
        ReplayVerificationMode, ReplayVerificationPlan,
    };
}

#[path = "facade/schema.rs"]
pub mod schema;

pub mod snapshots {
    pub use crate::snapshots::data::{
        SnapshotHandle, SnapshotId, SnapshotInspectionSummary, SnapshotReadPolicy,
    };
    pub use crate::visibility::{
        RelationalSnapshotAdmissionDenial, RelationalSnapshotReleaseDenial,
        RelationalSnapshotReleaseReceipt,
    };
}

pub mod visibility {
    pub use crate::visibility::store_correlation_reference::*;
}

pub mod storage {
    /// SDK-internal incremental maintenance contracts.
    pub use crate::branch::maintenance;
    pub use crate::storage::data::{
        authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
        AuthoritativeFieldComparisonKeyDecodeDenial,
        AuthoritativeFieldComparisonKeyDecodeDenialKind, RecordLifecycleState,
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
}

pub mod symbols {
    pub use crate::runtime::RelationalSymbolTableForTest;
    pub use crate::symbols::data::selected_field_text::RelationalFieldText;
    pub use crate::symbols::data::RelationalSymbolExhausted;
    pub use crate::symbols::data::{
        ClientKey, ClientKeySymbolPolicy, InternedString, RelationalFieldTextAdmissionDenial,
        StoredSymbolText, StringInterner, Symbol, SymbolTableSnapshot,
    };
}

#[path = "facade/transactions.rs"]
pub mod transactions;

#[path = "facade/mvcc.rs"]
pub mod mvcc;
