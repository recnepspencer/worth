//! Public API boundary for `worth-relational`.
#[path = "facade/authorization.rs"]
pub mod authorization;
#[path = "facade/branch.rs"]
pub mod branch;
mod runtime_validation_exports;
pub mod config {
    pub use crate::config::data::{
        AdjacencyBackend, AdjacencyPolicy, CascadeDeletePolicy, CheckpointPolicy,
        CommitStrategiesConfig, CompiledLanePolicy, ConfigProvenance, ConfigProvenanceEntry,
        ConfigValueSource, CrossContextPolicy, DiagnosticsBoundary, DurabilityPolicy,
        DurableLogPolicy, DurableLogRetentionMode, MvccConfig, PublicationConfig,
        RelationIntegrityScopeBudget, RelationalConfigOverride, RelationalRuntimeProfile,
        RetentionBackend, RetentionPolicy, RuntimeExecutionLane, RuntimeProfileBoundaryPolicy,
        SnapshotReleasePolicy, StorageLayoutConfig, VisibilityCachePolicy,
    };
}
pub mod grouped_truth {
    pub use crate::grouped_truth::{
        encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
        project_relational_grouped_truth, GroupedProjectionContract,
        RelationalAuthoritativeRowArtifact, RelationalAuthoritativeRowSetArtifact,
        RelationalGroupedMemberRow, RelationalGroupedProjectionArtifact,
        RelationalGroupedProjectionDigest, RelationalGroupedTruthError,
        RelationalProjectedAspectValueSet, RelationalRowIdentity, RelationalRowSetDigest,
    };
}
pub mod commit_strategies {
    pub use crate::commit_strategies::data::{
        CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact,
        CanonicalStrategyInputDigest, CanonicalStrategyOutputArtifact,
        CanonicalStrategyOutputDigest, CommitStrategyDescriptor, CommitStrategyDescriptorDigest,
        CommitStrategyExecutionRegistration, CommitStrategyExecutor, CommitStrategyFamilyName,
        CommitStrategyId, CommitStrategyRegistration, CommitStrategyRegistrationError,
        CommitStrategySemanticName, CommitStrategyVersion, LoweredStrategyCommitPlan,
        NativeStrategyCommitRequest, PersistentArtifactName, StrategyCallerProvenance,
        StrategyCommitArtifactBundle, StrategyCommitRequestError, StrategyEntityAspectReadRecord,
        StrategyExecutionDraft, StrategyExecutionResult, StrategyExecutionSummary,
        StrategyExecutorFailure, StrategyExecutorFailureClass, StrategyInputSchemaName,
        StrategyInputSchemaVersion, StrategyIntentName, StrategyIntentScopeDigest,
        StrategyLoweringError, StrategyLoweringProvenance, StrategyLoweringSummary,
        StrategyMergeConflictClass, StrategyMergeDescriptor, StrategyMergeSemantics,
        StrategyMutationProgram, StrategyMutationProgramDigest, StrategyObservationContext,
        StrategyOutputSchemaName, StrategyPacketContract, StrategyProjectedAspectReadSet,
        StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass,
        StrategyReadScopeClass, StrategyReplayDescriptor, StrategyRequestOrigin,
        StrategyTraversalBasis, StrategyVisibilityReadView,
    };
    pub use crate::commit_strategies::facade::{
        CommitStrategiesAuthorityFacade, CommitStrategiesFacade,
    };
    pub use crate::commit_strategies::strategies::{
        AspectFieldReconciliationInput, AspectFieldReconciliationOutput,
        AspectFieldReconciliationStrategy, EntityReplacementReconciliationAction,
        EntityReplacementReconciliationInput, EntityReplacementReconciliationOutput,
        EntityReplacementReconciliationStrategy, IntentReconciliationAction,
        IntentReconciliationInput, IntentReconciliationOutput, IntentReconciliationStrategy,
        ReplicaConvergenceAction, ReplicaConvergenceInput, ReplicaConvergenceOutput,
        ReplicaConvergenceStrategy,
    };
    pub use crate::commit_strategies::{FrozenCommitStrategyRegistry, StrategyExecutionError};
}

#[path = "facade/bridge.rs"]
pub mod bridge;

