//! Preserve WAL allocation causes outside integrity/source-binding outcomes.

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::{discovery_limit, DiscoveryFailure};
use worth_store::physical_runtime::{
    FundedRecoveryWalReadFailure, PhysicalRecoveryRejoinResidentAdmissionDenial,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryByteLimitScope, RecoveryWalAllocationDenial,
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

pub(super) fn map_read_failure(failure: FundedRecoveryWalReadFailure) -> DiscoveryFailure {
    let mut mapped = classify_read_failure(failure.diagnostic());
    mapped
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalRead { failure });
    mapped
}

fn classify_read_failure(view: RecoveryWalReadFailureView<'_>) -> DiscoveryFailure {
    match view {
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
        ) => discovery_limit(
            match scope {
                RecoveryDiscoveryByteLimitScope::Observation => {
                    PhysicalRecoveryLimitDimension::ObservationBytes
                }
                RecoveryDiscoveryByteLimitScope::Requested => {
                    PhysicalRecoveryLimitDimension::WalBytes
                }
            },
            observed,
            admitted,
        ),
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
