//! The common scope execution, control, admission and settlement lifecycle,
//! compiled once. Caller code runs only through [`ScopeBody`], always under the
//! panic boundary.
use super::admission::{admit_scope, AdmittedScope, ScopeRequest, ScopeStorage};
use super::body::{KernelVerdict, ScopeBody, ScopeVerdict};
use super::settlement::{
    admit_measured_payload, discard_payload, settle_hold, settle_measured_hold, weigh_payload,
};
use super::ScopeStop;
use crate::authority::{ExecutionResourceLease, LeaseDenial};
use crate::backend::meter::{
    record_nested_outcome, KernelContext, KernelFailure, KernelStop, NestedStopOwner, RunLimits,
};
use crate::backend::panic_boundary::contain;
use std::panic::AssertUnwindSafe;
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

pub(super) fn run_concrete_scope(
    request: ScopeRequest<'_, '_>,
    body: &mut dyn ScopeBody,
) -> (ScopeVerdict, ExecutionReport) {
    let charge_parent = request.charge_parent;
    let accounted = request.is_accounted();
    match admit_scope(request) {
        Ok(admitted) => run_admitted_scope(admitted, body),
        Err(denial) => {
            discard_payload(body);
            denied(denial, charge_parent, accounted)
        }
    }
}

fn run_admitted_scope(
    admitted: AdmittedScope<'_, '_>,
    body: &mut dyn ScopeBody,
) -> (ScopeVerdict, ExecutionReport) {
    let AdmittedScope {
        lease,
        limits,
        memory_bytes,
        max_result_bytes,
        charge_parent,
        storage,
        mut hold,
        reservation,
        serial_reservation,
    } = admitted;
    let measures_result = storage != ScopeStorage::Native && max_result_bytes.is_none();
    if measures_result {
        hold.as_deref_mut()
            .expect("measured scope retains its original reservation")
            .track_activity(&limits);
    }
    let _memory_activity = (limits.is_accounted()
        && !measures_result
        && (reservation.is_some() || serial_reservation.is_some() || hold.is_some()))
    .then(|| limits.enter_memory(memory_bytes));
    let newly_active = lease.map_or_else(
        || !RunLimits::has_parent(),
        |value| !value.is_current_worker_in_lineage(),
    );
    let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
    let _physical_worker = limits.enter_physical_worker(newly_active);
    let mut context = KernelContext::new(limits.clone());
    let result_memory = hold
        .as_deref_mut()
        .filter(|_| measures_result)
        .map(|reservation| crate::ExecutionResultMemory::new(reservation, memory_bytes));
    let invoked = invoke_kernel(&mut context, result_memory, body);
    let invoked = apply_control_stops(&context, invoked, body);
    let (invoked, payload_bytes) = if storage == ScopeStorage::Metered {
        weigh_payload(invoked, max_result_bytes, body)
    } else {
        (invoked, 0)
    };
    // Measured scopes have reserved known input/framework bytes already. The callback's
    // actual heap is admitted before framework retention/publication. Cooperating
    // callback owners still reserve their allocations before cloning or growth.
    let verdict = match hold.as_deref_mut().filter(|_| measures_result) {
        Some(owner) => admit_measured_payload(owner, memory_bytes, payload_bytes, invoked, body),
        None => invoked.map_err(ScopeStop::Failure),
    };
    let report = scope_report(&context, &limits);
    let verdict = match hold {
        Some(owner) if measures_result => settle_measured_hold(owner, payload_bytes, verdict),
        Some(owner) => settle_hold(owner, verdict, body),
        None => verdict,
    };
    let stop_owner = context.settle_result(&verdict, |stop| match stop {
        ScopeStop::Admission(_) => NestedStopOwner::Request,
        ScopeStop::Failure(failure) => context.failure_owner(failure),
    });
    record_nested_outcome(report, stop_owner, charge_parent);
    drop(reservation);
    drop(serial_reservation);
    (verdict, report)
}

/// Enters the kernel only past the safe point; a stopped entry never runs it.
fn invoke_kernel(
    context: &mut KernelContext<'_, '_>,
    result_memory: Option<crate::ExecutionResultMemory<'_>>,
    body: &mut dyn ScopeBody,
) -> KernelVerdict {
    if let Err(stop) = context.safe_point() {
        discard_payload(body);
        return Err(KernelFailure::Stop(stop));
    }
    let _meter_context = context.enter();
    contain(AssertUnwindSafe(|| body.invoke(context, result_memory)))
        .unwrap_or(Err(KernelFailure::Panic))
}

/// A stop the kernel swallowed still ends the scope.
fn apply_control_stops(
    context: &KernelContext<'_, '_>,
    invoked: KernelVerdict,
    body: &mut dyn ScopeBody,
) -> KernelVerdict {
    if let Some(stop) = context.checkpoint_stop() {
        discard_payload(body);
        Err(KernelFailure::Stop(stop))
    } else if context.nested_stopped() && invoked.is_ok() {
        discard_payload(body);
        Err(KernelFailure::Stop(KernelStop::NestedStopped))
    } else {
        invoked
    }
}

fn scope_report(context: &KernelContext<'_, '_>, limits: &RunLimits) -> ExecutionReport {
    let (work, span) = context.cost();
    let posture = if limits.worker_high_watermark() > 1 {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let physical = ExecutionPhysicalReport::new(
        limits.worker_high_watermark(),
        limits.peak_memory(),
        None,
        0,
        limits.discarded_work(),
    );
    if limits.is_accounted() {
        ExecutionReport::new(posture, work, span, physical)
    } else {
        ExecutionReport::native(posture, physical)
    }
}

fn denied(
    denial: LeaseDenial,
    charge_parent: bool,
    accounted: bool,
) -> (ScopeVerdict, ExecutionReport) {
    let report = if accounted {
        ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        )
    } else {
        ExecutionReport::native(ExecutionPosture::Serial, ExecutionPhysicalReport::default())
    };
    record_nested_outcome(report, Some(NestedStopOwner::Request), charge_parent);
    (Err(ScopeStop::Admission(denial)), report)
}
