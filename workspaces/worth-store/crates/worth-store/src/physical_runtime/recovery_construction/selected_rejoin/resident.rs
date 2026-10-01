//! Independent-rejoin translation at the qualified recovery discovery boundary.

pub(in crate::physical_runtime) use crate::physical_runtime::recovery_residency::{
    PhysicalRecoveryRejoinResidentDenial, StoreRejoinResidentLedger,
};

pub(super) fn discovery_allocation_denial(
    failure: worth_store_physical_backend::RecoveryDiscoveryAllocationFailure<
        PhysicalRecoveryRejoinResidentDenial,
    >,
) -> super::SelectedMediaRejoinDenial {
    use super::SelectedMediaRejoinDenial as Denial;
    use worth_store_physical_backend::RecoveryDiscoveryAllocationFailure as Failure;
    match failure {
        Failure::Discovery(failure) => Denial::Discovery(failure),
        Failure::Allocation {
            artifact,
            offset,
            requested,
            cause,
        } => Denial::ResidentRead {
            artifact,
            offset,
            requested,
            cause,
        },
        Failure::BufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        } => Denial::ReadBufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        },
    }
}
