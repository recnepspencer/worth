use super::RelationalExecutionDenialCause as Cause;

pub(crate) fn check_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    request.consult().map_err(|denial| match denial {
        worth_execution::WorkCeilingDenial::Admission(denial) => {
            super::denial::lease_denial(denial)
        }
        worth_execution::WorkCeilingDenial::Stopped(stop) => super::denial::kernel_stop(stop),
        worth_execution::WorkCeilingDenial::Panicked => Cause::WorkerFailed,
    })
}
