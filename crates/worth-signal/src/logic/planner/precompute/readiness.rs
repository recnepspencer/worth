use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{
    InvalidationReadinessEpoch, InvalidationStageOrder,
};
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::logic::invalidation::scheduling::{
    admit_current_readiness, lower_current_work, ReadyInvalidationQueue, ReadyQueueEntry,
};
use crate::logic::planner::EligibleTask;
use crate::logic::prepared::PreparedEvaluation;

use super::eligibility::PrevalidatedTask;

pub(super) fn attach_ready_invalidation(
    graph: &mut SignalGraph,
    tasks: &[EligibleTask],
    stage_index: u32,
    task_offset: usize,
    readiness_epoch: InvalidationReadinessEpoch,
    prevalidated: &mut [PrevalidatedTask],
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<(), SignalError> {
    let mut queue = ReadyInvalidationQueue::new();
    let result = (|| {
        let mut duplicates = Vec::new();
        for (task_index, (task, posture)) in tasks.iter().zip(prevalidated.iter()).enumerate() {
            if !matches!(posture, PrevalidatedTask::NeedsCompute { .. }) {
                continue;
            }
            let NodeInvalidationInput::Resolved(input) = graph
                .node_invalidation_input_with_execution_work(
                    task.node,
                    work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )?
            else {
                continue;
            };
            let stage_order = InvalidationStageOrder {
                stage: stage_index,
                order: u32::try_from(task_offset.saturating_add(task_index))
                    .map_err(|_| SignalError::invalid_input("graph stage order overflow"))?,
            };
            charge_lowering(&input, work.as_deref_mut(), preparation.as_deref_mut())?;
            let ready = lower_and_admit(
                graph,
                task.node,
                input.clone(),
                readiness_epoch,
                stage_order,
            )?;
            if !queue.insert_with_execution_work(
                graph,
                ReadyQueueEntry { task_index, ready },
                work.as_deref_mut(),
                preparation.as_deref_mut(),
            )? {
                crate::data::request_preparation::push(
                    &mut duplicates,
                    task_index,
                    preparation.as_deref_mut(),
                )?;
            }
            for _ in 0..graph.take_repeated_invalidation_admissions(task.node) {
                charge_lowering(&input, work.as_deref_mut(), preparation.as_deref_mut())?;
                let ready = lower_and_admit(
                    graph,
                    task.node,
                    input.clone(),
                    readiness_epoch,
                    stage_order,
                )?;
                if queue.insert_with_execution_work(
                    graph,
                    ReadyQueueEntry { task_index, ready },
                    work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )? {
                    return Err(SignalError::internal(
                        "repeated same-epoch admission did not merge with current ready work",
                    ));
                }
            }
        }
        suppress_duplicate_tasks(
            graph,
            tasks,
            prevalidated,
            duplicates,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
        assign_ready_work(graph, prevalidated, &mut queue, work.as_deref_mut())
    })();
    queue.release(graph);
    result
}

fn charge_lowering(
    input: &crate::data::proof::invalidation::revalidation::CanonicalDependencyCauseSet,
    work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<(), SignalError> {
    let Some(work) = work else {
        return Ok(());
    };
    let units = input.lowering_copy_work();
    work.checkpoint(units).map_err(|_| {
        SignalError::invalid_input("invalidation lowering stopped before copying authority")
    })?;
    if let Some(budget) = preparation {
        budget.claim(input.lowering_heap_bound()?)?;
    }
    Ok(())
}

fn lower_and_admit(
    graph: &SignalGraph,
    target: crate::data::handle::NodeId,
    input: crate::data::proof::invalidation::revalidation::CanonicalDependencyCauseSet,
    epoch: InvalidationReadinessEpoch,
    order: InvalidationStageOrder,
) -> Result<crate::data::proof::invalidation::progression::ReadyInvalidationBatch, SignalError> {
    let lowered = lower_current_work(graph, target, input, epoch, order)?;
    admit_current_readiness(graph, lowered, epoch, order)
}

fn suppress_duplicate_tasks(
    graph: &SignalGraph,
    tasks: &[EligibleTask],
    prevalidated: &mut [PrevalidatedTask],
    duplicates: Vec<usize>,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<(), SignalError> {
    for task_index in duplicates {
        let dependencies = super::super::validation::capture_current_dependencies_without_refresh(
            graph,
            tasks[task_index].node,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
        prevalidated[task_index] = PrevalidatedTask::Prepared(
            PreparedEvaluation::validated_clean().with_dependencies(dependencies),
        );
    }
    Ok(())
}

fn assign_ready_work(
    graph: &mut SignalGraph,
    prevalidated: &mut [PrevalidatedTask],
    queue: &mut ReadyInvalidationQueue,
    mut work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
) -> Result<(), SignalError> {
    while let Some(entry) = queue.pop_with_execution_work(graph, work.as_deref_mut())? {
        let PrevalidatedTask::NeedsCompute {
            ready_invalidation, ..
        } = &mut prevalidated[entry.task_index]
        else {
            return Err(SignalError::internal(
                "ready invalidation queue targeted a non-compute task",
            ));
        };
        *ready_invalidation = Some(entry.ready);
    }
    Ok(())
}
