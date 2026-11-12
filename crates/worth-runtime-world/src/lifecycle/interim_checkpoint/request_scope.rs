//! Whole World capture/readmission uses one original control and work account.
use super::*;

pub(super) fn run<T>(
    request: &ExecutionRequest<'_, '_>,
    body: impl FnOnce() -> Result<T, Stop>,
) -> Result<T, Stop> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| answer = Some(body()));
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the World interim request produced its answer"),
        (Err(cause), Some(Err(inner)))
            if worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause)
                == worth_relational::facade::transactions::RelationalExecutionDenialCause::NestedStopped => Err(inner),
        (Err(cause), _) => Err(Stop::Request(cause)),
    }
}
