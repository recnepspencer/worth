use super::super::execution::task_reporting::record_execution_failure_if_enabled;
use super::super::types::{ExecutionReport, PlanSummary};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::diagnostics::failure::{ExecutionFailureContext, ExecutionFailurePhase};

pub(in crate::logic::planner) fn record_stage_precompute_telemetry(
    graph: &mut SignalGraph,
    count: usize,
    snapshot_nanos: u128,
    precompute_nanos: u128,
    reports: &[worth_foundational::ExecutionReport],
) {
    let count = count as u64;
    let parallel = crate::logic::planner::stage_recording::execution_resolved_parallel(reports);
    graph.with_telemetry(|telemetry| {
        telemetry.execution.execution_snapshots_built += 1;
        telemetry.execution.execution_snapshot_nanos += snapshot_nanos;
        telemetry.execution.stage_precompute_nanos += precompute_nanos;
        telemetry.execution.prepared_evaluations_produced += count;
        if parallel {
            telemetry.execution.parallel_stage_dispatch_count += 1;
            telemetry.execution.parallel_precompute_task_count += count;
        } else {
            telemetry.execution.serial_precompute_task_count += count;
        }
    });
}

pub(in crate::logic::planner) fn record_stage_precompute_report(
    report: &mut ExecutionReport,
    count: usize,
    snapshot_nanos: u128,
    precompute_nanos: u128,
) {
    report.execution_snapshots_built += 1;
    report.execution_snapshot_nanos += snapshot_nanos;
    report.prepared_evaluations_produced += count as u32;
    report.stage_precompute_nanos += precompute_nanos;
}

pub(crate) fn record_stage_precompute_failure(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_index: u32,
    error: &SignalError,
) {
    record_execution_failure_if_enabled(graph, || {
        ExecutionFailureContext::new(
            ExecutionFailurePhase::Precompute,
            Some(stage_index),
            None,
            None,
            None,
            Some(*summary),
            error.to_string(),
        )
    });
}
