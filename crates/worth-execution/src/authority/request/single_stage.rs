//! One stage of its request: the request's own nested native stages join it.
//!
//! A native stage or scope adaptation this request opens inside its declared
//! single stage, with no ceiling of its own, narrows nothing that stage does
//! not already bound: its control, remaining work and memory lineage are the
//! stage's. It therefore charges, admits memory and stops on that stage's
//! meter. Work is charged once, cumulatively, so the request's ceiling bounds
//! the whole operation and a stop belongs to the stage it occurred in. A
//! joined measured result keeps its caller's reservation, which already
//! draws on the stage's lineage, and admits its retained and payload bytes
//! there exactly as its own scope would.
use super::{ExecutionRequest, RequestBacking};
use crate::authority::StageOrigin;
use crate::backend::{KernelFailure, KernelStop, ScopeOutcome, ScopeStop};
use crate::{ChargedBytes, ExecutionMemoryReservation, ExecutionResultMemory, LeaseDenial};
use crate::{MapKernelContext, WorkCeilingDenial};
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

impl ExecutionRequest<'_, '_> {
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

    /// This request's declared single stage, when it is the innermost one.
    pub(super) fn joined_stage<'a, 'lease>(self) -> Option<MapKernelContext<'a, 'lease>> {
        crate::backend::joined_stage(&self.stage_origin())
    }
}

/// Runs a native kernel on the joined stage and ends it as its own scope
/// would: a control stop before entry, a latched checkpoint stop or a nested
/// stop refuses it. The report carries the work this kernel charged.
pub(super) fn run_joined<R, E>(
    stage: &mut MapKernelContext<'_, '_>,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    let (work, span) = stage.cost();
    let invoked = match stage.safe_point() {
        Ok(()) => operation(stage),
        Err(stop) => Err(KernelFailure::Stop(stop)),
    };
    let invoked = match (stage.checkpoint_stop(), invoked) {
        (Some(stop), _) => Err(KernelFailure::Stop(stop)),
        (None, Ok(_)) if stage.nested_stopped() => {
            Err(KernelFailure::Stop(KernelStop::NestedStopped))
        }
        (None, invoked) => invoked,
    };
    ScopeOutcome {
        result: invoked.map_err(ScopeStop::Failure),
        report: report(stage, (work, span)),
    }
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

/// A measured result on the joined stage: the caller's reservation holds its
/// retained bytes before the kernel, its result memory during it, and its
/// value's or domain error's heap after it; any stop leaves it holding nothing.
pub(super) fn run_measured_joined<R: ChargedBytes, E: ChargedBytes>(
    stage: &mut MapKernelContext<'_, '_>,
    retained_bytes: u64,
    memory: &mut ExecutionMemoryReservation,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        &mut ExecutionResultMemory<'_>,
    ) -> Result<R, KernelFailure<E>>,
) -> ScopeOutcome<R, E> {
    if let Err(denial) = memory.resize(retained_bytes) {
        return ScopeOutcome {
            result: Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(denial))),
            report: report(stage, stage.cost()),
        };
    }
    let mut outcome = run_joined(stage, |stage| {
        kernel(
            stage,
            &mut ExecutionResultMemory::new(memory, retained_bytes),
        )
    });
    let payload = match &outcome.result {
        Ok(value) => Some(value.additional_charged_bytes()),
        Err(ScopeStop::Failure(KernelFailure::Domain(error))) => {
            Some(error.additional_charged_bytes())
        }
        Err(_) => None,
    };
    let admitted = payload.map(|payload| {
        retained_bytes
            .checked_add(payload)
            .ok_or_else(LeaseDenial::charged_bytes_overflow)
            .and_then(|total| memory.resize(total).map_err(LeaseDenial::MemoryExhausted))
            .map(|()| payload)
    });
    let kept = match admitted {
        Some(Ok(payload)) => payload,
        Some(Err(denial)) => {
            outcome.result = Err(ScopeStop::Admission(denial));
            0
        }
        None => 0,
    };
    memory
        .resize(kept)
        .expect("measured settlement only releases bytes");
    outcome
}

#[cfg(test)]
#[path = "single_stage_tests.rs"]
mod tests;
