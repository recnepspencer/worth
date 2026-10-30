#[allow(unused_imports)]
pub(crate) use {worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest, crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::exercise_advance_boundary, crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::exercise_fork_boundary, crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::exercise_fork_post_capture_boundary, crate::signal_owner_services::ordinary_context::adversarial::operation_control::independent_progress::exercise_retirement_boundary};

#[test]
fn unrelated_branch_advances_while_fork_registry_and_source_capture_are_parked() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BranchRegistryReservation,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ForkSourceCapture,
    ] {
        exercise_fork_boundary(boundary);
    }
}

#[test]
fn every_reachable_advance_boundary_preserves_unrelated_progress() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::TargetCellAdmission,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    ] {
        exercise_advance_boundary(boundary);
    }
}

#[test]
fn source_advances_while_post_capture_fork_work_is_parked() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ForkDestinationInstallation,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        exercise_fork_post_capture_boundary(boundary);
    }
}

#[test]
fn unrelated_branch_advances_while_retirement_cell_work_is_parked() {
    for boundary in [
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    ] {
        exercise_retirement_boundary(boundary);
    }
}
