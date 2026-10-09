use super::{
    WorthQueryInvariantExecutionDenialKind as Kind, WorthQueryInvariantExecutionFailure as Failure,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};

pub(in crate::domain_computation) fn check_live(
    request: Option<&WorthQueryRequestScope>,
) -> Result<(), Failure> {
    let Some(stop) = request.and_then(WorthQueryRequestScope::interruption) else {
        return Ok(());
    };
    use worth_relational::facade::mvcc::RelationalOperationInterruption;
    let stop = match stop {
        WorthQueryRequestInterruption::Cancelled => RelationalOperationInterruption::Cancelled,
        WorthQueryRequestInterruption::DeadlineExceeded => {
            RelationalOperationInterruption::TimedOut
        }
    };
    Err(Failure::new(
        Kind::RequestInterrupted(stop),
        "request interrupted invariant observation",
    ))
}