pub mod diagnostics {
    pub use crate::diagnostics::data::RelationalDiagnosticsFacade;
    pub use crate::diagnostics::data::{
        DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsDeliveryClass,
        DiagnosticsScope, RelationalArtifactPolicy, RelationalDiagnosticArtifact,
        RelationalDiagnosticsEntry, RelationalDiagnosticsProfile,
    };
}

pub mod durability {
    pub use crate::durability::data::{
        CheckpointCoverage, CompactionOutcome, CompactionPlan, CompactionPolicy, DurabilityError,
        DurabilityMode, DurableCheckpoint, DurableCheckpointId, DurableCheckpointManifest,
        DurableIntegrityStatus, DurableSegmentId, DurableSegmentManifest, DurableStore,
        DurableStoreLayout, PartitionCheckpointImage, RecoveryAuthorityContinuityCheck,
        RecoveryAuthorityContinuityMismatch, RecoveryAuthorityParity, RecoveryCoverage,
        RecoveryCursor, RecoveryFailureClass, RecoveryIntegrityReport, RecoveryPlan,
        RecoveryVerificationMode, RecoveryVerificationOutcome, RecoveryVerificationPlan,
        RelationIntegrityContractFamily, SegmentRetentionClass,
    };
}

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
        BranchCreateErrorClass, BranchId, CanonicalCommitEnvelope, CommitId,
        CommittedVersionSummary, HistoryAspectQueryTarget, HistoryDriftClass,
        HistoryRetentionClass, HistoryShapeClassification, LineageAspectHistory,
        LineageAspectHistoryQueryResult, LineageAspectResolutionDigest, MergeConflictRecord,
        MergeInspection, OrderedParentList, RelationalCommitReceipt,
        RelationalMergeBranchBasisDenial, VersionGraphPolicy,
    };
    pub use crate::history::{HistoryAccess, HistoryAuthority};
    pub use crate::history::{
        RelationalCommitArtifactDenial, RelationalCommitCatalogAppendDenial,
        RelationalCommitCatalogEntry, RelationalCommitIdentity, RelationalCommitParentage,
        RelationalCommitParentageDenial, RelationalCommitRootDescriptor,
    };
}

#[path = "facade/identity.rs"]
pub mod identity;

pub mod identity_authority {
    pub use crate::identity_authority::*;
}

#[path = "facade/inspection.rs"]
pub mod inspection;

#[path = "facade/indexes.rs"]
pub mod indexes;

pub mod lineage {
    pub use crate::lineage::data::{
        HistoricalLineageResolution, HistoricalLineageResolutionDigestBasis,
        HistoricalLineageResolutionMetrics, HistoricalResolutionBoundednessBasis,
        HistoricalResolutionDigestMode, HistoricalResolutionRequest, HistoricalResolutionTrace,
        LineageArtifactCounters, LineageCheckpointArtifact, LineageCheckpointCounters,
        LineageCheckpointDigestBasis, LineageDecisionKind, LineageDecisionLogDigestBasis,
        LineageDigestBasis, LineageDivergenceMetrics, LineageDivergenceRequest,
        LineageDivergenceSummary, LineageDivergenceTraversalBasis, LineageEventBatchDigestBasis,
        LineageEventKind, LineageEventRecord, LineageGraphDigestBasis, LineageGraphDigestMode,
        LineageGraphMetrics, LineageGraphRequest, LineageGraphSnapshot, LineageGraphTraversalBasis,
        LineageInvariant, LineageNode, RecordHistoryRequest,
    };
}

