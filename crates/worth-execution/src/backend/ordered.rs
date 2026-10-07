use std::{mem::size_of, panic::AssertUnwindSafe};

use worth_foundational::{
    ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture, ExecutionReport,
    PartitionIdentity,
};
use worth_proof::CanonicalUniqueVec;

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial, MemoryLimitDenial, MemoryLimitLevel},
    report::ChargedBytes,
};

use super::{
    meter::{record_nested, KernelContext, KernelFailure, KernelStop, RunLimits},
    port::BatchStop,
};

pub(crate) struct OrderedOutcome<S, O, E> {
    pub(crate) state: S,
    pub(crate) outputs: Vec<O>,
    pub(crate) completed_early: bool,
    pub(crate) stop: Option<BatchStop<E>>,
    pub(crate) boundary: Option<PartitionIdentity>,
    pub(crate) report: ExecutionReport,
}

pub(crate) enum OrderedStep<S, O> {
    Continue(S, O),
    Complete(S, O),
}

/// A sequential dependency chain under the same accounting authority as map.
/// Each step sees only the preceding committed state and returns a replacement.
pub(crate) fn run_ordered<S, O, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    identities: &CanonicalUniqueVec<PartitionIdentity>,
    initial: S,
    max_state_bytes: u64,
    max_output_bytes: u64,
    max_error_bytes: u64,
    scratch_bytes: u64,
    input_bytes: u64,
    mut step: F,
) -> OrderedOutcome<S, O, E>
where
    S: ChargedBytes,
    O: ChargedBytes,
    E: ChargedBytes,
    F: FnMut(&S, usize, &mut KernelContext<'_, '_>) -> Result<(S, O), KernelFailure<E>>,
{
    run_ordered_until(
        lease,
        identities,
        initial,
        max_state_bytes,
        max_output_bytes,
        max_error_bytes,
        scratch_bytes,
        input_bytes,
        |state, index, context| {
            step(state, index, context).map(|(next, output)| OrderedStep::Continue(next, output))
        },
    )
}

