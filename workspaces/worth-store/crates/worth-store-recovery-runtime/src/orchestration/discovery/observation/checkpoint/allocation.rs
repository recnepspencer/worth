//! Preserve checkpoint read/parser resource boundaries without relabeling them as root slots.

use worth_store::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial, PhysicalRecoveryRejoinResidentAdmissionDenial,
    PhysicalRecoveryRejoinResidentDenial, RecoveryDiscoveryAllocationFailure,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::DiscoveryFailure;

pub(in super::super) fn window_admission_failure(
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    failure(
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

pub(super) fn map_read_failure(
    denial: RecoveryDiscoveryAllocationFailure<PhysicalRecoveryObservationAllocationDenial>,
    map_discovery: impl FnOnce(RecoveryDiscoveryFailure) -> DiscoveryFailure,
) -> DiscoveryFailure {
    match denial {
        RecoveryDiscoveryAllocationFailure::Discovery(denial) => map_discovery(denial),
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
                _ => Boundary::ReadBuffer,
            };
            failure(
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
            artifact,
            Boundary::ReadBuffer,
            requested as u64,
            Cause::ReadBufferLengthMismatch {
                requested,
                observed,
            },
        ),
    }
}

pub(super) fn parser_failure(
    requested: u64,
    cause: PhysicalRecoveryRejoinResidentDenial,
) -> DiscoveryFailure {
    failure(
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::ParserRecords,
        requested,
        Cause::Residency(cause),
    )
}

pub(super) fn parser_capacity_failure(requested: u64, actual: u64) -> DiscoveryFailure {
    failure(
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::ParserRecords,
        requested,
        Cause::AllocatorExceededReservation { requested, actual },
    )
}

pub(super) fn binding_decode_failure(
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
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::BindingDecode,
        requested,
        Cause::BindingDecode(denial),
    )
}

pub(super) fn binding_basis_failure(
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
        RecoveryDiscoveryArtifact::CurrentCheckpoint,
        Boundary::BindingBasis,
        requested,
        Cause::BindingBasis(denial),
    )
}

fn failure(
    artifact: RecoveryDiscoveryArtifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit).with_root_protocol_denials(&[
        PhysicalRecoverySourceDenial::CheckpointReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        },
    ])
}
