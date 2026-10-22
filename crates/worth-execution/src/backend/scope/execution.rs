//! The common scope execution, control, admission and settlement lifecycle.
use super::{settlement::settle_hold, ScopeHold, ScopeOutcome, ScopeStop};
use crate::authority::{ExecutionResourceLease, LeaseDenial, SerialRequest};
use crate::backend::meter::{
    record_nested_outcome, KernelContext, KernelFailure, KernelStop, NestedStopOwner, RunLimits,
};
use crate::backend::payload_storage::PayloadStorage;
use std::{mem::size_of, panic::AssertUnwindSafe};
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

pub(super) fn run_scope_inner<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: Option<u64>,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    hold: Option<ScopeHold<'_, R>>,
    kernel: F,
    payload: PayloadStorage<(), R, E>,
) -> ScopeOutcome<R, E>
where
    F: FnOnce(
        &mut KernelContext<'_, '_>,
        Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>>,
{
    if lease.is_none() && RunLimits::has_leased_parent() {
        return denied(LeaseDenial::unrelated_nested_lease(), charge_parent);
    }
    let limits = RunLimits::for_run(lease, false)
        .within_work_ceiling(work_ceiling)
        .within_serial(serial);
    let memory_bytes = u64::try_from(size_of::<R>())
        .ok()
        .and_then(|bytes| bytes.checked_add(u64::try_from(size_of::<E>()).ok()?))
        .and_then(|bytes| bytes.checked_add(retained_bytes))
        .and_then(|bytes| bytes.checked_add(max_result_bytes.unwrap_or(0)))
        .and_then(|bytes| bytes.checked_add(limits.framework_context_bytes(1, 1)?));
    let Some(memory_bytes) = memory_bytes else {
        return denied(LeaseDenial::charged_bytes_overflow(), charge_parent);
    };
    // A held scope's memory is the caller's reservation; the entry holds
    // only its worker.
    let entry_bytes = if hold.is_some() { 0 } else { memory_bytes };
    let reservation = if let Some(lease) = lease {
        match lease.reserve_entry(entry_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => return denied(denial, charge_parent),
        }
    } else {
        None
    };
    let mut hold = match hold {
        Some(hold) => {
            if let Err(denial) =
                hold.reservation
                    .take_over(lease, limits.serial_memory(), memory_bytes)
            {
                return denied(denial, charge_parent);
            }
            Some(hold)
        }
        None => None,
    };
    let serial_reservation = match (lease, limits.serial_memory(), &hold) {
        (None, Some(budget), None) => match budget.reserve(memory_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => return denied(LeaseDenial::MemoryExhausted(denial), charge_parent),
        },
        _ => None,
    };
    let measures_result = payload.result_measure().is_some() && max_result_bytes.is_none();
    if measures_result {
        hold.as_mut()
            .expect("measured scope retains its original reservation")
            .reservation
            .track_activity(&limits);
    }
    let _memory_activity = (!measures_result
        && (reservation.is_some() || serial_reservation.is_some() || hold.is_some()))
    .then(|| limits.enter_memory(memory_bytes));
    let newly_active = lease.map_or_else(
        || !RunLimits::has_parent(),
        |value| !value.is_current_worker_in_lineage(),
    );
    let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
    let _physical_worker = limits.enter_physical_worker(newly_active);
    let mut context = KernelContext::new(limits.clone());
    let result = match context.safe_point() {
        Err(stop) => Err(KernelFailure::Stop(stop)),
        Ok(()) => {
            let _meter_context = context.enter();
            let result_memory = measures_result.then(|| {
                crate::ExecutionResultMemory::new(
                    hold.as_mut()
                        .expect("measured scope owns its reservation")
                        .reservation,
                    memory_bytes,
                )
            });
            crate::backend::panic_boundary::contain(AssertUnwindSafe(|| {
                kernel(&mut context, result_memory)
            }))
            .unwrap_or(Err(KernelFailure::Panic))
        }
    };
    let result = if let Some(stop) = context.checkpoint_stop() {
        let _ = crate::backend::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
        Err(KernelFailure::Stop(stop))
    } else if context.nested_stopped() && result.is_ok() {
        let _ = crate::backend::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
        Err(KernelFailure::Stop(KernelStop::NestedStopped))
    } else {
        result
    };
    let result = match result {
        Ok(value)
            if payload.result_measure().is_some_and(|measure| {
                max_result_bytes.is_some_and(|cap| measure(&value) > cap)
            }) =>
        {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        Err(KernelFailure::Domain(error))
            if payload.error_measure().is_some_and(|measure| {
                max_result_bytes.is_some_and(|cap| measure(&error) > cap)
            }) =>
        {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        other => other,
    };
    // Measured scopes have reserved known input/framework bytes already. The callback's
    // actual heap is admitted before framework retention/publication. Cooperating
    // callback owners still reserve their allocations before cloning or growth.
    let measured = measures_result.then(|| match &result {
        Ok(value) => payload.result_measure().expect("measured result callback")(value),
        Err(KernelFailure::Domain(error)) => {
            payload.error_measure().expect("measured error callback")(error)
        }
        Err(_) => 0,
    });
    let result = if let Some(bytes) = measured {
        let owner = hold
            .as_mut()
            .expect("measured scope retains its original reservation");
        let admitted = memory_bytes
            .checked_add(bytes)
            .ok_or_else(LeaseDenial::charged_bytes_overflow)
            .and_then(|total| {
                owner
                    .reservation
                    .resize(total)
                    .map_err(LeaseDenial::MemoryExhausted)
            });
        match admitted {
            Ok(()) => result.map_err(ScopeStop::Failure),
            Err(denial) => {
                // Keep the original reservation until the unretained materialized payload is gone.
                let _ = crate::backend::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
                Err(ScopeStop::Admission(denial))
            }
        }
    } else {
        result.map_err(ScopeStop::Failure)
    };
    let (work, span) = context.cost();
    let report = ExecutionReport::new(
        if limits.worker_high_watermark() > 1 {
            ExecutionPosture::Automatic
        } else {
            ExecutionPosture::Serial
        },
        work,
        span,
        ExecutionPhysicalReport::new(
            limits.worker_high_watermark(),
            limits.peak_memory(),
            None,
            0,
            limits.discarded_work(),
        ),
    );
    let result = match (hold, measured) {
        (Some(hold), Some(_)) => {
            let kept = match &result {
                Ok(value) => payload.result_measure().expect("measured result callback")(value),
                Err(ScopeStop::Failure(KernelFailure::Domain(error))) => {
                    payload.error_measure().expect("measured error callback")(error)
                }
                Err(_) => 0,
            };
            // Shrinking cannot refuse. Keep the measured value/error charged until its caller releases it.
            hold.reservation
                .resize(kept)
                .expect("measured settlement only releases bytes");
            result
        }
        (Some(hold), None) => settle_hold(
            hold,
            result.map_err(|stop| match stop {
                ScopeStop::Failure(failure) => failure,
                ScopeStop::Admission(_) => unreachable!("declared scope has no measured admission"),
            }),
        ),
        (None, _) => result,
    };
    let stop_owner = context.settle_result(&result, |stop| match stop {
        ScopeStop::Admission(_) => NestedStopOwner::Request,
        ScopeStop::Failure(failure) => context.failure_owner(failure),
    });
    record_nested_outcome(report, stop_owner, charge_parent);
    drop(reservation);
    drop(serial_reservation);
    ScopeOutcome { result, report }
}

fn denied<R, E>(denial: LeaseDenial, charge_parent: bool) -> ScopeOutcome<R, E> {
    let report = ExecutionReport::new(
        ExecutionPosture::Serial,
        0,
        0,
        ExecutionPhysicalReport::default(),
    );
    record_nested_outcome(report, Some(NestedStopOwner::Request), charge_parent);
    ScopeOutcome {
        result: Err(ScopeStop::Admission(denial)),
        report,
    }
}
