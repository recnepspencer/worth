//! The request owns computed values before downstream capacity is lowered.
use super::super::callback::SignalPrecompute;
use super::super::eligibility::PrevalidatedTask;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationDisposition};
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::DisjointGraphBatch;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStorageMeasurement, RetainedStoragePreparation,
};
use crate::logic::planner::execution::CheckedAdmittedEpoch;
use crate::logic::planner::types::ResolvedSignalPlannerPolicy;
use crate::logic::prepared::{ExecutionSnapshot, PreparedEvaluation};
use std::sync::Mutex;
use worth_execution::{
    ExecutionMemoryReservation, MapKernelContext, MapKernelFailure, MapOutcome, MapStop,
};
use worth_foundational::PartitionIdentity;

struct ReservedOutput {
    outcome: Result<PreparedEvaluation, worth_execution::ScopeStop<SignalError>>,
    memory: ExecutionMemoryReservation,
}

type OutputSlot = Mutex<Option<Result<ReservedOutput, SignalError>>>;

pub(super) struct ComputedValues {
    pub(super) values: Vec<PreparedEvaluation>,
    pub(super) memory: Vec<ExecutionMemoryReservation>,
}

pub(super) fn compute(
    graph: &SignalGraph,
    validated: &[PrevalidatedTask],
    batch: &DisjointGraphBatch,
    stage: &CheckedAdmittedEpoch<'_, '_, '_>,
    precompute: &impl SignalPrecompute,
    policy: &ResolvedSignalPlannerPolicy,
    work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: &mut SignalPreparationBudget,
) -> Result<(ComputedValues, worth_foundational::ExecutionReport), SignalError> {
    let map = super::map_declaration::lower_checked_map(
        graph,
        validated,
        batch,
        stage,
        work,
        Some(&mut *preparation),
    )?;
    preparation.claim_vec::<OutputSlot>(stage.tasks().len())?;
    let slots: Vec<OutputSlot> = (0..stage.tasks().len()).map(|_| Mutex::new(None)).collect();
    let request = stage.request();
    let snapshot = ExecutionSnapshot::new(graph);
    let posture = if map.partition_count() < policy.parallel_min_tasks() {
        worth_foundational::ExecutionPosture::Serial
    } else {
        request.resolved_posture()
    };
    let map =
        crate::data::prepared_request_map::PreparedRequestMap::prepare(map, request, posture)?;
    let outcome = map.run(|item, work| {
        let task = &stage.tasks()[item.task_index];
        work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        let view = snapshot.read_view(task.node);
        let captured = compute_output(task.node, &view, precompute, item.capacity, request);
        let stopped = !matches!(&captured, Ok(ReservedOutput { outcome: Ok(_), .. }));
        *slots[item.task_index]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(captured);
        if stopped {
            Err(MapKernelFailure::<()>::Domain(()))
        } else {
            Ok(())
        }
    });
    let report = match outcome {
        MapOutcome::Complete { report, .. } => report,
        MapOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => {
            return Err(SignalError::execution_stopped(SignalExecutionStop::new(
                restore_stop(reason, &slots, stage.task_offset()).into(),
                boundary,
                SignalPublicationDisposition::WorkerLocal,
                report,
            )));
        }
    };
    Ok((
        collect_outputs(slots, map_count(validated), preparation)?,
        report,
    ))
}

fn collect_outputs(
    slots: Vec<OutputSlot>,
    count: usize,
    preparation: &mut SignalPreparationBudget,
) -> Result<ComputedValues, SignalError> {
    preparation.claim_vec::<PreparedEvaluation>(count)?;
    preparation.claim_vec::<ExecutionMemoryReservation>(count)?;
    let mut computed = ComputedValues {
        values: Vec::with_capacity(count),
        memory: Vec::with_capacity(count),
    };
    // Moving the original value into reconciliation keeps its original ticket.
    // Only the two new Vec backings require an additional reservation.
    for slot in slots {
        if let Some(captured) = slot
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            let ReservedOutput {
                outcome,
                memory: held,
            } = captured?;
            computed
                .values
                .push(outcome.expect("completed map retained a successful measured result"));
            computed.memory.push(held);
        }
    }
    Ok(computed)
}