pub mod merge {
    pub use crate::merge::data::{
        AspectComparisonState, AspectMergePolicyDeclaration, AspectMergePolicyKind,
        AspectPolicyResolutionRecord, AuthorizedAspectValueSurface, AuthorizedAspectValueUsage,
        BranchCausalDot, BranchDeltaSummary, CausalAnnotationSummary, CausalFrontier,
        CommitCausalMetadata, CommitCausalRelation, ConflictClassificationSummary,
        CustomIdentityBasisIdentity, CustomMergePolicyIdentity, DeletionExecutionClass,
        DeletionMergeClass, EndpointContinuityClass, ExecutedMergeAspectClass,
        ExecutedMergeAspectDiagnosticRow, ExecutedMergeRecordClass,
        ExecutedMergeRecordDiagnosticRow, IdentityBasisDeclaration, IdentityBasisKind,
        IdentityBasisScope, IdentityDiscoverySummary, IdentityMatchCandidate, IdentityMatchClass,
        IdentityResolutionReason, LoweredAspectAction, LoweredAspectOutcome, LoweredMergeAction,
        LoweredMergeBlockedReason, LoweredMergePlanRecord, LoweredMergePlanSummary,
        LoweredMergeRejectedReason, LoweredRecordDecision, LoweredRecordDecisionKind,
        LoweredRecordDenialBundle, LoweredRecordDenialKind, LoweredRecordExecutionBundle,
        LoweredRecordExecutionIntentKind, MergeAncestrySummary, MergeArtifactDigestBasis,
        MergeBaseSelectionRule, MergeCausalEvidenceModel, MergeConflictClass,
        MergeConflictClassification, MergeExecutableClass, MergeExecutionAuthorityContract,
        MergeExecutionAuthorizationRule, MergeExecutionCompilationError,
        MergeExecutionConsumptionRule, MergeExecutionDecisionSurface, MergeExecutionDeniedRecord,
        MergeExecutionDiagnosticsPlan, MergeExecutionError, MergeExecutionFreshnessPolicy,
        MergeExecutionMutationPlanError, MergeExecutionPreparationError, MergeExecutionReadiness,
        MergeExecutionReadinessReport, MergeExecutionRequest, MergeIntent,
        MergeManualResolutionClass, MergePlanningArtifactCore, MergePlanningDecisionKind,
        MergePlanningDecisionLog, MergePlanningDecisionLogDigestBasis, MergePlanningDecisionRecord,
        MergePlanningError, MergePlanningRequest, MergePlanningSummary,
        MergePolicyDecisionBoundary, MergePolicyOwnershipClass, MergePolicyOwnershipSurface,
        MergePolicyProofBoundary, MergePolicyRejectClass, MergePolicyResolution,
        MergeRecordCausalAnnotation, MergeRecordCausalDisposition, MergeRecordIdentity,
        MergeResolutionClass, MergeResolvedAspectValueStrategy, MergeSchemaKindClass,
        MergeSchemaKindSemanticSnapshot, MergeSchemaSnapshotDigestBasis, MergeVisibilityEvidence,
        MergeVisibilityEvidenceKind, MergeVisibilityState, NormalizedRelationalMergeRequest,
        OwnerBoundMergeExecutionRequest, OwnerBoundMergePlanningRequest, PreparedMergeExecution,
        RelationConflictPropagation, RelationContinuityClass, RelationalFoundationalMergeRequest,
        RelationalMergeAdmittedSurfaceRow, RelationalMergeAspectPolicyWitnessRow,
        RelationalMergeCorrespondencePosture, RelationalMergeCorrespondenceWitness,
        RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
        RelationalMergeDeletionStrategyWitnessRow, RelationalMergeInspectionAdmission,
        RelationalMergeInspectionArtifact, RelationalMergeInspectionInput,
        RelationalMergeInspectionRow, RelationalMergeProofPacket,
        RelationalMergeProofPacketAdmissionPosture, RelationalMergeProofPacketCanonicalBasis,
        RelationalMergeRequestBindingDenial, RelationalMergeRequestFamily,
        RelationalMergeRequestNormalizationDenial, RelationalMergeSchemaReconciliationPosture,
        RelationalMergeScope, RelationalMergeStrategyWitness, RelationalMergeTopologyIntent,
        RelationalMergeTopologyStrategyWitnessRow, RelationalSchemaReconciliationBasisRow,
        RelationalSchemaReconciliationCorrespondenceLinkRow, RelationalSchemaReconciliationWitness,
        RelationalSchemaReconciliationWitnessDenial, RelationalSchemaReconciliationWitnessPosture,
        RelationalSchemaReconciliationWitnessRow, ResolvedAspectMergePolicy, ResolvedMergeBase,
        SchemaDeclaredCorrespondenceValidationSummary, TopologyExecutionClass,
        TopologyRegionConflictReason, TopologyRewireAdmissionPolicy,
    };
    pub use crate::merge::MergeAccess;
    pub use crate::transactions::data::MergeExecutionOutcome;
}

