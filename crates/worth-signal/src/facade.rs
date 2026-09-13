//! Public API boundary for `worth-signal`.
//! External components should import through this module.
//!
//! The facade is an export map, not an implementation layer. Capability
//! modules below classify the stable public surface by audience and domain;
//! their behavior remains owned by the graph, runtime, diagnostics, and
//! integration modules that implement it.

pub mod adapters;
pub mod branch;
pub mod core;
pub mod diagnostics;
pub mod history;
pub mod runtime;
pub mod schema;
pub mod specialist;

pub use crate::runtime_policy::{
    compile_signal_runtime_policy, AdmittedSignalRuntimePolicy, InstalledSignalRuntimePolicy,
    ParallelAdmissionPolicy, ResolvedSignalRuntimePolicy, SignalObservationCapturePlan,
    SignalRuntimePolicy, SignalRuntimePolicyAdmissionDenial, SignalRuntimePolicyCompilationDenial,
    SignalRuntimePolicyRequest,
};

#[cfg(test)]
pub mod advanced;
#[cfg(test)]
pub mod integration;

#[cfg(test)]
pub use self::adapters::*;
#[cfg(test)]
pub use self::core::*;
#[cfg(not(test))]
pub use self::core::{
    apply_installed_scoped_changes, mark_changed, mark_changed_with_regions, mark_dirty,
    mark_dirty_with_regions, resolve_signal_delta_threshold, AdmittedHostComputedReadSet,
    AfterCondition, Aspect, AspectMask, AspectVersion, AtOrAfterCondition,
    BoundedTemporalReadyPromotionSummary, CanonicalChangedRegions, ChangedRegion,
    ClockAdvanceOrdinal, ClockAdvanceRequest, ClockAuthority, ClockCheckpointId, ClockDomain,
    ClockTick, CommittedHostComputedArtifact, ComparatorPolicyResolver, ConditionEvaluationContext,
    DebounceCondition, DeferredTemporalEligibility, DeniedHostComputedEvaluation,
    DeniedHostComputedReadSet, DependencyEdge, EvaluationCondition, HostComputedApiFamily,
    HostComputedDenialClass, HostComputedDependencyPatch, HostComputedDescriptor,
    HostComputedDescriptorId, HostComputedDiagnosticsSummary, HostComputedEvaluationOutcome,
    HostComputedEvaluationRequest, HostComputedEvaluationResponse, HostComputedEvaluator,
    HostComputedFailure, HostComputedFailureClass, HostComputedOutcomeClass,
    HostComputedPreparedResponse, InstalledSignalAspectCapability,
    InstalledSignalAspectLoweringAuthority, InstalledSignalAspectSetCapability,
    InstalledSignalAuthorizationPolicy, InstalledSignalComparatorIdentity,
    InstalledSignalComparatorUse, InstalledSignalConditionDecision,
    InstalledSignalConditionIdentity, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, InstalledSignalGraphCapability,
    InstalledSignalNodeCapability, InstalledSignalScopedChange, InstalledSignalScopedChangeSet,
    InstalledSignalScopedChangeView, IntervalAnchor, IntervalCondition, IntervalPeriod,
    LoweredTemporalEligibility, MissedTickPolicy, NodeBuilder, NodeEvaluationResult, NodeId,
    NodeState, OutputChange, OutputIdentity, PartitionMatchMode, PartitionSubscription,
    PartitionToken, PreparedHostComputedEvaluation, ReadyTemporalEligibility, RuntimeClockBasis,
    SignalAspectLoweringOwner, SignalAspectLoweringOwnershipDenial,
    SignalAuthorizationClauseContract, SignalAuthorizationClauseObservation,
    SignalAuthorizationDecision, SignalAuthorizationDecisionEvidence, SignalAuthorizationDenial,
    SignalAuthorizationDependencyCardinality, SignalAuthorizationEvaluationCounters,
    SignalAuthorizationObservation, SignalAuthorizationPolicyDefinition,
    SignalAuthorizationPolicyIdentity, SignalAuthorizationRequirementContract,
    SignalAuthorizationRequirementObservation, SignalAuthorizationRuleContract,
    SignalAuthorizationRuleDecisionEvidence, SignalAuthorizationRuleEffect,
    SignalAuthorizationRuleObservation, SignalConditionalArtifactReuse,
    SignalConditionalArtifactReuseClass, SignalConditionalArtifactReusePolicy,
    SignalConditionalComparatorClass, SignalConditionalComparatorPosition,
    SignalConditionalComparisonWork, SignalConditionalCondition, SignalConditionalConditionClass,
    SignalConditionalContractDefinition, SignalConditionalContractDenial,
    SignalConditionalDecisionClass, SignalConditionalDecisionCounters,
    SignalConditionalDecisionEvidence, SignalConditionalDecisionIdentityKind,
    SignalConditionalDecisionProjectionIdentity, SignalConditionalExecutionAffinity,
    SignalConditionalExecutionAffinityComparisonMismatch,
    SignalConditionalExecutionAffinityMismatch, SignalConditionalExecutionFailure,
    SignalConditionalExecutionRequest, SignalConditionalSemanticComparisonMismatch,
    SignalConditionalSemanticContinuity, SignalConditionalSemanticMismatch,
    SignalConditionalVersionComparator, SignalDeltaThresholdContract, SignalError, SignalGraph,
    SignalGraphLifecycleProbe, SignalInstalledScopedChangeDenial,
    SignalInstalledScopedChangeOutcome, SignalThresholdBoundary, SignalThresholdComparisonDomain,
    SignalThresholdValueFamily, StagedHostComputedArtifact, StaleAfterCondition,
    TemporalClockAdvanceSummary, TemporalCondition, TemporalDuration, TemporalEligibilityAuthority,
    TemporalExecutionSummary, TemporalReadyPromotionSummary, TemporalWakeAdmissionSummary,
    TemporalWakeOwner, TemporalWakeRetirementBatch, ThrottleCondition, ValidatedClockAdvance,
    VersionComparatorPolicy, VersionComparatorResolver, CORE_STORAGE_PROFILE_ID, MAX_ASPECTS,
};