pub(crate) fn run_ordered_until<S, O, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    identities: &CanonicalUniqueVec<PartitionIdentity>,
    initial: S,
    max_state_bytes: u64,
    max_output_bytes: u64,
    max_error_bytes: u64,
    scratch_bytes: u64,
    input_bytes: u64,
    mut step: F,
) -> OrderedOutcome<S, O, E>
where
    S: ChargedBytes,
    O: ChargedBytes,
    E: ChargedBytes,
    F: FnMut(&S, usize, &mut KernelContext<'_, '_>) -> Result<OrderedStep<S, O>, KernelFailure<E>>,
{
    if identities.as_slice().is_empty() {
        let report = ExecutionReport::new(
            ExecutionPosture::Serial,
            0,
            0,
            ExecutionPhysicalReport::default(),
        )
        .with_fallback(ExecutionFallbackCause::NoWork);
        record_nested(report, false);
        return OrderedOutcome {
            state: initial,
            outputs: Vec::new(),
            completed_early: false,
            stop: None,
            boundary: None,
            report,
        };
    }
    if lease.is_none() && RunLimits::has_leased_parent() {
        return denied(initial, LeaseDenial::UnrelatedNestedLease);
    }
    // The chain itself is ordered; an independent nested pattern may still
    // spend the lease's parallel worker budget at a round boundary.
    let limits = RunLimits::for_run(lease, false);
    let memory_bytes = u64::try_from(size_of::<S>())
        .ok()
        .and_then(|bytes| bytes.checked_mul(2))
        .and_then(|bytes| bytes.checked_add(max_state_bytes.checked_mul(2)?))
        .and_then(|bytes| {
            bytes.checked_add(
                u64::try_from(identities.as_slice().len().checked_mul(size_of::<O>())?).ok()?,
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                max_output_bytes.checked_mul(u64::try_from(identities.as_slice().len()).ok()?)?,
            )
        })
        .and_then(|bytes| bytes.checked_add(scratch_bytes))
        .and_then(|bytes| bytes.checked_add(u64::try_from(size_of::<E>()).ok()?))
        .and_then(|bytes| bytes.checked_add(max_error_bytes))
        .and_then(|bytes| bytes.checked_add(input_bytes))
        .and_then(|bytes| bytes.checked_add(limits.framework_context_bytes(1, 1)?));
    let Some(memory_bytes) = memory_bytes else {
        return denied(initial, LeaseDenial::ChargedBytesOverflow);
    };
    let state_bytes = initial.additional_charged_bytes();
    if state_bytes > max_state_bytes {
        let denial = MemoryLimitDenial {
            requested: state_bytes,
            admitted: max_state_bytes,
            level: MemoryLimitLevel::Declared,
        };
        return denied(initial, LeaseDenial::MemoryExhausted(denial));
    }
    let reservation = if let Some(lease) = lease {
        match lease.reserve_entry(memory_bytes) {
            Ok(guard) => Some(guard),
            Err(denial) => return denied(initial, denial),
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
    let mut state = initial;
    let mut outputs = Vec::with_capacity(identities.as_slice().len());
    let mut stop = None;
    let mut boundary = None;
    let mut completed_early = false;
    for (index, identity) in identities.as_slice().iter().copied().enumerate() {
        let result = match limits.safe_point() {
            Err(reason) => Err(KernelFailure::Stop(reason)),
            Ok(()) => {
                let _meter_context = context.enter();
                super::panic_boundary::contain(AssertUnwindSafe(|| {
                    step(&state, index, &mut context)
                }))
                .unwrap_or(Err(KernelFailure::Panic))
            }
        };
        let result = if let Some(reason) = context.checkpoint_stop() {
            Err(KernelFailure::Stop(reason))
        } else if context.nested_stopped() && result.is_ok() {
            Err(KernelFailure::Stop(KernelStop::NestedStopped))
        } else {
            result
        };
        match result {
            Ok(step) => {
                let (next, output, done) = match step {
                    OrderedStep::Continue(next, output) => (next, output, false),
                    OrderedStep::Complete(next, output) => (next, output, true),
                };
                if next.additional_charged_bytes() > max_state_bytes
                    || output.additional_charged_bytes() > max_output_bytes
                {
                    stop = Some(BatchStop::Failure {
                        identity,
                        cause: KernelFailure::ResultCapacityExceeded,
                    });
                    boundary = Some(identity);
                    break;
                }
                state = next;
                outputs.push(output);
                if done {
                    completed_early = true;
                    break;
                }
            }
            Err(KernelFailure::Stop(KernelStop::WorkCeiling)) => {
                stop = Some(BatchStop::WorkExhausted { identity });
                boundary = Some(identity);
                break;
            }
            Err(KernelFailure::Domain(error))
                if error.additional_charged_bytes() > max_error_bytes =>
            {
                stop = Some(BatchStop::Failure {
                    identity,
                    cause: KernelFailure::ResultCapacityExceeded,
                });
                boundary = Some(identity);
                break;
            }
            Err(cause) => {
                stop = Some(BatchStop::Failure { identity, cause });
                boundary = Some(identity);
                break;
            }
        }
    }
    let (work, span) = context.cost();
    let physical_parallel = limits.worker_high_watermark() > 1;
    let report = ExecutionReport::new(
        if physical_parallel {
            ExecutionPosture::Automatic
        } else {
            ExecutionPosture::Serial
        },
        work,
        span,
        ExecutionPhysicalReport::new(
            limits.worker_high_watermark(),
            limits.peak_memory(),
            if physical_parallel { None } else { Some(0) },
            0,
            limits.discarded_work(),
        ),
    );
    record_nested(report, stop.is_some());
    drop(reservation);
    OrderedOutcome {
        state,
        outputs,
        completed_early,
        stop,
        boundary,
        report,
    }
}

fn denied<S, O, E>(state: S, denial: LeaseDenial) -> OrderedOutcome<S, O, E> {
    let report = ExecutionReport::new(
        ExecutionPosture::Serial,
        0,
        0,
        ExecutionPhysicalReport::default(),
    );
    record_nested(report, true);
    OrderedOutcome {
        state,
        outputs: Vec::new(),
        completed_early: false,
        stop: Some(BatchStop::Admission(denial)),
        boundary: None,
        report,
    }
}
