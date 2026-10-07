use crate::data::comparator::TierPolicyResolver;
use crate::data::error::{SignalError, SignalPublicationProgress};
use crate::data::graph::{EvaluationStrategy, GcPressure, ScratchLeaseKind, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::telemetry::RuntimeTelemetry;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::precompute::callback::{LegacyPrecompute, SignalPrecompute};
use crate::logic::planner::{
    execute_evaluation_session_in_scope, execute_prepared_plan_with_policy_and_temporal_lowering,
    EvaluationPlan, ExecutionReport, PlanSummary, TemporalLoweringContext,
};
use crate::logic::prepared::{ExecutionReadView, PreparedEvaluation};
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use super::super::config::SignalRuntimeConfig;

pub(super) struct SessionExecutionError {
    pub error: SignalError,
    pub plan_summary: PlanSummary,
}

impl From<SignalError> for SessionExecutionError {
    fn from(error: SignalError) -> Self {
        Self {
            error,
            plan_summary: PlanSummary::default(),
        }
    }
}

pub(super) fn absorb_execution_report_telemetry(
    telemetry: &mut RuntimeTelemetry,
    report: &ExecutionReport,
) {
    let mut stage_execution_nanos = 0_u128;
    // Stage outcomes are classified from their child pattern postures. The
    // final request-scope report describes the enclosing scan and must not
    // count as a dispatched stage.
    let mut parallel_stages = 0_u64;
    let mut parallel_tasks = 0_u64;

    for stage in &report.stages {
        stage_execution_nanos += stage.duration_nanos;
        if matches!(
            stage.outcome,
            crate::logic::planner::StageExecutionOutcome::CompletedParallel
        ) {
            parallel_stages += 1;
            parallel_tasks += stage.task_records.len() as u64;
        }
    }

    telemetry.planner.plans_built += 1;
    telemetry.planner.stages_built += report.stage_count as u64;
    telemetry.planner.tasks_scheduled += report.task_count as u64;
    telemetry.planner.tasks_pruned_before_execution += report.tasks_pruned as u64;
    telemetry.planner.maybe_stale_validation_tasks += report.maybe_stale_validation_tasks as u64;
    telemetry.temporal.deferred_by_time_count += report.temporal_summary.deferred_count() as u64;
    telemetry.execution.stage_execution_count += report.stage_count as u64;
    telemetry.execution.stage_execution_nanos += stage_execution_nanos;
    telemetry.execution.execution_snapshots_built += report.execution_snapshots_built as u64;
    telemetry.execution.prepared_evaluations_produced +=
        report.prepared_evaluations_produced as u64;
    telemetry.execution.prepared_evaluations_applied += report.prepared_evaluations_applied as u64;
    telemetry.execution.dependency_capture_updates += report.dependency_capture_updates as u64;
    telemetry.execution.execution_snapshot_nanos += report.execution_snapshot_nanos;
    telemetry.execution.stage_precompute_nanos += report.stage_precompute_nanos;
    telemetry.execution.stage_apply_nanos += report.stage_apply_nanos;
    telemetry.execution.last_execution_report = report.execution.last().copied();
    if parallel_stages > 0 {
        telemetry.execution.parallel_executor_usage_count += 1;
        telemetry.execution.parallel_stage_dispatch_count += parallel_stages;
        telemetry.execution.parallel_precompute_task_count += parallel_tasks;
    } else if !report.execution.is_empty() {
        telemetry.execution.serial_executor_usage_count += 1;
        telemetry.execution.serial_precompute_task_count += report.task_count as u64;
    }
    telemetry.execution.max_tasks_in_stage = telemetry
        .execution
        .max_tasks_in_stage
        .max(report.plan_summary.max_stage_width as u64);
}

pub(super) fn apply_strategy_maintenance(graph: &mut SignalGraph, strategy: EvaluationStrategy) {
    if matches!(strategy.gc_pressure, GcPressure::CompactAfterEvaluation) {
        graph.run_gc_epoch();
    }
}

fn prepare_with_context<Ctx, F, O>(
    graph: &SignalGraph,
    domain_ctx: &Ctx,
    node: NodeId,
    evaluator: &F,
) -> Result<PreparedEvaluation, SignalError>
where
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    let mut eval_ctx = EvaluationContext::new(graph, node, domain_ctx);
    let output = evaluator(&mut eval_ctx)?;
    Ok(eval_ctx.into_prepared(output))
}

pub(super) fn execute_targets_with_runtime_config<T, Ctx, F, O>(
    graph: &mut SignalGraph,
    config: &SignalRuntimeConfig<T>,
    temporal_lowering: TemporalLoweringContext,
    domain_ctx: &Ctx,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    evaluator: &F,
    lease: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ExecutionReport, SignalError>
where
    T: Copy + Ord,
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    execute_targets_with_runtime_config_detailed(
        graph,
        config,
        temporal_lowering,
        domain_ctx,
        targets,
        request_mode,
        evaluator,
        lease,
    )
    .map_err(|failure| failure.error)
}

