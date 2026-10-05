mod conditions;

use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::logic::prepared::PreparedEvaluation;

use super::super::types::EligibleTask;
use super::super::validation::capture_current_dependencies_without_refresh;
use super::temporal::TemporalLoweringContext;

#[derive(Debug)]
pub(super) enum PrevalidatedTask {
    Prepared(PreparedEvaluation),
    NeedsCompute {
        temporal_ready: Option<crate::data::temporal::ReadyTemporalEligibility>,
        ready_invalidation:
            Option<crate::data::proof::invalidation::progression::ReadyInvalidationBatch>,
    },
}

pub(super) fn prevalidate_stage_tasks(
    graph: &mut SignalGraph,
    tasks: &[EligibleTask],
    stage_index: u32,
    task_offset: usize,
    readiness_epoch: crate::data::proof::invalidation::progression::InvalidationReadinessEpoch,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: &TemporalLoweringContext,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<Vec<PrevalidatedTask>, SignalError> {
    super::work::checkpoint(work.as_deref_mut(), tasks.len())?;
    let mut prevalidated = Vec::with_capacity(tasks.len());
    for task in tasks {
        let dependency_count = graph.current_runtime_dependencies_of(task.node)?.len();
        let snapshot_count = graph.get_dep_snapshot(task.node)?.entries().len();
        super::work::checkpoint(
            work.as_deref_mut(),
            dependency_count
                .saturating_add(snapshot_count)
                .saturating_add(1),
        )?;
        let prepared = if let Some(prepared) = prepare_invalidation_outcome(
            graph,
            task,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )? {
            prepared
        } else if let Some(prepared) = conditions::prepare_condition_outcome_if_blocked(
            graph,
            task,
            temporal_lowering,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )? {
            prepared
        } else {
            prepare_validated_clean_if_unchanged(
                graph,
                task,
                comparator_resolver,
                work.as_deref_mut(),
                preparation.as_deref_mut(),
            )?
            .unwrap_or(PrevalidatedTask::NeedsCompute {
                temporal_ready: None,
                ready_invalidation: None,
            })
        };
        prevalidated.push(prepared);
    }
    super::readiness::attach_ready_invalidation(
        graph,
        tasks,
        stage_index,
        task_offset,
        readiness_epoch,
        &mut prevalidated,
        work,
        preparation,
    )?;
    Ok(prevalidated)
}

fn prepare_invalidation_outcome(
    graph: &SignalGraph,
    task: &EligibleTask,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<Option<PrevalidatedTask>, SignalError> {
    match graph.node_invalidation_input_with_execution_work(
        task.node,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )? {
        NodeInvalidationInput::Pending(_) => {
            let dependencies = capture_current_dependencies_without_refresh(
                graph,
                task.node,
                work.as_deref_mut(),
                preparation.as_deref_mut(),
            )?;
            Ok(Some(PrevalidatedTask::Prepared(
                PreparedEvaluation::deferred_by_invalidation().with_dependencies(dependencies),
            )))
        }
        NodeInvalidationInput::Resolved(_) => {
            let structural = graph
                .node_pending_revalidation(task.node)?
                .is_some_and(|pending| pending.requires_structural_recompute());
            Ok(structural.then_some(PrevalidatedTask::NeedsCompute {
                temporal_ready: None,
                ready_invalidation: None,
            }))
        }
        NodeInvalidationInput::ResolvedNoChange(_) => {
            let validates_clean = matches!(
                task.admission.node_state_at_admission,
                Some(
                    crate::data::node::NodeState::MaybeStale | crate::data::node::NodeState::Clean
                )
            ) && !matches!(
                task.request_mode,
                crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand
            );
            if !validates_clean {
                return Ok(None);
            }
            let dependencies =
                capture_current_dependencies_without_refresh(graph, task.node, work, preparation)?;
            Ok(Some(PrevalidatedTask::Prepared(
                PreparedEvaluation::validated_clean().with_dependencies(dependencies),
            )))
        }
    }
}

fn prepare_validated_clean_if_unchanged(
    graph: &mut SignalGraph,
    task: &EligibleTask,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<Option<PrevalidatedTask>, SignalError> {
    if matches!(
        graph.node_invalidation_input_with_execution_work(task.node, work.as_deref_mut(), preparation.as_deref_mut())?,
        NodeInvalidationInput::Resolved(ref causes) if causes.is_source_recompute()
    ) {
        return Ok(None);
    }
    if matches!(
        task.request_mode,
        crate::logic::evaluation::EvaluationRequestMode::ForceOnDemand
    ) {
        return Ok(None);
    }

    if !matches!(
        task.admission.node_state_at_admission,
        Some(crate::data::node::NodeState::MaybeStale)
    ) {
        return Ok(None);
    }

    if task.admission.dirty_partition_scopes_present {
        return Ok(None);
    }

    let preview = super::super::validation::preview_maybe_stale(
        graph,
        task.node,
        comparator_resolver,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    if !preview.unchanged {
        return Ok(None);
    }

    let dependencies =
        capture_current_dependencies_without_refresh(graph, task.node, work, preparation)?;
    Ok(Some(PrevalidatedTask::Prepared(
        PreparedEvaluation::validated_clean().with_dependencies(dependencies),
    )))
}

fn max_dependency_delta(
    graph: &SignalGraph,
    node: crate::data::handle::NodeId,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
) -> Result<u64, SignalError> {
    let mut max_delta = 0;
    for snapshot_entry in graph.get_dep_snapshot(node)?.entries() {
        super::work::checkpoint(
            work.as_deref_mut(),
            snapshot_entry.scope.as_ref().map_or(1, |scope| {
                scope.path().total_segment_bytes().saturating_add(9)
            }),
        )?;
        if !graph.is_alive(snapshot_entry.source) {
            continue;
        }
        let current_version = graph.node_version_for_scope(
            snapshot_entry.source,
            snapshot_entry.aspect,
            snapshot_entry.scope.as_ref(),
        )?;
        max_delta = max_delta.max(current_version.abs_diff(snapshot_entry.cached_version));
    }
    Ok(max_delta)
}
