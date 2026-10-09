use crate::data::error::SignalError;
use worth_execution::{ExecutionRequest, MapKernelContext};

/// Meter the caller's request without changing the conditional port's unwind
/// contract. The conditional owner has already rolled back and released its
/// slot before an unwind reaches this boundary.
pub(super) fn run_conditional_request<R>(
    request: ExecutionRequest<'_, '_>,
    execute: impl FnOnce(&mut MapKernelContext<'_, '_>) -> R,
) -> Result<R, SignalError> {
    let mut execute = Some(execute);
    let mut unwind = None;
    let result = crate::logic::planner::run_signal_preparation_request(request, |work, _, _| {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute.take().expect("one conditional request scan step")(work)
        })) {
            Ok(result) => Ok(Some(result)),
            Err(payload) => {
                unwind = Some(payload);
                Ok(None)
            }
        }
    });
    // Close the execution owner's accounting first, then retain the original
    // public-port behavior and panic payload.
    if let Some(payload) = unwind {
        std::panic::resume_unwind(payload);
    }
    Ok(result?
        .0
        .expect("a completed conditional request has its result"))
}
