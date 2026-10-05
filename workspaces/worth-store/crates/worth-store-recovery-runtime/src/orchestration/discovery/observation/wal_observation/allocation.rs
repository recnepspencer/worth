//! Preserve WAL allocation causes outside integrity/source-binding outcomes.

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::{discovery_limit, refused_read, DiscoveryFailure};
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};
use worth_store::physical_runtime::{
    FilesystemObservationBound, FundedRecoveryWalReadFailure,
    PhysicalRecoveryRejoinResidentAdmissionDenial, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryFailure, RecoveryWalAllocationDenial, RecoveryWalDiscoveryFailureView,
    RecoveryWalReadFailureView,
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
        RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past))
            if past.dimension() == FilesystemObservationBound::Entries =>
        {
            discovery_limit(
                PhysicalRecoveryLimitDimension::WalSegments,
                past.observed(),
                past.admitted(),
            )
        }
        RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past)) => {
            let refused = RecoveryDiscoveryFailure::Limit(past);
            match refused_read(refused, wal_bytes, PhysicalRecoveryLimitDimension::WalBytes) {
                Err(limit) => limit,
                // No WAL file has a ceiling of its own to pass.
                Ok(OversizedArtifact) => {
                    DiscoveryFailure::from(PhysicalRecoveryBlockKind::MediaObservation)
                }
            }
        }
        // A count past every count is no limit: the media observation stops.
        RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::Media { .. }
            | RecoveryWalDiscoveryFailureView::InvalidAddress { .. }
            | RecoveryWalDiscoveryFailureView::CountOverflow(_),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::PhysicalRecoveryLimitFailure;
    use worth_store::physical_runtime::{
        filesystem_observation_limit_for_test, RecoveryDiscoveryCount,
    };

    /// The limit a WAL read's refusal of `observed` past `admitted` names.
    fn classified(
        bound: FilesystemObservationBound,
        observed: u64,
        admitted: u64,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        let past = filesystem_observation_limit_for_test(bound, observed, admitted);
        let view =
            RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Limit(past));
        classify_read_failure(view, ReadCeiling::of_budget_alone(8, 8)).limit
    }

    fn limit(
        dimension: PhysicalRecoveryLimitDimension,
        observed: u64,
        admitted: u64,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        Some(PhysicalRecoveryLimitFailure {
            dimension,
            observed,
            admitted,
        })
    }

    #[test]
    fn each_wal_refusal_names_its_own_limit_with_both_counts() {
        use FilesystemObservationBound::{Entries, ObservationBytes as Observed, RequestedBytes};
        use PhysicalRecoveryLimitDimension::{ObservationBytes, WalBytes, WalSegments};
        assert_eq!(classified(Entries, 3, 2), limit(WalSegments, 3, 2));
        assert_eq!(classified(RequestedBytes, 10, 8), limit(WalBytes, 10, 8));
        assert_eq!(classified(Observed, 10, 9), limit(ObservationBytes, 10, 9));
    }

    #[test]
    fn a_wal_count_past_every_count_stops_the_observation_as_no_limit() {
        let view = RecoveryWalReadFailureView::Discovery(
            RecoveryWalDiscoveryFailureView::CountOverflow(RecoveryDiscoveryCount::WalBytesRead),
        );
        let failure = classify_read_failure(view, ReadCeiling::of_budget_alone(8, 8));
        assert_eq!(failure.limit, None);
        assert_eq!(failure.kind, PhysicalRecoveryBlockKind::MediaObservation);
    }
}
