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
    RetainedCompanionCapacityExhausted {
        requested: u64,
        retained: u64,
        maximum: u64,
    },
}
pub(in crate::domain_computation::primary_graph) fn preflight_execution_cause(
    stop: Stop,
) -> Result<Cause, PreflightReadRefusal> {
    match stop {
        Stop::Execution(cause) => Ok(cause),
        Stop::WorkExhausted { .. } | Stop::WorkExhaustedWithoutRequirement { .. } => {
            Ok(Cause::WorkExhausted)
        }
        Stop::WorkCounterOverflow => Ok(Cause::WorkCounterOverflow),
        Stop::PreparationMemoryExhausted { required, maximum } => {
            Ok(Cause::DeclaredMemoryExhausted {
                requested: required,
                admitted: maximum,
            })
        }
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
        Stop::RetainedCompanionCapacityExhausted {
            requested,
            retained,
            maximum,
        } => Err(PreflightReadRefusal::RetainedCompanionCapacityExhausted {
            requested,
            retained,
            maximum,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_ceilings_keep_the_actual_memory_and_counter_causes() {
        for (stop, cause) in [
            (
                Stop::PreparationMemoryExhausted {
                    required: 43,
                    maximum: 17,
                },
                Cause::DeclaredMemoryExhausted {
                    requested: 43,
                    admitted: 17,
                },
            ),
            (
                Stop::PreparationMemoryCounterOverflow,
                Cause::ChargedBytesOverflow,
            ),
            (Stop::WorkCounterOverflow, Cause::WorkCounterOverflow),
            (
                Stop::WorkExhausted {
                    required: 43,
                    maximum: 17,
                },
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
