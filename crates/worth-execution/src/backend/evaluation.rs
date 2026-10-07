use super::{
    meter::{KernelContext, KernelFailure, RunLimits},
    panic_boundary::contain,
    port::TaskOutcome,
};
use crate::{authority::ExecutionResourceLease, report::ChargedBytes};
use std::panic::AssertUnwindSafe;

/// Both input modes cross the same checkpoint, panic, capacity, and cost boundary.
pub(super) fn evaluate<I, R, E, F>(
    input: I,
    lease: Option<&ExecutionResourceLease<'_>>,
    limits: &RunLimits,
    result_capacity: u64,
    kernel: &F,
) -> TaskOutcome<R, E>
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: Fn(I, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    let newly_active_worker = lease.is_some_and(|value| !value.is_current_worker_in_lineage());
    let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
    let mut context = KernelContext::new(limits.clone());
    let result = match limits.safe_point() {
        Err(stop) => {
            if std::mem::needs_drop::<I>() {
                // Only owned destruction adds a panic boundary at a skipped item.
                // A destructor panic replaces the safe-point stop with Panic.
                contain(AssertUnwindSafe(|| drop(input)))
                    .map(|()| Err(KernelFailure::Stop(stop)))
                    .unwrap_or(Err(KernelFailure::Panic))
            } else {
                drop(input);
                Err(KernelFailure::Stop(stop))
            }
        }
        Ok(()) => {
            let _physical_worker = limits.enter_physical_worker(newly_active_worker);
            let _meter_context = context.enter();
            contain(AssertUnwindSafe(|| kernel(input, &mut context)))
                .unwrap_or(Err(KernelFailure::Panic))
        }
    };
    let result = if let Some(stop) = context.checkpoint_stop() {
        Err(KernelFailure::Stop(stop))
    } else if context.nested_stopped() && result.is_ok() {
        Err(KernelFailure::Stop(super::meter::KernelStop::NestedStopped))
    } else {
        result
    };
    let result = match result {
        Ok(value) if value.additional_charged_bytes() > result_capacity => {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        Err(KernelFailure::Domain(error)) if error.additional_charged_bytes() > result_capacity => {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        other => other,
    };
    let (work, span) = context.cost();
    TaskOutcome { result, work, span }
}
