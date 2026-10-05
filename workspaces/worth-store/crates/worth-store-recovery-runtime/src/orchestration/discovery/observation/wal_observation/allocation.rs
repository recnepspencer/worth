//! Preserve WAL allocation causes outside integrity/source-binding outcomes.

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::{discovery_limit, refused_read, DiscoveryFailure};
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};
use worth_store::physical_runtime::{
    FundedRecoveryWalReadFailure, PhysicalRecoveryRejoinResidentAdmissionDenial,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure, RecoveryWalAllocationDenial,
    RecoveryWalDiscoveryFailureView, RecoveryWalReadFailureView,
};

pub(super) fn window_admission_failure(
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    read_failure(
        RecoveryDiscoveryArtifact::WalDirectory,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

pub(super) fn map_read_failure(
    failure: FundedRecoveryWalReadFailure,
    wal_bytes: ReadCeiling,
) -> DiscoveryFailure {
    let mut mapped = classify_read_failure(failure.diagnostic(), wal_bytes);
    mapped
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalRead { failure });
    mapped
}

fn classify_read_failure(
    view: RecoveryWalReadFailureView<'_>,
    wal_bytes: ReadCeiling,
) -> DiscoveryFailure {
    match view {
        // The listing counts the segments it found against the caller's.
        RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::EntryLimitExceeded { observed, admitted },
        ) => discovery_limit(
            PhysicalRecoveryLimitDimension::WalSegments,
            observed,
            admitted,
        ),
        RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::ByteLimitExceeded {
                observed,
                admitted,
                scope,
            },
        ) => {
            let refused = RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed,
                admitted,
                scope,
            };
            match refused_read(refused, wal_bytes, PhysicalRecoveryLimitDimension::WalBytes) {
                Err(limit) => limit,
                // No WAL file has a ceiling of its own to pass.
                Ok(OversizedArtifact) => {
                    DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation)
                }
            }
        }
        RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::Media { .. }
            | RecoveryWalDiscoveryFailureView::InvalidAddress { .. },
        ) => DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation),
        RecoveryWalReadFailureView::Allocation { .. }
        | RecoveryWalReadFailureView::BufferLengthMismatch { .. } => {
            DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit)
        }
    }
}

pub(super) fn admission_failure(cause: RecoveryWalAllocationDenial) -> DiscoveryFailure {
    let mut failure = DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit);
    failure
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalAdmissionAllocation { cause });
    failure
}

pub(super) fn inventory_failure(
    boundary: crate::entry::PhysicalRecoveryWalInventoryAllocationBoundary,
    cause: RecoveryWalAllocationDenial,
) -> DiscoveryFailure {
    let mut failure = DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit);
    failure
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalInventoryAllocation { boundary, cause });
    failure
}

fn read_failure(
    artifact: RecoveryDiscoveryArtifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    let mut failure = DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit);
    failure
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        });
    failure
}
