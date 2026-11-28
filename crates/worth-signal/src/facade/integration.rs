pub use crate::data::dirty_set::{BatchedDirtySet, DomainImpact};
pub use crate::data::effect_mapping::EffectMapping;
pub use crate::data::evaluator::CheckpointEvaluator;
pub use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
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
    SignalInvalidationCausalObservation, SignalInvalidationExecutionObservation,
    SignalInvalidationExecutionReceipt,
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
    branch_state_proof_report, canonical_digest, replay_artifact_proof_report,
    replay_parity_proof_report, runtime_proof_report, BranchMutationLedger,
    BranchStateDenseGridProofBasis, BranchStateProofBasis, BranchStateProofReport,
    ReplayArtifactProofInput, ReplayArtifactProofReport, ReplayMismatchClass,
    ReplayParityProofReport, RuntimeMaterializer, RuntimeProofReport,
    BRANCH_STATE_PROOF_BASIS_VERSION, BRANCH_STATE_PROOF_SCHEMA_VERSION,
};
