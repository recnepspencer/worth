use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::recorder::record_lineage_transition;
use crate::logic::evaluation::EvaluationExecutionMetadata;
use crate::logic::prepared::ExecutionSnapshot;

use super::super::execution::diagnostics::{record_successful_execution, summarize_recorded_plan};
use super::super::reporting::{accumulate_report_counters, classify_task_record};
use super::super::types::{
    EvaluationPlan, ExecutionRecordId, ExecutionReport, ParallelAdmissionReason, SemanticSegmentId,
    StageExecutionOutcome, StageExecutionRecord, TaskReason,
};
use super::execution_preparation::{
    apply_test_precompute_telemetry, prepare_test_task, TestPrecomputeTelemetry,
};
use super::report_seed::empty_execution_report;

pub(super) fn execute_plan_serial<F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    compute: &mut F,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    condition_resolver: &mut impl crate::logic::evaluation::ConditionResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
) -> Result<ExecutionReport, SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: crate::data::output::IntoNodeEvaluationResult,
{
    let maybe_stale_tasks = plan
        .stages
        .iter()
        .flat_map(|stage| &stage.tasks)
        .filter(|task| matches!(task.reason, TaskReason::MaybeStaleValidation))
        .count() as u64;
    graph.with_telemetry(|telemetry| {
        telemetry.planner.plans_built += 1;
        telemetry.planner.stages_built += plan.stages.len() as u64;
        telemetry.planner.tasks_scheduled += plan.summary.task_count as u64;
        telemetry.execution.max_tasks_in_stage = telemetry
            .execution
            .max_tasks_in_stage
            .max(plan.summary.max_stage_width as u64);
        telemetry.execution.serial_executor_usage_count += 1;
        telemetry.evaluation.evaluation_calls += 1;
        telemetry.evaluation.evaluation_stack_peak = telemetry
            .evaluation
            .evaluation_stack_peak
            .max(plan.summary.task_count as u64);
        telemetry.planner.maybe_stale_validation_tasks += maybe_stale_tasks;
    });

    let mut next_record_id = 1_u64;
    let mut report = empty_execution_report(plan);

    for stage in &plan.stages {
        let stage_start = std::time::Instant::now();
        let snapshot_start = std::time::Instant::now();
        graph.with_telemetry(|telemetry| telemetry.execution.execution_snapshots_built += 1);
        let mut prepared_tasks = Vec::with_capacity(stage.tasks.len());
        let mut precompute_telemetry = TestPrecomputeTelemetry::default();
        let precompute_start = std::time::Instant::now();
        {
            let snapshot = ExecutionSnapshot::new(&*graph);
            for task in &stage.tasks {
                let prepared = prepare_test_task(
                    snapshot.graph(),
                    task.node,
                    compute,
                    comparator_resolver,
                    condition_resolver,
                    task.request_mode,
                )?;
                precompute_telemetry.accumulate(&prepared.telemetry);
                prepared_tasks.push(prepared.prepared);
            }
        }
        apply_test_precompute_telemetry(graph, &precompute_telemetry);
        let snapshot_nanos = snapshot_start.elapsed().as_nanos();
        let precompute_nanos = precompute_start.elapsed().as_nanos();
        let prepared_count = prepared_tasks.len() as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.execution.execution_snapshot_nanos += snapshot_nanos;
            telemetry.execution.stage_precompute_nanos += precompute_nanos;
            telemetry.execution.prepared_evaluations_produced += prepared_count;
            telemetry.execution.serial_precompute_task_count += prepared_count;
        });
        report.execution_snapshots_built += 1;
        report.execution_snapshot_nanos += snapshot_nanos;
        report.prepared_evaluations_produced += prepared_tasks.len() as u32;
        report.stage_precompute_nanos += precompute_nanos;

        let apply_start = std::time::Instant::now();
        let mut stage_record = StageExecutionRecord {
            stage_index: stage.index,
            outcome: StageExecutionOutcome::CompletedSerial,
            authority_policy: None,
            parallel_admission_reason: Some(ParallelAdmissionReason::SerialExecutor),
            #[cfg(feature = "parallel")]
            parallel_kind: None,
            #[cfg(feature = "parallel")]
            apply_mode: None,
            #[cfg(feature = "parallel")]
            apply_group_count: 0,
            #[cfg(feature = "parallel")]
            serial_apply_rejection_reason: None,
            #[cfg(feature = "parallel")]
            serial_fallback_group_count: 0,
            #[cfg(feature = "parallel")]
            concurrent_apply_task_count: 0,
            #[cfg(feature = "parallel")]
            serial_apply_task_count: 0,
            snapshot_duration_nanos: snapshot_nanos,
            precompute_duration_nanos: precompute_nanos,
            apply_duration_nanos: 0,
            semantic_finalize_duration_nanos: 0,
            duration_nanos: 0,
            semantic_task_range: None,
            semantic_segment_count: 0,
            task_records: Vec::new(),
        };

        for (task, prepared) in stage.tasks.iter().zip(prepared_tasks.into_iter()) {
            let record_id = ExecutionRecordId(next_record_id);
            next_record_id += 1;
            let before_state = graph.get_state(task.node)?;
            let before_trace = graph.node_runtime_artifact_finalize_image(task.node)?;
            let apply_result = crate::logic::evaluation::apply_prepared_evaluation_with_policy(
                graph,
                task.node,
                prepared,
                comparator_resolver,
                execution_metadata.filter(|_| task.direct_request),
            )?;
            record_lineage_transition(
                graph,
                task.node,
                before_trace.as_ref(),
                record_id,
                SemanticSegmentId(record_id.0),
            )?;
            let after_state = graph.get_state(task.node)?;
            let after_trace = graph.node_runtime_artifact_finalize_image(task.node)?;
            let after_warm = graph.node_runtime_artifact_warm(task.node)?;
            let task_record = classify_task_record(
                record_id,
                SemanticSegmentId(record_id.0),
                task,
                before_state,
                after_state,
                before_trace.as_ref(),
                after_trace.as_ref(),
                apply_result.report.verdict.clone(),
                apply_result.temporal_eligibility.clone(),
                after_warm
                    .as_ref()
                    .map(|trace| trace.memoized_origin)
                    .unwrap_or(crate::data::output::MemoizedResultOrigin::DirectCompute),
                after_warm
                    .as_ref()
                    .map(|trace| trace.reuse_basis.clone_inner())
                    .unwrap_or(crate::data::reuse::ReuseBasis::fresh_compute()),
            );
            accumulate_report_counters(&mut report, &task_record.record);
            stage_record.task_records.push(task_record.record);
            graph.with_telemetry(|telemetry| {
                telemetry.execution.prepared_evaluations_applied += 1;
                telemetry.execution.dependency_capture_updates +=
                    apply_result.dependency_updates as u64;
            });
            report.prepared_evaluations_applied += 1;
            report.dependency_capture_updates += apply_result.dependency_updates;
        }

        stage_record.apply_duration_nanos = apply_start.elapsed().as_nanos();
        stage_record.duration_nanos = stage_start.elapsed().as_nanos();
        report.stage_apply_nanos += stage_record.apply_duration_nanos;
        report.semantic_finalize_nanos += stage_record.semantic_finalize_duration_nanos;
        graph.with_telemetry(|telemetry| {
            telemetry.execution.stage_apply_nanos += stage_record.apply_duration_nanos;
            telemetry.execution.stage_execution_count += 1;
            telemetry.execution.stage_execution_nanos += stage_record.duration_nanos;
        });
        report.stages.push(stage_record);
    }

    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_plan(plan, profile);
    record_successful_execution(graph, plan_summary, first_target, &report);
    Ok(report)
}
