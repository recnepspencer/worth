pub use crate::data::comparator::DefaultComparatorPolicyResolver as DefaultComparatorResolver;
pub use crate::data::comparator::DefaultComparatorResolver as ComparatorHookResolver;
pub use crate::data::comparator::VersionComparatorPolicy as ComparatorPolicy;
pub use crate::data::comparator::VersionComparatorResolver as ComparatorResolver;
pub use crate::data::comparator::{ComparatorPolicyResolver, TierPolicyResolver};
pub use crate::data::graph::{
    EvaluationStrategy, GcPressure, GraphMaterializer, GraphObserver, ObservationLevel,
};
pub use crate::data::proof::FrontierValidationDecision;
pub use crate::logic::checked_context::CheckedEvaluationContext;
pub use crate::logic::checkpoint::CheckpointRuntime;
pub use crate::logic::context::EvaluationContext;
pub use crate::logic::evaluation::EvaluationRequestMode as RunMode;
pub use crate::logic::evaluation::{
    AppliedEffectReport, ConditionEvaluationContext, ConditionResolver, DefaultConditionResolver,
    DeferralReason, DiagnosticEnvelope, EvaluationExecutionMetadata, EvaluationOutput,
    EvaluationVerdict, IntoEvaluationOutput, OperationalEffect, SuppressionReason,
    TemporalConditionResolver,
};
pub use crate::logic::events::{EventBus, EventFlushError, SubscriberRegistryError};
pub use crate::logic::planner::{
    build_evaluation_plan, execute_prepared_plan, CandidateTask, EligibleTask, EvaluationPlan,
    ExecutedTask, ExecutionPruneReason, ExecutionRecordId, ExecutionReport, ExecutionStage,
    FrontierRouteEvidenceReason, FrontierRouteEvidenceReceipt, FrontierRouteEvidenceReceiptError,
    FrontierRouteSerialFallbackReason, ParallelAdmissionReason, PlanSummary,
    ResolvedExecutionStrategy, ResolvedMaintenanceStrategy, ResolvedSignalPlannerPolicy,
    SemanticSegmentId, SemanticTaskRange, StageBarrier, StageExecutionOutcome,
    StageExecutionRecord, TaskExecutionOutcome, TaskExecutionRecord, TaskReason,
};
pub use crate::logic::prepared::ExecutionReadView as ReadView;
pub use crate::logic::prepared::PreparedEvaluation as PlannedRun;
pub use crate::state::SignalBranchHandle as RuntimeBranch;
pub use crate::state::SignalBranchId as RuntimeBranchId;
pub use crate::state::SignalSnapshotMeta as RuntimeSnapshotMeta;
pub use crate::state::SignalSnapshotV1 as RuntimeSnapshot;
pub use crate::state::{
    SignalSnapshotDiagnostics, SignalSnapshotId, SnapshotArtifactRestoreMode,
    SnapshotArtifactRetentionPolicy, SnapshotDependencyRestoreMode, SnapshotRestoreCoarseReason,
    SnapshotRestoreIntent, SnapshotRestorePlan, SnapshotStateRestoreMode,
};
