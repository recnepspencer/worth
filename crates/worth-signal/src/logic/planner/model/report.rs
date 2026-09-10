mod retained_charge;

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::node::AuthorityPolicy;
use crate::data::output::MemoizedResultOrigin;
use crate::data::reuse::{ReuseBasis, ReuseOrigin};
use crate::data::temporal::{LoweredTemporalEligibility, TemporalExecutionSummary};
use crate::logic::evaluation::{DeferralReason, EvaluationVerdict, SuppressionReason};

use super::admission::ParallelAdmissionReason;
#[cfg(feature = "parallel")]
use super::apply::ApplyPlanSerialFallbackReason;
use super::plan::PlanSummary;
use super::task::{EligibleTask, SemanticSegmentId, SemanticTaskRange, TaskReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionPruneReason {
    CleanAtPlanTime,
    CleanAfterValidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskExecutionOutcome {
    Recomputed,
    ValidatedClean,
    ConditionDeferred,
    ConditionRevertedClean,
    MemoizedReuse,
    SnapshotRestoreReuse,
    ReconciliationAdoption,
    CrossIdentityPersistentReuse,
    PartialArtifactSplice,
    PropagationSuppressed,
    Pruned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StageExecutionOutcome {
    CompletedSerial,
    #[cfg(feature = "parallel")]
    CompletedParallel,
}

#[cfg(feature = "parallel")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelExecutionKind {
    StagedParallelPrecompute,
    FullParallel,
}

#[cfg(feature = "parallel")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelApplyMode {
    SerialApply,
    GroupedConcurrentApply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskExecutionRecord {
    pub id: super::task::ExecutionRecordId,
    pub semantic_segment_id: SemanticSegmentId,
    pub node: NodeId,
    pub scheduled_reason: TaskReason,
    pub direct_request: bool,
    pub outcome: TaskExecutionOutcome,
    pub verdict: Option<EvaluationVerdict>,
    pub suppression_reason: Option<SuppressionReason>,
    pub deferral_reason: Option<DeferralReason>,
    pub temporal_eligibility: Option<LoweredTemporalEligibility>,
    pub prune_reason: Option<ExecutionPruneReason>,
    pub recomputed: bool,
    pub memoized_origin: MemoizedResultOrigin,
    pub reuse_basis: ReuseBasis,
    pub reuse_origin: ReuseOrigin,
    pub propagation_suppressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutedTask {
    pub task: EligibleTask,
    pub record: TaskExecutionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageExecutionRecord {
    pub stage_index: u32,
    pub outcome: StageExecutionOutcome,
    pub authority_policy: Option<AuthorityPolicy>,
    pub parallel_admission_reason: Option<ParallelAdmissionReason>,
    #[cfg(feature = "parallel")]
    pub parallel_kind: Option<ParallelExecutionKind>,
    #[cfg(feature = "parallel")]
    pub apply_mode: Option<ParallelApplyMode>,
    #[cfg(feature = "parallel")]
    pub apply_group_count: u32,
    #[cfg(feature = "parallel")]
    pub serial_apply_rejection_reason: Option<ApplyPlanSerialFallbackReason>,
    #[cfg(feature = "parallel")]
    pub serial_fallback_group_count: u32,
    #[cfg(feature = "parallel")]
    pub concurrent_apply_task_count: u32,
    #[cfg(feature = "parallel")]
    pub serial_apply_task_count: u32,
    pub snapshot_duration_nanos: u128,
    pub precompute_duration_nanos: u128,
    pub apply_duration_nanos: u128,
    pub semantic_finalize_duration_nanos: u128,
    pub duration_nanos: u128,
    pub semantic_task_range: Option<SemanticTaskRange>,
    pub semantic_segment_count: u32,
    pub task_records: Vec<TaskExecutionRecord>,
}

impl StageExecutionRecord {
    pub fn parallel_admission_message(&self) -> &'static str {
        self.parallel_admission_reason
            .map(ParallelAdmissionReason::message)
            .unwrap_or("parallel admission reason was not recorded")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub plan_summary: PlanSummary,
    pub stage_count: u32,
    pub task_count: u32,
    pub maybe_stale_validation_tasks: u32,
    pub latest_execution_record_id: Option<u64>,
    pub temporal_summary: TemporalExecutionSummary,
    pub reuse_origin_counts: BTreeMap<ReuseOrigin, u32>,
    pub tasks_executed: u32,
    pub tasks_pruned: u32,
    pub tasks_validated_clean: u32,
    pub tasks_deferred_by_condition: u32,
    pub tasks_reverted_clean_by_condition: u32,
    pub tasks_satisfied_by_memoization: u32,
    pub tasks_with_suppressed_propagation: u32,
    pub execution_snapshots_built: u32,
    pub prepared_evaluations_produced: u32,
    pub prepared_evaluations_applied: u32,
    pub dependency_capture_updates: u32,
    pub execution_snapshot_nanos: u128,
    pub stage_precompute_nanos: u128,
    pub stage_apply_nanos: u128,
    pub semantic_finalize_nanos: u128,
    pub semantic_segment_count: u32,
    pub stages: Vec<StageExecutionRecord>,
}

impl fmt::Display for ExecutionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ExecutionReport stages={} tasks={} executed={} pruned={} validated_clean={} deferred={} memoized={} suppressed={}",
            self.stage_count,
            self.task_count,
            self.tasks_executed,
            self.tasks_pruned,
            self.tasks_validated_clean,
            self.tasks_deferred_by_condition,
            self.tasks_satisfied_by_memoization,
            self.tasks_with_suppressed_propagation
        )?;
        for stage in &self.stages {
            writeln!(
                f,
                "  stage {} outcome={:?} tasks={}",
                stage.stage_index,
                stage.outcome,
                stage.task_records.len()
            )?;
        }
        Ok(())
    }
}
