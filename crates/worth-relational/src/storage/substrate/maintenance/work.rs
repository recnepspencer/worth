//! Original-request work and memory admission for a change-scoped maintainer.
use crate::execution::{request_stage, RelationalExecutionDenialCause};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceRefusal {
    MissingPredecessor,
    UnavailableChange,
    Execution(RelationalExecutionDenialCause),
}

/// Only the driver can construct a work scope, using the original request.
pub struct ChangeWork<'scope> {
    prepare: &'scope mut dyn FnMut(u64, u64) -> Result<(), MaintenanceRefusal>,
}
impl ChangeWork<'_> {
    #[cfg(test)]
    pub(super) fn oracle<T>(operation: impl FnOnce(&mut ChangeWork<'_>) -> T) -> T {
        let mut prepare = |_, _| Ok(());
        operation(&mut ChangeWork {
            prepare: &mut prepare,
        })
    }
    /// Attribute the adopter's real nested Execution charges to a named phase.
    /// Inspection observes it only inside `observe_maintenance_work`.
    pub fn run_phase<T, E: From<MaintenanceRefusal>>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &'static str,
        operation: impl FnOnce(&mut ChangeWork<'_>) -> Result<T, E>,
    ) -> Result<T, E> {
        crate::execution::phase_observation::run(
            execution,
            phase,
            || Self::run(execution, operation),
            |cause| MaintenanceRefusal::Execution(cause).into(),
        )
    }
    pub fn prepare(&mut self, units: u64, bytes: u64) -> Result<(), MaintenanceRefusal> {
        (self.prepare)(units, bytes)
    }
    pub fn run<T, E: From<MaintenanceRefusal>>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        operation: impl FnOnce(&mut ChangeWork<'_>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut first_stop = None;
        let mut failure = None;
        let mut operation = Some(operation);
        let result = request_stage::optional_stage(
            execution,
            |context, memory| {
                let mut prepare = |units, bytes| {
                    let result = request_stage::checkpoint(context, units)
                        .map_err(RelationalExecutionDenialCause::from)
                        .and_then(|()| memory.claim(bytes));
                    result.map_err(|cause| {
                        first_stop.get_or_insert(cause);
                        MaintenanceRefusal::Execution(cause)
                    })
                };
                match operation
                    .take()
                    .expect("original-request stage invokes its operation once")(
                    &mut ChangeWork {
                        prepare: &mut prepare,
                    },
                ) {
                    Ok(value) => Some(value),
                    Err(cause) => {
                        failure = Some(cause);
                        None
                    }
                }
            },
            |_| 0,
        );
        match (result, first_stop, failure) {
            (_, Some(cause), _) => Err(MaintenanceRefusal::Execution(cause).into()),
            // The SDK's post-stage checkpoint may replace a nested operation's
            // concrete denial with its generic latch. Keep the owned denial.
            (Err(RelationalExecutionDenialCause::NestedStopped), None, Some(cause))
            | (Ok(None), None, Some(cause)) => Err(cause),
            (Err(cause), None, _) => Err(MaintenanceRefusal::Execution(cause).into()),
            (Ok(Some(value)), None, _) => Ok(value),
            (Ok(None), None, None) => {
                // Proof: a completed stage invokes the callback, which stores
                // either a value or its owned denial before returning.
                unreachable!("a completed maintenance operation produces a value or a denial")
            }
        }
    }
}
