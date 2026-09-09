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
pub use crate::data::checkpoint::CheckpointBarrier;
pub use crate::data::checkpoint_policy::CheckpointPolicy as RuntimeCheckpointPolicy;
pub use crate::data::output::ComputationFamily as RecipeFamily;
pub use crate::data::output::ComputationKey as RecipeKey;
pub use crate::data::output::KeyedComputation as KeyedRecipe;
pub use crate::data::proof::DirtyBatch as ChangeBatch;
pub use crate::data::proof::SourceRecomputeAdmission as ChangeBatchAdmission;
#[deprecated(note = "use ChangeBatchAdmission; this records root admission, not commit")]
pub type ChangeBatchCommit = ChangeBatchAdmission;
pub use crate::data::resource::{
    resource_certification_builder, resource_certification_bundle,
    resource_certification_bundle_parity_report, resource_milestone_b_certification_run,
    resource_milestone_b_hostile_scenario_evidence, resource_milestone_b_performance_closeout,
    resource_milestone_b_scenario_matrix, CommittedResourceCompletionArtifact,
    DeniedResourcePolicyRestoreCompatibility, DeniedResourceRejection,
    FrozenResourcePolicyDescriptor, FrozenResourcePolicyDescriptorSet, LoweredResourceDescriptor,
    LoweredResourcePolicyBundle, RejectedResourceRequest, ResourceBoundaryKind,
    ResourceBoundaryPerformanceEnvelope, ResourceBranchRestoreReport,
    ResourceCancellationDecisionClass, ResourceCancellationDecisionPlan,
    ResourceCertificationBuilder, ResourceCertificationBundle,
    ResourceCertificationBundleMismatchClass, ResourceCertificationBundleParityReport,
    ResourceCertificationFailure, ResourceCertificationFamily, ResourceCertificationRecord,
    ResourceCertificationSummary, ResourceCompletionBatchAdmissionReport,
    ResourceCompletionCommitReport, ResourceCompletionDenialStagingReport,
    ResourceCompletionRollbackReport, ResourceCompletionRollbackSubject,
    ResourceCompletionStagingReport, ResourceDeclarationReport, ResourceDensityStrategy,
    ResourceDependentCancellationPropagation, ResourceDescriptorId, ResourceDescriptorVersion,
    ResourceDiagnosticsDecisionClass, ResourceDiagnosticsDecisionPlan,
    ResourceDiagnosticsExpansionBudget, ResourceDiagnosticsExpansionDenial,
    ResourceDiagnosticsExpansionDenialClass, ResourceDiagnosticsPolicyDeclaration,
    ResourceDiagnosticsSummary, ResourceHostCancellationAdvisory,
    ResourceIntentEquivalenceCoalescing, ResourceLifecycleRetentionCompactionReport,
    ResourceLifecycleSummary, ResourceMilestoneBCertificationRun,
    ResourceMilestoneBCertificationRunSummary, ResourceMilestoneBHostileScenarioEvidence,
    ResourceMilestoneBHostileScenarioEvidenceRow, ResourceMilestoneBPerformanceClaimId,
    ResourceMilestoneBPerformanceCloseout, ResourceMilestoneBPerformanceCloseoutRow,
    ResourceMilestoneBPerformanceCloseoutSummary, ResourceMilestoneBScenarioEvidenceKind,
    ResourceMilestoneBScenarioId, ResourceMilestoneBScenarioMatrix,
    ResourceMilestoneBScenarioMatrixSummary, ResourceMilestoneBScenarioRow,
    ResourceOldHostWorkCancellationAdvisory, ResourceOverlappingGenerationAdmission,
    ResourcePayloadContractDigest, ResourcePolicyCompatibilityClass,
    ResourcePolicyCompatibilityFamilyReport, ResourcePolicyCompatibilityReport,
    ResourcePolicyDescriptor, ResourcePolicyDigest, ResourcePolicyKind,
    ResourcePolicyRegistryFreezeReport, ResourcePolicyRestoreCompatibilityDenialClass,
    ResourcePolicyRestoreCompatibilityProof, ResourcePolicySelectionBasis, ResourceRejectionReport,
    ResourceReplayReconstructionReport, ResourceRequestAdmissionReport,
    ResourceRequestIntentDigest, ResourceResolvedPolicy, ResourceResolvedPolicyBundle,
    ResourceRetainedDeniedCompletionAvailability,
    ResourceRetainedDeniedCompletionAvailabilityClass, ResourceRetainedHistoryAvailability,
    ResourceRetainedHistoryAvailabilityClass, ResourceRetainedRetryLineageAvailability,
    ResourceRetainedRetryLineageAvailabilityClass, ResourceRetentionCompactionBudget,
    ResourceRetentionDecisionClass, ResourceRetentionDecisionPlan, ResourceRetryAdmissionReport,
    ResourceRetryBudgetScope, ResourceRetryDecisionClass, ResourceRetryDecisionPlan,
    ResourceRetryScheduleReport, ResourceRevalidationReport, ResourceRuntimeSummary,
    ResourceRuntimeSummaryReadReport, ResourceSupersessionDecisionClass,
    ResourceSupersessionDecisionPlan, ResourceSupersessionOldHostWorkPosture,
    ResourceSupersessionOverlapDisposition, ResourceSupersessionRecord,
    ResourceTimeoutDecisionClass, ResourceTimeoutDecisionPlan, ResourceTimeoutReport,
    RetainedResourceRetryLineage, RolledBackResourceCompletionArtifact,
    StagedDeniedResourceCompletionEffect, StagedResourceCompletionEffect,
    ValidatedResourcePolicyDeclaration, ValidatedResourcePolicyReference,
    REQUIRED_RESOURCE_CERTIFICATION_FAMILIES, REQUIRED_RESOURCE_MILESTONE_B_HOSTILE_SCENARIOS,
    REQUIRED_RESOURCE_MILESTONE_B_PERFORMANCE_CLAIMS, REQUIRED_RESOURCE_MILESTONE_B_SCENARIOS,
    RESOURCE_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    RESOURCE_CERTIFICATION_BUNDLE_SCHEMA_VERSION, RESOURCE_DIAGNOSTICS_SUMMARY_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_CERTIFICATION_RUN_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_HOSTILE_SCENARIO_EVIDENCE_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_PERFORMANCE_CLOSEOUT_SCHEMA_VERSION,
    RESOURCE_MILESTONE_B_SCENARIO_MATRIX_SCHEMA_VERSION,
};
pub use crate::data::temporal::{
    BoundedTemporalReadyPromotionSummary, IntervalWakeRegeneration, PreviousValueRevision,
    ReadyTemporalWake, RetiredTemporalWake, ScheduledTemporalWake, TemporalClockAdvanceSummary,
    TemporalFrontierSnapshot, TemporalPreviousValueAccess, TemporalPreviousValueReference,
    TemporalReadyPromotionSummary, TemporalWakeAdmissionSummary, TemporalWakeId, TemporalWakeOwner,
    TemporalWakeReschedule, TemporalWakeRetirementBatch, TemporalWakeRetirementReason,
    TemporalWakeReuse, TemporalWakeSummary, WakeOrdinal,
};
pub use crate::data::tier::TierPolicy as RuntimeTierPolicy;
pub use crate::data::tier::{DependencyMode, DirtyPropagation, EvaluationTrigger};
pub use crate::logic::invalidation::mark_dirty_batch;
pub use crate::logic::transaction::DefinedComputation as RecipeInstance;
pub use crate::logic::transaction::DefinedKeyedComputation as KeyedRecipeInstance;
pub use crate::logic::transaction::EvaluationSummary as RunSummary;
pub use crate::logic::transaction::RuntimeExecutionRequest as RuntimeRunRequest;
pub use crate::logic::transaction::RuntimeHistory as History;
#[cfg(test)]
pub use crate::logic::transaction::SignalRuntimeConfig;
pub use crate::logic::transaction::SignalRuntimeConfig as RuntimeConfig;
pub use crate::logic::transaction::TransactionExecutionRequest as TransactionRunRequest;
#[cfg(test)]
pub use crate::logic::transaction::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
    SignalBranchBasis, SignalBranchBasisArtifact, SignalBranchBasisCompactExplanation,
    SignalBranchBasisDenial, SignalBranchBasisIdentity, SignalBranchBasisValidationOutcome,
    SignalBranchHeadPosture, SignalBranchRestorePosture, SignalBranchTransactionHead,
    StaleSignalBranchBasisArtifact,
};
pub use crate::logic::transaction::{
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
    TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
pub use crate::logic::transaction::{
    BatchChangeSession, PlannedRuntimeMerge, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, Recipe, RequiredDerivedRebuildSet, RuntimeMerge,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
    SignalObservationAdmissionDenial, SignalObservationCompletion, SignalObservationRequest,
    SignalObservationSession, SignalObservationSurface, SignalRuntime, SignalRuntimeBuilder,
    SignalTransaction, TemporalCertificationBuilder, TemporalCertificationBundle,
    TemporalCertificationBundleMismatchClass, TemporalCertificationBundleParityReport,
    TemporalCertificationFailure, TemporalCertificationFamily, TemporalCertificationRecord,
    TemporalCertificationSummary, TemporalEligibilityFact, TemporalReconstructabilityArtifact,
    TemporalReplayMismatchClass, TemporalReplayParityReport, TemporalStateRebuildProof,
    TemporalTransactionEvidence, TransactionOutcome, TransactionResult, TransactionTiming,
};
#[cfg(test)]
pub use crate::logic::transaction::{
    BranchTargetedTransactionDenial, BranchTargetedTransactionRequest, SignalBranchForkDenial,
    SignalBranchForkReceipt, SignalBranchForkRequest, SignalBranchForkRequestBasis,
    ValidatedBranchTargetedTransactionRequest,
};
pub use crate::logic::transaction::{
    CommittedObservationEventSummary, MatchingObserverSet, ObservationBoundaryOutcome,
    ObservationBoundarySummary, ObservationDeliveryMode, ObservationHandle, ObservationHandleId,
    ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
    ObservationRegistrySummary, ObservationTrigger, ObservedNodeSet, ObserverId,
};
pub use crate::runtime_policy::SignalConditionalEvaluationBudget;
pub use crate::runtime_policy::SignalConditionalTemporalBudget;
pub use crate::runtime_policy::SignalObservationCapturePlan;
pub use crate::runtime_policy::SignalRuntimePolicy;
pub use crate::runtime_policy::SignalRuntimePolicy as RuntimePolicy;
pub use crate::schema::data::SignalSchemaRegistry;
pub type BatchChange = ChangeBatch;
pub type BatchChangeResult = ChangeBatchAdmission;
#[cfg(test)]
pub type CheckpointPolicy<D> = RuntimeCheckpointPolicy<D>;
#[cfg(test)]
pub type ComputationFamily = RecipeFamily;
#[cfg(test)]
pub type ComputationKey = RecipeKey;
#[cfg(test)]
pub type KeyedComputation = KeyedRecipe;
#[cfg(test)]
pub type DirtyBatch = ChangeBatch;
#[cfg(test)]
#[deprecated(note = "use ChangeBatchAdmission; this records root admission, not commit")]
pub type SemanticBatchCommit = ChangeBatchAdmission;
#[cfg(test)]
pub type TierPolicy<T> = RuntimeTierPolicy<T>;
#[cfg(test)]
pub type DefinedComputation<T, F> = RecipeInstance<T, F>;
#[cfg(test)]
pub type DefinedKeyedComputation<'a, T, F> = KeyedRecipeInstance<'a, T, F>;
#[cfg(test)]
pub type EvaluationSummary = RunSummary;
#[cfg(test)]
pub type RuntimeExecutionRequest<'a, D, I, E, Ctx, T> = RuntimeRunRequest<'a, D, I, E, Ctx, T>;
#[cfg(test)]
pub type RuntimeHistory<'a, D, I, E, Ctx, T> = History<'a, D, I, E, Ctx, T>;
#[cfg(test)]
pub type TransactionExecutionRequest<'tx, 'a, D, I, E, Ctx, T> =
    TransactionRunRequest<'tx, 'a, D, I, E, Ctx, T>;