pub mod runtime {
    pub use super::runtime_validation_exports::*;
    pub use crate::config::data::{
        PlanningContract, RelationIntegrityScopeBudget, RelationalExecutionModel,
    };
    pub use crate::presentation::facade::runtime::{
        ImmutableReadContract, RelationalBoundaryContract, RelationalRuntimeApi,
    };
    pub use crate::publication::{
        PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
    };
    pub use crate::runtime::builder::RelationalRuntimeBuilder;
    #[cfg(test)]
    pub use crate::runtime::HarnessAuditMode;
    pub use crate::runtime::{
        custom_invariant_inventory_digest, CompiledArtifactAuthorityStatus, CompiledArtifactError,
        CompiledExecutionArtifact, ComplexityContract, ComplexityStatus, EntityProjectionRecord,
        EntityRecordProjection, InvariantAccess, RelationProjectionRecord,
        RelationRecordProjection, RelationalInitialSchemaInstallation,
        RelationalInitialSchemaInstallationDenial, RelationalInitialSchemaInstallationDenialKind,
        RelationalInitialSchemaInstallationReceipt, RelationalPatchPositionReservationCounters,
        RelationalPhase4ReferenceCostCounters, RelationalReplayRecord, RelationalRuntime,
        RelationalRuntimeConfig, RelationalRuntimeForkDenial,
        RelationalSchemaTransitionAdmissionDenial, RelationalSchemaTransitionAdmissionDenialKind,
        ReplaySchemaVersion, RuntimeComplexityCounters, SimulationAccess, SimulationAuthority,
        SnapshotGuard, TopologyFreezeMode, VisibilityProjectionView, VisibilityReadContext,
        VisibilityRetentionAuthority,
    };
    pub use crate::storage::data::{
        ChunkVisibilitySummary, ChunkedStorageSummary, EntityReadRecord, PartitionStorageStats,
        RelationReadRecord, RelationalReadView, StorageStats,
    };
    pub use crate::visibility::authority::VisibilityAuthority as SnapshotAuthority;
    pub use crate::visibility::exact_commit_snapshot::{
        RelationalRetainedCommitEntityProjection, RelationalRetainedCommitProjectionWork,
        RelationalRetainedCommitSnapshot, RelationalRetainedCommitSnapshotDenial,
        RelationalRetainedCommitSnapshotDenialKind,
    };
    pub use crate::visibility::materialization::read_records::{
        AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead,
        BoundedFrontierAdjacencyTruthRead, BoundedFrontierFieldEqualityTruthRead,
        FrontierAdjacencyTruthReadLimitExceeded, FrontierFieldEqualityTruthReadLimitExceeded,
        ProjectionAspectFilter, ProjectionAspectFilterMode, ProjectionAspectRequirement,
        ProjectionAspectScope,
    };
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
    pub use crate::publication::bundle::{PublicationBundle, PublicationStage, PublicationStatus};
    pub use crate::publication::cdc::facade::{
        SubscriberBoundaryAssessment, SubscriberCheckpoint, SubscriberContinuationAssessment,
        SubscriberRecoveryDecision, SubscriberRecoveryDisposition, SubscriberRecoverySource,
        SubscriberResumeRequest, SubscriberStreamBatch, SubscriberStreamFailure,
        SubscriberStreamFailureClass,
    };
    pub use crate::publication::data::{
        DeferredPublicationSettlement, DeferredPublicationSettlementError,
        PublicationArtifactSnapshot, PublicationDiagnosticsSnapshot, PublicationError,
        PublicationObservationSnapshot,
    };
    pub use crate::publication::patch::data::{
        PatchDetail, PatchFragmentBudget, PatchOrdering, PatchPublicationMode, PatchStreamBatch,
        PatchStreamPosition, PatchStreamReadError, PatchStreamReadErrorClass, PatchStreamRequest,
        PublishedAspectChangePrecision, PublishedAuthoritativeAspectChange,
        PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
        PublishedAuthoritativePatchEnvelope, PublishedAuthoritativeRecordPatch,
        RecordStructuralChange,
    };
    pub use crate::publication::{
        PublicationArtifactsAccess, PublicationDiagnosticsAccess, PublicationPatchStreamAccess,
        PublicationSubscriberStreamAccess, PublicationSurface,
    };
}

