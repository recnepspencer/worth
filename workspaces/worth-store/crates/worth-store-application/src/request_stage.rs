//! One original request scope for a complete pre-effect preparation stage.
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

pub(crate) fn prepare<T, E>(
    execution: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce() -> Result<T, E>,
    stopped: impl FnOnce(WorkCeilingDenial) -> E,
) -> Result<T, E> {
    match execution.prepare_native(|_| operation()) {
        Ok(result) => result,
        Err(cause) => Err(stopped(cause)),
    }
}
