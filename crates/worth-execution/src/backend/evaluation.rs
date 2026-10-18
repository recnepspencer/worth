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
    capture: Option<&super::ExecutionKernelReceiptStorage>,
    kernel: &F,
) -> (
    TaskOutcome<R, E>,
    Option<super::ExecutionKernelReceipt>,
    Option<u64>,
)
where
    R: ChargedBytes,
    E: ChargedBytes,
    F: Fn(I, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
{
    let newly_active_worker = lease.is_some_and(|value| !value.is_current_worker_in_lineage());
    let _worker_context = lease.map(ExecutionResourceLease::enter_worker_context);
    let mut context = KernelContext::new(limits.clone());
    let mut receipt = None;
    let result = match context.safe_point() {
        Err(stop) => {
            // Entry safe points already latch these request facts, even if
            // skipped input destruction replaces the public cause with Panic.
            if std::mem::needs_drop::<I>() {
                // Only owned destruction adds a panic boundary at a skipped item.
                // A destructor panic replaces the public stop with Panic;
                // it cannot release the interruption's request ownership.
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
            contain(AssertUnwindSafe(|| match capture {
                Some(storage) => {
                    let captured = context.record_kernel(storage, |context| kernel(input, context));
                    receipt = captured.1;
                    captured.0
                }
                None => kernel(input, &mut context),
            }))
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
    let mut result_heap = None;
    let result = match result {
        Ok(value) => {
            let heap = value.additional_charged_bytes();
            if heap > result_capacity {
                Err(KernelFailure::ResultCapacityExceeded)
            } else {
                result_heap = Some(heap);
                Ok(value)
            }
        }
        Err(KernelFailure::Domain(error)) if error.additional_charged_bytes() > result_capacity => {
            Err(KernelFailure::ResultCapacityExceeded)
        }
        other => other,
    };
    let (work, span) = context.cost();
    let stop_owner = context.settle_result(&result, |failure| context.failure_owner(failure));
    if result.is_err() {
        receipt = None;
    }
    (
        TaskOutcome {
            result: super::task_result::TaskResult::new(result, stop_owner),
            work,
            span,
        },
        receipt,
        result_heap,
    )
}
