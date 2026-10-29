#[allow(unused_imports)]
pub(crate) use crate::fast::signal_owner_services::adversarial::operation_control::independent_progress::*;

#[test]
fn unrelated_branch_advances_while_fork_registry_and_source_capture_are_parked() {
    for boundary in [
        SignalOwnerOperationBoundaryForTest::BranchRegistryReservation,
        SignalOwnerOperationBoundaryForTest::ForkSourceCapture,
    ] {
        exercise_fork_boundary(boundary);
    }
}

#[test]
fn every_reachable_advance_boundary_preserves_unrelated_progress() {
    for boundary in [
        SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
        SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
        SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        SignalOwnerOperationBoundaryForTest::TargetCellAdmission,
        SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    ] {
        exercise_advance_boundary(boundary);
    }
}

#[test]
fn source_advances_while_post_capture_fork_work_is_parked() {
    for boundary in [
        SignalOwnerOperationBoundaryForTest::ForkDestinationInstallation,
        SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
    ] {
        exercise_fork_post_capture_boundary(boundary);
    }
}

#[test]
fn unrelated_branch_advances_while_retirement_cell_work_is_parked() {
    for boundary in [
        SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
    ] {
        exercise_retirement_boundary(boundary);
    }
}
