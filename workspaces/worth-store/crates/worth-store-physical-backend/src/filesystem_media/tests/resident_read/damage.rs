//! Damage on media is the tree's failure, never a limit of this reader: a
//! read or listing the tree refused for what it found names no bound, however
//! the read's ceiling and the reader's bytes stand.

use super::*;

/// The failure a read or listing refused for, when it is the tree's.
fn tree_failure(failure: &RecoveryDiscoveryFailure) -> Option<ArtifactTreeFailureKind> {
    match failure {
        RecoveryDiscoveryFailure::Media { failure, .. } => Some(failure.kind()),
        _ => None,
    }
}

/// A directory where a root artifact belongs: neither read of it is the
/// requested-bytes limit its ceiling would name for an artifact one byte past.
#[test]
fn a_whole_read_of_damaged_media_is_the_trees_failure_under_any_ceiling() {
    let address = RecordArtifactFile::RootManifest { generation: 6 };
    // The ceiling is all the reader holds, and less than it holds.
    for (ceiling, reader) in [(6, 6), (6, 32)] {
        let (_parent, _observer, mut discovery) = discovery(
            |root| std::fs::create_dir_all(record_path(root, address)).unwrap(),
            4,
            reader,
        );
        let managed = discovery.read_root_manifest(6, ceiling).unwrap_err();
        let kind = tree_failure(&managed);
        assert!(
            kind.is_some_and(|kind| kind != ArtifactTreeFailureKind::Absent),
            "addressed read under ceiling {ceiling} with {reader} bytes: {managed:?}",
        );
        let allocated = discovery.read_record_artifact_with_allocator(address, ceiling, |_| {
            Ok::<_, DeniedAllocation>(Vec::new())
        });
        match &allocated {
            Err(RecoveryDiscoveryAllocationFailure::Discovery(failure)) => {
                assert_eq!(tree_failure(failure), kind, "{allocated:?}");
            }
            _ => panic!("allocated read of damaged media: {allocated:?}"),
        }
        assert_eq!(discovery.counters().bytes_read, 0);
    }
}

/// A file where the WAL directory belongs: the inventory is the tree's
/// failure on that directory, not a count past the segments it was admitted.
#[test]
fn a_wal_directory_that_is_not_one_is_the_trees_failure() {
    let (_parent, _observer, mut discovery) = discovery(
        |root| std::fs::write(root.join("families/wal"), b"not a directory").unwrap(),
        8,
        64,
    );
    let managed = discovery.read_wal_artifacts(segments(2), 64).unwrap_err();
    assert!(
        matches!(
            &managed,
            RecoveryDiscoveryFailure::Media {
                artifact: RecoveryDiscoveryArtifact::WalDirectory,
                ..
            }
        ),
        "{managed:?}",
    );
    let allocated = discovery.read_wal_artifacts_with_allocators(
        segments(2),
        64,
        |count| Ok::<_, DeniedAllocation>(Vec::with_capacity(count)),
        |length| Ok(vec![0; length]),
        |count| Ok(std::ffi::OsString::with_capacity(count)),
    );
    match &allocated {
        Err(RecoveryDiscoveryAllocationFailure::Discovery(failure)) => {
            assert_eq!(
                tree_failure(failure),
                tree_failure(&managed),
                "{allocated:?}"
            );
        }
        _ => panic!("allocated inventory of damaged media: {allocated:?}"),
    }
}
