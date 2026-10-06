//! Damage on media is the tree's failure, never a limit of this reader: a
//! read or listing the tree refused for what it found names no bound, however
//! the read's ceiling and the reader's bytes stand.

use super::*;

/// The failure a read or listing refused for, when it is the tree's.
fn tree_failure(failure: &RecoveryDiscoveryFailure) -> Option<ArtifactTreeFailureKind> {
    match failure {
        RecoveryDiscoveryFailure::Damage(ArtifactDamage::Media { failure, .. }) => {
            Some(failure.kind())
        }
        _ => None,
    }
}

/// A directory where a root artifact belongs: neither read of it is a limit,
/// however much its grant and the observation hold.
#[test]
fn a_whole_read_of_damaged_media_is_the_trees_failure_under_any_grant() {
    let address = RecordArtifactFile::RootManifest { generation: 6 };
    for (granted, reader) in [(6, 6), (6, 32), (32, 6)] {
        let (_parent, _observer, mut discovery) = discovery(
            |root| std::fs::create_dir_all(record_path(root, address)).unwrap(),
            4,
            reader,
        );
        let managed = stopped(&discovery.read(root_ceiling(6), grant(granted)));
        assert!(
            matches!(managed, Some(Stop::Media(kind)) if kind != ArtifactTreeFailureKind::Absent),
            "addressed read under grant {granted} with {reader} bytes: {managed:?}",
        );
        let allocated = discovery.read_with_allocator(root_ceiling(6), grant(granted), |_| {
            Ok::<_, DeniedAllocation>(Vec::new())
        });
        assert_eq!(stopped(&allocated), managed, "{allocated:?}");
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
            RecoveryDiscoveryFailure::Damage(ArtifactDamage::Media {
                artifact: RecoveryDiscoveryArtifact::WalDirectory,
                ..
            })
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
