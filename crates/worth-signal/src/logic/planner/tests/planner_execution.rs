use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::recorder::record_lineage_transition;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationExecutionMetadata, IntoEvaluationOutput};
use crate::logic::prepared::ExecutionSnapshot;

use super::super::execution::diagnostics::{record_successful_execution, summarize_recorded_plan};
use super::super::reporting::{accumulate_report_counters, classify_task_record};
use super::super::types::{
    EvaluationPlan, ExecutionRecordId, ExecutionReport, ParallelAdmissionReason, SemanticSegmentId,
    StageExecutionOutcome, StageExecutionRecord, StageExecutor, TaskReason,
};
use super::execution_preparation::{
    apply_test_precompute_telemetry, prepare_test_precomputed_task, TestPrecomputeTelemetry,
};
use super::planner_serial_execution::execute_plan_serial;
use super::report_seed::empty_execution_report;

pub(crate) fn execute_plan_with_policy_and_condition<F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    compute: &mut F,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    condition_resolver: &mut impl crate::logic::evaluation::ConditionResolver,
    executor: StageExecutor,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
) -> Result<ExecutionReport, SignalError>
where
    F: FnMut(NodeId, &SignalGraph) -> Result<O, SignalError>,
    O: crate::data::output::IntoNodeEvaluationResult,
{
    match executor {
        StageExecutor::Serial => execute_plan_serial(
            graph,
            plan,
            compute,
            comparator_resolver,
            condition_resolver,
            execution_metadata,
        ),
        #[cfg(feature = "parallel")]
        StageExecutor::StagedParallelPrecompute { .. } | StageExecutor::FullParallel { .. } => {
            Err(SignalError::invalid_input(
                "parallel stage execution is not yet supported by the current mutable graph engine",
            ))
        }
    }
}

pub(crate) fn execute_test_prepared_plan_with_resolvers<Ctx, F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    domain_ctx: &Ctx,
    evaluator: &F,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    condition_resolver: &mut impl crate::logic::evaluation::ConditionResolver,
) -> Result<ExecutionReport, SignalError>
where
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .planner
        .plans_built += 1;
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .planner
        .stages_built += plan.stages.len() as u64;
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .planner
        .tasks_scheduled += plan.summary.task_count as u64;
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .execution
        .max_tasks_in_stage = graph
        .telemetry()
        .execution
        .max_tasks_in_stage
        .max(plan.summary.max_stage_width as u64);
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .execution
        .serial_executor_usage_count += 1;
    graph
        .telemetry_mut()
        .expect("telemetry admitted")
        .planner
        .maybe_stale_validation_tasks += plan
        .stages
        .iter()
        .flat_map(|stage| &stage.tasks)
        .filter(|task| matches!(task.reason, TaskReason::MaybeStaleValidation))
        .count() as u64;

    let mut next_record_id = 1_u64;
    let mut report = empty_execution_report(plan);

    for stage in &plan.stages {
        let stage_start = std::time::Instant::now();
        let snapshot_start = std::time::Instant::now();
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .execution_snapshots_built += 1;
        let mut prepared_tasks = Vec::with_capacity(stage.tasks.len());
        let mut precompute_telemetry = TestPrecomputeTelemetry::default();
        let precompute_start = std::time::Instant::now();
        {
            let snapshot = ExecutionSnapshot::new(&*graph);
            for task in &stage.tasks {
                let prepared = prepare_test_precomputed_task(
                    &snapshot,
                    task.node,
                    &|node, view| {
                        let mut eval_ctx = EvaluationContext::new(view.graph(), node, domain_ctx);
                        let output = evaluator(&mut eval_ctx)?;
                        Ok(eval_ctx.into_prepared(output))
                    },
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
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .execution_snapshot_nanos += snapshot_nanos;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .stage_precompute_nanos += precompute_nanos;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .prepared_evaluations_produced += prepared_tasks.len() as u64;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .serial_precompute_task_count += prepared_tasks.len() as u64;
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
                None,
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
            graph
                .telemetry_mut()
                .expect("telemetry admitted")
                .execution
                .prepared_evaluations_applied += 1;
            graph
                .telemetry_mut()
                .expect("telemetry admitted")
                .execution
                .dependency_capture_updates += apply_result.dependency_updates as u64;
            report.prepared_evaluations_applied += 1;
            report.dependency_capture_updates += apply_result.dependency_updates;
        }

        stage_record.apply_duration_nanos = apply_start.elapsed().as_nanos();
        stage_record.duration_nanos = stage_start.elapsed().as_nanos();
        report.stage_apply_nanos += stage_record.apply_duration_nanos;
        report.semantic_finalize_nanos += stage_record.semantic_finalize_duration_nanos;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .stage_apply_nanos += stage_record.apply_duration_nanos;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .stage_execution_count += 1;
        graph
            .telemetry_mut()
            .expect("telemetry admitted")
            .execution
            .stage_execution_nanos += stage_record.duration_nanos;
        report.stages.push(stage_record);
    }

    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_plan(plan, profile);
    record_successful_execution(graph, plan_summary, first_target, &report);
    Ok(report)
}
