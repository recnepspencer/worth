//! A completion phase that ran out of a limit names that limit with the
//! value recovery admitted. A failed verification names none, and neither
//! does an artifact that outgrew the ceiling of its own read.

use worth_store::physical_runtime::{
    FilesystemObservationBound, RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

use super::*;
use crate::entry::PhysicalRecoverySelectedRecordReadDenial;
use crate::orchestration::reader_limit::refused_past;
use crate::orchestration::recovery_budget::recovery_limit_for_test;
use PhysicalRecoveryLimitDimension::{ManifestEntries, ObservationBytes, StagingBytes};

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

/// A reader handed 4,000 bytes that observed 4,001.
fn reader_out_of_bytes() -> HistoricalFailure {
    let bytes = ReaderBytes::of(&outgrown(FilesystemObservationBound::ObservationBytes));
    HistoricalFailure::Limit(PageLimit::Reader(bytes.expect("the reader's own bytes")))
}

fn named(
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> Option<PhysicalRecoveryLimitFailure> {
    Some(recovery_limit_for_test(dimension, observed, admitted).into())
}

#[test]
fn an_exhausted_limit_is_reported_with_recovery_s_own_counts() {
    let limits = limits();
    let mut budget = ManifestEntryBudget::new(limits.manifest_entries, 497);
    assert_eq!(HistoricalFailure::Invalid.limit(&limits, &budget), None);
    assert_eq!(
        HistoricalFailure::CountOverflow.limit(&limits, &budget),
        None
    );
    // A budget that refused nothing has no counts to report.
    assert_eq!(
        HistoricalFailure::ManifestEntries.limit(&limits, &budget),
        None
    );
    assert!(budget.charge(5).is_err());
    assert_eq!(
        HistoricalFailure::ManifestEntries.limit(&limits, &budget),
        named(ManifestEntries, 502, 500),
    );
    // The reader started with 4,000 of the 10,000 bytes left and had
    // observed 4,001 at the crossing: 6,000 were spent before it.
    assert_eq!(
        reader_out_of_bytes().limit(&limits, &budget),
        named(ObservationBytes, 10_001, 10_000),
    );
    let staging =
        recovery_limit_for_test(StagingBytes, limits.staging_bytes + 7, limits.staging_bytes);
    assert_eq!(
        HistoricalFailure::Limit(PageLimit::Recovery(staging)).limit(&limits, &budget),
        Some(staging.into()),
    );
    // T2: a phase with nothing left to observe is short of the least read.
    assert_eq!(
        HistoricalFailure::ObservationSpent.limit(&limits, &budget),
        named(ObservationBytes, 10_001, 10_000),
    );
}

#[test]
fn only_a_reader_out_of_its_own_bytes_is_a_limit() {
    assert_eq!(
        discovery_failure(outgrown(FilesystemObservationBound::ObservationBytes)),
        reader_out_of_bytes(),
    );
    for damage in [
        // A phase's reader counts no reads.
        refused_past(FilesystemObservationBound::Reads, 2, 1),
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
    let limit = PageLimit::Recovery(recovery_limit_for_test(ManifestEntries, 9, 8));
    assert_eq!(
        HistoricalFailure::from(PageObservationFailure::Limit(limit)),
        HistoricalFailure::Limit(limit),
    );
    assert_eq!(
        HistoricalFailure::from(PageObservationFailure::CountOverflow),
        HistoricalFailure::CountOverflow,
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
    assert_eq!(left_to_observe(0), Err(HistoricalFailure::ObservationSpent));
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
            reader_out_of_bytes(),
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
