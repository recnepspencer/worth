//! Original-request admission for variable lexical work and temporary value rows.
use super::*;
use worth_execution::ExecutionRequest;

pub(super) type Request<'a, 'request, 'authority> =
    Option<&'a ExecutionRequest<'request, 'authority>>;

pub(super) fn poll(request: Request<'_, '_, '_>) -> Result<(), PartitionContentDigestError> {
    if let Some(request) = request {
        request
            .consult()
            .map_err(|cause| PartitionContentDigestError::Request(cause.into()))?;
    }
    Ok(())
}

/// Charge the separately varying resolved-name work before hashing/comparing.
pub(super) fn charge(
    request: Request<'_, '_, '_>,
    units: u64,
) -> Result<(), PartitionContentDigestError> {
    let Some(request) = request else {
        return Ok(());
    };
    let mut outcome = None;
    let run = request.run_native_with_checkpoint(|context| {
        outcome = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (run, outcome) {
        (_, Some(Err(cause))) => Err(PartitionContentDigestError::Request(
            worth_execution::WorkCeilingDenial::Stopped(cause).into(),
        )),
        (Err(cause), _) => Err(PartitionContentDigestError::Request(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed commitment charge executes callback"),
    }
}
