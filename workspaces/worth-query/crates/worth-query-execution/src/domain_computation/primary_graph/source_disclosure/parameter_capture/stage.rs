//! Preserve the original request's accounted or native parameter-capture posture.
use super::{request_stopped, stopped, Denial};
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionRequest, ExecutionResultMemory,
    ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop, ScopeStop,
    WorkCeilingDenial,
};

pub(super) fn checkpoint(
    context: &mut MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), MapKernelStop> {
    if context.has_accounting() {
        context.checkpoint(work)
    } else {
        context.poll()
    }
}

pub(super) fn run_stage<Value: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    framework: u64,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut ExecutionResultMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<Denial>>,
) -> Result<(Value, Option<ExecutionMemoryReservation>), Denial> {
    let mut settled = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        settled = Some(if context.has_accounting() {
            run_accounted(execution, framework, kernel)
        } else {
            kernel(context, None)
                .map(|value| (value, None))
                .map_err(|failure| stopped(ScopeStop::Failure(failure)))
        });
    });
    match (handoff, settled) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(request_stopped(cause)),
        (Ok(_), None) => unreachable!("successful source capture invokes its stage"),
    }
}

fn run_accounted<Value: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    framework: u64,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut ExecutionResultMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<Denial>>,
) -> Result<(Value, Option<ExecutionMemoryReservation>), Denial> {
    let mut memory = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(request_stopped)?
        .map_err(|cause| request_stopped(WorkCeilingDenial::Admission(cause)))?;
    let outcome = execution.run_measured_result(
        ExecutionWorkCeiling::new(None),
        framework,
        &mut memory,
        |context, memory| kernel(context, Some(memory)),
    );
    outcome
        .result
        .map(|value| (value, Some(memory)))
        .map_err(stopped)
}
