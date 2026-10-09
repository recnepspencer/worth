//! A range is its own real length, so every bound is decided before anything
//! is opened: past the caller's grant is that budget's, and only a range the
//! grant admits runs the observation out of bytes.

use super::record_storage::storage;
use super::*;

const ARENA: RecordArtifactFile = RecordArtifactFile::ExtentArena { arena: 10 };

fn arena_discovery(reader: u64) -> (tempfile::TempDir, BoundedRecoveryFilesystemDiscovery) {
    let (parent, _observer, discovery) = discovery(
        |root| {
            let directory = root.join("families/records/arenas");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(ARENA.file_name()), b"0123456789abcdef").unwrap();
        },
        4,
        reader,
    );
    (parent, discovery)
}

/// Where the three range readers stopped for `length` bytes under a grant of
/// `granted`, or no grant.
fn stops(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    length: u32,
    granted: Option<u64>,
) -> [Option<Stop>; 3] {
    let range = ExtentArenaRange::new(ExtentArenaId::new(10).unwrap(), 2, 8).unwrap();
    let allocate = |bytes| Ok::<_, DeniedAllocation>(vec![0; bytes]);
    let mut storage = storage(None, false);
    match granted {
        None => [
            stopped(&discovery.read_extent_range(range, 0, length, uncharged())),
            stopped(&discovery.read_record_range_with_allocator(
                ARENA,
                2,
                length,
                uncharged(),
                allocate,
            )),
            stopped(&discovery.read_record_artifact_range_with_storage(
                ARENA,
                2,
                length,
                uncharged(),
                &mut storage,
            )),
        ],
        Some(bytes) => [
            stopped(&discovery.read_extent_range(range, 0, length, grant(bytes))),
            stopped(&discovery.read_record_range_with_allocator(
                ARENA,
                2,
                length,
                grant(bytes),
                allocate,
            )),
            stopped(&discovery.read_record_artifact_range_with_storage(
                ARENA,
                2,
                length,
                grant(bytes),
                &mut storage,
            )),
        ],
    }
}

/// What the storage reader meets on a platform whose path storage is never
/// qualified.
fn stored(stop: Option<Stop>) -> Option<Stop> {
    if cfg!(windows) {
        stop
    } else {
        Some(Stop::InvalidAddress)
    }
}

fn each(stop: Option<Stop>) -> [Option<Stop>; 3] {
    [stop.clone(), stop.clone(), stored(stop)]
}

#[test]
fn a_range_exactly_at_its_grant_reads_and_one_past_names_its_length() {
    let (_parent, mut discovery) = arena_discovery(64);
    assert_eq!(stops(&mut discovery, 7, Some(7)), each(None));
    assert_eq!(
        stops(&mut discovery, 7, Some(6)),
        each(Some(Stop::PastGrant {
            granted: 6,
            length: 7
        })),
    );
    assert_eq!(
        discovery.counters().bytes_read,
        if cfg!(windows) { 21 } else { 14 }
    );
}

/// A range the grant admits, refused by the observation's bytes, is the
/// observation's limit with the backend's own counts; a grant never stands
/// in for the observation.
#[test]
fn a_range_within_its_grant_past_the_observation_is_the_observations() {
    use FilesystemObservationBound::ObservationBytes;
    for (granted, reader) in [(Some(7), 6), (None, 6), (Some(8), 4), (None, 0)] {
        let (_parent, mut discovery) = arena_discovery(reader);
        assert_eq!(
            stops(&mut discovery, 7, granted),
            each(Some(Stop::Observation(ObservationBytes, 7, reader))),
            "grant {granted:?} with {reader} observation bytes",
        );
        assert_eq!(discovery.counters().bytes_read, 0);
    }
}

#[test]
fn a_range_counts_the_bytes_the_observation_already_gave() {
    let (_parent, mut discovery) = arena_discovery(if cfg!(windows) { 9 } else { 6 });
    assert_eq!(stops(&mut discovery, 3, None), each(None));
    let spent = if cfg!(windows) { 9 } else { 6 };
    let refused = Some(Stop::Observation(
        FilesystemObservationBound::ObservationBytes,
        spent + 1,
        spent,
    ));
    assert_eq!(stops(&mut discovery, 1, None), each(refused));
}

#[test]
fn an_empty_range_addresses_nothing() {
    let (_parent, mut discovery) = arena_discovery(32);
    assert_eq!(
        stops(&mut discovery, 0, None),
        each(Some(Stop::InvalidAddress))
    );
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
}
