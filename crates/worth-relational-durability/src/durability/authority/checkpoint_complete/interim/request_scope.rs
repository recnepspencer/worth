//! One original work account encloses each complete interim admission operation.
use super::*;
use worth_relational_contracts::internal_port as contracts;

pub fn run<T>(
    request: &ExecutionRequest<'_, '_>,
    body: impl FnOnce() -> Result<T, Refusal>,
) -> Result<T, Refusal> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| answer = Some(body()));
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the interim request produced its answer"),
        (Err(cause), Some(Err(inner)))
            if contracts::execution::denial_cause::RelationalExecutionDenialCause::from(cause)
                == contracts::execution::denial_cause::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(Refusal::Request(cause.into())),
    }
}

pub fn run_recovery<T>(
    request: &ExecutionRequest<'_, '_>,
    body: impl FnOnce() -> Result<T, contracts::durability::data::recovery_stop::RelationalRecoveryStop>,
) -> Result<T, contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| answer = Some(body()));
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the interim recovery produced its answer"),
        (Err(cause), Some(Err(inner)))
            if contracts::execution::denial_cause::RelationalExecutionDenialCause::from(cause)
                == contracts::execution::denial_cause::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(
            cause.into(),
        )),
    }
}
