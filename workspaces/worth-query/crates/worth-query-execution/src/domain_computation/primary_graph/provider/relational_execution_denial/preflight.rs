//! Actual request stops in a carried native preparation budget.
use worth_relational::facade::{
    mvcc::CompanionPreflightStop as Stop, transactions::RelationalExecutionDenialCause as Cause,
};

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
