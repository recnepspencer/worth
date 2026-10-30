use worth_store::physical_runtime::{
    FundedRecoveryWalReadFailure, PhysicalRecoveryObservationAllocationDenial,
    RecoveryDiscoveryAllocationFailure,
};

fn escape(
    failure: FundedRecoveryWalReadFailure,
) -> RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial> {
    failure.into_error()
}

fn main() {}
