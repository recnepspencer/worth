use std::collections::BTreeMap;

use crate::data::temporal::TemporalExecutionSummary;

use super::super::types::{EvaluationPlan, ExecutionReport, TaskReason};

pub(super) fn empty_execution_report(plan: &EvaluationPlan) -> ExecutionReport {
    ExecutionReport {
        execution: Vec::new(),
        plan_summary: plan.summary,
        stage_count: plan.summary.stage_count,
        task_count: plan.summary.task_count,
        maybe_stale_validation_tasks: plan
            .stages
            .iter()
            .flat_map(|stage| &stage.tasks)
            .filter(|task| matches!(task.reason, TaskReason::MaybeStaleValidation))
            .count() as u32,
        latest_execution_record_id: None,
        temporal_summary: TemporalExecutionSummary::default(),
        reuse_origin_counts: BTreeMap::new(),
        tasks_executed: 0,
        tasks_pruned: 0,
        tasks_validated_clean: 0,
        tasks_deferred_by_condition: 0,
        tasks_reverted_clean_by_condition: 0,
        tasks_satisfied_by_memoization: 0,
        tasks_with_suppressed_propagation: 0,
        execution_snapshots_built: 0,
        prepared_evaluations_produced: 0,
        prepared_evaluations_applied: 0,
        dependency_capture_updates: 0,
        execution_snapshot_nanos: 0,
        stage_precompute_nanos: 0,
        stage_apply_nanos: 0,
        semantic_finalize_nanos: 0,
        semantic_segment_count: 0,
        stages: Vec::new(),
    }
}
