use std::{mem::size_of, panic::AssertUnwindSafe};

use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, SerialRequest},
    report::ChargedBytes,
};

mod settlement;
use settlement::settle_hold;

use super::meter::{
    record_nested_outcome, KernelContext, KernelFailure, KernelStop, NestedStopOwner, RunLimits,
};

/// The original admission or callback stop of one executed scope.
#[derive(Debug)]
pub enum ScopeStop<E> {
    Admission(LeaseDenial),
    Failure(KernelFailure<E>),
}

/// One scope result and its original measured execution report.
pub struct ScopeOutcome<R, E> {
    pub result: Result<R, ScopeStop<E>>,
    pub report: ExecutionReport,
}

/// A caller's reservation a scope's memory admission takes over, in one
/// ledger step as a run takes over its inputs. The scope leaves it holding
/// the bytes `kept` says its result keeps, and nothing when it fails, so what
/// the caller keeps is never unreserved between the two holds.
pub(crate) struct ScopeHold<'h, R> {
    pub(crate) reservation: &'h mut ExecutionMemoryReservation,
    pub(crate) kept: &'h dyn Fn(&R) -> Option<u64>,
}

/// One parent meter and reservation for a multi-stage computation. Descendant
/// patterns inherit its remaining work, deadline, cancellation and memory cap.
pub(crate) fn run_scope<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    run_scope_with_charge(lease, retained_bytes, max_result_bytes, true, kernel)
}

/// A serial oracle still meters its descendants and inherits the invoking
/// ceiling, but its completed report is not charged to the invoking meter.
pub(crate) fn run_scope_with_charge<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    charge_parent: bool,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    run_scope_within(
        lease,
        retained_bytes,
        max_result_bytes,
        charge_parent,
        None,
        None,
        kernel,
    )
}

/// [`run_scope_with_charge`], whose memory admission takes over `hold`.
pub(crate) fn run_scope_holding<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    charge_parent: bool,
    hold: Option<ScopeHold<'_, R>>,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    run_scope_inner(
        lease,
        retained_bytes,
        Some(max_result_bytes),
        charge_parent,
        None,
        None,
        hold,
        |context, _| kernel(context),
    )
}

/// A scope whose work, and every descendant's, is also bounded by
/// `work_ceiling`: the narrower of it and the inherited or leased ceiling.
/// A lease-free scope given a `serial` request observes its cancellation and
/// deadline, and holds its own bytes, and every lease-free descendant's, on
/// its memory budget.
pub(crate) fn run_scope_within<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    run_scope_inner(
        lease,
        retained_bytes,
        Some(max_result_bytes),
        charge_parent,
        work_ceiling,
        serial,
        None,
        |context, _| kernel(context),
    )
}

pub(crate) fn run_scope_measured<R: ChargedBytes, E: ChargedBytes>(
    lease: Option<&ExecutionResourceLease<'_>>,
    serial: Option<&SerialRequest>,
    work: Option<u64>,
    retained_bytes: u64,
    reservation: &mut ExecutionMemoryReservation,
    kernel: impl FnOnce(
        &mut KernelContext<'_, '_>,
        &mut crate::ExecutionResultMemory<'_>,
    ) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    run_scope_inner(
        lease,
        retained_bytes,
        None,
        true,
        work,
        serial,
        Some(ScopeHold {
            reservation,
            kept: &|value: &R| Some(value.additional_charged_bytes()),
        }),
        |context, memory| {
            kernel(
                context,
                &mut memory.expect("measured scope lends result memory"),
            )
        },
    )
}

fn run_scope_inner<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: Option<u64>,
    charge_parent: bool,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
    hold: Option<ScopeHold<'_, R>>,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
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
    if max_result_bytes.is_none() {
        hold.as_mut()
            .expect("measured scope retains its original reservation")
            .reservation
            .track_activity(&limits);
    }
    let _memory_activity = (max_result_bytes.is_some()
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
            let result_memory = max_result_bytes.is_none().then(|| {
                crate::ExecutionResultMemory::new(
                    hold.as_mut()
                        .expect("measured scope owns its reservation")
                        .reservation,
                    memory_bytes,
                )
            });
            super::panic_boundary::contain(AssertUnwindSafe(|| kernel(&mut context, result_memory)))
                .unwrap_or(Err(KernelFailure::Panic))
        }
    };
    let result = if let Some(stop) = context.checkpoint_stop() {
        let _ = super::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
        Err(KernelFailure::Stop(stop))
    } else if context.nested_stopped() && result.is_ok() {
        let _ = super::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
        Err(KernelFailure::Stop(KernelStop::NestedStopped))
    } else {
        result
    };
    let result = match result {
        Ok(value) if max_result_bytes.is_some_and(|cap| value.additional_charged_bytes() > cap) => {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        Err(KernelFailure::Domain(error))
            if max_result_bytes.is_some_and(|cap| error.additional_charged_bytes() > cap) =>
        {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        other => other,
    };
    // Measured scopes have reserved known input/framework bytes already. The callback's
    // actual heap is admitted before framework retention/publication. Cooperating
    // callback owners still reserve their allocations before cloning or growth.
    let measured = max_result_bytes.is_none().then(|| match &result {
        Ok(value) => value.additional_charged_bytes(),
        Err(KernelFailure::Domain(error)) => error.additional_charged_bytes(),
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
                let _ = super::panic_boundary::contain(AssertUnwindSafe(|| drop(result)));
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
                Ok(value) => value.additional_charged_bytes(),
                Err(ScopeStop::Failure(KernelFailure::Domain(error))) => {
                    error.additional_charged_bytes()
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
