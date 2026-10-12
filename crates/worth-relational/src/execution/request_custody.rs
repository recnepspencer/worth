use super::RelationalExecutionDenialCause as Cause;

pub(crate) fn check_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    request.consult().map_err(Cause::from)
}
