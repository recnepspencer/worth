use std::{
    mem::size_of,
    panic::{catch_unwind, AssertUnwindSafe},
};

use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    report::ChargedBytes,
};

use super::meter::{record_nested, KernelContext, KernelFailure, KernelStop, RunLimits};

#[derive(Debug)]
pub(crate) enum ScopeStop<E> {
    Admission(LeaseDenial),
    Failure(KernelFailure<E>),
}

pub(crate) struct ScopeOutcome<R, E> {
    pub(crate) result: Result<R, ScopeStop<E>>,
    pub(crate) report: ExecutionReport,
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
        u64::MAX,
        kernel,
    )
}

/// A scope whose work, and every descendant's, is also bounded by
/// `work_ceiling`: the narrower of it and the inherited or leased ceiling.
pub(crate) fn run_scope_within<R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    max_result_bytes: u64,
    charge_parent: bool,
    work_ceiling: u64,
    kernel: F,
) -> ScopeOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: FnOnce(&mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    if lease.is_none() && RunLimits::has_leased_parent() {
        return denied(LeaseDenial::UnrelatedNestedLease, charge_parent);
    }
    let limits = RunLimits::for_run(lease, false).within_work_ceiling(work_ceiling);
    let memory_bytes = u64::try_from(size_of::<R>())
        .ok()
        .and_then(|bytes| bytes.checked_add(u64::try_from(size_of::<E>()).ok()?))
        .and_then(|bytes| bytes.checked_add(retained_bytes))
        .and_then(|bytes| bytes.checked_add(max_result_bytes))
        .and_then(|bytes| bytes.checked_add(limits.framework_context_bytes(1, 1)?));
    let Some(memory_bytes) = memory_bytes else {
        return denied(LeaseDenial::ResourceExhausted, charge_parent);
    };
    let reservation = if let Some(lease) = lease {
        match lease.reserve_entry(memory_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => return denied(denial, charge_parent),
        }
    } else {
        None
    };
    let _memory_activity = reservation
        .as_ref()
        .map(|_| limits.enter_memory(memory_bytes));
    let newly_active = lease.map_or_else(
        || !RunLimits::has_parent(),
        |value| !value.is_current_worker_in_lineage(),
    );
    let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
    let _physical_worker = limits.enter_physical_worker(newly_active);
    let mut context = KernelContext::new(limits.clone());
    let result = match limits.safe_point() {
        Err(stop) => Err(KernelFailure::Stop(stop)),
        Ok(()) => {
            let _meter_context = context.enter();
            catch_unwind(AssertUnwindSafe(|| kernel(&mut context)))
                .unwrap_or(Err(KernelFailure::Panic))
        }
    };
    let result = if let Some(stop) = context.checkpoint_stop() {
        Err(KernelFailure::Stop(stop))
    } else if context.nested_stopped() && result.is_ok() {
        Err(KernelFailure::Stop(KernelStop::NestedStopped))
    } else {
        result
    };
    let result = match result {
        Ok(value) if value.additional_charged_bytes() > max_result_bytes => {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        Err(KernelFailure::Domain(error))
            if error.additional_charged_bytes() > max_result_bytes =>
        {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        other => other,
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
    if charge_parent {
        record_nested(report, result.is_err());
    }
    drop(reservation);
    ScopeOutcome {
        result: result.map_err(ScopeStop::Failure),
        report,
    }
}

fn denied<R, E>(denial: LeaseDenial, charge_parent: bool) -> ScopeOutcome<R, E> {
    let report = ExecutionReport::new(
        ExecutionPosture::Serial,
        0,
        0,
        ExecutionPhysicalReport::default(),
    );
    if charge_parent {
        record_nested(report, true);
    }
    ScopeOutcome {
        result: Err(ScopeStop::Admission(denial)),
        report,
    }
}
