//! Pure interruption observation of the caller's original execution request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomInvariantInterruption {
    Cancelled,
    DeadlineElapsed,
}

pub(super) fn check_live(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), CustomInvariantInterruption> {
    execution.check_live().map_err(|stop| match stop {
        worth_execution::MapKernelStop::Cancelled => CustomInvariantInterruption::Cancelled,
        worth_execution::MapKernelStop::DeadlineElapsed => {
            CustomInvariantInterruption::DeadlineElapsed
        }
        _ => unreachable!("the execution interruption observer never observes resource accounting"),
    })
}
