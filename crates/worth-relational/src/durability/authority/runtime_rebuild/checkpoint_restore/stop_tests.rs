use super::restored_index_stop;
use crate::branch::RelationalBranchBasisDenial as Denial;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::execution::RelationalExecutionDenialCause as Cause;

#[test]
fn restored_index_interruption_is_request_control_by_value() {
    assert_eq!(
        restored_index_stop(Denial::Cancelled),
        Stop::Request(Cause::Cancelled)
    );
    assert_eq!(
        restored_index_stop(Denial::TimedOut),
        Stop::Request(Cause::DeadlineElapsed)
    );
}

#[test]
fn restored_index_bad_evidence_keeps_its_durability_failure_by_value() {
    assert_eq!(
        restored_index_stop(Denial::MalformedDescriptor),
        Stop::Durability(crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::ReplayFailure,
            "recovered unique-index basis admission failed: MalformedDescriptor"
        ))
    );
}
