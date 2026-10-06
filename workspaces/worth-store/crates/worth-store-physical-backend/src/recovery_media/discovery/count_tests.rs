//! Each count an observation keeps names itself when it goes past every
//! count, and refuses as its bound's limit when it goes past its ceiling.

use super::{
    ArtifactTreeFailure, ArtifactTreeFailureKind, FilesystemObservation,
    FilesystemObservationBound, RecoveryDiscoveryCount, RecoveryDiscoveryCounters,
    RecoveryDiscoveryFailure,
};

/// An observation with no media behind it, `entries` and `bytes` admitted
/// and `reads` and `spent` of them already taken.
fn observation(entries: u64, reads: u64, bytes: u64, spent: u64) -> FilesystemObservation<()> {
    FilesystemObservation {
        parts: (),
        remaining_entries: entries - reads,
        maximum_entries: entries,
        remaining_bytes: bytes - spent,
        maximum_bytes: bytes,
        discovery_incarnation: 0,
        wal_observations_issued: 0,
        counters: RecoveryDiscoveryCounters::default(),
    }
}

fn overflow(count: RecoveryDiscoveryCount) -> Result<(), RecoveryDiscoveryFailure> {
    Err(RecoveryDiscoveryFailure::overflow(count))
}

fn limit(
    failure: Result<(), RecoveryDiscoveryFailure>,
) -> Option<(FilesystemObservationBound, u64, u64)> {
    match failure {
        Err(RecoveryDiscoveryFailure::Limit(past)) => {
            Some((past.dimension(), past.observed(), past.admitted()))
        }
        _ => None,
    }
}

#[test]
fn the_next_read_is_a_limit_past_the_reads_and_no_limit_past_every_count() {
    assert_eq!(observation(3, 2, 8, 0).admit_read(), Ok(()));
    assert_eq!(
        limit(observation(3, 3, 8, 0).admit_read()),
        Some((FilesystemObservationBound::Reads, 4, 3))
    );
    assert_eq!(
        observation(u64::MAX, u64::MAX, 8, 0).admit_read(),
        overflow(RecoveryDiscoveryCount::Reads)
    );
}

#[test]
fn spent_bytes_are_a_limit_past_the_observation_and_no_limit_past_every_count() {
    let mut within = observation(3, 0, 8, 2);
    assert_eq!(within.spend_read_bytes(6), Ok(()));
    assert_eq!((within.remaining_bytes, within.counters.bytes_read), (0, 6));
    assert_eq!(
        limit(observation(3, 0, 8, 2).spend_read_bytes(7)),
        Some((FilesystemObservationBound::ObservationBytes, 9, 8))
    );
    assert_eq!(
        observation(3, 0, u64::MAX, u64::MAX).spend_read_bytes(1),
        overflow(RecoveryDiscoveryCount::ObservationBytes)
    );
    let mut counted = observation(3, 0, 8, 0);
    counted.counters.bytes_read = u64::MAX;
    assert_eq!(
        counted.spend_read_bytes(1),
        overflow(RecoveryDiscoveryCount::BytesRead)
    );
}

#[test]
fn a_refused_read_past_every_count_names_the_count_it_passed() {
    let spent = observation(3, 0, u64::MAX, u64::MAX);
    assert_eq!(
        spent.read_refused(u64::MAX, 1),
        overflow(RecoveryDiscoveryCount::ObservationBytes)
    );
    // The tree tells no length: one past every ceiling is past every count.
    let unbounded = observation(3, 0, u64::MAX, 0);
    let refused = ArtifactTreeFailure::recovery_io(
        ArtifactTreeFailureKind::AccessLimitExceeded,
        std::io::ErrorKind::Other,
    );
    assert_eq!(
        unbounded.whole_read_refused(&refused, u64::MAX, u64::MAX),
        overflow(RecoveryDiscoveryCount::ReadLength).err()
    );
    assert_eq!(
        unbounded
            .whole_read_refused(&refused, 4, 4)
            .map(|failure| limit(Err(failure))),
        Some(Some((FilesystemObservationBound::RequestedBytes, 5, 4)))
    );
}

#[test]
fn wal_counts_name_themselves_past_every_count() {
    let mut wal = observation(3, 0, 8, 0);
    assert_eq!(wal.count_wal_bytes(5), Ok(()));
    assert_eq!(wal.count_wal_observation(), Ok(()));
    assert_eq!(
        (wal.counters.wal_bytes_read, wal.wal_observations_issued),
        (5, 1)
    );
    wal.counters.wal_bytes_read = u64::MAX;
    wal.wal_observations_issued = u64::MAX;
    assert_eq!(
        wal.count_wal_bytes(1),
        overflow(RecoveryDiscoveryCount::WalBytesRead)
    );
    assert_eq!(
        wal.count_wal_observation(),
        overflow(RecoveryDiscoveryCount::WalObservations)
    );
}
