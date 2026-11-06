use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use {
    crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionDenialKind as Kind,
    crate::domain_computation::vocabulary::provider_session::invariant_execution_denial::WorthQueryInvariantExecutionFailure as Failure,
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
