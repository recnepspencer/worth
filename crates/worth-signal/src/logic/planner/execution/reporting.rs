use crate::data::graph::SignalGraph;
use std::collections::BTreeMap;

use super::super::types::{ExecutionReport, PlanSummary, StageExecutionRecord};

pub(crate) fn begin_execution_report(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_count: usize,
    maybe_stale_validation_tasks: u64,
) -> ExecutionReport {
    let max_stage_width = summary.max_stage_width as u64;
    graph.with_telemetry(|telemetry| {
        telemetry.planner.plans_built += 1;
        telemetry.planner.stages_built += stage_count as u64;
        telemetry.planner.tasks_scheduled += summary.task_count as u64;
        telemetry.execution.max_tasks_in_stage =
            telemetry.execution.max_tasks_in_stage.max(max_stage_width);
        telemetry.planner.maybe_stale_validation_tasks += maybe_stale_validation_tasks;
    });

    ExecutionReport {
        execution: Vec::new(),
        plan_summary: *summary,
        stage_count: summary.stage_count,
        task_count: summary.task_count,
        maybe_stale_validation_tasks: maybe_stale_validation_tasks as u32,
        latest_execution_record_id: None,
        temporal_summary: crate::data::temporal::TemporalExecutionSummary::default(),
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

pub(crate) fn record_stage_execution_completion(
    graph: &mut SignalGraph,
    report: &mut ExecutionReport,
    mut stage_record: StageExecutionRecord,
    apply_elapsed_nanos: u128,
    stage_elapsed_nanos: u128,
) {
    stage_record.apply_duration_nanos =
        apply_elapsed_nanos.saturating_sub(stage_record.semantic_finalize_duration_nanos);
    report.stage_apply_nanos += stage_record.apply_duration_nanos;
    let apply_nanos = stage_record.apply_duration_nanos;
    graph.with_telemetry(|telemetry| telemetry.execution.stage_apply_nanos += apply_nanos);
    report.semantic_finalize_nanos += stage_record.semantic_finalize_duration_nanos;

    stage_record.duration_nanos = stage_elapsed_nanos;
    let duration_nanos = stage_record.duration_nanos;
    graph.with_telemetry(|telemetry| {
        telemetry.execution.stage_execution_count += 1;
        telemetry.execution.stage_execution_nanos += duration_nanos;
    });
    report.stages.push(stage_record);
}