#[cfg(test)]
pub use self::diagnostics::*;
#[cfg(not(test))]
pub use self::diagnostics::{diagnostics_for_graph, diagnostics_for_runtime};
#[cfg(test)]
pub use self::diagnostics::{
    DiagnosticsLevel as DiagnosticsTier, LineageEvent as LineageRecord, ReplayView as ReplaySlice,
};

#[cfg(not(test))]
pub use self::history::RuntimeBranchId as SignalBranchId;
#[cfg(test)]
pub use self::history::{
    RuntimeBranch as SignalBranchHandle, RuntimeBranchId as SignalBranchId,
    RuntimeSnapshot as SignalSnapshotV1, RuntimeSnapshotMeta as SignalSnapshotMeta,
};

#[cfg(test)]
pub use self::runtime::*;
#[cfg(test)]
#[allow(deprecated)]
pub use self::runtime::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
    SignalBranchBasis, SignalBranchBasisArtifact, SignalBranchBasisCompactExplanation,
    SignalBranchBasisDenial, SignalBranchBasisIdentity, SignalBranchBasisValidationOutcome,
    SignalBranchHeadPosture, SignalBranchRestorePosture, SignalBranchTransactionHead,
    StaleSignalBranchBasisArtifact,
};
#[cfg(not(test))]
#[allow(deprecated)]
pub use self::runtime::{
    mark_dirty_batch, BatchChange, BatchChangeResult, BatchChangeSession, ChangeBatch,
    ChangeBatchAdmission, ChangeBatchCommit, History, IntervalWakeRegeneration,
    PlannedSignalBranchRetirement, PlannedSignalBranchRetirementBatch, PreviousValueRevision,
    ReadyTemporalWake, RecipeInstance, RetiredTemporalWake, RunSummary, RuntimeConfig,
    RuntimeMerge, RuntimePolicy, ScheduledTemporalWake, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementReason, SignalBranchRetirementReceipt, SignalObservationAdmissionDenial,
    SignalObservationCompletion, SignalObservationRequest, SignalObservationSession,
    SignalObservationSurface, SignalRuntime, SignalTransaction, TemporalFrontierSnapshot,
    TemporalPreviousValueAccess, TemporalPreviousValueReference, TemporalWakeId,
    TemporalWakeReschedule, TemporalWakeRetirementReason, TemporalWakeReuse, TemporalWakeSummary,
    TransactionOutcome, TransactionResult, TransactionTiming, WakeOrdinal,
};
#[cfg(test)]
#[allow(deprecated)]
pub use self::runtime::{
    ChangeBatch as DirtyBatch, ChangeBatchCommit as SemanticBatchCommit, History as RuntimeHistory,
    KeyedRecipe as KeyedComputation, KeyedRecipeInstance as DefinedKeyedComputation,
    RecipeFamily as ComputationFamily, RecipeInstance as DefinedComputation,
    RunSummary as EvaluationSummary, RuntimeCheckpointPolicy as CheckpointPolicy,
    RuntimeConfig as SignalRuntimeConfig, RuntimeRunRequest as RuntimeExecutionRequest,
    RuntimeTierPolicy as TierPolicy, TransactionRunRequest as TransactionExecutionRequest,
};
#[cfg(all(feature = "parallel", not(test)))]
pub use self::specialist::ParallelExecutionPolicy;
#[cfg(test)]
pub use self::specialist::{
    ComparatorPolicy as VersionComparatorPolicy, ComparatorResolver as VersionComparatorResolver,
    ConditionEvaluationContext, ConditionResolver, DefaultConditionResolver, EvaluationContext,
    EvaluationOutput, PlannedRun as PreparedEvaluation, ReadView as ExecutionReadView,
    RunMode as EvaluationRequestMode, TemporalConditionResolver,
};
#[cfg(not(test))]
pub use self::specialist::{EvaluationContext, RunMode};

