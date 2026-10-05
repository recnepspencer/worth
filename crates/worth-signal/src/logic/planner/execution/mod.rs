use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationProgress};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::IntoEvaluationOutput;

use self::diagnostics::{summarize_recorded_plan, summarize_recorded_session};
use super::precompute::callback::{LegacyPrecompute, SignalPrecompute};
use super::types::{EvaluationPlan, ExecutionReport, PlanSummary, SessionScratch};
use super::TemporalLoweringContext;
use crate::data::request_preparation::SignalPreparationBudget;
use worth_execution::{
    ExecutionResourceLease, ExecutionScan, MapKernelContext, MapKernelFailure, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

mod context;
pub(crate) mod diagnostics;
mod epochs;
mod prepared_plan;
mod reporting;
mod request_scope;
mod stage;
pub(crate) use epochs::{
    run_stage_slices, AdmittedEpoch, CheckedAdmittedEpoch, EpochMetadata, LegacySerialEpoch,
    PlannedStageSlice,
};
pub use prepared_plan::execute_prepared_plan;
use prepared_plan::prepare_with_context;
pub(crate) use request_scope::run_signal_request_scope;
pub(crate) mod task_reporting;

pub(crate) fn execute_prepared_plan_with_precompute<P: SignalPrecompute>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    precompute: &P,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<ExecutionReport, SignalError> {
    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_plan(plan, profile);
    let maybe_stale_validation_tasks = plan
        .stages
        .iter()
        .flat_map(|stage| stage.tasks.iter())
        .filter(|task| matches!(task.reason, super::types::TaskReason::MaybeStaleValidation))
        .count() as u64;
    execute_plan_stage_slices_with_policy(
        graph,
        &plan.summary,
        plan.stages.len(),
        maybe_stale_validation_tasks,
        plan.stages.iter().map(|stage| PlannedStageSlice {
            index: stage.index,
            task_offset: 0,
            tasks: &stage.tasks,
        }),
        precompute,
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        lease,
    )
}

pub(crate) fn execute_prepared_plan_in_scope<P: SignalPrecompute>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    precompute: &P,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: &ExecutionResourceLease<'_>,
    request_work: &mut MapKernelContext<'_, '_>,
    progress: &mut SignalPublicationProgress,
    preparation: &mut SignalPreparationBudget,
) -> Result<ExecutionReport, SignalError> {
    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_plan(plan, profile);
    let maybe_stale_validation_tasks = plan
        .stages
        .iter()
        .flat_map(|stage| stage.tasks.iter())
        .filter(|task| matches!(task.reason, super::types::TaskReason::MaybeStaleValidation))
        .count() as u64;
    let policy = super::types::ResolvedSignalPlannerPolicy::for_graph(graph, Some(lease));
    run_stage_slices(
        graph,
        &plan.summary,
        plan.stages.len(),
        maybe_stale_validation_tasks,
        plan.stages.iter().map(|stage| PlannedStageSlice {
            index: stage.index,
            task_offset: 0,
            tasks: &stage.tasks,
        }),
        precompute,
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        Some(lease),
        policy,
        Some(request_work),
        Some(preparation),
        progress,
    )
}

pub fn execute_prepared_plan_with_policy<Ctx, F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    domain_ctx: &Ctx,
    evaluator: &F,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<ExecutionReport, SignalError>
where
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    execute_prepared_plan_with_policy_and_temporal_lowering(
        graph,
        plan,
        domain_ctx,
        evaluator,
        comparator_resolver,
        TemporalLoweringContext::graph_only(),
        lease,
    )
}

pub(crate) fn execute_prepared_plan_with_policy_and_temporal_lowering<Ctx, F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    domain_ctx: &Ctx,
    evaluator: &F,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<ExecutionReport, SignalError>
where
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_plan(plan, profile);
    let maybe_stale_validation_tasks = plan
        .stages
        .iter()
        .flat_map(|stage| stage.tasks.iter())
        .filter(|task| matches!(task.reason, super::types::TaskReason::MaybeStaleValidation))
        .count() as u64;
    execute_plan_stage_slices_with_policy(
        graph,
        &plan.summary,
        plan.stages.len(),
        maybe_stale_validation_tasks,
        plan.stages.iter().map(|stage| PlannedStageSlice {
            index: stage.index,
            task_offset: 0,
            tasks: &stage.tasks,
        }),
        &LegacyPrecompute::new(
            |node, view: &crate::logic::prepared::ExecutionReadView<'_>| {
                prepare_with_context(view.graph(), domain_ctx, node, evaluator)
            },
        ),
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        lease,
    )
}

