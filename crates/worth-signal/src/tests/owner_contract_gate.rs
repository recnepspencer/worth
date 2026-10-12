use crate::facade::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference,
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchForkOutcome,
    SignalBranchRetentionLease, SignalOwnerLifecycleObservation, SignalOwnerServiceCostSnapshot,
    SignalOwnerUnavailable,
};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_copy_eq<T: Copy + Eq>() {}
fn assert_public_type<T>() {}

fn lifecycle_ordinal(observation: SignalOwnerLifecycleObservation) -> u8 {
    match observation {
        SignalOwnerLifecycleObservation::Open => 0,
        SignalOwnerLifecycleObservation::Closing => 1,
        SignalOwnerLifecycleObservation::Closed => 2,
    }
}

fn operation_boundary_ordinal(
    boundary: crate::facade::branch::SignalOwnerOperationBoundaryForTest,
) -> u8 {
    use crate::facade::branch::SignalOwnerOperationBoundaryForTest;

    match boundary {
        SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission => 0,
        SignalOwnerOperationBoundaryForTest::BranchRegistryLookup => 1,
        SignalOwnerOperationBoundaryForTest::BranchRegistryReservation => 2,
        SignalOwnerOperationBoundaryForTest::ExactBasisPreflight => 3,
        SignalOwnerOperationBoundaryForTest::TargetCellAdmission => 4,
        SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement => 5,
        SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement => 6,
        SignalOwnerOperationBoundaryForTest::ForkSourceCapture => 7,
        SignalOwnerOperationBoundaryForTest::ForkDestinationInstallation => 8,
        SignalOwnerOperationBoundaryForTest::OutcomeConstruction => 9,
        SignalOwnerOperationBoundaryForTest::OwnerCloseBatch => 10,
    }
}

#[test]
fn branch_facade_preserves_existing_exports_and_adds_owner_vocabulary() {
    assert_public_type::<AdmittedSignalBranchBasis>();
    assert_public_type::<SignalBranchForkOutcome>();
    assert_public_type::<SignalBranchRetentionLease>();
    assert_send_sync::<ManagedSignalBranchReference>();
    assert_copy_eq::<ManagedSignalBranchReferenceAdmissionDenial>();
    assert_send_sync::<SignalOwnerServiceCostSnapshot>();
    assert_copy_eq::<SignalOwnerLifecycleObservation>();
    assert_copy_eq::<SignalOwnerUnavailable>();

    assert_eq!(lifecycle_ordinal(SignalOwnerLifecycleObservation::Open), 0);
    assert_eq!(
        lifecycle_ordinal(SignalOwnerLifecycleObservation::Closing),
        1
    );
    assert_eq!(
        lifecycle_ordinal(SignalOwnerLifecycleObservation::Closed),
        2
    );
}

#[test]
fn operation_control_feature_exports_the_frozen_progression_roster() {
    use crate::facade::branch::SignalOwnerOperationBoundaryForTest;

    let boundaries = [
        SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
        SignalOwnerOperationBoundaryForTest::BranchRegistryLookup,
        SignalOwnerOperationBoundaryForTest::BranchRegistryReservation,
        SignalOwnerOperationBoundaryForTest::ExactBasisPreflight,
        SignalOwnerOperationBoundaryForTest::TargetCellAdmission,
        SignalOwnerOperationBoundaryForTest::BeforeCanonicalMovement,
        SignalOwnerOperationBoundaryForTest::AfterCanonicalMovement,
        SignalOwnerOperationBoundaryForTest::ForkSourceCapture,
        SignalOwnerOperationBoundaryForTest::ForkDestinationInstallation,
        SignalOwnerOperationBoundaryForTest::OutcomeConstruction,
        SignalOwnerOperationBoundaryForTest::OwnerCloseBatch,
    ];

    let ordinals = boundaries.map(operation_boundary_ordinal);

    assert_eq!(ordinals, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
