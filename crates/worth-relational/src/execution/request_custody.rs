use super::RelationalExecutionDenialCause as Cause;

pub(crate) fn check_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    request.check_live().map_err(|stop| match stop {
        worth_execution::MapKernelStop::Cancelled => Cause::Cancelled,
        worth_execution::MapKernelStop::DeadlineElapsed => Cause::DeadlineElapsed,
        worth_execution::MapKernelStop::WorkCounterOverflow => Cause::WorkCounterOverflow,
        worth_execution::MapKernelStop::WorkCeiling => Cause::WorkExhausted,
        worth_execution::MapKernelStop::NestedStopped => Cause::NestedStopped,
    })
}
