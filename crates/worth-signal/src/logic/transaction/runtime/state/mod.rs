mod async_capability;
mod branching;
mod builder;
mod canonical_merge_guidance;
mod guided;
mod inspection;
mod merge;
mod mutation;
mod observation;
mod observer;
mod reconstructability;
mod resource;
mod resource_observation;
mod runtime_observation;
mod runtime_state;
mod temporal;
pub(crate) use temporal::TemporalRuntimeState;

pub(crate) use crate::observation::session::admit as admit_signal_observation_request;
pub(crate) use crate::observation::session::{
    SignalObservationCaptureGate, SignalObservationDropCleanup, SignalObservationSessionState,
};
pub use branching::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
    BranchTargetedTransactionDenial, BranchTargetedTransactionExecutionOutcome,
    BranchTargetedTransactionRequest, ExecutedBranchTargetedTransactionReceipt,
    LoweredBranchTargetedTransactionPlan, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, SignalBranchBasis, SignalBranchBasisArtifact,
    SignalBranchBasisAuthority, SignalBranchBasisCompactExplanation, SignalBranchBasisDenial,
    SignalBranchBasisIdentity, SignalBranchBasisReady, SignalBranchBasisValidationOutcome,
    SignalBranchForkDenial, SignalBranchForkReceipt, SignalBranchForkRequest,
    SignalBranchForkRequestBasis, SignalBranchHeadPosture, SignalBranchRestorePosture,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
    SignalBranchTransactionHead, StaleSignalBranchBasisArtifact,
    ValidatedBranchTargetedTransactionRequest, SIGNAL_BRANCH_BASIS_SCHEMA_VERSION,
};
pub(crate) use branching::{
    BranchState, SignalCanonicalCallerUnwind, SignalOwnerMetadataCloseBatch,
    SignalOwnerMetadataState, SignalOwnerPartition, SignalOwnerRetirementCleanup,
    SignalOwnerSnapshotReservationDenial, SnapshotBranchState, SnapshotStatePacket,
};
pub use builder::SignalRuntimeBuilder;
pub use canonical_merge_guidance::{PlannedRuntimeMerge, RuntimeMerge};
#[cfg(test)]
pub(crate) use guided::RawRuntimeMerge;
pub use guided::RuntimeHistory;
pub use inspection::{
    SignalBranchBasisInspectionRow, SignalCompatibilityInspectionRow,
    SignalMergeSupportInspectionAbsence, SignalMergeSupportInspectionAbsenceKind,
    SignalMergeSupportInspectionOutcome, SignalMergeSupportInspectionWitness,
    SignalMergeSupportReadinessPosture, SignalScopedMergeInspectionRow,
    SignalStrategyInspectionRow,
};
pub use merge::bridge_signal_merge_compatibility_trust_boundary;
pub use merge::{branch_state_proof_report, canonical_digest};
pub use merge::{
    lowered_strategy_bundle_digest, merge_lineage_digest, replay_artifact_proof_report,
    replay_parity_proof_report,
};
#[allow(unused_imports)]
pub use merge::{
    merge_plan_proof_report, merge_result_proof_report, runtime_proof_report, ArtifactMergeAction,
    ArtifactMergeComparable, AspectMergeDecisionOutcome, AspectMergePolicy,
    AspectMergePolicyBinding, AspectMergePolicyDescriptor, AspectMergePolicyId,
    AspectMergePolicyName, AspectMergePolicyRegistration, AspectMergePolicySelectionBasis,
    AspectMergePolicyVersion, BoundaryBridgedSignalMergeCompatibilityArtifact,
    BranchConflictResolutionPlan, BranchMergeBase, BranchMergeConflictEvidence,
    BranchMergeConflictKind, BranchMergeConflictRecord, BranchMergeConflictSummary,
    BranchMergeCounters, BranchMergeDeletionFailureEvidence, BranchMergeDivergence,
    BranchMergeExecutionSummary, BranchMergeFailureEvidence, BranchMergeFailureKind,
    BranchMergeIdentityFailureEvidence, BranchMergeKind, BranchMergePlan,
    BranchMergeReconciliationPolicy, BranchMergeRequest, BranchMergeRequestDenial,
    BranchMergeRequestScope, BranchMergeRequestScopeFamily, BranchMergeResolutionRequirement,
    BranchMergeResult, BranchMergeScopedDenialFailureEvidence, BranchMergeScopedDenialKind,
    BranchMergeScopedDeniedLocus, BranchMergeScopedUnavailableFailureEvidence,
    BranchMergeScopedUnavailableOutcomeKind, BranchMergeScopedUnavailableReason,
    BranchMergeStrategy, BranchMutationJournalSlice, BranchMutationLedger,
    BranchStateDenseGridProofBasis, BranchStateProofBasis, BranchStateProofReport,
    CausalityCarryPolicy, ConflictIsolationGranularity, ConflictIsolationPolicyDescriptor,
    ConflictIsolationPolicyId, ConflictIsolationPolicyName, ConflictIsolationPolicyRegistration,
    ConflictIsolationPolicyVersion, ConflictIsolationSelectionBasis, ConflictMergePolicy,
    ConflictPolicyDescriptor, ConflictPolicyId, ConflictPolicyName, ConflictPolicyRegistration,
    ConflictPolicySelectionBasis, ConflictPolicyVersion, ConflictResolutionRecord,
    ConflictResolutionStrategy, ConservativeOverlapExpansion, DeletionMergePolicy,
    DeletionPolicyDescriptor, DeletionPolicyId, DeletionPolicyName, DeletionPolicyRegistration,
    DeletionPolicySelectionBasis, DeletionPolicyVersion, DependencyFingerprint,
    DependencyRemapRecord, DuplicateAspectMergePolicyRegistration,
    DuplicateConflictIsolationPolicyRegistration, DuplicateConflictPolicyRegistration,
    DuplicateDeletionPolicyRegistration, DuplicateIdentityMatcherRegistration,
    DuplicateMergeBaseStrategyRegistration, DuplicateMergeStrategyRegistration,
    DuplicateSourceOnlyPolicyRegistration, ExistingTargetMergePolicy,
    FoundationalScopeLoweringDenial, FrozenAspectMergePolicyRegistry,
    FrozenConflictIsolationRegistry, FrozenConflictPolicyRegistry, FrozenDeletionPolicyRegistry,
    FrozenIdentityMatcherRegistry, FrozenMergeBaseStrategyRegistry, FrozenMergeStrategyRegistry,
    FrozenSourceOnlyPolicyRegistry, IdentityCorrespondenceBasis, IdentityCorrespondenceRecord,
    IdentityCorrespondenceStatus, IdentityMatchPolicy, IdentityMatcherDescriptor,
    IdentityMatcherId, IdentityMatcherName, IdentityMatcherRegistration,
    IdentityMatcherSelectionBasis, IdentityMatcherVersion, LoweredAspectMergeDecisionPlan,
    LoweredAspectMergeDecisionRecord, LoweredConflictIsolationPlan, LoweredConflictIsolationRecord,
    LoweredDeletionPolicyPlan, LoweredFoundationalMergeRequest, LoweredIdentityCorrespondencePlan,
    LoweredMergeBasePlan, LoweredMergePlan, LoweredScopedMergeCandidateSet,
    MergeBaseSelectionBasis, MergeBaseSelectionPolicy, MergeBaseStrategyDescriptor,
    MergeBaseStrategyId, MergeBaseStrategyName, MergeBaseStrategyRegistration,
    MergeBaseStrategyVersion, MergeBoundaryWitness, MergeBoundaryWitnessKind, MergeDecisionBasis,
    MergeNodeMap, MergePlanProofReport, MergeResultProofReport, MergeStrategyDescriptor,
    MergeStrategyId, MergeStrategyName, MergeStrategyRegistration, MergeStrategySelectionBasis,
    MergeStrategyVersion, MergeTouchedNodeSet, MergedArtifactRecord, NodeMergeInputState,
    NodeMergePlan, NodeReconciliationDecision, NodeReconciliationShape,
    NormalizedBranchMergeRequest, NormalizedBranchMergeRequestScope, PlannedMergeCandidateSet,
    ProofMinimalOverlapBasis, ReplayArtifactProofInput, ReplayArtifactProofReport,
    ReplayMismatchClass, ReplayParityProofReport, RetainedArtifactCarryPolicy,
    RuntimeArtifactCarryPolicy, RuntimeProofReport, ScopedMergeCandidateBreadthSummary,
    ScopedMergeProofPacket, SelectedMergeSemanticsBundle, SignalAspectPolicyInventoryEntry,
    SignalDeliveryStrategyIdentity, SignalInvalidationStrategyIdentity,
    SignalMergeCompatibilityArtifact, SignalMergeCompatibilityAuthority,
    SignalMergeCompatibilityBasis, SignalMergeCompatibilityDenial,
    SignalMergeCompatibilityDenialKind, SignalMergeCompatibilityFactInventory,
    SignalMergeCompatibilityPostureKind, SignalMergeCompatibilityReadmissionAuthority,
    SignalMergeCompatibilityReady, SignalMergeCompatibilityWitness, SignalMergeStrategyIdentity,
    SignalMergeStrategyWitness, SignalMergeStrategyWitnessDenial,
    SignalMergeStrategyWitnessDenialKind, SignalScopedMergeCanonicalBasisBundle,
    SignalScopedMergeCanonicalLocatorBundle, SignalScopedMergeDiagnosticRow,
    SignalScopedMergeLocatorBundle, SignalSelectedAspectRequestEntry,
    SourceNodeAdoptionCarryPolicy, SourceNodeAdoptionPlanCore, SourceOnlyMergePolicy,
    SourceOnlyPolicyDescriptor, SourceOnlyPolicyId, SourceOnlyPolicyName,
    SourceOnlyPolicyRegistration, SourceOnlyPolicySelectionBasis, SourceOnlyPolicyVersion,
    StructuralMergeCandidateRecord, StructuralMergeJournalSlice, TargetNodeIdentityIntent,
    TopologyRepairSummary, BRANCH_STATE_PROOF_BASIS_VERSION, MERGE_PROOF_SCHEMA_VERSION,
    SIGNAL_MERGE_COMPATIBILITY_SCHEMA_VERSION,
};
pub use observation::{
    SignalObservationAdmissionDenial, SignalObservationCompletion, SignalObservationRequest,
    SignalObservationSession, SignalObservationSurface,
};
pub use observer::{RuntimeMaterializer, RuntimeObserver};
#[allow(unused_imports)]
pub use reconstructability::{
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, BoundedJournalSegment, CheckpointBoundary,
    DependencyIndexRebuildProof, MergeSupportRebuildProof, ReconstructabilityProof,
    ReplaySuffixRebuildProof, RequiredDerivedRebuildSet, TemporalCertificationBuilder,
    TemporalCertificationBundle, TemporalCertificationBundleMismatchClass,
    TemporalCertificationBundleParityReport, TemporalCertificationFailure,
    TemporalCertificationFamily, TemporalCertificationRecord, TemporalCertificationSummary,
    TemporalReconstructabilityArtifact, TemporalReplayMismatchClass, TemporalReplayParityReport,
    TemporalStateRebuildProof, REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
    TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
#[allow(unused_imports)]
pub use reconstructability::{CheckpointRecord, JournalSegment, ReconstructabilityRecord};
pub(in crate::logic::transaction::runtime) use resource::ResourceRuntimeState;
pub(crate) use runtime_observation::RuntimeObservationRegistry;
pub use runtime_observation::{
    MatchingObserverSet, ObservationDeliveryMode, ObservationHandle, ObservationHandleId,
    ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
    ObservationRegistrySummary, ObservationTrigger, ObservedNodeSet, ObserverId,
};
pub use runtime_state::SignalRuntime;