#[cfg(not(test))]
pub use crate::data::async_node::{
    async_node_compile_time_boundary_proof, async_node_milestone_d_certification_run,
    async_node_milestone_d_performance_closeout, async_node_milestone_d_scenario_matrix,
    AsyncKeyedNodeCapabilityBinding, AsyncKeyedNodeCapabilityEquivalenceDenialClass,
    AsyncKeyedNodeCapabilityEquivalenceReport, AsyncKeyedNodeHistoricalParityDenialClass,
    AsyncKeyedNodeHistoricalParityReport, AsyncNodeAdmissionClass,
    AsyncNodeAdmissionClassification, AsyncNodeCapabilityAliasLoweringProof,
    AsyncNodeCapabilityDeclaration, AsyncNodeCapabilityEquivalenceDenialClass,
    AsyncNodeCapabilityEquivalenceReport, AsyncNodeCompileTimeBoundaryProof,
    AsyncNodeConditionBlockClass, AsyncNodeDownstreamDependenceFact, AsyncNodeGateStateReport,
    AsyncNodeHierarchyCancellationReport, AsyncNodeHierarchyHistoricalParityDenialClass,
    AsyncNodeHierarchyHistoricalParityReport, AsyncNodeHierarchyReplaySummary,
    AsyncNodeMilestoneDCertificationRun, AsyncNodeMilestoneDCertificationRunSummary,
    AsyncNodeMilestoneDPerformanceClaimId, AsyncNodeMilestoneDPerformanceCloseout,
    AsyncNodeMilestoneDPerformanceCloseoutRow, AsyncNodeMilestoneDPerformanceCloseoutSummary,
    AsyncNodeMilestoneDScenarioEvidenceKind, AsyncNodeMilestoneDScenarioId,
    AsyncNodeMilestoneDScenarioInputs, AsyncNodeMilestoneDScenarioMatrix,
    AsyncNodeMilestoneDScenarioMatrixSummary, AsyncNodeMilestoneDScenarioRow,
    AsyncNodePayloadContract, AsyncNodePayloadContractId, AsyncNodeRequestAdmissionReport,
    AsyncNodeRequestIntent, AsyncNodeRevalidationIntent, AsyncNodeRevalidationReport,
    DeniedAsyncKeyedNodeCapabilityEquivalence, DeniedAsyncKeyedNodeHistoricalParity,
    DeniedAsyncNodeCapabilityEquivalence, DeniedAsyncNodeHierarchyHistoricalParity,
    FrozenAsyncNodeCapabilityDescriptor, LoweredAsyncNodeCapabilityBundle,
    ValidatedAsyncNodeCapabilityDeclaration, ASYNC_NODE_COMPILE_TIME_BOUNDARY_PROOF_SCHEMA_VERSION,
    ASYNC_NODE_MILESTONE_D_CERTIFICATION_RUN_SCHEMA_VERSION,
    ASYNC_NODE_MILESTONE_D_PERFORMANCE_CLOSEOUT_SCHEMA_VERSION,
    ASYNC_NODE_MILESTONE_D_SCENARIO_MATRIX_SCHEMA_VERSION,
    REQUIRED_ASYNC_NODE_COMPILE_TIME_FIXTURES, REQUIRED_ASYNC_NODE_MILESTONE_D_PERFORMANCE_CLAIMS,
    REQUIRED_ASYNC_NODE_MILESTONE_D_SCENARIOS,
};
#[cfg(test)]
pub use crate::data::comparator::DefaultComparatorPolicyResolver;
#[cfg(test)]
pub use crate::data::comparator::DefaultComparatorResolver;
#[cfg(test)]
pub use crate::data::dependency::CanonicalDependencies;
#[cfg(test)]
pub use crate::data::graph::{GcPressure, ObservationLevel, ParallelismHint};
#[cfg(test)]
pub use crate::data::output::MemoizedResultOrigin;

