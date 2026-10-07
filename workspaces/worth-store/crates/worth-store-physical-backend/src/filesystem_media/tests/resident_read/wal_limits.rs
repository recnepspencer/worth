//! A WAL inventory's stops, named the same way for every reader: the
//! listing's segment ceiling and the observation's bytes are the
//! observation's, with the backend's counts; one grant shared by the whole
//! inventory refuses a file with what the files before it took plus its own
//! real length.

use super::*;
use FilesystemObservationBound::{Entries, ObservationBytes};

fn wal_discovery(
    files: &[(&str, usize)],
    reader: u64,
) -> (tempfile::TempDir, BoundedRecoveryFilesystemDiscovery) {
    let (parent, _observer, discovery) = discovery(
        |root| {
            let wal = root.join("families/wal");
            std::fs::create_dir_all(&wal).expect("real WAL directory");
            for (name, length) in files {
                std::fs::write(wal.join(name), vec![5; *length]).expect("real WAL file");
            }
        },
        8,
        reader,
    );
    (parent, discovery)
}

/// Where the caller-managed and the allocated inventory each stopped, on
/// their own observations; `None` for an inventory that read.
fn inventory_stops(
    files: &[(&str, usize)],
    maximum_segments: u64,
    granted: Option<u64>,
    reader: u64,
) -> [Option<Stop>; 2] {
    let maximum_segments = segments(maximum_segments);
    let (_managed_parent, mut managed) = wal_discovery(files, reader);
    let (_allocated_parent, mut allocated) = wal_discovery(files, reader);
    let read = |discovery: &mut BoundedRecoveryFilesystemDiscovery, granted| {
        discovery.read_wal_artifacts_with_allocators(
            maximum_segments,
            grant(granted),
            |count| Ok::<_, DeniedAllocation>(Vec::with_capacity(count)),
            |length| Ok(vec![0; length]),
            |count| Ok(std::ffi::OsString::with_capacity(count)),
        )
    };
    match granted {
        Some(granted) => [
            stopped(&managed.read_wal_artifacts(maximum_segments, grant(granted))),
            stopped(&read(&mut allocated, granted)),
        ],
        None => [
            stopped(&managed.read_wal_artifacts(maximum_segments, uncharged())),
            stopped(&allocated.read_wal_artifacts_with_allocators(
                maximum_segments,
                uncharged(),
                |count| Ok::<_, DeniedAllocation>(Vec::with_capacity(count)),
                |length| Ok(vec![0; length]),
                |count| Ok(std::ffi::OsString::with_capacity(count)),
            )),
        ],
    }
}

fn both(stop: Option<Stop>) -> [Option<Stop>; 2] {
    [stop.clone(), stop]
}

const THREE: [(&str, usize); 3] = [("a.wal", 5), ("b.wal", 5), ("c.wal", 5)];
const TWO: [(&str, usize); 2] = [("a.wal", 5), ("b.wal", 5)];

#[test]
fn a_listing_past_its_segment_ceiling_names_both_counts() {
    assert_eq!(
        inventory_stops(&THREE, 2, None, 64),
        both(Some(Stop::Observation(Entries, 3, 2)))
    );
}

/// Two five-byte files under a grant of ten read; under nine, the second is
/// asked for the four bytes left and refused with the five the first took
/// plus its own five.
#[test]
fn an_inventory_exactly_at_its_grant_reads_and_one_past_counts_what_earlier_reads_took() {
    assert_eq!(inventory_stops(&TWO, 2, Some(10), 64), [None, None]);
    assert_eq!(
        inventory_stops(&TWO, 2, Some(9), 64),
        both(Some(Stop::PastGrant {
            granted: 9,
            length: 10
        }))
    );
    // One file alone past the whole grant names its own length.
    assert_eq!(
        inventory_stops(&[("a.wal", 7)], 1, Some(6), 64),
        both(Some(Stop::PastGrant {
            granted: 6,
            length: 7
        }))
    );
}

/// The grant admits the files; the observation's bytes do not: the
/// observation's limit with the backend's counts, the bytes it read before
/// the file plus the file's real length.
#[test]
fn an_inventory_within_its_grant_past_the_observation_is_the_observations() {
    assert_eq!(
        inventory_stops(&TWO, 2, Some(64), 9),
        both(Some(Stop::Observation(ObservationBytes, 10, 9)))
    );
    assert_eq!(
        inventory_stops(&TWO, 2, None, 9),
        both(Some(Stop::Observation(ObservationBytes, 10, 9)))
    );
    assert_eq!(inventory_stops(&TWO, 2, None, 10), [None, None]);
}
