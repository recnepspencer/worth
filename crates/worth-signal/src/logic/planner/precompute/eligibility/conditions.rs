use super::super::super::types::EligibleTask;
use super::super::super::validation::capture_current_dependencies_without_refresh;
use super::super::temporal::TemporalLoweringContext;
use super::PrevalidatedTask;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::EvaluationCondition;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::temporal::{
    DeferredTemporalEligibility, LoweredTemporalEligibility, ReadyTemporalEligibility,
    TemporalCondition,
};
use crate::logic::evaluation::{
    ConditionEvaluationContext, ConditionResolver, DefaultConditionResolver,
};
use crate::logic::prepared::PreparedEvaluation;
use worth_execution::MapKernelContext;

pub(super) fn prepare_condition_outcome_if_blocked(
    graph: &mut SignalGraph,
    task: &EligibleTask,
    temporal_lowering: &TemporalLoweringContext,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<Option<PrevalidatedTask>, SignalError> {
    let invalidation = graph.node_invalidation_input_with_execution_work(
        task.node,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let Some(dirty_aspects) = invalidation.resolved_dirty_aspects() else {
        let dependencies = capture_current_dependencies_without_refresh(
            graph,
            task.node,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
        return Ok(Some(PrevalidatedTask::Prepared(
            PreparedEvaluation::deferred_by_invalidation().with_dependencies(dependencies),
        )));
    };
    let required_context = graph.get_contract(task.node)?.semantics.required_context;
    let max_dependency_delta = if matches!(
        graph.node_eval_config(task.node)?.condition,
        EvaluationCondition::DeltaThreshold(_)
    ) {
        super::max_dependency_delta(graph, task.node, work.as_deref_mut())?
    } else {
        0
    };
    let ctx = ConditionEvaluationContext {
        node: task.node,
        request_mode: task.request_mode,
        dirty_aspects,
        max_dependency_delta,
        required_context,
    };
    let has_dependency_snapshot = !graph.get_dep_snapshot(task.node)?.entries().is_empty();
    let mut default_resolver = DefaultConditionResolver;

    if let Some(work) = work.as_deref_mut() {
        use crate::data::retained_storage::{
            RetainedStorageMeasurement, RetainedStoragePreparation,
        };
        let condition = &graph.node_eval_config(task.node)?.condition;
        let mut measurement = RetainedStoragePreparation::new(usize::MAX);
        let bytes = condition
            .retained_heap_charge(&mut measurement)
            .map_err(|_| SignalError::invalid_input("condition preparation memory overflow"))?
            .bytes();
        work.checkpoint(
            bytes
                .saturating_mul(3)
                .saturating_add(measurement.visits() as u64),
        )
        .map_err(SignalError::execution_checkpoint_stopped)?;
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim(bytes.saturating_mul(3))?;
        }
    }
    match graph.node_eval_config(task.node)?.condition.clone() {
        EvaluationCondition::Always | EvaluationCondition::OnDemand => Ok(None),
        EvaluationCondition::AspectFilter(mask) => {
            if !has_dependency_snapshot
                || dirty_aspects.is_empty()
                || dirty_aspects.intersects(mask)
            {
                Ok(None)
            } else {
                prepare_condition_blocked_result(
                    graph,
                    task.node,
                    PreparedEvaluation::deferred_by_condition(),
                    work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )
            }
        }
        EvaluationCondition::DeltaThreshold(threshold) => {
            if !has_dependency_snapshot
                || dirty_aspects.is_empty()
                || (max_dependency_delta as f64) > threshold
            {
                Ok(None)
            } else {
                prepare_condition_blocked_result(
                    graph,
                    task.node,
                    PreparedEvaluation::reverted_clean_by_condition(),
                    work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )
            }
        }
        EvaluationCondition::Temporal(condition) => {
            graph.with_telemetry(|telemetry| {
                telemetry.temporal.temporal_eligibility_lowering_count += 1;
            });
            lower_temporal_condition(
                graph,
                task.node,
                condition,
                &ctx,
                temporal_lowering,
                work,
                preparation,
            )
        }
        EvaluationCondition::Custom(key) => {
            if default_resolver.resolve_custom(&key, &ctx)? {
                Ok(None)
            } else {
                prepare_condition_blocked_result(
                    graph,
                    task.node,
                    PreparedEvaluation::deferred_by_condition(),
                    work,
                    preparation,
                )
            }
        }
        EvaluationCondition::Installed(_) => Err(SignalError::invalid_input(
            "installed conditions require the owner-bound conditional execution entry point",
        )),
    }
}

fn prepare_condition_blocked_result(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<Option<PrevalidatedTask>, SignalError> {
    let dependencies =
        capture_current_dependencies_without_refresh(graph, node, work, preparation)?;
    Ok(Some(PrevalidatedTask::Prepared(
        prepared.with_dependencies(dependencies),
    )))
}

fn lower_temporal_condition(
    graph: &mut SignalGraph,
    node: NodeId,
    condition: TemporalCondition,
    ctx: &ConditionEvaluationContext,
    temporal_lowering: &TemporalLoweringContext,
    work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<Option<PrevalidatedTask>, SignalError> {
    if let Some(prevalidated) =
        lower_temporal_condition_from_runtime_clock(condition.clone(), temporal_lowering)
    {
        return match prevalidated {
            PrevalidatedTask::Prepared(prepared) => {
                prepare_condition_blocked_result(graph, node, prepared, work, preparation)
            }
            other => Ok(Some(other)),
        };
    }

    if let Some(ready) = temporal_lowering.ready_wake_for_node(node) {
        if ready.condition() == &condition {
            return Ok(Some(PrevalidatedTask::NeedsCompute {
                temporal_ready: Some(ReadyTemporalEligibility::runtime_wake_backed(
                    condition,
                    ready.id(),
                    ready.ready_ordinal(),
                    ready.ready_tick(),
                )),
                ready_invalidation: None,
            }));
        }
        return Err(SignalError::internal(format!(
            "ready temporal wake {} for node {} carried a descriptor different from the node declaration",
            ready.id().get(),
            node
        )));
    }

    let Some(authority_tick) = temporal_lowering.current_runtime_tick() else {
        return Err(SignalError::invalid_input(format!(
            "temporal condition for node {node} requires runtime-owned temporal lowering"
        )));
    };

    let _ = ctx;
    prepare_condition_blocked_result(
        graph,
        node,
        PreparedEvaluation::deferred_by_time(LoweredTemporalEligibility::Deferred(
            DeferredTemporalEligibility::runtime_wake_deferred(condition, authority_tick),
        )),
        work,
        preparation,
    )
}

fn lower_temporal_condition_from_runtime_clock(
    condition: TemporalCondition,
    temporal_lowering: &TemporalLoweringContext,
) -> Option<PrevalidatedTask> {
    match condition.clone() {
        TemporalCondition::AtOrAfter(at_or_after) => {
            let authority_tick = temporal_lowering.runtime_tick_for(at_or_after.clock_domain())?;
            if authority_tick >= at_or_after.tick() {
                Some(PrevalidatedTask::NeedsCompute {
                    temporal_ready: Some(ReadyTemporalEligibility::runtime_clock_backed(
                        condition,
                        authority_tick,
                    )),
                    ready_invalidation: None,
                })
            } else {
                Some(PrevalidatedTask::Prepared(
                    PreparedEvaluation::deferred_by_time(LoweredTemporalEligibility::Deferred(
                        DeferredTemporalEligibility::runtime_clock_backed(
                            condition,
                            authority_tick,
                        ),
                    )),
                ))
            }
        }
        _ => None,
    }
}
