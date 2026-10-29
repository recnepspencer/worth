//! The one compiled scope lifecycle. Typed shells pass sizes and trampolines.
use crate::authority::{
    ActiveWorkerGuard, ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial,
    ResourceReservation, SerialRequest,
};
use crate::backend::meter::{
    record_nested_outcome, KernelContext, KernelStop, MemoryActivityGuard, NestedStopOwner,
    RunLimits, WorkerActivityGuard,
};
use crate::backend::panic_boundary::contain;
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

use super::settlement::{apply_held_charge, release_held_reservation};
use super::typed_slot::ScopeSlot;

pub(super) struct ScopeDrive<'a> {
    pub lease: Option<&'a ExecutionResourceLease<'a>>,
    pub retained_bytes: u64,
    pub max_result_bytes: Option<u64>,
    pub charge_parent: bool,
    pub work_ceiling: Option<u64>,
    pub serial: Option<&'a SerialRequest>,
    pub native: bool,
    /// `size_of` of the real result and error. `None` means that sum overflowed.
    pub static_payload_bytes: Option<u64>,
    pub measures_result: bool,
}

/// Guards and limits that stay live across the typed kernel call.
///
/// Fields drop in declaration order: entry reservations first, then the kernel
/// context, then the worker and memory activity that outlive settlement.
pub(super) struct ScopeRun<'a> {
    entry: Option<ResourceReservation>,
    serial: Option<ExecutionMemoryReservation>,
    pub(super) context: KernelContext<'a, 'a>,
    physical: Option<WorkerActivityGuard>,
    worker: Option<ActiveWorkerGuard>,
    memory_activity: Option<MemoryActivityGuard>,
    limits: RunLimits,
    pub(super) memory_bytes: u64,
    pub(super) measures_result: bool,
    max_result_bytes: Option<u64>,
    charge_parent: bool,
}

/// Admits the scope and installs its guards. The shell then runs the kernel.
#[inline(never)]
pub(super) fn begin_scope<'a>(
    drive: &ScopeDrive<'a>,
    reservation: Option<&mut ExecutionMemoryReservation>,
    slot: &mut dyn ScopeSlot,
) -> Result<ScopeRun<'a>, ExecutionReport> {
    let accounted = scope_is_accounted(drive.native, drive.lease, drive.work_ceiling, drive.serial);
    if drive.lease.is_none() && RunLimits::has_leased_parent() {
        return Err(deny(
            slot,
            LeaseDenial::unrelated_nested_lease(),
            drive.charge_parent,
            accounted,
        ));
    }
    let limits = scope_limits(drive.native, drive.lease, drive.work_ceiling, drive.serial);
    let Some(memory_bytes) = payload_memory_bytes(
        &limits,
        drive.static_payload_bytes,
        drive.retained_bytes,
        drive.max_result_bytes,
    ) else {
        return Err(deny(
            slot,
            LeaseDenial::charged_bytes_overflow(),
            drive.charge_parent,
            accounted,
        ));
    };
    // A held scope's memory is the caller's reservation; the entry holds
    // only its worker.
    let entry_bytes = if reservation.is_some() {
        0
    } else {
        memory_bytes
    };
    let entry = match reserve_scope_entry(drive.lease, &limits, entry_bytes) {
        Ok(guard) => guard,
        Err(denial) => return Err(deny(slot, denial, drive.charge_parent, accounted)),
    };
    let mut reservation = reservation;
    if let Some(held) = reservation.as_mut()
        && let Err(denial) = held.take_over(drive.lease, limits.serial_memory(), memory_bytes)
    {
        return Err(deny(slot, denial, drive.charge_parent, accounted));
    }
    let serial = match reserve_serial_memory(
        drive.lease,
        reservation.is_some(),
        limits.serial_memory(),
        memory_bytes,
    ) {
        Ok(guard) => guard,
        Err(denial) => return Err(deny(slot, denial, drive.charge_parent, accounted)),
    };
    if drive.measures_result {
        reservation
            .as_mut()
            .expect("measured scope retains its original reservation")
            .track_activity(&limits);
    }
    let memory_activity = (limits.is_accounted()
        && !drive.measures_result
        && (entry.is_some() || serial.is_some() || reservation.is_some()))
    .then(|| limits.enter_memory(memory_bytes));
    let newly_active = drive.lease.map_or_else(
        || !RunLimits::has_parent(),
        |value| !value.is_current_worker_in_lineage(),
    );
    let worker = drive
        .lease
        .map(ExecutionResourceLease::enter_worker_context);
    let physical = limits.enter_physical_worker(newly_active);
    let context = KernelContext::new(limits.clone());
    Ok(ScopeRun {
        entry,
        serial,
        context,
        physical,
        worker,
        memory_activity,
        limits,
        memory_bytes,
        measures_result: drive.measures_result,
        max_result_bytes: drive.max_result_bytes,
        charge_parent: drive.charge_parent,
    })
}

