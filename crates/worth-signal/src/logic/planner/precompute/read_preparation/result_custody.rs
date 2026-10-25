//! Callback results retain their pre-effect reservation through publication.
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
    outcome: Result<PreparedEvaluation, MapKernelFailure<SignalError>>,
    memory: ExecutionMemoryReservation,
}

type OutputSlot = Mutex<Option<ReservedOutput>>;

pub(super) struct ComputedValues {
    pub(super) values: Vec<PreparedEvaluation>,
    pub(super) memory: Vec<ExecutionMemoryReservation>,
}

pub(super) struct PreparedComputation<'request, 'authority> {
    map: crate::data::prepared_request_map::PreparedRequestMap<
        'request,
        'authority,
        super::map_declaration::GraphWorkItem,
        crate::data::proof::invalidation::progression::GraphProposalKey,
        (),
        (),
    >,
    slots: Vec<OutputSlot>,
    computed: ComputedValues,
    offset: usize,
}

pub(super) fn prepare<'request, 'authority>(
    graph: &SignalGraph,
    validated: &[PrevalidatedTask],
    batch: &DisjointGraphBatch,
    stage: &CheckedAdmittedEpoch<'_, 'request, 'authority>,
    policy: &ResolvedSignalPlannerPolicy,
    work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: &mut SignalPreparationBudget,
) -> Result<PreparedComputation<'request, 'authority>, SignalError> {
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
    let computed = allocate_outputs(map_count(validated), preparation)?;
    let request = stage.request();
    let posture = if map.partition_count() < policy.parallel_min_tasks() {
        worth_foundational::ExecutionPosture::Serial
    } else {
        request.resolved_posture()
    };
    let map =
        crate::data::prepared_request_map::PreparedRequestMap::prepare(map, request, posture)?;
    Ok(PreparedComputation {
        map,
        slots,
        computed,
        offset: stage.task_offset(),
    })
}

impl PreparedComputation<'_, '_> {
    pub(super) fn compute(
        self,
        graph: &SignalGraph,
        tasks: &[crate::logic::planner::EligibleTask],
        precompute: &impl SignalPrecompute,
    ) -> Result<(ComputedValues, worth_foundational::ExecutionReport), SignalError> {
        let Self {
            map,
            slots,
            computed,
            offset,
        } = self;
        let snapshot = ExecutionSnapshot::new(graph);
        let outcome = map.run(|item, work| {
            let task = &tasks[item.task_index];
            work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
            let view = snapshot.read_view(task.node);
            let memory = item
                .memory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .expect("one callback consumes its admitted result custody");
            let captured =
                compute_output(task.node, &view, precompute, item.capacity, work, memory);
            let stopped = captured.outcome.is_err();
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
                    restore_stop(reason, &slots, offset).into(),
                    boundary,
                    SignalPublicationDisposition::WorkerLocal,
                    report,
                )));
            }
        };
        Ok((collect_outputs(slots, computed), report))
    }
}

fn allocate_outputs(
    count: usize,
    preparation: &mut SignalPreparationBudget,
) -> Result<ComputedValues, SignalError> {
    preparation.claim_vec::<PreparedEvaluation>(count)?;
    preparation.claim_vec::<ExecutionMemoryReservation>(count)?;
    Ok(ComputedValues {
        values: Vec::with_capacity(count),
        memory: Vec::with_capacity(count),
    })
}

fn collect_outputs(slots: Vec<OutputSlot>, mut computed: ComputedValues) -> ComputedValues {
    // Moving the original value into reconciliation keeps its original ticket.
    // Both Vec backings were allocated before dispatch.
    for slot in slots {
        if let Some(captured) = slot
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            let ReservedOutput {
                outcome,
                memory: held,
            } = captured;
            computed
                .values
                .push(outcome.expect("completed map retained a successful measured result"));
            computed.memory.push(held);
        }
    }
    computed
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
    work: &mut MapKernelContext<'_, '_>,
    mut memory: ExecutionMemoryReservation,
) -> ReservedOutput {
    let outcome = (|| {
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
        // Finite capacities were reserved before the map was dispatched.
        // An unbounded authority has no capacity refusal; still retain the
        // exact measured bytes, including dependency capture.
        memory.resize(bytes).map_err(|denial| {
            MapKernelFailure::Domain(SignalError::execution_admission_denied(
                worth_execution::LeaseDenial::MemoryExhausted(denial),
            ))
        })?;
        Ok(prepared)
    })();
    ReservedOutput { outcome, memory }
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
                    match captured.outcome {
                        Err(cause) => cause,
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
        MapStop::Allocation(kind) => MapStop::Allocation(kind),
    }
}

#[cfg(test)]
mod tests;
