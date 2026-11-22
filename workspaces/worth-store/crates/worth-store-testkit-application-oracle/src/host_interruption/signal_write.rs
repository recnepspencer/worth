//! Park the installed Signal owner before its next durable write.

/// The next durable Signal write parks before its candidate commits; any
/// earlier owner effect of the same original is already durable.
pub fn pause_before_signal_durable_write(
    control: &worth_signal::facade::branch::SignalOwnerOperationControlForTest,
) -> worth_signal::facade::branch::SignalOwnerOperationPauseForTest {
    control.arm_pause_once_for_test(
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::DurableCandidateCellWait,
    )
}
