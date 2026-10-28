//! Actual request stops in a carried native preparation budget.
use worth_relational::facade::{
    mvcc::CompanionPreflightStop as Stop, transactions::RelationalExecutionDenialCause as Cause,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum PreflightReadRefusal {
    TopologyPending,
    SelectedSourceMismatch,
    SelectedPositionUnavailable {
        commit_id: worth_relational::facade::history::CommitId,
    },
    ForeignCell,
    RegistrationChanged,
}
pub(in crate::domain_computation::primary_graph) fn preflight_execution_cause(
    stop: Stop,
) -> Result<Cause, PreflightReadRefusal> {
    match stop {
        Stop::Execution(cause) => Ok(cause),

        Stop::WorkCounterOverflow => Ok(Cause::WorkCounterOverflow),

        Stop::PreparationMemoryCounterOverflow => Ok(Cause::ChargedBytesOverflow),
        Stop::Interrupted(event) => Ok(match event.interruption() {
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
                Cause::Cancelled
            }
            worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
                Cause::DeadlineElapsed
            }
        }),
        Stop::TopologyPending => Err(PreflightReadRefusal::TopologyPending),
        Stop::SelectedSourceMismatch => Err(PreflightReadRefusal::SelectedSourceMismatch),
        Stop::SelectedPositionUnavailable { commit_id } => {
            Err(PreflightReadRefusal::SelectedPositionUnavailable { commit_id })
        }
        Stop::ForeignCell => Err(PreflightReadRefusal::ForeignCell),
        Stop::RegistrationChanged => Err(PreflightReadRefusal::RegistrationChanged),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_ceilings_keep_the_actual_memory_and_counter_causes() {
        for (stop, cause) in [
            (
                Stop::PreparationMemoryCounterOverflow,
                Cause::ChargedBytesOverflow,
            ),
            (Stop::WorkCounterOverflow, Cause::WorkCounterOverflow),
            (
                Stop::Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
                Cause::WorkExhausted,
            ),
        ] {
            assert_eq!(preflight_execution_cause(stop), Ok(cause));
        }
        assert_eq!(
            preflight_execution_cause(Stop::TopologyPending),
            Err(PreflightReadRefusal::TopologyPending)
        );
    }
}
