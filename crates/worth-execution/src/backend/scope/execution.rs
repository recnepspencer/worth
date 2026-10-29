//! The common scope execution, control, admission and settlement lifecycle.
//!
//! This function is the typed shell. [`super::drive`] admits and settles one
//! compiled lifecycle; the shell only names the payload and runs the kernel.
use std::mem::size_of;

use crate::authority::{ExecutionMemoryReservation, ExecutionResourceLease, SerialRequest};
use crate::backend::meter::{KernelContext, KernelFailure};
use crate::backend::panic_boundary::contain;
use crate::backend::payload_storage::PayloadStorage;

use super::drive::{begin_scope, finish_scope, ScopeDrive, ScopeRun};
use super::typed_slot::TypedSlot;
use super::{ScopeHold, ScopeOutcome};

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
    let static_payload_bytes = u64::try_from(size_of::<R>())
        .ok()
        .and_then(|bytes| bytes.checked_add(u64::try_from(size_of::<E>()).ok()?));
    let (mut reservation, kept) = match hold {
        Some(hold) => (Some(hold.reservation), Some(hold.kept)),
        None => (None, None),
    };
    let mut slot = TypedSlot::new(payload.result_measure(), payload.error_measure(), kept);
    let mut kernel = Some(kernel);
    let report = match begin_scope(
        &ScopeDrive {
            lease,
            retained_bytes,
            max_result_bytes,
            charge_parent,
            work_ceiling,
            serial,
            native: matches!(payload, PayloadStorage::Native),
            static_payload_bytes,
            measures_result: payload.result_measure().is_some() && max_result_bytes.is_none(),
        },
        reservation.as_deref_mut(),
        &mut slot,
    ) {
        Err(report) => report,
        Ok(mut run) => {
            invoke_kernel(&mut run, reservation.as_deref_mut(), &mut kernel, &mut slot);
            finish_scope(run, reservation.as_deref_mut(), &mut slot)
        }
    };
    ScopeOutcome {
        result: slot.into_result(),
        report,
    }
}

fn invoke_kernel<R, E, F>(
    run: &mut ScopeRun<'_>,
    reservation: Option<&mut ExecutionMemoryReservation>,
    kernel: &mut Option<F>,
    slot: &mut TypedSlot<'_, R, E>,
) where
    F: FnOnce(
        &mut KernelContext<'_, '_>,
        Option<crate::ExecutionResultMemory<'_>>,
    ) -> Result<R, KernelFailure<E>>,
{
    match run.context.safe_point() {
        Err(stop) => slot.store_produced(Err(KernelFailure::Stop(stop))),
        Ok(()) => {
            let _meter_context = run.context.enter();
            let mut memory = None;
            if run.measures_result {
                memory = Some(crate::ExecutionResultMemory::new(
                    reservation.expect("measured scope owns its reservation"),
                    run.memory_bytes,
                ));
            }
            let mut produced = None;
            let panicked = contain(&mut || {
                let kernel = kernel.take().expect("scope kernel runs once");
                produced = Some(kernel(&mut run.context, memory.take()));
            })
            .is_err();
            if panicked {
                slot.store_produced(Err(KernelFailure::Panic));
            } else {
                slot.store_produced(produced.expect("contained kernel returned"));
            }
        }
    }
}
