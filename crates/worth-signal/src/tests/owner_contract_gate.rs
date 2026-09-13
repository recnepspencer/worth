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

#[cfg(feature = "test-operation-control")]
fn operation_boundary_ordinal(boundary: crate::facade::branch::SignalOwnerOperationBoundary) -> u8 {
    use crate::facade::branch::SignalOwnerOperationBoundary;

    match boundary {
        SignalOwnerOperationBoundary::OwnerLifecycleAdmission => 0,
        SignalOwnerOperationBoundary::BranchRegistryLookup => 1,
        SignalOwnerOperationBoundary::BranchRegistryReservation => 2,
        SignalOwnerOperationBoundary::ExactBasisPreflight => 3,
        SignalOwnerOperationBoundary::TargetCellAdmission => 4,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement => 5,
        SignalOwnerOperationBoundary::AfterCanonicalMovement => 6,
        SignalOwnerOperationBoundary::ForkSourceCapture => 7,
        SignalOwnerOperationBoundary::ForkDestinationInstallation => 8,
        SignalOwnerOperationBoundary::OutcomeConstruction => 9,
        SignalOwnerOperationBoundary::OwnerCloseBatch => 10,
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

#[cfg(feature = "test-operation-control")]
#[test]
fn operation_control_feature_exports_the_frozen_progression_roster() {
    use crate::facade::branch::SignalOwnerOperationBoundary;

    let boundaries = [
        SignalOwnerOperationBoundary::OwnerLifecycleAdmission,
        SignalOwnerOperationBoundary::BranchRegistryLookup,
        SignalOwnerOperationBoundary::BranchRegistryReservation,
        SignalOwnerOperationBoundary::ExactBasisPreflight,
        SignalOwnerOperationBoundary::TargetCellAdmission,
        SignalOwnerOperationBoundary::BeforeCanonicalMovement,
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        SignalOwnerOperationBoundary::ForkSourceCapture,
        SignalOwnerOperationBoundary::ForkDestinationInstallation,
        SignalOwnerOperationBoundary::OutcomeConstruction,
        SignalOwnerOperationBoundary::OwnerCloseBatch,
    ];

    let ordinals = boundaries.map(operation_boundary_ordinal);

    assert_eq!(ordinals, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
