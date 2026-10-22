use super::{
    admission::BatchDeclaration,
    custody::RunCustody,
    evaluation,
    input::InputMode,
    meter::{KernelContext, KernelFailure, NestedStopOwner, RunLimits},
    payload_storage::PayloadStorage,
    port::{BackendKind, BatchOutcome, BatchStop, TaskOutcome},
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
    if lease.is_none() && RunLimits::has_leased_parent() {
        let outcome = denied(LeaseDenial::unrelated_nested_lease());
        return custody.finish(outcome, batch, charge_parent);
    }
    if custody
        .prepared
        .as_ref()
        .is_some_and(|resources| !resources.matches_current_parent())
    {
        let outcome = denied(LeaseDenial::unrelated_nested_lease());
        return custody.finish(outcome, batch, charge_parent);
    }
    let limits = if let Some(resources) = custody.prepared.as_ref() {
        resources.dispatch_limits(lease.expect("prepared map requires its lease"), backend)
    } else {
        RunLimits::for_run(lease, matches!(backend, BackendKind::Serial))
    };
    let requested_posture = if lease
        .is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic)
        && !limits.force_serial()
        && !matches!(backend, BackendKind::Serial)
    {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let initial_fallback = if requested_posture == ExecutionPosture::Serial {
        // A lease that resolves serial names why. An automatic lease runs
        // serial only under a serial oracle: this run's, or the enclosing
        // run's that its limits inherit.
        Some(match lease {
            None => ExecutionFallbackCause::NoLease,
            Some(value) => value
                .serial_cause()
                .unwrap_or(ExecutionFallbackCause::OracleSerial),
        })
    } else {
        None
    };
    if batch.len() == 0 && retained.is_none() {
        let physical = if custody.prepared.is_some() {
            ExecutionPhysicalReport::new(
                limits.worker_high_watermark(),
                limits.peak_memory(),
                Some(0),
                0,
                limits.discarded_work(),
            )
        } else {
            ExecutionPhysicalReport::default()
        };
        let outcome = BatchOutcome {
            values: Vec::new(),
            stop: None,
            stop_owner: None,
            prefix_boundary: None,
            report: ExecutionReport::new(ExecutionPosture::Serial, 0, 0, physical)
                .with_fallback(ExecutionFallbackCause::NoWork),
        };
        return custody.finish(outcome, batch, charge_parent);
    }
    // A lease-free run holds its framework at one worker: its own thread.
    let max_workers = lease.map_or(1, |value| value.policy().budget().max_workers().get());
    let memory_bytes = custody
        .inputs
        .as_ref()
        .expect("inputs precede accounting")
        .memory_bytes::<R, E>(batch, payload.input_measure())
        .and_then(|bytes| {
            // Retained settlement may enter one native carried-slot context
            // after all changed workers join, including an empty frontier.
            let contexts = batch.len().checked_add(usize::from(retained.is_some()))?;
            bytes.checked_add(limits.framework_context_bytes(contexts, max_workers)?)
        });
    let Some(memory_bytes) = memory_bytes else {
        let outcome = denied(LeaseDenial::charged_bytes_overflow());
        return custody.finish(outcome, batch, charge_parent);
    };
    if custody
        .prepared
        .as_ref()
        .is_some_and(|resources| !resources.matches_dispatch(memory_bytes))
    {
        let outcome = denied(LeaseDenial::unrelated_nested_lease());
        return custody.finish(outcome, batch, charge_parent);
    }
    custody.reservation = if let Some(lease) = lease {
        let admission_bytes = if custody.prepared.is_some() || custody.taken.is_some() {
            0
        } else {
            memory_bytes
        };
        match lease.reserve_entry(admission_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => {
                let outcome = denied(denial);
                return custody.finish(outcome, batch, charge_parent);
            }
        }
    } else {
        None
    };
    if let Some(held) = custody.taken.as_mut()
        && let Err(denial) = held.take_over(lease, limits.serial_memory(), memory_bytes)
    {
        // The caller still covers every input. Close the zero-byte entry
        // before refused inputs can clean up otherwise empty ledger nodes.
        drop(custody.reservation.take());
        let outcome = denied(denial);
        return custody.finish(outcome, batch, charge_parent);
    }
    custody.serial_reservation = match (lease, limits.serial_memory(), &custody.taken) {
        (None, Some(budget), None) => match budget.reserve(memory_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => {
                let outcome = denied(LeaseDenial::MemoryExhausted(denial));
                return custody.finish(outcome, batch, charge_parent);
            }
        },
        _ => None,
    };
    let _memory_activity = if custody.prepared.is_some() {
        None
    } else {
        (custody.reservation.is_some()
            || custody.serial_reservation.is_some()
            || custody.taken.is_some())
        .then(|| limits.enter_memory(memory_bytes))
    };
    // The reservation remains alive through settlement and result-vector
    // allocation; the returned values leave the lease at this boundary.
    let mut outcomes: Vec<Mutex<Option<TaskOutcome<R, E>>>> = Vec::new();
    let mut values = Vec::<R>::new();
    if outcomes.try_reserve_exact(batch.len()).is_err()
        || values.try_reserve_exact(batch.len()).is_err()
    {
        let outcome = allocation_refused();
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
            records.map(|records| records.storage()),
            kernel,
            &payload,
        );
        if let Some(records) = records {
            records.record(index, &outcome, receipt, result_heap);
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
        .dispatch(lease, parallel, seed, &execute);
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

fn allocation_refused<R, E>() -> BatchOutcome<R, E> {
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
        report: ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        ),
    }
}

fn denied<R, E>(denial: LeaseDenial) -> BatchOutcome<R, E> {
    BatchOutcome {
        values: Vec::new(),
        stop: Some(BatchStop::Admission(denial)),
        stop_owner: Some(NestedStopOwner::Request),
        prefix_boundary: None,
        report: ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        )
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
            let refused = custody.finish(allocation_refused::<(), ()>(), batch.declaration(), true);
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
