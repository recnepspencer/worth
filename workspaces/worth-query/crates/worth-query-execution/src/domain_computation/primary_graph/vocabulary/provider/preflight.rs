//! Moved whole from `domain_computation::primary_graph::provider::relational_execution_denial::preflight`: the items below and every impl they own.
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum PreflightReadRefusal {
    TopologyPending,
    SelectedSourceMismatch,
    SelectedPositionUnavailable {
        commit_id: worth_relational::facade::history::CommitId,
    },
    ForeignCell,
    RegistrationChanged,
    ManagedSourceNotPrepared,
    ManagedSourceChanged,
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
        Stop::ManagedSourceNotPrepared => Err(PreflightReadRefusal::ManagedSourceNotPrepared),
        Stop::ManagedSourceChanged => Err(PreflightReadRefusal::ManagedSourceChanged),
    }
}
