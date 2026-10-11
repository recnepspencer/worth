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