pub use crate::data::resource::{
    resource_certification_builder, resource_certification_bundle,
    resource_certification_bundle_parity_report, resource_milestone_b_certification_run,
    resource_milestone_b_hostile_scenario_evidence, resource_milestone_b_performance_closeout,
    resource_milestone_b_scenario_matrix, resource_milestone_c_certification_run,
    resource_milestone_c_policy_certification_builder,
    resource_milestone_c_policy_certification_bundle,
    resource_milestone_c_policy_performance_closeout, resource_milestone_c_policy_scenario_matrix,
    ActiveResourceRevalidationProof, AdmittedResourceCompletion, AdmittedResourceRequest,
    AdmittedResourceRetry, AdmittedResourceRevalidation, AsyncDenialId, CancelledResourceRequest,
    CommittedResourceCompletionArtifact, CompletionDenialClass, DeniedResourceCancellation,
    DeniedResourceCompletion, DeniedResourcePolicyRestoreCompatibility, DeniedResourceRejection,
    DeniedResourceRetry, DeniedResourceRevalidation, DeniedResourceTimeout,
    DeniedResourceTimeoutHeartbeatExtension, DependencyChangeResourceRevalidationProof,
    ExtendedResourceTimeoutHeartbeat, FrozenResourcePolicyDescriptor,
    FrozenResourcePolicyDescriptorSet, FrozenResourcePolicyRegistry,
    FulfilledLifecycleResourceRevalidationProof, InFlightResourceRequest,
    LoweredResourceDescriptor, LoweredResourcePolicyBundle, ObservedResourceNodeState,
    ObserverDemandResourceRevalidationProof, RawCompletionEnvelope, RejectedResourceRequest,
    ResourceAttemptId, ResourceBoundaryKind, ResourceBoundaryPerformanceEnvelope,
    ResourceBranchEpoch, ResourceBranchRestoreReport, ResourceCancellationDecisionClass,
    ResourceCancellationDecisionPlan, ResourceCancellationDenialClass,
    ResourceCancellationGraceWindow, ResourceCancellationOrdinal,
    ResourceCancellationPolicyDeclaration, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCertificationBuilder, ResourceCertificationBundle,
    ResourceCertificationBundleMismatchClass, ResourceCertificationBundleParityReport,
    ResourceCertificationFailure, ResourceCertificationFamily, ResourceCertificationRecord,
    ResourceCertificationSummary, ResourceCompletionAdmissionReport,
    ResourceCompletionBatchAdmissionReport, ResourceCompletionCommitReport,
    ResourceCompletionDenialStagingReport, ResourceCompletionOrdinal,
    ResourceCompletionRollbackReport, ResourceCompletionRollbackSubject,
    ResourceCompletionStagingReport, ResourceCostContractId, ResourceCostPosture,
    ResourceDeclarationReport, ResourceDensityStrategy, ResourceDependentCancellationPropagation,
    ResourceDescriptorId, ResourceDescriptorVersion, ResourceDiagnosticsDecisionClass,
    ResourceDiagnosticsDecisionPlan, ResourceDiagnosticsExpansionBudget,
    ResourceDiagnosticsExpansionDenial, ResourceDiagnosticsExpansionDenialClass,
    ResourceDiagnosticsPolicyDeclaration, ResourceDiagnosticsSummary, ResourceGeneration,
    ResourceHostCancellationAdvisory, ResourceInFlightStatus, ResourceInitialLifecycleClass,
    ResourceIntentEquivalenceCoalescing, ResourceLifecycleClass, ResourceLifecycleOrdinal,
    ResourceLifecyclePolicyDeclaration, ResourceLifecycleRetentionCompactionReport,
    ResourceLifecycleSummary, ResourceLifecycleTransition, ResourceLifecycleTransitionKind,
    ResourceManagedQueueBinding, ResourceManagedQueueCounters, ResourceManagedQueueDenial,
    ResourceManagedQueueDenialClass, ResourceManagedQueueMutationKind,
    ResourceManagedQueueMutationReport, ResourceMilestoneBCertificationRun,
    ResourceMilestoneBCertificationRunSummary, ResourceMilestoneBHostileScenarioEvidence,
    ResourceMilestoneBHostileScenarioEvidenceRow, ResourceMilestoneBPerformanceClaimId,
    ResourceMilestoneBPerformanceCloseout, ResourceMilestoneBPerformanceCloseoutRow,
    ResourceMilestoneBPerformanceCloseoutSummary, ResourceMilestoneBScenarioEvidenceKind,
    ResourceMilestoneBScenarioId, ResourceMilestoneBScenarioMatrix,
    ResourceMilestoneBScenarioMatrixSummary, ResourceMilestoneBScenarioRow,
    ResourceMilestoneCCertificationRun, ResourceMilestoneCCertificationRunSummary,
    ResourceMilestoneCPolicyCertificationBuilder, ResourceMilestoneCPolicyCertificationBundle,
    ResourceMilestoneCPolicyCertificationFamily, ResourceMilestoneCPolicyCertificationRecord,
    ResourceMilestoneCPolicyCertificationSummary, ResourceMilestoneCPolicyPerformanceClaimId,
    ResourceMilestoneCPolicyPerformanceCloseout, ResourceMilestoneCPolicyPerformanceCloseoutRow,
    ResourceMilestoneCPolicyPerformanceCloseoutSummary,
    ResourceMilestoneCPolicyScenarioEvidenceKind, ResourceMilestoneCPolicyScenarioId,
    ResourceMilestoneCPolicyScenarioMatrix, ResourceMilestoneCPolicyScenarioMatrixSummary,
    ResourceMilestoneCPolicyScenarioRow, ResourceNodeDeclaration, ResourceNodeId,
    ResourceObservationBatchReport, ResourceObservationDecisionClass,
    ResourceObservationDecisionPlan, ResourceObservationEvent,
    ResourceObservationPolicyDeclaration, ResourceOldHostWorkCancellationAdvisory,
    ResourceOutputContinuity, ResourceOutputContinuityDecisionClass,
    ResourceOutputContinuityDecisionPlan, ResourceOutputContinuityPolicyDeclaration,
    ResourceOverlappingGenerationAdmission, ResourcePayloadContract, ResourcePayloadContractDigest,
    ResourcePayloadContractId, ResourcePolicyCompatibilityClass,
    ResourcePolicyCompatibilityFamilyReport, ResourcePolicyCompatibilityPosture,
    ResourcePolicyCompatibilityReport, ResourcePolicyDescriptor, ResourcePolicyDescriptorId,
    ResourcePolicyDigest, ResourcePolicyKind, ResourcePolicyName, ResourcePolicyRegistration,
    ResourcePolicyRegistryError, ResourcePolicyRegistryFreezeReport, ResourcePolicyResolutionError,
    ResourcePolicyRestoreCompatibilityDenialClass, ResourcePolicyRestoreCompatibilityProof,
    ResourcePolicySelectionBasis, ResourcePolicyVersion, ResourceQueuePressureClass,
    ResourceQueuePressureObservation, ResourceRejectionDenialClass, ResourceRejectionOrdinal,
    ResourceRejectionReason, ResourceRejectionReport, ResourceReplayAvailabilityClass,
    ResourceReplayAvailabilityDenialClass, ResourceReplayAvailabilityReport,
    ResourceReplayDecisionClass, ResourceReplayDecisionPlan, ResourceReplayPolicyDeclaration,
    ResourceReplayReconstructionReport, ResourceRequestAdmissionReport, ResourceRequestHandle,
    ResourceRequestId, ResourceRequestIntent, ResourceRequestIntentDigest, ResourceResolvedPolicy,
    ResourceResolvedPolicyBundle, ResourceRetainedDeniedCompletionAvailability,
    ResourceRetainedDeniedCompletionAvailabilityClass, ResourceRetainedHistoryAvailability,
    ResourceRetainedHistoryAvailabilityClass, ResourceRetainedRetryLineageAvailability,
    ResourceRetainedRetryLineageAvailabilityClass, ResourceRetentionCompactionBudget,
    ResourceRetentionDecisionClass, ResourceRetentionDecisionPlan,
    ResourceRetentionPolicyDeclaration, ResourceRetryAdmissionReport, ResourceRetryBudgetScope,
    ResourceRetryDecisionClass, ResourceRetryDecisionPlan, ResourceRetryDenialClass,
    ResourceRetryOrdinal, ResourceRetryPolicyDeclaration, ResourceRetryReason,
    ResourceRetryScheduleReport, ResourceRevalidationCoalescing, ResourceRevalidationDecisionClass,
    ResourceRevalidationDecisionPlan, ResourceRevalidationDenialClass,
    ResourceRevalidationEvidence, ResourceRevalidationFreshnessClass,
    ResourceRevalidationFreshnessDecision, ResourceRevalidationIntent,
    ResourceRevalidationPolicyDeclaration, ResourceRevalidationReport, ResourceRuntimeSummary,
    ResourceRuntimeSummaryReadReport, ResourceSafePointObservationCounters,
    ResourceSafePointObservationDenial, ResourceSafePointObservationDenialClass,
    ResourceSafePointObservationOrdinal, ResourceSafePointObservationReport,
    ResourceStaleAfterDecisionClass, ResourceStaleAfterDecisionPlan,
    ResourceStaleAfterPolicyDeclaration, ResourceSupersessionDecisionClass,
    ResourceSupersessionDecisionPlan, ResourceSupersessionOldHostWorkPosture,
    ResourceSupersessionOrdinal, ResourceSupersessionOverlapDisposition,
    ResourceSupersessionPolicyDeclaration, ResourceSupersessionRecord,
    ResourceTimeoutDeadlineAuthority, ResourceTimeoutDecisionClass, ResourceTimeoutDecisionPlan,
    ResourceTimeoutDenialClass, ResourceTimeoutHeartbeatExtensionDenialClass,
    ResourceTimeoutHeartbeatExtensionReport, ResourceTimeoutOrdinal, ResourceTimeoutOutcomeClass,
    ResourceTimeoutPolicyDeclaration, ResourceTimeoutReport, RetainedResourceRetryLineage,
    RolledBackResourceCompletionArtifact, ScheduledResourceRetry,
    StagedDeniedResourceCompletionEffect, StagedResourceCompletionEffect,
    TerminalStateResourceRevalidationProof, TimedOutResourceRequest, ValidatedCompletionEnvelope,
    ValidatedResourcePolicyDeclaration, ValidatedResourcePolicyReference,
    REQUIRED_RESOURCE_CERTIFICATION_FAMILIES, REQUIRED_RESOURCE_MILESTONE_B_HOSTILE_SCENARIOS,
    REQUIRED_RESOURCE_MILESTONE_B_PERFORMANCE_CLAIMS, REQUIRED_RESOURCE_MILESTONE_B_SCENARIOS,
    REQUIRED_RESOURCE_MILESTONE_C_POLICY_CERTIFICATION_FAMILIES,
    REQUIRED_RESOURCE_MILESTONE_C_POLICY_PERFORMANCE_CLAIMS,
    REQUIRED_RESOURCE_MILESTONE_C_POLICY_SCENARIOS,
    RESOURCE_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    RESOURCE_CERTIFICATION_BUNDLE_SCHEMA_VERSION, RESOURCE_DIAGNOSTICS_SUMMARY_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_CERTIFICATION_RUN_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_HOSTILE_SCENARIO_EVIDENCE_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_PERFORMANCE_CLOSEOUT_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_SCENARIO_MATRIX_SCHEMA_VERSION,
    RESOURCE_MILESTONE_C_CERTIFICATION_RUN_SCHEMA_VERSION,
    RESOURCE_MILESTONE_C_POLICY_CERTIFICATION_BUNDLE_SCHEMA_VERSION,
    RESOURCE_MILESTONE_C_POLICY_PERFORMANCE_CLOSEOUT_SCHEMA_VERSION,
    RESOURCE_MILESTONE_C_POLICY_SCENARIO_MATRIX_SCHEMA_VERSION,
};