pub(super) fn execute_targets_with_prepared_runtime_config_detailed<T, P>(
    graph: &mut SignalGraph,
    config: &SignalRuntimeConfig<T>,
    temporal_lowering: TemporalLoweringContext,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    precompute: &P,
    lease: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ExecutionReport, SessionExecutionError>
where
    T: Copy + Ord,
    P: SignalPrecompute,
{
    let mut plan_summary = PlanSummary::default();
    graph.with_telemetry(|telemetry| telemetry.execution.last_execution_report = None);
    let result = crate::logic::planner::run_signal_execution_request_scope(
        lease,
        |request_work, progress, preparation| {
            let mut resolver = TierPolicyResolver::new(
                config.node_meta(),
                config.tier_policies(),
                config.fallback_comparator(),
            );
            graph.with_scratch(ScratchLeaseKind::Evaluation, |graph, scratch| {
                let session = crate::logic::planner::planning::build_evaluation_session_with_policy_resolver_and_work(
                    graph,
                    scratch.traversal_mut(),
                    targets,
                    request_mode,
                    &mut resolver,
                    Some(&mut *request_work),
                    Some(&mut *preparation),
                )?;
                plan_summary = session.summary;
                execute_evaluation_session_in_scope(
                    graph,
                    &session,
                    precompute,
                    &mut resolver,
                    temporal_lowering.clone(),
                    lease,
                    request_work,
                    progress,
                    preparation,
                )
            })
        },
    );
    graph.with_telemetry(|telemetry| {
        telemetry.execution.last_execution_report = match &result {
            Ok(report) => report.execution.last().copied(),
            Err(SignalError::ExecutionStopped(stop)) => Some(stop.execution()),
            Err(_) => None,
        };
    });
    result.map_err(|error| SessionExecutionError {
        error,
        plan_summary,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_targets_with_prepared_runtime_config_in_scope<T, P>(
    graph: &mut SignalGraph,
    config: &SignalRuntimeConfig<T>,
    temporal_lowering: TemporalLoweringContext,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    precompute: &P,
    lease: &ExecutionResourceLease<'_>,
    request_work: &mut MapKernelContext<'_, '_>,
    disposition: &mut SignalPublicationProgress,
    preparation: &mut SignalPreparationBudget,
) -> Result<ExecutionReport, SignalError>
where
    T: Copy + Ord,
    P: SignalPrecompute,
{
    let mut resolver = TierPolicyResolver::new(
        config.node_meta(),
        config.tier_policies(),
        config.fallback_comparator(),
    );
    if !precompute.allows_bounded_inputs() {
        return Err(SignalError::invalid_input(
            "leased Signal evaluation requires a checked bounded-input callback",
        ));
    }
    graph.with_scratch(ScratchLeaseKind::Evaluation, |graph, scratch| {
        let session = crate::logic::planner::planning::build_evaluation_session_with_policy_resolver_and_work(
            graph,
            scratch.traversal_mut(),
            targets,
            request_mode,
            &mut resolver,
            Some(&mut *request_work),
            Some(&mut *preparation),
        )?;
        execute_evaluation_session_in_scope(
            graph,
            &session,
            precompute,
            &mut resolver,
            temporal_lowering,
            worth_execution::ExecutionRequest::leased(lease),
            request_work,
            disposition,
            preparation,
        )
    })
}

pub(super) fn execute_targets_with_runtime_config_detailed<T, Ctx, F, O>(
    graph: &mut SignalGraph,
    config: &SignalRuntimeConfig<T>,
    temporal_lowering: TemporalLoweringContext,
    domain_ctx: &Ctx,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    evaluator: &F,
    lease: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ExecutionReport, SessionExecutionError>
where
    T: Copy + Ord,
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    execute_targets_with_prepared_runtime_config_detailed(
        graph,
        config,
        temporal_lowering,
        targets,
        request_mode,
        &LegacyPrecompute::new(|node, view: &ExecutionReadView<'_>| {
            prepare_with_context(view.graph(), domain_ctx, node, evaluator)
        }),
        lease,
    )
}

pub(super) fn execute_plan_with_runtime_config<T, Ctx, F, O>(
    graph: &mut SignalGraph,
    config: &SignalRuntimeConfig<T>,
    temporal_lowering: TemporalLoweringContext,
    domain_ctx: &Ctx,
    plan: &EvaluationPlan,
    evaluator: &F,
    lease: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ExecutionReport, SignalError>
where
    T: Copy + Ord,
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    let mut resolver = TierPolicyResolver::new(
        config.node_meta(),
        config.tier_policies(),
        config.fallback_comparator(),
    );
    execute_prepared_plan_with_policy_and_temporal_lowering(
        graph,
        plan,
        domain_ctx,
        evaluator,
        &mut resolver,
        temporal_lowering,
        lease,
    )
}
