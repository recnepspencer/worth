pub use crate::data::dirty_set::{BatchedDirtySet, DomainImpact};
pub use crate::data::effect_mapping::EffectMapping;
pub use crate::data::evaluator::CheckpointEvaluator;
pub use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
pub use crate::data::graph::{SignalGraphReconstitution, SignalGraphReconstitutionReport};
pub use crate::data::node::{
    ArtifactPolicyClass, AuthorityPolicy, CanonicalDependencyOrder, ComparatorBasis,
    CompileTimePerformanceContract, ContextRequirement, EquivalenceContract, IdentityBasis,
    MaintenanceMode, NodeAuthorityContract, NodeContract, NodeEvaluationConfig,
    NodeExecutionContract, NodeProjectionContract, NodeReuseContract, NodeSemanticContract,
    PathClass, PerformanceCounterSurface, PerformanceEnforcementLayer, ResolvedPerformancePolicy,
    SuppressionBasis,
};
pub use crate::data::proof::{
    attach_foundational_invalidation_performance_receipt,
    FoundationalInvalidationPerformanceReceipt, InvalidationExecutionSummary,
    InvalidationFoundationalReceiptDenial, InvalidationPlanningEstimate,
    SignalInvalidationExecutionObservation, SignalInvalidationExecutionReceipt,
};
#[allow(deprecated)]
pub use crate::data::proof::{
    CanonicalForm, DedupedNodeBatch, DeltaForm, DependencyBatchEdit, DependencySetEdit,
    DesiredState, DirtyBatch, DirtyBatchEntry, DirtyDelta, FrontierEntryClassification,
    FrontierInclusionBasis, FrontierSeedCause, InvalidationSeed, InvalidationSeedBatch,
    InvalidationTraceRecord, LocalityFootprint, LocallyOrderedShard, LoweredForm,
    MergeableOrderedStream, MixedSnapshotBatchCommit, OrderedStreamItem, OrderedStreamMergeError,
    PartitionScopeSet, PatchPlan, PendingSnapshotBatch, ResolvedForm, SemanticBatchCommit,
    SingleConsumer, SnapshotBatchCommit, SortedSourceBatch, SourceRecomputeAdmission,
    StableShapeSnapshotBatchCommit, StructuralDelta, SubscriberRepair, SubscriberRepairBatch,
    SummaryForm, TouchedScopeSummary,
};
pub use crate::data::reuse::{
    ArtifactEquivalenceContract, ArtifactSemanticBoundary, PersistentCorrespondenceEvidence,
    ReuseBasis, ReuseBoundaryAuthority, ReuseBoundaryContext, ReuseBoundaryEvidence,
    ReuseBoundaryFailure, ReuseBoundaryProof, ReuseCertificationFailure, ReuseCertificationRecord,
    ReuseCrossing, ReuseOrigin, ReuseSemanticRegionIdentity, ReuseSource, ReuseStrategy,
    ReuseStrategyBoundaryAuthority,
};
pub use crate::data::subscriber_context::{SubscriberContext, SubscriberContextError};
pub use crate::data::telemetry::{
    HostComputedTelemetry, ResourceTelemetry, RuntimeTelemetry, RuntimeTelemetryMutation,
};
pub use crate::data::telemetry::{
    InvalidationPerformedCounter, SignalInvalidationRealizedCounters,
};
pub use crate::data::tier_policy_table::TierPolicyTable;
pub use crate::logic::planner::{
    FrontierRouteEvidenceReason, FrontierRouteEvidenceReceipt, FrontierRouteEvidenceReceiptError,
    FrontierRouteSerialFallbackReason,
};
pub use crate::logic::transaction::{
    branch_state_proof_report, canonical_digest, lowered_strategy_bundle_digest,
    merge_lineage_digest, merge_plan_proof_report, merge_result_proof_report,
    replay_artifact_proof_report, replay_parity_proof_report, runtime_proof_report,
    ArtifactMergeAction, ArtifactMergeComparable, AspectMergeDecisionOutcome, AspectMergePolicy,
    AspectMergePolicyBinding, AspectMergePolicyDescriptor, AspectMergePolicyId,
    AspectMergePolicyName, AspectMergePolicyRegistration, AspectMergePolicySelectionBasis,
    AspectMergePolicyVersion, BranchMergeBase, BranchMergeConflictEvidence,
    BranchMergeConflictKind, BranchMergeConflictRecord, BranchMergeConflictSummary,
    BranchMergeCounters, BranchMergeDeletionFailureEvidence, BranchMergeDivergence,
    BranchMergeExecutionSummary, BranchMergeFailureEvidence, BranchMergeFailureKind,
    BranchMergeIdentityFailureEvidence, BranchMergeKind, BranchMergePlan,
    BranchMergeReconciliationPolicy, BranchMergeRequest, BranchMergeRequestDenial,
    BranchMergeRequestScope, BranchMergeRequestScopeFamily, BranchMergeResult, BranchMergeStrategy,
    BranchMutationJournalSlice, BranchMutationLedger, BranchStateDenseGridProofBasis,
    BranchStateProofBasis, BranchStateProofReport, CausalityCarryPolicy,
    ConflictIsolationGranularity, ConflictIsolationPolicyDescriptor, ConflictIsolationPolicyId,
    ConflictIsolationPolicyName, ConflictIsolationPolicyRegistration,
    ConflictIsolationPolicyVersion, ConflictIsolationSelectionBasis, ConflictMergePolicy,
    ConflictPolicyDescriptor, ConflictPolicyId, ConflictPolicyName, ConflictPolicyRegistration,
    ConflictPolicySelectionBasis, ConflictPolicyVersion, ConservativeOverlapExpansion,
    DeletionMergePolicy, DeletionPolicyDescriptor, DeletionPolicyId, DeletionPolicyName,
    DeletionPolicyRegistration, DeletionPolicySelectionBasis, DeletionPolicyVersion,
    DependencyFingerprint, DependencyRemapRecord, DuplicateAspectMergePolicyRegistration,
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
    LoweredMergeBasePlan, LoweredMergePlan, MergeBaseSelectionBasis, MergeBaseSelectionPolicy,
    MergeBaseStrategyDescriptor, MergeBaseStrategyId, MergeBaseStrategyName,
    MergeBaseStrategyRegistration, MergeBaseStrategyVersion, MergeBoundaryWitness,
    MergeBoundaryWitnessKind, MergeDecisionBasis, MergeNodeMap, MergePlanProofReport,
    MergeResultProofReport, MergeStrategyDescriptor, MergeStrategyId, MergeStrategyName,
    MergeStrategyRegistration, MergeStrategySelectionBasis, MergeStrategyVersion,
    MergeTouchedNodeSet, MergedArtifactRecord, NodeMergeInputState, NodeMergePlan,
    NodeReconciliationDecision, NodeReconciliationShape, NormalizedBranchMergeRequest,
    NormalizedBranchMergeRequestScope, PlannedMergeCandidateSet, ProofMinimalOverlapBasis,
    ReplayArtifactProofInput, ReplayArtifactProofReport, ReplayMismatchClass,
    ReplayParityProofReport, RetainedArtifactCarryPolicy, RuntimeArtifactCarryPolicy,
    RuntimeMaterializer, RuntimeProofReport, ScopedMergeProofPacket, SelectedMergeSemanticsBundle,
    SignalAspectPolicyInventoryEntry, SignalBranchBasisInspectionRow,
    SignalCompatibilityInspectionRow, SignalDeliveryStrategyIdentity,
    SignalInvalidationStrategyIdentity, SignalMergeStrategyIdentity, SignalMergeStrategyWitness,
    SignalMergeStrategyWitnessDenial, SignalMergeStrategyWitnessDenialKind,
    SignalMergeSupportInspectionAbsence, SignalMergeSupportInspectionAbsenceKind,
    SignalMergeSupportInspectionOutcome, SignalMergeSupportInspectionWitness,
    SignalMergeSupportReadinessPosture, SignalScopedMergeCanonicalBasisBundle,
    SignalScopedMergeCanonicalLocatorBundle, SignalScopedMergeDiagnosticRow,
    SignalScopedMergeInspectionRow, SignalScopedMergeLocatorBundle,
    SignalSelectedAspectRequestEntry, SignalStrategyInspectionRow, SourceNodeAdoptionPlanCore,
    SourceOnlyMergePolicy, SourceOnlyPolicyDescriptor, SourceOnlyPolicyId, SourceOnlyPolicyName,
    SourceOnlyPolicyRegistration, SourceOnlyPolicySelectionBasis, SourceOnlyPolicyVersion,
    StructuralMergeCandidateRecord, StructuralMergeJournalSlice, BRANCH_STATE_PROOF_BASIS_VERSION,
    MERGE_PROOF_SCHEMA_VERSION,
};
