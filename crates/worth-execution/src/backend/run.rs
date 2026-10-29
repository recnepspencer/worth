use super::{
    admission::BatchDeclaration,
    custody::RunCustody,
    evaluation,
    input::InputMode,
    meter::{KernelContext, KernelFailure, NestedStopOwner, RunLimits},
    payload_storage::PayloadStorage,
    port::{BackendKind, BatchOutcome, BatchStop, TaskOutcome},
    run_admission::{admit_batch_prefix, BatchRunPrefixStop},
    settlement, PreparedBatchResources,
};
use crate::authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial};
use std::sync::Mutex;
use worth_foundational::{
    ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture, ExecutionReport,
};

pub(super) fn run_batch<I, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &BatchDeclaration,
    inputs: I,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
    prepared: Option<PreparedBatchResources>,
    taken: Option<ExecutionMemoryReservation>,
    records: Option<&settlement::records::SettlementRecords>,
    retained: Option<&mut settlement::RetainedSettlement<'_, R>>,
    payload: PayloadStorage<I::Payload, R, E>,
) -> BatchOutcome<R, E>
where
    I: InputMode,
    R: Send,
    E: Send,
    F: Fn(I::Item, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let mut custody = RunCustody::new(inputs, taken, prepared);
    let native = matches!(payload, PayloadStorage::Native);
    let prefix = {
        let inputs = &custody.inputs;
        let mut memory_bytes = |limits: &RunLimits, max_workers: usize| -> Option<u64> {
            if !limits.is_accounted() {
                return Some(0);
            }
            inputs
                .as_ref()
                .expect("inputs precede accounting")
                .memory_bytes::<R, E>(batch, payload.input_measure())
                .and_then(|bytes| {
                    // Retained settlement may enter one native carried-slot context
                    // after all changed workers join, including an empty frontier.
                    let contexts = batch.len().checked_add(usize::from(retained.is_some()))?;
                    bytes.checked_add(limits.framework_context_bytes(contexts, max_workers)?)
                })
        };
        admit_batch_prefix(
            lease,
            batch.len(),
            retained.is_some(),
            backend,
            native,
            custody.prepared.as_ref(),
            &mut custody.reservation,
            &mut custody.serial_reservation,
            &mut custody.taken,
            &mut memory_bytes,
        )
    };
    let prefix = match prefix {
        Ok(prefix) => prefix,
        Err(stop) => return release_prefix_stop(&mut custody, stop, batch, charge_parent),
    };
    // Activity drops with `prefix`, before limits and before custody's entry.
    let _memory_activity = &prefix.memory_activity;
    let limits = &prefix.limits;
    let requested_posture = prefix.requested_posture;
    let initial_fallback = prefix.initial_fallback;
    let accounted = prefix.accounted;
    // The reservation remains alive through settlement and result-vector
    // allocation; the returned values leave the lease at this boundary.
    let mut outcomes: Vec<Mutex<Option<TaskOutcome<R, E>>>> = Vec::new();
    let mut values = Vec::<R>::new();
    if outcomes.try_reserve_exact(batch.len()).is_err()
        || values.try_reserve_exact(batch.len()).is_err()
    {
        let outcome = allocation_refused(accounted);
        return custody.finish(outcome, batch, charge_parent);
    }
    outcomes.extend((0..batch.len()).map(|_| Mutex::new(None)));
    let measures_results = payload.result_measure().is_some() && batch.measures_results();
    let result_memory = Mutex::new(Vec::with_capacity(if measures_results {
        batch.len()
    } else {
        0
    }));
    let execute = |(index, input)| {
        let (outcome, receipt, result_heap) = evaluation::evaluate_with_storage(
            input,
            lease,
            &limits,
            batch.result_capacity(index),
            measures_results.then_some(&result_memory),
            records.and_then(|records| records.storage()),
            kernel,
            &payload,
        );
        if let Some(records) = records {
            records.record(index, &outcome, receipt, result_heap, limits.is_accounted());
        }
        let mut slot = outcomes[index]
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *slot = Some(outcome);
    };
    let seed = match (requested_posture, backend) {
        (ExecutionPosture::Automatic, BackendKind::Perturbation(seed)) => Some(seed),
        _ => None,
    };
    let parallel =
        requested_posture == ExecutionPosture::Automatic && !matches!(backend, BackendKind::Serial);
    let admitted_workers = custody
        .inputs
        .as_mut()
        .expect("inputs dispatch once")
        .dispatch(lease, parallel, seed, limits.is_accounted(), &execute);
    let resolved_posture = if admitted_workers > 1 {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let fallback = if requested_posture == ExecutionPosture::Automatic && admitted_workers == 1 {
        Some(ExecutionFallbackCause::Capacity)
    } else {
        initial_fallback
    };
    let active_high_water = limits.worker_high_watermark();
    let queue_width = if !matches!(backend, BackendKind::Serial)
        && requested_posture == ExecutionPosture::Automatic
    {
        batch.len()
    } else {
        0
    };
    custody.result_memory = result_memory
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let outcome = settlement::settle(
        batch,
        outcomes,
        &limits,
        resolved_posture,
        active_high_water,
        limits.peak_memory(),
        queue_width,
        if matches!(backend, BackendKind::Serial) || requested_posture == ExecutionPosture::Serial {
            Some(0)
        } else {
            None
        },
        fallback,
        retained,
        lease,
        values,
    );
    custody.finish(outcome, batch, charge_parent)
}

fn release_prefix_stop<I: InputMode, R, E>(
    custody: &mut RunCustody<I>,
    stop: BatchRunPrefixStop,
    batch: &BatchDeclaration,
    charge_parent: bool,
) -> BatchOutcome<R, E> {
    let (outcome, limits) = match stop {
        BatchRunPrefixStop::Denied {
            denial,
            accounted,
            limits,
        } => (denied(denial, accounted), limits),
        BatchRunPrefixStop::NoWork {
            physical,
            accounted,
            limits,
        } => (
            BatchOutcome {
                values: Vec::new(),
                stop: None,
                stop_owner: None,
                prefix_boundary: None,
                report: (if accounted {
                    ExecutionReport::new(ExecutionPosture::Serial, 0, 0, physical)
                } else {
                    ExecutionReport::native(ExecutionPosture::Serial, physical)
                })
                .with_fallback(ExecutionFallbackCause::NoWork),
            },
            Some(limits),
        ),
    };
    let outcome = custody.finish(outcome, batch, charge_parent);
    drop(limits);
    outcome
}

fn allocation_refused<R, E>(accounted: bool) -> BatchOutcome<R, E> {
    BatchOutcome {
        values: Vec::new(),
        stop: Some(BatchStop::Allocation(
            crate::ExecutionPhysicalAllocationDenialKind::Allocator,
        )),
        stop_owner: Some(NestedStopOwner::Computation(
            super::NestedComputationStop::Allocation(
                crate::ExecutionPhysicalAllocationDenialKind::Allocator,
            ),
        )),
        prefix_boundary: None,
        report: if accounted {
            ExecutionReport::new(
                ExecutionPosture::Serial,
                0,
                0,
                ExecutionPhysicalReport::default(),
            )
        } else {
            ExecutionReport::native(ExecutionPosture::Serial, ExecutionPhysicalReport::default())
        },
    }
}

fn denied<R, E>(denial: LeaseDenial, accounted: bool) -> BatchOutcome<R, E> {
    BatchOutcome {
        values: Vec::new(),
        stop: Some(BatchStop::Admission(denial)),
        stop_owner: Some(NestedStopOwner::Request),
        prefix_boundary: None,
        report: (if accounted {
            ExecutionReport::new(
                ExecutionPosture::Serial,
                0,
                0,
                ExecutionPhysicalReport::default(),
            )
        } else {
            ExecutionReport::native(ExecutionPosture::Serial, ExecutionPhysicalReport::default())
        })
        .with_fallback(ExecutionFallbackCause::Capacity),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        input::OwnedInputs, meter::NestedComputationStop, run_scope, AdmittedBatch,
    };

    #[test]
    fn physical_allocation_refusal_keeps_original_nested_cause_through_custody() {
        let kind = crate::ExecutionPhysicalAllocationDenialKind::Allocator;
        let outer = run_scope(None, 0, 0, |context| {
            let batch = AdmittedBatch::<()>::try_admit(Vec::new(), 0).unwrap();
            let mut custody = RunCustody::new(OwnedInputs(Vec::<()>::new()), None, None);
            let refused = custody.finish(
                allocation_refused::<(), ()>(true),
                batch.declaration(),
                true,
            );
            assert_eq!(refused.stop, Some(BatchStop::Allocation(kind)));
            assert_eq!(
                context.poll(),
                Err(super::super::meter::KernelStop::NestedStopped)
            );
            assert_eq!(
                context.take_nested_computation_stop(),
                Some(NestedComputationStop::Allocation(kind))
            );
            assert_eq!(context.poll(), Ok(()));
            Ok::<(), KernelFailure<()>>(())
        });
        assert!(outer.result.is_ok());
    }
}