pub(crate) fn execute_evaluation_session_with_policy<P: SignalPrecompute>(
    graph: &mut SignalGraph,
    session: &SessionScratch<'_>,
    precompute: &P,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<ExecutionReport, SignalError> {
    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_session(session, profile);
    let maybe_stale_validation_tasks = session
        .tasks
        .iter()
        .filter(|task| matches!(task.reason, super::types::TaskReason::MaybeStaleValidation))
        .count() as u64;
    let report = execute_plan_stage_slices_with_policy(
        graph,
        &session.summary,
        session.stages.len(),
        maybe_stale_validation_tasks,
        session.stages.iter().map(|stage| PlannedStageSlice {
            index: stage.index,
            task_offset: 0,
            tasks: &session.tasks[stage.start..stage.end],
        }),
        precompute,
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        lease,
    )?;
    Ok(report)
}

/// Execute a planned session inside a caller-owned request scan. The caller
/// appends the scan's exact report after planning and publication settle.
pub(crate) fn execute_evaluation_session_in_scope<P: SignalPrecompute>(
    graph: &mut SignalGraph,
    session: &SessionScratch<'_>,
    precompute: &P,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: &ExecutionResourceLease<'_>,
    request_work: &mut MapKernelContext<'_, '_>,
    progress: &mut SignalPublicationProgress,
    preparation: &mut SignalPreparationBudget,
) -> Result<ExecutionReport, SignalError> {
    let profile = graph.diagnostics_profile();
    let (plan_summary, first_target) = summarize_recorded_session(session, profile);
    let maybe_stale_validation_tasks = session
        .tasks
        .iter()
        .filter(|task| matches!(task.reason, super::types::TaskReason::MaybeStaleValidation))
        .count() as u64;
    let policy = super::types::ResolvedSignalPlannerPolicy::for_graph(graph, Some(lease));
    run_stage_slices(
        graph,
        &session.summary,
        session.stages.len(),
        maybe_stale_validation_tasks,
        session.stages.iter().map(|stage| PlannedStageSlice {
            index: stage.index,
            task_offset: 0,
            tasks: &session.tasks[stage.start..stage.end],
        }),
        precompute,
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        Some(lease),
        policy,
        Some(request_work),
        Some(preparation),
        progress,
    )
}

fn execute_plan_stage_slices_with_policy<'a, P: SignalPrecompute>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_count: usize,
    maybe_stale_validation_tasks: u64,
    stages: impl IntoIterator<Item = PlannedStageSlice<'a>>,
    precompute: &P,
    plan_summary: crate::diagnostics::summary::EvaluationPlanSummary,
    first_target: Option<NodeId>,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<ExecutionReport, SignalError> {
    graph.with_telemetry(|telemetry| telemetry.execution.last_execution_report = None);
    let policy = super::types::ResolvedSignalPlannerPolicy::for_graph(graph, lease);
    if let Some(lease) = lease {
        let identity = PartitionIdentity::new(1);
        let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).map_err(
            |denial| SignalError::internal(format!("signal request admission failed: {denial:?}")),
        )?;
        let memory = lease.policy().budget().charged_memory_bytes();
        let mut progress = SignalPublicationProgress::default();
        let mut preparation = SignalPreparationBudget::new(memory / 8);
        let mut complete = None;
        let mut stages = Some(stages);
        let mut plan_summary = Some(plan_summary);
        let outcome = scan.run(
            Some(lease),
            (),
            0,
            0,
            memory / 8,
            memory / 8,
            |_, _, request_work| {
                request_work.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                preparation
                    .claim_retained_vec::<worth_foundational::ExecutionReport>(1)
                    .map_err(MapKernelFailure::Domain)?;
                if !precompute.allows_bounded_inputs() {
                    return Err(MapKernelFailure::Domain(SignalError::invalid_input(
                        "leased Signal evaluation requires a checked bounded-input callback",
                    )));
                }
                let mut report = run_stage_slices(
                    graph,
                    summary,
                    stage_count,
                    maybe_stale_validation_tasks,
                    stages.take().expect("one Signal request scan step"),
                    precompute,
                    plan_summary.take().expect("one Signal request scan step"),
                    first_target,
                    comparator_resolver,
                    temporal_lowering.clone(),
                    Some(lease),
                    policy,
                    Some(request_work),
                    Some(&mut preparation),
                    &mut progress,
                )
                .map_err(MapKernelFailure::Domain)?;
                if report.execution.len() == report.execution.capacity() {
                    report.execution.reserve_exact(1);
                }
                complete = Some(report);
                Ok(((), ()))
            },
        );
        return match outcome {
            ScanOutcome::Complete {
                report: physical, ..
            } => {
                graph.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = Some(physical)
                });
                let mut report = complete.expect("completed Signal scan has a report");
                report.execution.push(physical);
                Ok(report)
            }
            ScanOutcome::Stopped {
                reason,
                boundary,
                report,
                ..
            } => {
                graph.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = Some(report)
                });
                Err(SignalError::execution_stopped(SignalExecutionStop::new(
                    reason.into(),
                    boundary,
                    progress,
                    report,
                )))
            }
        };
    }
    let mut progress = SignalPublicationProgress::default();
    run_stage_slices(
        graph,
        summary,
        stage_count,
        maybe_stale_validation_tasks,
        stages,
        precompute,
        plan_summary,
        first_target,
        comparator_resolver,
        temporal_lowering,
        None,
        policy,
        None,
        None,
        &mut progress,
    )
}
