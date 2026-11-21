//! One stage of its request: the request's own nested native stages join it.
//!
//! A native stage or scope adaptation this request opens inside its declared
//! single stage, with no ceiling of its own, narrows nothing that stage does
//! not already bound: its control, remaining work and memory lineage are the
//! stage's. It therefore charges, admits memory and stops on that stage's
//! meter. Work is charged once, cumulatively, so the request's ceiling bounds
//! the whole operation and a stop belongs to the stage it occurred in. A
//! joined call ends as its own scope would: a latched or control stop refuses
//! it before entry, a panic is contained, and a failure or a refusal it
//! raised stops the stage as a failed child stops its parent. A joined
//! measured result takes its caller's reservation over onto the stage's
//! lineage, refusing a foreign one, and admits its retained and payload bytes
//! there exactly as its own scope would. Only an accounted stage joins one.
use super::{ExecutionRequest, ExecutionResourceLease, RequestBacking};
use crate::authority::StageOrigin;
use crate::backend::{contain, KernelFailure, KernelStop, ScopeOutcome, ScopeStop};
use crate::{ChargedBytes, ExecutionMemoryReservation, ExecutionResultMemory, LeaseDenial};
use crate::{MapKernelContext, WorkCeilingDenial};
use std::panic::AssertUnwindSafe;
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

impl<'request, 'authority> ExecutionRequest<'request, 'authority> {
    /// Run `operation` as the single stage of this request. Inside it, this
    /// request's native stages and scope adaptations that declare no ceiling,
    /// and its measured results whose ceiling leaves the stage's remaining
    /// work, join the stage instead of setting up another; any other scope,
    /// and anything that scope runs, keeps its own stage.
    pub fn run_single_stage<R>(
        self,
        operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> R,
    ) -> Result<(R, ExecutionPhysicalReport), WorkCeilingDenial> {
        let origin = self.stage_origin();
        self.run_native_with_checkpoint(|stage| {
            let _single = stage.declare_single_stage(origin);
            operation(stage)
        })
    }

    fn stage_origin(self) -> StageOrigin {
        match self.form {
            RequestBacking::Leased(lease) => {
                let active = lease.current_descendant();
                active.as_ref().unwrap_or(lease).stage_origin()
            }
            RequestBacking::Serial(serial) => StageOrigin::serial(serial),
        }
    }

    pub(super) fn lease(self) -> Option<&'request ExecutionResourceLease<'authority>> {
        match self.form {
            RequestBacking::Leased(lease) => Some(lease),
            RequestBacking::Serial(_) => None,
        }
    }

    pub(super) fn active_lease(self) -> Option<ExecutionResourceLease<'authority>> {
        self.lease()
            .and_then(ExecutionResourceLease::current_descendant)
    }

    /// This request's declared single stage, when it is the innermost one.
    pub(super) fn joined_stage<'a, 'lease>(self) -> Option<MapKernelContext<'a, 'lease>> {
        crate::backend::joined_stage(&self.stage_origin())
    }
}

/// Runs a native kernel on the joined stage and ends it as its own scope
/// would. The report carries the work this kernel charged.
pub(super) fn run_joined<R, E>(
    stage: &mut MapKernelContext<'_, '_>,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    let cost = stage.cost();
    let pending = stage.refusal_pending();
    let invoked = match stage
        .checkpoint_stop()
        .map_or_else(|| stage.safe_point(), Err)
    {
        Ok(()) => {
            contain(AssertUnwindSafe(|| operation(stage))).unwrap_or(Err(KernelFailure::Panic))
        }
        Err(stop) => Err(KernelFailure::Stop(stop)),
    };
    // A stop the kernel swallowed still ends it; its payload is discarded.
    let stop = stage.checkpoint_stop().or_else(|| {
        (stage.nested_stopped() && invoked.is_ok()).then_some(KernelStop::NestedStopped)
    });
    let invoked = match (stop, invoked) {
        (Some(stop), invoked) => {
            discard(invoked);
            Err(KernelFailure::Stop(stop))
        }
        (None, invoked) => invoked,
    };
    let result = invoked.map_err(ScopeStop::Failure);
    stage.settle_joined(pending, result.as_ref().err());
    ScopeOutcome {
        result,
        report: report(stage, cost),
    }
}

/// A panicking destructor cannot replace the outcome that disposed of it.
fn discard<T>(payload: T) {
    let _ = contain(AssertUnwindSafe(|| drop(payload)));
}

/// The work and span the stage charged since the given cost.
fn report(stage: &MapKernelContext<'_, '_>, (work, span): (u64, u64)) -> ExecutionReport {
    let (after_work, after_span) = stage.cost();
    if stage.has_accounting() {
        ExecutionReport::new(
            ExecutionPosture::Serial,
            after_work - work,
            after_span - span,
            ExecutionPhysicalReport::default(),
        )
    } else {
        ExecutionReport::native(ExecutionPosture::Serial, ExecutionPhysicalReport::default())
    }
}

/// A measured result on the joined stage: the caller's reservation is taken
/// over for its retained bytes before the kernel, holds its result memory
/// during it and its value's or domain error's heap after it; any other end
/// leaves it holding nothing.
pub(super) fn run_measured_joined<R: ChargedBytes, E: ChargedBytes>(
    stage: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    retained_bytes: u64,
    memory: &mut ExecutionMemoryReservation,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        &mut ExecutionResultMemory<'_>,
    ) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    let cost = stage.cost();
    let pending = stage.refusal_pending();
    if let Err(denial) = stage.hold_joined(memory, lease, retained_bytes) {
        let result = Err(ScopeStop::Admission(denial));
        stage.settle_joined(pending, result.as_ref().err());
        return ScopeOutcome {
            result,
            report: report(stage, cost),
        };
    }
    let mut outcome = run_joined(stage, |stage| {
        kernel(
            stage,
            &mut ExecutionResultMemory::new(memory, retained_bytes),
        )
    });
    let measured = match &outcome.result {
        Ok(value) => Some(contain(AssertUnwindSafe(|| {
            value.additional_charged_bytes()
        }))),
        Err(ScopeStop::Failure(KernelFailure::Domain(error))) => {
            Some(contain(AssertUnwindSafe(|| {
                error.additional_charged_bytes()
            })))
        }
        Err(_) => None,
    };
    let pending = stage.refusal_pending();
    let refusal = match measured {
        None => None,
        Some(Err(())) => Some(ScopeStop::Failure(KernelFailure::Panic)),
        Some(Ok(payload)) => retained_bytes
            .checked_add(payload)
            .ok_or_else(LeaseDenial::charged_bytes_overflow)
            .and_then(|total| memory.resize(total).map_err(LeaseDenial::MemoryExhausted))
            .err()
            .map(ScopeStop::Admission),
    };
    if let Some(refusal) = refusal {
        discard(std::mem::replace(&mut outcome.result, Err(refusal)));
        stage.settle_joined(pending, outcome.result.as_ref().err());
    }
    let kept = match &outcome.result {
        Ok(_) | Err(ScopeStop::Failure(KernelFailure::Domain(_))) => {
            memory.bytes() - retained_bytes
        }
        Err(_) => 0,
    };
    memory
        .resize(kept)
        .expect("measured settlement only releases bytes");
    outcome
}
