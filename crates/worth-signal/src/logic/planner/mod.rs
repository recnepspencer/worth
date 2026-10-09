pub(crate) mod apply;
mod execution;
pub(crate) mod model;
pub(crate) mod planning;
pub(crate) mod precompute;
pub(crate) mod semantic;
#[cfg(test)]
mod tests;

pub(crate) use self::apply::stage as stage_apply;
pub(crate) use self::execution::task_reporting as reporting;
pub(crate) use self::model as types;
pub(crate) use self::planning as plan_builder;
pub(crate) use self::planning::validation;
pub(crate) use self::precompute::TemporalLoweringContext;
pub(crate) use self::precompute::{reporting as precompute_reporting, stage as stage_precompute};
pub(crate) use self::semantic::stage_recording;

pub use crate::data::performance::{ResolvedExecutionStrategy, ResolvedMaintenanceStrategy};
pub(crate) use execution::{
    execute_evaluation_session_in_scope, execute_prepared_plan_in_scope,
    execute_prepared_plan_with_policy_and_temporal_lowering, execute_prepared_plan_with_precompute,
    run_signal_execution_request_scope, run_signal_preparation_request, run_signal_request_scope,
};
#[allow(unused_imports)]
pub use execution::{execute_prepared_plan, execute_prepared_plan_with_policy};
pub(crate) use model::SessionScratch;
pub(crate) use model::StageCursor;
#[allow(unused_imports)]
pub use model::{
    ApplyFootprint, CandidateTask, ConcurrentApplyPlan, ConcurrentApplyReductionPlan,
    DisjointApplyGroup, EligibleTask, EligibleTaskAdmission, EvaluationPlan, ExecutedTask,
    ExecutionPruneReason, ExecutionRecordId, ExecutionReport, ExecutionStage,
    FrontierRouteEvidenceReason, FrontierRouteEvidenceReceipt, FrontierRouteEvidenceReceiptError,
    FrontierRouteSerialFallbackReason, LoweredApplyPlan, LoweredStagePlan, LoweredTask,
    MaybeStaleAdmission, ParallelAdmissionReason, PlanSummary, ReductionOrderingContract,
    ReductionWorkClass, ResolvedSignalPlannerPolicy, SemanticSegmentId, SemanticTaskRange,
    SerialApplyPlan, StageBarrier, StageExecutionOutcome, StageExecutionRecord,
    TaskExecutionOutcome, TaskExecutionRecord, TaskReason,
};
#[allow(unused_imports)]
pub use model::{ParallelApplyMode, ParallelExecutionKind};
pub(crate) use plan_builder::admit_direct_task_with_policy_resolver;
pub use plan_builder::{build_evaluation_plan, build_evaluation_plan_with_policy_resolver};
#[cfg(test)]
pub(crate) use tests::{
    execute_plan_with_policy_and_condition, execute_test_prepared_plan_with_resolvers,
};
