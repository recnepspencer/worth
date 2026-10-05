//! A completion phase that ran out of a limit names that limit with the
//! value recovery admitted. A failed verification names none, and neither
//! does an artifact that outgrew the ceiling of its own read.

use worth_store::physical_runtime::{
    FilesystemObservationBound, RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

use super::*;
use crate::entry::PhysicalRecoverySelectedRecordReadDenial;
use crate::orchestration::reader_limit::refused_past;

fn limits() -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 1 << 20,
        manifest_entries: 500,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 1 << 20,
        redo_targets: 4096,
        redo_bytes: 1 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 1 << 20,
        recovery_memory_bytes: 1 << 20,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 1 << 20,
        observation_bytes: 10_000,
    }
}

fn outgrown(bound: FilesystemObservationBound) -> RecoveryDiscoveryFailure {
    refused_past(bound, 4_001, 4_000)
}

#[test]
fn an_exhausted_limit_is_reported_with_the_value_recovery_admitted() {
    let limits = limits();
    assert_eq!(HistoricalFailure::Invalid.limit(&limits, 4_000), None);
    assert_eq!(
        HistoricalFailure::ManifestEntries.limit(&limits, 4_000),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
            observed: 501,
            admitted: 500,
        }),
    );
    // The reader started with 4,000 of the 10,000 bytes left and had
    // observed 4,001 at the crossing: 6,000 were spent before it.
    assert_eq!(
        HistoricalFailure::ObservationBytes(Some(4_001)).limit(&limits, 4_000),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: 10_001,
            admitted: 10_000,
        }),
    );
    assert_eq!(
        HistoricalFailure::StagingBytes(limits.staging_bytes + 7).limit(&limits, 4_000),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::StagingBytes,
            observed: limits.staging_bytes + 7,
            admitted: limits.staging_bytes,
        }),
    );
    // A crossing nobody counted is one past the limit.
    assert_eq!(
        HistoricalFailure::ObservationBytes(None).limit(&limits, 4_000),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: 10_001,
            admitted: 10_000,
        }),
    );
    assert_eq!(
        HistoricalFailure::ObservationBytes(Some(2_500))
            .limit(&limits, 2_000)
            .map(|limit| limit.observed),
        Some(10_500),
    );
}

#[test]
fn only_a_reader_out_of_its_own_budget_is_a_limit() {
    assert_eq!(
        discovery_failure(outgrown(FilesystemObservationBound::ObservationBytes)),
        HistoricalFailure::ObservationBytes(Some(4_001)),
    );
    assert_eq!(
        discovery_failure(refused_past(FilesystemObservationBound::Reads, 1, 0)),
        HistoricalFailure::ManifestEntries,
    );
    for damage in [
        // The artifact outgrew the ceiling of its own read.
        outgrown(FilesystemObservationBound::RequestedBytes),
        RecoveryDiscoveryFailure::InvalidAddress {
            artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
        },
    ] {
        assert_eq!(discovery_failure(damage), HistoricalFailure::Invalid);
    }
}

#[test]
fn a_page_observation_out_of_a_limit_stays_that_limit() {
    assert_eq!(
        HistoricalFailure::from(PageObservationFailure::ManifestEntryLimit),
        HistoricalFailure::ManifestEntries,
    );
    assert_eq!(
        HistoricalFailure::from(PageObservationFailure::ByteLimit),
        HistoricalFailure::ObservationBytes(None),
    );
    assert_eq!(
        HistoricalFailure::from(PageObservationFailure::InvalidManifest {
            target: None,
            artifact: worth_store_physical_format::RecordArtifactFile::RootManifest {
                generation: 1
            },
        }),
        HistoricalFailure::Invalid,
    );
}

#[test]
fn a_phase_starting_with_no_observation_bytes_left_has_met_that_limit() {
    assert_eq!(left_to_observe(9), Ok(9));
    assert_eq!(left_to_observe(1), Ok(1));
    assert_eq!(
        left_to_observe(0),
        Err(HistoricalFailure::ObservationBytes(None))
    );
}

#[test]
fn a_record_that_could_not_be_read_is_a_limit_only_where_its_reader_met_one() {
    use PhysicalRecoverySelectedRecordReadDenial as Denial;
    for (denial, failure) in [
        (
            Denial::ManifestEntryLimit,
            HistoricalFailure::ManifestEntries,
        ),
        (
            Denial::ManifestRead(outgrown(FilesystemObservationBound::ObservationBytes)),
            HistoricalFailure::ObservationBytes(Some(4_001)),
        ),
        (
            Denial::ManifestRead(outgrown(FilesystemObservationBound::RequestedBytes)),
            HistoricalFailure::Invalid,
        ),
        (Denial::InvalidPayload, HistoricalFailure::Invalid),
        // A refused resident allowance is read back from the allowance.
        (Denial::ResidentBoundExceeded, HistoricalFailure::Invalid),
    ] {
        assert_eq!(HistoricalFailure::from(denial), failure);
    }
}