/// Settles a scope after the shell has stored the kernel result.
#[inline(never)]
pub(super) fn finish_scope(
    run: ScopeRun<'_>,
    mut reservation: Option<&mut ExecutionMemoryReservation>,
    slot: &mut dyn ScopeSlot,
) -> ExecutionReport {
    let ScopeRun {
        entry,
        serial,
        context,
        physical,
        worker,
        memory_activity,
        limits,
        memory_bytes,
        measures_result,
        max_result_bytes,
        charge_parent,
    } = run;
    override_stops(&context, slot);
    if let Some(cap) = max_result_bytes {
        slot.enforce_result_cap(cap);
    }
    if measures_result {
        admit_measured_payload(
            slot,
            reservation
                .as_mut()
                .expect("measured scope retains its original reservation"),
            memory_bytes,
        );
    } else {
        slot.map_produced_to_stop();
    }
    let report = execution_report(&limits, &context);
    settle_after_report(&mut reservation, measures_result, slot);
    let stop_owner = slot.stop_owner(&context);
    record_nested_outcome(report, stop_owner, charge_parent);
    drop(entry);
    drop(serial);
    drop(context);
    drop(physical);
    drop(worker);
    drop(memory_activity);
    drop(limits);
    report
}

fn scope_is_accounted(
    native: bool,
    lease: Option<&ExecutionResourceLease<'_>>,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
) -> bool {
    !native
        || RunLimits::native_accounting(lease)
        || work_ceiling.is_some()
        || serial.is_some_and(|serial| serial.memory().is_some())
}

fn scope_limits(
    native: bool,
    lease: Option<&ExecutionResourceLease<'_>>,
    work_ceiling: Option<u64>,
    serial: Option<&SerialRequest>,
) -> RunLimits {
    let limits = if native {
        RunLimits::for_native(lease, false)
    } else {
        RunLimits::for_run(lease, false)
    };
    limits
        .within_work_ceiling(work_ceiling)
        .within_serial(serial)
}

fn payload_memory_bytes(
    limits: &RunLimits,
    static_payload_bytes: Option<u64>,
    retained_bytes: u64,
    max_result_bytes: Option<u64>,
) -> Option<u64> {
    if !limits.is_accounted() {
        return Some(0);
    }
    static_payload_bytes?
        .checked_add(retained_bytes)?
        .checked_add(max_result_bytes.unwrap_or(0))?
        .checked_add(limits.framework_context_bytes(1, 1)?)
}

fn reserve_scope_entry(
    lease: Option<&ExecutionResourceLease<'_>>,
    limits: &RunLimits,
    entry_bytes: u64,
) -> Result<Option<ResourceReservation>, LeaseDenial> {
    let Some(lease) = lease else {
        return Ok(None);
    };
    if limits.is_accounted() {
        lease.reserve_entry(entry_bytes).map(Some)
    } else {
        lease.reserve_worker_entry().map(Some)
    }
}

fn reserve_serial_memory(
    lease: Option<&ExecutionResourceLease<'_>>,
    held: bool,
    serial: Option<&crate::SerialMemoryBudget>,
    memory_bytes: u64,
) -> Result<Option<ExecutionMemoryReservation>, LeaseDenial> {
    match (lease, serial, held) {
        (None, Some(budget), false) => budget
            .reserve(memory_bytes)
            .map(Some)
            .map_err(LeaseDenial::MemoryExhausted),
        _ => Ok(None),
    }
}

fn deny(
    slot: &mut dyn ScopeSlot,
    denial: LeaseDenial,
    charge_parent: bool,
    accounted: bool,
) -> ExecutionReport {
    slot.store_admission(denial);
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
    report
}

fn override_stops(context: &KernelContext<'_, '_>, slot: &mut dyn ScopeSlot) {
    if let Some(stop) = context.checkpoint_stop() {
        discard_produced(slot);
        slot.store_stop(stop);
    } else if context.nested_stopped() && slot.produced_is_ok() {
        discard_produced(slot);
        slot.store_stop(KernelStop::NestedStopped);
    }
}

fn discard_produced(slot: &mut dyn ScopeSlot) {
    let _ = contain(&mut || slot.drop_produced());
}

fn admit_measured_payload(
    slot: &mut dyn ScopeSlot,
    reservation: &mut ExecutionMemoryReservation,
    memory_bytes: u64,
) {
    let bytes = slot.measured_from_produced();
    let admitted = memory_bytes
        .checked_add(bytes)
        .ok_or_else(LeaseDenial::charged_bytes_overflow)
        .and_then(|total| {
            reservation
                .resize(total)
                .map_err(LeaseDenial::MemoryExhausted)
        });
    match admitted {
        Ok(()) => slot.map_produced_to_stop(),
        Err(denial) => {
            // Keep the original reservation until the unretained payload is gone.
            discard_produced(slot);
            slot.store_admission(denial);
        }
    }
}

fn execution_report(limits: &RunLimits, context: &KernelContext<'_, '_>) -> ExecutionReport {
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

fn settle_after_report(
    reservation: &mut Option<&mut ExecutionMemoryReservation>,
    measures_result: bool,
    slot: &mut dyn ScopeSlot,
) {
    if measures_result {
        let kept = slot.measured_kept_bytes();
        reservation
            .as_mut()
            .expect("measured scope retains its original reservation")
            .resize(kept)
            .expect("measured settlement only releases bytes");
        return;
    }
    let Some(reservation) = reservation.as_mut() else {
        return;
    };
    let kept = slot.kept_for_hold();
    let succeeded = slot.settled_succeeded();
    if let Err(denial) = apply_held_charge(reservation, kept) {
        if succeeded {
            slot.refuse_success(denial);
            release_held_reservation(reservation);
        }
    }
}
