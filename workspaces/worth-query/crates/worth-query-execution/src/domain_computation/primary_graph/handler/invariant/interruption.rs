//! Failed decision polls stay with preparation even when a handler catches them.
use std::cell::Cell;
use {super::HandlerInterruption, crate::domain_computation::request_execution::QueryRequestExecution};

pub(super) fn checkpoint(
    execution: &QueryRequestExecution<'_>,
    interruption: &Cell<Option<HandlerInterruption>>,
) -> Result<(), HandlerInterruption> {
    let observed = execution.interruption().map(|interruption| {
        use crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption as Stop;
        match interruption {
            Stop::Cancelled => HandlerInterruption::Cancelled,
            Stop::DeadlineExceeded => HandlerInterruption::DeadlineExceeded,
        }
    });
    let first = interruption.get().or(observed);
    interruption.set(first);
    first.map_or(Ok(()), Err)
}
