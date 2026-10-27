//! One original-request capture posture for opaque authored projections.
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionRequest, ExecutionResultMemory,
    ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop, ScopeStop,
    WorkCeilingDenial,
};

pub(super) enum CaptureStop<E> {
    Request(WorkCeilingDenial),
    Kernel(ScopeStop<E>),
}

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

pub(super) fn run_stage<Value: ChargedBytes, E: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    framework: u64,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut ExecutionResultMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<E>>,
) -> Result<(Value, Option<ExecutionMemoryReservation>), CaptureStop<E>> {
    let mut settled = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        settled = Some(if context.has_accounting() {
            run_accounted(execution, framework, kernel)
        } else {
            kernel(context, None)
                .map(|value| (value, None))
                .map_err(|failure| CaptureStop::Kernel(ScopeStop::Failure(failure)))
        });
    });
    match (handoff, settled) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(CaptureStop::Request(cause)),
        (Ok(_), None) => unreachable!("successful original request invokes its capture stage"),
    }
}

fn run_accounted<Value: ChargedBytes, E: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    framework: u64,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut ExecutionResultMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<E>>,
) -> Result<(Value, Option<ExecutionMemoryReservation>), CaptureStop<E>> {
    let mut memory = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(CaptureStop::Request)?
        .map_err(|cause| CaptureStop::Request(WorkCeilingDenial::Admission(cause)))?;
    let outcome = execution.run_measured_result(
        ExecutionWorkCeiling::new(None),
        framework,
        &mut memory,
        |context, memory| kernel(context, Some(memory)),
    );
    outcome
        .result
        .map(|value| (value, Some(memory)))
        .map_err(CaptureStop::Kernel)
}
