use super::RelationalExecutionDenialCause as Cause;

pub(crate) fn check_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    request.check_live().map_err(super::denial::kernel_stop)
}
