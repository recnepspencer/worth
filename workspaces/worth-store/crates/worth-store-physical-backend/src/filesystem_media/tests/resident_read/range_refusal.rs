//! A range is refused by the rule a whole artifact is: one longer than the
//! ceiling its caller named passed that ceiling, however few bytes the reader
//! has left, and only one within its ceiling runs the reader out of bytes.

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

fn discovered<E: std::fmt::Debug>(
    failure: RecoveryDiscoveryAllocationFailure<E>,
) -> RecoveryDiscoveryFailure {
    match failure {
        RecoveryDiscoveryAllocationFailure::Discovery(failure) => failure,
        other => panic!("refused before any allocation: {other:?}"),
    }
}

/// The three range readers' refusals of `length` bytes under `ceiling`.
fn refusals(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    length: u32,
    ceiling: u64,
) -> [Option<RecoveryDiscoveryFailure>; 3] {
    let range = ExtentArenaRange::new(ExtentArenaId::new(10).unwrap(), 2, 8).unwrap();
    let addressed = discovery.read_extent_range(range, 0, length, ceiling);
    let allocated =
        discovery.read_record_range_with_allocator(ARENA, 2, length, ceiling, |bytes| {
            Ok::<_, DeniedAllocation>(vec![0; bytes])
        });
    let stored = discovery.read_record_artifact_range_with_storage(
        ARENA,
        2,
        length,
        ceiling,
        &mut storage(None, false),
    );
    [
        addressed.err(),
        allocated.err().map(discovered),
        stored.err().map(discovered),
    ]
}

#[test]
fn a_range_past_its_ceiling_is_never_the_readers_own_limit() {
    use RecoveryDiscoveryByteLimitScope::{Observation, Requested};
    // A range of seven bytes, asked under (ceiling, reader bytes).
    for (ceiling, reader, observed, admitted, scope) in [
        (6, 32, 7, 6, Requested),
        (6, 4, 7, 6, Requested),
        (6, 6, 7, 6, Requested),
        (7, 6, 7, 6, Observation),
        (8, 4, 7, 4, Observation),
    ] {
        let refused = Some(RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed,
            admitted,
            scope,
        });
        let (_parent, mut discovery) = arena_discovery(reader);
        assert_eq!(
            refusals(&mut discovery, 7, ceiling),
            [refused.clone(), refused.clone(), refused],
            "ceiling {ceiling} with {reader} reader bytes",
        );
        assert_eq!(discovery.counters().bytes_read, 0);
    }
}

#[test]
fn a_range_within_its_ceiling_counts_the_bytes_the_reader_already_gave() {
    let (_parent, mut discovery) = arena_discovery(9);
    assert_eq!(refusals(&mut discovery, 3, 8), [None, None, None]);
    // Nine bytes are spent; one more passes the reader's nine, not the ceiling.
    let refused = Some(RecoveryDiscoveryFailure::ByteLimitExceeded {
        observed: 10,
        admitted: 9,
        scope: RecoveryDiscoveryByteLimitScope::Observation,
    });
    assert_eq!(
        refusals(&mut discovery, 1, 8),
        [refused.clone(), refused.clone(), refused],
    );
}

#[test]
fn an_empty_range_addresses_nothing() {
    let (_parent, mut discovery) = arena_discovery(32);
    let invalid = Some(RecoveryDiscoveryFailure::InvalidAddress {
        artifact: RecoveryDiscoveryArtifact::Record(ARENA),
    });
    assert_eq!(
        refusals(&mut discovery, 0, 8),
        [invalid.clone(), invalid.clone(), invalid],
    );
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
}