pub mod query {
    pub use crate::query::data::{
        CanonicalQueryResult, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
        IndexParityMode, IndexParityVerifiedQueryOutcome, IndexQueryRejectionClass, PartitionHint,
        PlannedQueryPacket, QueryAccessContract, QueryAccessPath, QueryComplexitySummary,
        QueryExecutionOutcome, QueryExecutionShape, QueryFragmentCounters, QueryLocalityClass,
        QueryOrderingContract, QueryParallelLegality, QueryParallelProfitability,
        QueryPlanContextId, QueryPlanEvidenceBasis, QueryScope, QuerySerialReason,
        QueryWorkerFragment, ReductionDiscipline, SnapshotPinnedQueryPlan,
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

pub mod schema {
    pub use crate::schema::data::{
        AcyclicityContractDeclaration, AllowedCycleClass, AspectBinding,
        AspectContractPlanRevision, AspectDeclarationTrace, AspectDeclarationTraceRow,
        AspectLoweringTrace, AspectLoweringTraceRow, CardinalityContractDeclaration,
        ConnectivityMinimumContractDeclaration, ConnectivityMinimumEnforcement, ContractId,
        DeclaredAspectContractBinding, DescriptorCanonicalBasisSupportPolicy,
        DescriptorCanonicalBasisVersion, DescriptorSemanticsSupportPolicy,
        DescriptorSemanticsVersion, DirectedTraversalKind, EndpointDeletionIntegrityDeclaration,
        EndpointDeletionIntegrityMode, EndpointKindContractDeclaration, EntityKindRegistration,
        FreeFormSchemaDiffIntent, HistoricalInterpretationSensitivity,
        KindAspectContractDeclarations, KindResolution, LoweredAcyclicityContract,
        LoweredAspectContractBinding, LoweredAspectContractPlan, LoweredCardinalityMaximumContract,
        LoweredCardinalityMinimumContract, LoweredConnectivityMinimumContract,
        LoweredEndpointDeletionIntegrityContract, LoweredEndpointKindContract,
        LoweredPartitionIsolationContract, LoweredRelationIntegrityPlan,
        LoweredSchemaTransitionPlan, LoweredSymmetryContract, LoweredUniquenessContract,
        MinimumCardinalityEnforcement, PairMinimumSemantics, PartitionIsolationContractDeclaration,
        PartitionIsolationMode, ProposedSchemaTransition, RelationIntegrityDeclarations,
        RelationIntegrityPlanCatalog, RelationIntegrityPlanRevision, RelationKindRegistration,
        RelationalAspectChangeKind, RelationalSchemaRegistry, SchemaAuthoritySnapshot,
        SchemaBoundaryFingerprint, SchemaBridgeDescriptor, SchemaBridgeabilityClassification,
        SchemaContinuationAdmissionObservation, SchemaContinuationClassification,
        SchemaContinuationDescriptor, SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind,
        SchemaElementRef, SchemaId, SchemaLineageArtifact, SchemaLineageOrderingSemantics,
        SchemaPublicationImpact, SchemaReconciliationClassification,
        SchemaReconciliationDescriptor, SchemaReconciliationOrderingMode,
        SchemaReconciliationPolicy, SchemaRegistryError, SchemaRegistryErrorClass, SchemaStratum,
        SchemaSubscriberImpact, SchemaTransitionArtifact, SchemaTransitionBarrier,
        SchemaTransitionSummary, SchemaVersionId, SubscriberBoundaryVisibility,
        SymmetryContractDeclaration, SymmetryMode, UniquenessContractDeclaration, UniquenessScope,
        ValidatedSchemaTransition,
    };
}

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
    pub use crate::storage::data::{
        authoritative_aspect_value_field_comparison_key, AuthoritativeFieldComparisonKey,
        RecordLifecycleState,
    };
}

pub mod symbols {
    pub use crate::symbols::data::{
        ClientKey, ClientKeySymbolPolicy, InternedString, StringInterner, Symbol,
        SymbolTableSnapshot,
    };
}

#[path = "facade/transactions.rs"]
pub mod transactions;

#[path = "facade/mvcc.rs"]
pub mod mvcc;