fn map_count(validated: &[PrevalidatedTask]) -> usize {
    validated
        .iter()
        .filter(|task| matches!(task, PrevalidatedTask::NeedsCompute { .. }))
        .count()
}

fn compute_output(
    node: crate::data::handle::NodeId,
    view: &crate::logic::prepared::ExecutionReadView<'_>,
    precompute: &impl SignalPrecompute,
    capacity: super::super::callback::CheckedKernelCapacity,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ReservedOutput, SignalError> {
    let mut memory = request
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(SignalError::execution_scope_denied)?
        .map_err(SignalError::execution_admission_denied)?;
    let outcome = request.run_measured_result(
        worth_execution::ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |work, _memory| {
            let prepared = precompute
                .prepare(node, view, work, Some(capacity))
                .map_err(MapKernelFailure::Domain)?;
            let bytes = measure(work, |observed| prepared.retained_heap_charge(observed))
                .map_err(MapKernelFailure::Domain)?;
            if capacity
                .scratch_bytes
                .is_some_and(|capacity| bytes > capacity)
            {
                return Err(MapKernelFailure::ResultCapacityExceeded);
            }
            Ok(prepared)
        },
    );
    Ok(ReservedOutput {
        outcome: outcome.result,
        memory,
    })
}

pub(super) fn measure_result_maximum(
    values: &[PreparedEvaluation],
    work: &mut MapKernelContext<'_, '_>,
) -> Result<u64, SignalError> {
    values.iter().try_fold(0, |maximum, value| {
        measure(work, |observed| value.checked_result_heap_charge(observed))
            .map(|bytes| maximum.max(bytes))
    })
}

fn measure(
    work: &mut MapKernelContext<'_, '_>,
    measure: impl FnOnce(
        &mut RetainedStoragePreparation<'_>,
    ) -> Result<
        RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    >,
) -> Result<u64, SignalError> {
    let mut preparation = RetainedStoragePreparation::new(None);
    let mut checkpoint = |visits: usize| {
        work.checkpoint(visits as u64).map_err(|stop| {
            crate::data::retained_storage::RetainedStoragePreparationDenial::ExecutionStopped(
                stop.into(),
            )
        })
    };
    measure(&mut preparation.reborrow_with_checkpoint(&mut checkpoint))
        .map(RetainedStorageCharge::bytes)
        .map_err(SignalError::retained_storage_denied)
}

fn restore_stop(reason: MapStop<()>, slots: &[OutputSlot], offset: usize) -> MapStop<SignalError> {
    match reason {
        MapStop::Failure { identity, cause } => {
            let cause = match cause {
                MapKernelFailure::Domain(()) => {
                    let slot = slots
                        .iter()
                        .enumerate()
                        .find_map(|(index, slot)| {
                            (PartitionIdentity::new((offset + index) as u64) == identity)
                                .then_some(slot)
                        })
                        .expect("failed callback belongs to its request slot");
                    let captured = slot
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .take()
                        .expect("domain stop captured once");
                    match captured {
                        Ok(ReservedOutput {
                            outcome: Err(worth_execution::ScopeStop::Admission(denial)),
                            ..
                        }) => return MapStop::Admission(denial),
                        Ok(ReservedOutput {
                            outcome: Err(worth_execution::ScopeStop::Failure(cause)),
                            ..
                        }) => cause,
                        Err(error) => MapKernelFailure::Domain(error),
                        _ => unreachable!("successful output cannot cause a domain stop"),
                    }
                }
                MapKernelFailure::Stop(stop) => MapKernelFailure::Stop(stop),
                MapKernelFailure::Panic => MapKernelFailure::Panic,
                MapKernelFailure::ResultCapacityExceeded => {
                    MapKernelFailure::ResultCapacityExceeded
                }
            };
            MapStop::Failure { identity, cause }
        }
        MapStop::WorkExhausted { identity } => MapStop::WorkExhausted { identity },
        MapStop::Admission(denial) => MapStop::Admission(denial),
    }
}

#[cfg(test)]
mod tests;
