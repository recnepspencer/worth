#[allow(unused_imports)] use crate::signal_owner_services::reference_context::adversarial::operation_control::snapshot_progress::*;

#[test]
fn unrelated_branch_advances_while_snapshot_capture_crosses_every_work_boundary() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        exercise_capture_boundary(boundary);
    }
}

#[test]
fn unrelated_branch_advances_while_restoration_crosses_every_work_boundary() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        exercise_restore_boundary(boundary);
    }
}
