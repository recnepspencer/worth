use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
pub(crate) mod validation;

mod admission;
mod depths;
mod evidence;
mod required_inputs;
mod stage_formation;
mod topology;
mod unsettled_paths;

use crate::data::comparator::{
    ComparatorPolicyResolver, DefaultComparatorPolicyResolver, DefaultComparatorResolver,
    VersionComparatorPolicy,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::graph::TraversalScratch;
use crate::data::handle::NodeId;
use crate::logic::evaluation::EvaluationRequestMode;

use super::types::{EvaluationCursor, EvaluationPlan, SessionScratch};

pub fn build_evaluation_plan(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
) -> Result<EvaluationPlan, SignalError> {
    let mut comparator = DefaultComparatorResolver;
    let mut resolver = DefaultComparatorPolicyResolver {
        fallback: VersionComparatorPolicy::Exact,
        custom: &mut comparator,
    };
    build_evaluation_plan_with_policy_resolver(graph, targets, request_mode, &mut resolver)
}

pub fn build_evaluation_plan_with_policy_resolver(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
) -> Result<EvaluationPlan, SignalError> {
    let serial = crate::data::host_execution::declared_serial_request(
        graph.installed_runtime_policy().requested_policy(),
    );
    super::execution::run_signal_preparation_request(
        worth_execution::ExecutionRequest::serial(&serial),
        |work, _, preparation| {
            build_evaluation_plan_with_policy_resolver_and_work(
                graph,
                targets,
                request_mode,
                resolver,
                Some(work),
                Some(preparation),
            )
        },
    )
    .map(|(plan, _)| plan)
}

pub(crate) fn build_evaluation_plan_with_policy_resolver_and_work(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
    work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<EvaluationPlan, SignalError> {
    let cursor = build_evaluation_cursor_with_work(
        graph,
        targets,
        request_mode,
        resolver,
        work,
        preparation.as_deref_mut(),
    )?;
    preparation_budget::claim_vec::<super::types::ExecutionStage>(
        preparation.as_deref_mut(),
        cursor.stages.len(),
    )?;
    preparation_budget::claim_vec::<super::types::EligibleTask>(preparation, cursor.tasks.len())?;
    Ok(evidence::materialize_plan_from_cursor(cursor))
}

fn build_evaluation_cursor_with_work(
    graph: &mut SignalGraph,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
    work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<EvaluationCursor, SignalError> {
    let mut deduped_targets = Vec::new();
    let mut flat_tasks = Vec::new();
    let mut stages = Vec::new();
    let summary = stage_formation::populate_plan_buffers(
        graph,
        targets,
        request_mode,
        resolver,
        &mut deduped_targets,
        &mut flat_tasks,
        &mut stages,
        work,
        preparation,
    )?;

    Ok(EvaluationCursor {
        request_mode,
        targets: deduped_targets,
        tasks: flat_tasks,
        stages,
        summary,
    })
}

pub(crate) fn build_evaluation_session_with_policy_resolver_and_work<'a>(
    graph: &mut SignalGraph,
    scratch: &'a mut TraversalScratch,
    targets: &[NodeId],
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
    work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<SessionScratch<'a>, SignalError> {
    let summary = stage_formation::populate_plan_buffers(
        graph,
        targets,
        request_mode,
        resolver,
        &mut scratch.planner_targets,
        &mut scratch.planner_tasks,
        &mut scratch.planner_stages,
        work,
        preparation,
    )?;

    Ok(SessionScratch {
        targets: &scratch.planner_targets,
        tasks: &scratch.planner_tasks,
        stages: &scratch.planner_stages,
        summary,
    })
}

pub(crate) use admission::admit_direct_task_with_policy_resolver;
#[cfg(test)]
pub(crate) use validation::partition_scope_untouched;
