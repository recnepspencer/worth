//! One original work account encloses each complete interim admission operation.
use super::*;

pub(super) fn run<T>(
    request: &ExecutionRequest<'_, '_>,
    body: impl FnOnce() -> Result<T, Refusal>,
) -> Result<T, Refusal> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| answer = Some(body()));
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the interim request produced its answer"),
        (Err(cause), Some(Err(inner)))
            if crate::execution::RelationalExecutionDenialCause::from(cause)
                == crate::execution::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(Refusal::Request(cause.into())),
    }
}

pub(super) fn run_recovery<T>(
    request: &ExecutionRequest<'_, '_>,
    body: impl FnOnce() -> Result<T, crate::durability::data::RelationalRecoveryStop>,
) -> Result<T, crate::durability::data::RelationalRecoveryStop> {
    let mut answer = None;
    let stage = request.run_native_with_checkpoint(|_| answer = Some(body()));
    match (stage, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("the interim recovery produced its answer"),
        (Err(cause), Some(Err(inner)))
            if crate::execution::RelationalExecutionDenialCause::from(cause)
                == crate::execution::RelationalExecutionDenialCause::NestedStopped =>
        {
            Err(inner)
        }
        (Err(cause), _) => Err(crate::durability::data::RelationalRecoveryStop::Request(
            cause.into(),
        )),
    }
}
