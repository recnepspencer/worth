mod async_keyed;
mod computation;
mod config;
mod execution;
mod state;
mod transaction;
pub(crate) use state::TemporalRuntimeState;

pub use computation::{DefinedComputation, DefinedKeyedComputation, Recipe};
pub use config::SignalRuntimeConfig;
pub use execution::{RuntimeExecutionRequest, TransactionExecutionRequest};
pub(crate) use state::admit_signal_observation_request;
pub(crate) use state::RuntimeObservationRegistry;
pub(crate) use state::SignalCanonicalCallerUnwind;
pub use state::{branch_state_proof_report, canonical_digest};
#[allow(unused_imports)]
pub use state::{
    bridge_signal_branch_basis_trust_boundary, bridge_signal_merge_compatibility_trust_boundary,
    lowered_strategy_bundle_digest, merge_lineage_digest, replay_artifact_proof_report,
    replay_parity_proof_report,
};
pub use state::{
    merge_plan_proof_report, merge_result_proof_report, runtime_proof_report, ArtifactMergeAction,
    ArtifactMergeComparable, AspectMergeDecisionOutcome, AspectMergePolicy,
    AspectMergePolicyBinding, AspectMergePolicyDescriptor, AspectMergePolicyId,
    AspectMergePolicyName, AspectMergePolicyRegistration, AspectMergePolicySelectionBasis,
    AspectMergePolicyVersion, BoundaryBridgedSignalBranchBasisArtifact,
    BoundaryBridgedSignalMergeCompatibilityArtifact, BranchConflictResolutionPlan, BranchMergeBase,
    BranchMergeConflictEvidence, BranchMergeConflictKind, BranchMergeConflictRecord,
    BranchMergeConflictSummary, BranchMergeCounters, BranchMergeDeletionFailureEvidence,
    BranchMergeDivergence, BranchMergeExecutionSummary, BranchMergeFailureEvidence,
    BranchMergeFailureKind, BranchMergeIdentityFailureEvidence, BranchMergeKind, BranchMergePlan,
    BranchMergeReconciliationPolicy, BranchMergeRequest, BranchMergeRequestDenial,
    BranchMergeRequestScope, BranchMergeRequestScopeFamily, BranchMergeResolutionRequirement,
    BranchMergeResult, BranchMergeScopedDenialFailureEvidence, BranchMergeScopedDenialKind,
    BranchMergeScopedDeniedLocus, BranchMergeScopedUnavailableFailureEvidence,
    BranchMergeScopedUnavailableOutcomeKind, BranchMergeScopedUnavailableReason,
    BranchMergeStrategy, BranchMutationJournalSlice, BranchMutationLedger,
    BranchStateDenseGridProofBasis, BranchStateProofBasis, BranchStateProofReport,
    BranchTargetedTransactionDenial, BranchTargetedTransactionExecutionOutcome,
    BranchTargetedTransactionRequest, CausalityCarryPolicy, ConflictIsolationGranularity,
    ConflictIsolationPolicyDescriptor, ConflictIsolationPolicyId, ConflictIsolationPolicyName,
    ConflictIsolationPolicyRegistration, ConflictIsolationPolicyVersion,
    ConflictIsolationSelectionBasis, ConflictMergePolicy, ConflictPolicyDescriptor,
    ConflictPolicyId, ConflictPolicyName, ConflictPolicyRegistration, ConflictPolicySelectionBasis,
    ConflictPolicyVersion, ConflictResolutionRecord, ConflictResolutionStrategy,
    ConservativeOverlapExpansion, DeletionMergePolicy, DeletionPolicyDescriptor, DeletionPolicyId,
    DeletionPolicyName, DeletionPolicyRegistration, DeletionPolicySelectionBasis,
    DeletionPolicyVersion, DependencyFingerprint, DependencyRemapRecord,
    DuplicateAspectMergePolicyRegistration, DuplicateConflictIsolationPolicyRegistration,
    DuplicateConflictPolicyRegistration, DuplicateDeletionPolicyRegistration,
    DuplicateIdentityMatcherRegistration, DuplicateMergeBaseStrategyRegistration,
    DuplicateMergeStrategyRegistration, DuplicateSourceOnlyPolicyRegistration,
    ExecutedBranchTargetedTransactionReceipt, ExistingTargetMergePolicy,
    FoundationalScopeLoweringDenial, FrozenAspectMergePolicyRegistry,
    FrozenConflictIsolationRegistry, FrozenConflictPolicyRegistry, FrozenDeletionPolicyRegistry,
    FrozenIdentityMatcherRegistry, FrozenMergeBaseStrategyRegistry, FrozenMergeStrategyRegistry,
    FrozenSourceOnlyPolicyRegistry, IdentityCorrespondenceBasis, IdentityCorrespondenceRecord,
    IdentityCorrespondenceStatus, IdentityMatchPolicy, IdentityMatcherDescriptor,
    IdentityMatcherId, IdentityMatcherName, IdentityMatcherRegistration,
    IdentityMatcherSelectionBasis, IdentityMatcherVersion, LoweredAspectMergeDecisionPlan,
    LoweredAspectMergeDecisionRecord, LoweredBranchTargetedTransactionPlan,
    LoweredConflictIsolationPlan, LoweredConflictIsolationRecord, LoweredDeletionPolicyPlan,
    LoweredFoundationalMergeRequest, LoweredIdentityCorrespondencePlan, LoweredMergeBasePlan,
    LoweredMergePlan, MergeBaseSelectionBasis, MergeBaseSelectionPolicy,
    MergeBaseStrategyDescriptor, MergeBaseStrategyId, MergeBaseStrategyName,
    MergeBaseStrategyRegistration, MergeBaseStrategyVersion, MergeBoundaryWitness,
    MergeBoundaryWitnessKind, MergeDecisionBasis, MergeNodeMap, MergePlanProofReport,
    MergeResultProofReport, MergeStrategyDescriptor, MergeStrategyId, MergeStrategyName,
    MergeStrategyRegistration, MergeStrategySelectionBasis, MergeStrategyVersion,
    MergeTouchedNodeSet, MergedArtifactRecord, NodeMergeInputState, NodeMergePlan,
    NodeReconciliationDecision, NodeReconciliationShape, NormalizedBranchMergeRequest,
    NormalizedBranchMergeRequestScope, PlannedMergeCandidateSet, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, ProofMinimalOverlapBasis, ReplayArtifactProofInput,
    ReplayArtifactProofReport, ReplayMismatchClass, ReplayParityProofReport,
    RetainedArtifactCarryPolicy, RuntimeArtifactCarryPolicy, RuntimeProofReport,
    ScopedMergeProofPacket, SelectedMergeSemanticsBundle, SignalAspectPolicyInventoryEntry,
    SignalBranchBasis, SignalBranchBasisArtifact, SignalBranchBasisAuthority,
    SignalBranchBasisCompactExplanation, SignalBranchBasisDenial, SignalBranchBasisIdentity,
    SignalBranchBasisInspectionRow, SignalBranchBasisReady, SignalBranchBasisValidationOutcome,
    SignalBranchForkDenial, SignalBranchForkReceipt, SignalBranchForkRequest,
    SignalBranchForkRequestBasis, SignalBranchHeadPosture, SignalBranchRestorePosture,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
    SignalBranchTransactionHead, SignalCompatibilityInspectionRow, SignalDeliveryStrategyIdentity,
    SignalInvalidationStrategyIdentity, SignalMergeCompatibilityArtifact,
    SignalMergeCompatibilityAuthority, SignalMergeCompatibilityBasis,
    SignalMergeCompatibilityDenial, SignalMergeCompatibilityDenialKind,
    SignalMergeCompatibilityFactInventory, SignalMergeCompatibilityPostureKind,
    SignalMergeCompatibilityReadmissionAuthority, SignalMergeCompatibilityReady,
    SignalMergeCompatibilityWitness, SignalMergeStrategyIdentity, SignalMergeStrategyWitness,
    SignalMergeStrategyWitnessDenial, SignalMergeStrategyWitnessDenialKind,
    SignalMergeSupportInspectionAbsence, SignalMergeSupportInspectionAbsenceKind,
    SignalMergeSupportInspectionOutcome, SignalMergeSupportInspectionWitness,
    SignalMergeSupportReadinessPosture, SignalScopedMergeCanonicalBasisBundle,
    SignalScopedMergeCanonicalLocatorBundle, SignalScopedMergeDiagnosticRow,
    SignalScopedMergeInspectionRow, SignalScopedMergeLocatorBundle,
    SignalSelectedAspectRequestEntry, SignalStrategyInspectionRow, SourceNodeAdoptionPlanCore,
    SourceOnlyMergePolicy, SourceOnlyPolicyDescriptor, SourceOnlyPolicyId, SourceOnlyPolicyName,
    SourceOnlyPolicyRegistration, SourceOnlyPolicySelectionBasis, SourceOnlyPolicyVersion,
    StaleSignalBranchBasisArtifact, StructuralMergeCandidateRecord, StructuralMergeJournalSlice,
    ValidatedBranchTargetedTransactionRequest, BRANCH_STATE_PROOF_BASIS_VERSION,
    MERGE_PROOF_SCHEMA_VERSION, SIGNAL_BRANCH_BASIS_SCHEMA_VERSION,
    SIGNAL_MERGE_COMPATIBILITY_SCHEMA_VERSION,
};
pub use state::{
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, BoundedJournalSegment, CheckpointBoundary, CheckpointRecord,
    DependencyIndexRebuildProof, JournalSegment, MatchingObserverSet, MergeSupportRebuildProof,
    ObservationDeliveryMode, ObservationHandle, ObservationHandleId, ObservationListener,
    ObservationNotice, ObservationPolicy, ObservationReadContext, ObservationRegistrySummary,
    ObservationTrigger, ObservedNodeSet, ObserverId, PlannedRuntimeMerge, ReconstructabilityProof,
    ReconstructabilityRecord, ReplaySuffixRebuildProof, RequiredDerivedRebuildSet, RuntimeHistory,
    RuntimeMaterializer, RuntimeMerge, RuntimeObserver, SignalObservationAdmissionDenial,
    SignalObservationCompletion, SignalObservationRequest, SignalObservationSession,
    SignalObservationSurface, SignalRuntime, SignalRuntimeBuilder, TemporalCertificationBuilder,
    TemporalCertificationBundle, TemporalCertificationBundleMismatchClass,
    TemporalCertificationBundleParityReport, TemporalCertificationFailure,
    TemporalCertificationFamily, TemporalCertificationRecord, TemporalCertificationSummary,
    TemporalReconstructabilityArtifact, TemporalReplayMismatchClass, TemporalReplayParityReport,
    TemporalStateRebuildProof, REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
    TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
pub(crate) use state::{
    BranchState, SignalOwnerMetadataCloseBatch, SignalOwnerMetadataState, SignalOwnerPartition,
    SignalOwnerRetirementCleanup, SignalOwnerSnapshotReservationDenial, SnapshotBranchState,
    SnapshotStatePacket,
};
pub(crate) use state::{
    SignalObservationCaptureGate, SignalObservationDropCleanup, SignalObservationSessionState,
};
#[allow(unused_imports)]
pub use transaction::{
    AdvisoryRecord, BatchChangeSession, CommittedObservationEventSummary, DecisionDetail,
    DecisionLog, DecisionRecord, DecisionSummary, EvaluationSummary, IntegrityMarkers,
    ObservationBoundaryOutcome, ObservationBoundarySummary, SignalTransaction,
    TemporalEligibilityFact, TemporalTransactionEvidence, TransactionOutcome,
    TransactionReplayEntry, TransactionResult, TransactionTiming,
};
