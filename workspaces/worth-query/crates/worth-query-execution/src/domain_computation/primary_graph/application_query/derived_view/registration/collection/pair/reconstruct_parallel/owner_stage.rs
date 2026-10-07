//! Calling-thread stages use the ordered execution pattern's meter and custody.

use super::super::Denial;
use std::num::NonZeroUsize;
use worth_execution::{
    ExecutionRounds, MapKernelContext, MapKernelFailure, MapStop, RoundsOutcome,
};

pub(super) fn run<T>(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    max_error_bytes: u64,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let stages =
        ExecutionRounds::try_new(NonZeroUsize::new(1).unwrap()).map_err(super::denial::rounds)?;
    let mut operation = Some(operation);
    let mut value = None;
    let outcome = stages.run(
        lease,
        (),
        0,
        max_error_bytes,
        0,
        |_, _, context| {
            value = Some(
                operation.take().expect("one declared owner stage")(context)
                    .map_err(MapKernelFailure::Domain)?,
            );
            Ok(())
        },
        |_, _| true,
    );
    match outcome {
        RoundsOutcome::Converged { .. } | RoundsOutcome::NotConverged { .. } => {
            Ok(value.expect("the owner stage completed"))
        }
        RoundsOutcome::Stopped { reason, .. } => Err(match reason {
            MapStop::Admission(cause) => super::denial::lease(cause),
            MapStop::WorkExhausted { .. } => Denial::WorkExhausted { root: None },
            MapStop::Failure { cause, .. } => super::denial::owner(cause),
        }),
    }
}
