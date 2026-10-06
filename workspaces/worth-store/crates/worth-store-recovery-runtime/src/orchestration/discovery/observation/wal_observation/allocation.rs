//! Preserve WAL allocation causes outside integrity/source-binding outcomes.

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension,
    PhysicalRecoverySourceDenial, PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::source_memory::{source_allocation, stopped};
use crate::orchestration::discovery::{refused_beside, refused_read, DiscoveryFailure};
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use worth_store::physical_runtime::{
    FilesystemObservationBound, FundedRecoveryWalReadFailure,
    PhysicalRecoveryRejoinResidentAdmissionDenial, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryFailure, RecoveryWalAllocationDenial, RecoveryWalDiscoveryFailureView,
    RecoveryWalReadFailureView,
};

pub(super) fn window_admission_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    read_failure(
        limits,
        RecoveryDiscoveryArtifact::WalDirectory,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

pub(super) fn map_read_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    failure: FundedRecoveryWalReadFailure,
    wal_bytes: ReadCeiling,
) -> DiscoveryFailure {
    let mut mapped = classify_read_failure(limits, failure.diagnostic(), wal_bytes);
    mapped
        .source_denials
        .push(PhysicalRecoverySourceDenial::WalRead { failure });
    mapped
}

fn classify_read_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    view: RecoveryWalReadFailureView<'_>,
    wal_bytes: ReadCeiling,
) -> DiscoveryFailure {
    match view {
        // The listing counts the segments it found against those it was
        // handed of recovery's.
        RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past))
            if past.dimension() == FilesystemObservationBound::Entries =>
        {
            refused_beside(
                RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::WalSegments),
                past.observed(),
                past.admitted(),
                PhysicalRecoveryBlockKind::MediaObservation,
            )
        }
        RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past)) => {
            let refused = RecoveryDiscoveryFailure::Limit(past);
            let wal = PhysicalRecoveryLimitDimension::WalBytes;
            match refused_read(refused, wal_bytes, limits, wal) {
                Err(limit) => limit,
                // No WAL file has a ceiling of its own to pass.
                Ok(OversizedArtifact) => {
                    DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation)
                }
            }
        }
        // A count past every count is no limit: the media observation stops.
        // T2b: a WAL file past its declared length is that file's damage.
        RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::PastCeiling { .. }
            | RecoveryWalDiscoveryFailureView::Media { .. }
            | RecoveryWalDiscoveryFailureView::InvalidAddress { .. }
            | RecoveryWalDiscoveryFailureView::CountOverflow(_),
        ) => DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation),
        RecoveryWalReadFailureView::Allocation { cause, .. } => stopped(limits, cause),
        // The backend returned a length other than the one it read into.
        RecoveryWalReadFailureView::BufferLengthMismatch { .. } => {
            DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation)
        }
    }
}

pub(super) fn admission_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: RecoveryWalAllocationDenial,
) -> DiscoveryFailure {
    source_allocation(limits, cause, |cause| {
        PhysicalRecoverySourceDenial::WalAdmissionAllocation { cause }
    })
}

pub(super) fn inventory_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    boundary: crate::entry::PhysicalRecoveryWalInventoryAllocationBoundary,
    cause: RecoveryWalAllocationDenial,
) -> DiscoveryFailure {
    source_allocation(limits, cause, |cause| {
        PhysicalRecoverySourceDenial::WalInventoryAllocation { boundary, cause }
    })
}

fn read_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    artifact: RecoveryDiscoveryArtifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    source_allocation(limits, cause, |cause| {
        PhysicalRecoverySourceDenial::WalReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{PhysicalRecoveryBlockCause, PhysicalRecoveryLimitFailure};
    use crate::orchestration::recovery_budget::recovery_limit_for_test;
    use worth_store::physical_runtime::{
        filesystem_observation_limit_for_test, RecoveryDiscoveryCount,
    };

    /// Recovery declared `wal_segments`, 8 WAL bytes and 9 observation
    /// bytes, and one of every other count.
    fn declared(wal_segments: u64) -> PhysicalRecoveryLimitDeclaration {
        let mut values = [1; 19];
        (values[4], values[6], values[18]) = (wal_segments, 8, 9);
        PhysicalRecoveryLimitDeclaration::from_values_for_test(values)
    }

    /// The limit a WAL read's refusal of `observed` past `admitted` names.
    fn classified(
        limits: &PhysicalRecoveryLimitDeclaration,
        bound: FilesystemObservationBound,
        observed: u64,
        admitted: u64,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        let past = filesystem_observation_limit_for_test(bound, observed, admitted);
        let view =
            RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past));
        let failure = classify_read_failure(limits, view, ReadCeiling::of_budget_alone(8, 8));
        assert_eq!(
            failure.cause.phase(),
            PhysicalRecoveryBlockKind::MediaObservation
        );
        failure.cause.limit()
    }

    fn limit(
        dimension: PhysicalRecoveryLimitDimension,
        observed: u64,
        admitted: u64,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        Some(recovery_limit_for_test(dimension, observed, admitted).into())
    }

    #[test]
    fn each_wal_refusal_names_its_own_limit_with_both_counts() {
        use FilesystemObservationBound::{Entries, ObservationBytes as Observed, RequestedBytes};
        use PhysicalRecoveryLimitDimension::{ObservationBytes, WalBytes, WalSegments};
        let limits = declared(2);
        assert_eq!(classified(&limits, Entries, 3, 2), limit(WalSegments, 3, 2));
        assert_eq!(
            classified(&limits, RequestedBytes, 10, 8),
            limit(WalBytes, 10, 8)
        );
        assert_eq!(
            classified(&limits, Observed, 10, 9),
            limit(ObservationBytes, 10, 9)
        );
        // A listing handed 2 of 5 segments found 3: recovery held 3 beside
        // it. A listing handed more than recovery declared is no part of it.
        assert_eq!(
            classified(&declared(5), Entries, 3, 2),
            limit(WalSegments, 6, 5)
        );
        assert_eq!(classified(&declared(1), Entries, 3, 2), None);
    }

    #[test]
    fn a_wal_count_past_every_count_stops_the_observation_as_no_limit() {
        let view = RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::CountOverflow(RecoveryDiscoveryCount::WalBytesRead),
        );
        let failure = classify_read_failure(&declared(2), view, ReadCeiling::of_budget_alone(8, 8));
        assert_eq!(
            failure.cause,
            PhysicalRecoveryBlockCause::Damage(PhysicalRecoveryBlockKind::MediaObservation)
        );
    }
}
