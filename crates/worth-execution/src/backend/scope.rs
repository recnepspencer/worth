use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, SerialRequest},
    report::ChargedBytes,
};

mod execution;
mod native_payload;
mod settlement;
use execution::run_scope_inner;
pub(crate) use native_payload::run_scope_native_within;

use super::meter::{KernelContext, KernelFailure};
use super::payload_storage::PayloadStorage;
use worth_foundational::ExecutionReport;

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
        PayloadStorage::<(), R, E>::metered(),
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
        PayloadStorage::<(), R, E>::metered(),
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
        PayloadStorage::<(), R, E>::metered(),
    )
}
