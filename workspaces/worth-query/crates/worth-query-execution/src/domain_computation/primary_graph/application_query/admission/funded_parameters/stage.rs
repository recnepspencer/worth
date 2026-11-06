//! Parameter-specific translation of the sole original-request capture owner.
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionRequest, ExecutionResultMemory,
    MapKernelContext, MapKernelFailure,
};
use {
    super::request_stopped, super::stopped,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial as Denial,
};
use {
    crate::domain_computation::primary_graph::admitted_capture,
    crate::domain_computation::vocabulary::primary_graph::admitted_capture::CaptureStop,
};

pub(super) use crate::domain_computation::vocabulary::primary_graph::admitted_capture::checkpoint;

pub(super) fn run_stage<Value: ChargedBytes>(
    execution: &ExecutionRequest<'_, '_>,
    framework: u64,
    kernel: impl FnOnce(
        &mut MapKernelContext<'_, '_>,
        Option<&mut ExecutionResultMemory<'_>>,
    ) -> Result<Value, MapKernelFailure<Denial>>,
) -> Result<(Value, Option<ExecutionMemoryReservation>), Denial> {
    admitted_capture::run_stage(execution, framework, kernel).map_err(|cause| match cause {
        CaptureStop::Request(cause) => request_stopped(cause),
        CaptureStop::Kernel(cause) => stopped(cause),
    })
}
