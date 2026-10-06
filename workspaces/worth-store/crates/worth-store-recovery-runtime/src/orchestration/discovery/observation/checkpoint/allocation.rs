//! Preserve checkpoint read/parser resource boundaries without relabeling them as root slots.

use worth_store::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial, PhysicalRecoveryRejoinResidentAdmissionDenial,
    PhysicalRecoveryRejoinResidentDenial, RecoveryDiscoveryAllocationFailure,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::source_memory::stopped;
use crate::orchestration::discovery::DiscoveryFailure;
use crate::orchestration::reader_limit::OversizedArtifact;

pub(in super::super) fn window_admission_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    failure(
        limits,
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

pub(super) fn refused(
    limits: &PhysicalRecoveryLimitDeclaration,
    denial: RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    refused_read: impl FnOnce(RecoveryDiscoveryFailure) -> Result<OversizedArtifact, DiscoveryFailure>,
) -> Result<OversizedArtifact, DiscoveryFailure> {
    Err(match denial {
        RecoveryDiscoveryAllocationFailure::Discovery(denial) => return refused_read(denial),
        RecoveryDiscoveryAllocationFailure::Allocation {
            artifact,
            requested,
            cause,
            ..
        } => {
            let boundary = match &cause {
                PhysicalRecoveryObservationAllocationDenial::PathResidency { boundary, .. } => {
                    Boundary::QualifiedPath(*boundary)
                }
                PhysicalRecoveryObservationAllocationDenial::StoreMismatch
                | PhysicalRecoveryObservationAllocationDenial::WalObservationUnavailable(_)
                | PhysicalRecoveryObservationAllocationDenial::Residency(_)
                | PhysicalRecoveryObservationAllocationDenial::ListingResidency { .. }
                | PhysicalRecoveryObservationAllocationDenial::UnqualifiedListingStorage
                | PhysicalRecoveryObservationAllocationDenial::UnqualifiedPathStorage
                | PhysicalRecoveryObservationAllocationDenial::AllocatorExceededReservation {
                    ..
                } => Boundary::ReadBuffer,
            };
            failure(
                limits,
                artifact,
                boundary,
                requested as u64,
                Cause::Observation(cause),
            )
        }
        RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact,
            requested,
            observed,
            ..
        } => failure(
            limits,
            artifact,
            Boundary::ReadBuffer,
            requested as u64,
            Cause::ReadBufferLengthMismatch {
                requested,
                observed,
            },
        ),
    })
}

pub(super) fn parser_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    requested: u64,
    cause: PhysicalRecoveryRejoinResidentDenial,
) -> DiscoveryFailure {
    failure(
        limits,
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::ParserRecords,
        requested,
        Cause::Residency(cause),
    )
}

pub(super) fn parser_capacity_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    requested: u64,
    actual: u64,
) -> DiscoveryFailure {
    failure(
        limits,
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::ParserRecords,
        requested,
        Cause::AllocatorExceededReservation { requested, actual },
    )
}

pub(super) fn binding_decode_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    denial: worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial,
) -> DiscoveryFailure {
    use worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial as Denial;
    let requested = match &denial {
        Denial::Backing { requested, .. }
        | Denial::AllocatorExceededReservation { requested, .. } => *requested,
        Denial::SizeOverflow
        | Denial::StoreMismatch
        | Denial::PoolMismatch
        | Denial::BackingMismatch => 0,
    };
    failure(
        limits,
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::BindingDecode,
        requested,
        Cause::BindingDecode(denial),
    )
}

pub(super) fn binding_basis_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    denial: worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial,
) -> DiscoveryFailure {
    use worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial as Denial;
    let requested = match &denial {
        Denial::Backing { requested, .. }
        | Denial::AllocatorExceededReservation { requested, .. } => *requested,
        Denial::SizeOverflow
        | Denial::StoreMismatch
        | Denial::PoolMismatch
        | Denial::BackingMismatch => 0,
    };
    failure(
        limits,
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::BindingBasis,
        requested,
        Cause::BindingBasis(denial),
    )
}

fn failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    artifact: RecoveryDiscoveryArtifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    stopped(limits, &cause).with_root_protocol_denials(&[
        PhysicalRecoverySourceDenial::CheckpointReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        },
    ])
}