#[cfg(test)]
pub use crate::data::trace::{
    ArtifactAuthorityClass, ArtifactMergeAuthority, CausalityMetadata, MergeAdoptability,
    RetainedDiagnosticArtifact,
};
#[cfg(test)]
pub use crate::logic::evaluation::IntoEvaluationOutput;
#[cfg(test)]
pub use crate::logic::events::*;
#[cfg(test)]
pub use crate::logic::explain::{
    CausalDisposition, ConditionDecision, NodeExplanation, ScopeProvenanceKind, UpstreamCause,
};
#[cfg(test)]
pub use crate::logic::planner::*;
#[cfg(test)]
pub use crate::logic::transaction::{DecisionDetail, DecisionRecord};
#[cfg(test)]
pub use crate::presentation::boundaries::contracts::*;
#[cfg(test)]
pub use crate::presentation::boundaries::transaction_contract::*;
#[cfg(test)]
pub use crate::presentation::harness::*;
#[cfg(test)]
pub use crate::presentation::metrics::{GraphMetrics, RuntimeMetrics};
#[cfg(test)]
pub use crate::presentation::outputs::deployment::*;
#[cfg(test)]
pub use crate::state::{
    SignalSnapshotId, SnapshotArtifactRestoreMode, SnapshotDependencyRestoreMode,
    SnapshotRestoreCoarseReason, SnapshotRestoreIntent, SnapshotStateRestoreMode,
};
#[cfg(test)]
pub use crate::tests::support::GraphDependencyBatchExt;
