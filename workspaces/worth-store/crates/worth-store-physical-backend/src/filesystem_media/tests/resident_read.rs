use std::cell::Cell;

use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange, RecordArtifactFile};

use super::super::{
    namespace_identity_admission, recovery_qualification, ArtifactTreeFailureKind,
    FilesystemMediaAdmissionAuthority, FilesystemMediaOwner, MediaCounterObserver,
};
use crate::recovery_media::{
    AdmittedRecoveryFilesystemMedia, BoundedRecoveryFilesystemDiscovery,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};

#[derive(Debug, PartialEq, Eq)]
struct DeniedAllocation;

fn discovery(
    prepare: impl FnOnce(&std::path::Path),
    entries: u64,
    bytes: u64,
) -> (
    tempfile::TempDir,
    MediaCounterObserver,
    BoundedRecoveryFilesystemDiscovery,
) {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("store");
    let owner = FilesystemMediaOwner::admit(&root, FilesystemMediaAdmissionAuthority::for_test())
        .expect("real backend owner");
    namespace_identity_admission::admit_store_identity(&owner).expect("persisted identity");
    owner.close();
    prepare(&root);
    let parts = recovery_qualification::qualify_existing_recovery(&root)
        .expect("qualified existing backend")
        .admit_persisted_store()
        .expect("admitted existing store");
    let observer = parts.owner.counter_observer();
    let media = AdmittedRecoveryFilesystemMedia::from_parts(parts);
    (
        parent,
        observer,
        media
            .bounded_discovery(entries, bytes)
            .expect("bounded C4 discovery"),
    )
}

fn record_path(root: &std::path::Path, artifact: RecordArtifactFile) -> std::path::PathBuf {
    root.join("families/records/roots")
        .join(artifact.file_name())
}

fn write_root_artifact(root: &std::path::Path, artifact: RecordArtifactFile, bytes: &[u8]) {
    std::fs::create_dir_all(root.join("families/records/roots")).unwrap();
    std::fs::write(record_path(root, artifact), bytes).unwrap();
}

#[test]
fn whole_reads_allocate_after_observed_length_and_preserve_c4_counters() {
    let address = RecordArtifactFile::RootManifest { generation: 1 };
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            write_root_artifact(root, address, b"root");
            std::fs::write(root.join("families/checkpoint.current"), b"checkpoint").unwrap();
        },
        4,
        32,
    );
    let record = discovery
        .read_record_artifact_with_allocator(address, 8, |length| {
            assert_eq!(length, 4);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .unwrap();
    assert_eq!(record.bytes(), Some(&b"root"[..]));
    let checkpoint = discovery
        .read_current_checkpoint_with_allocator(16, |length| {
            assert_eq!(length, 10);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .unwrap();
    assert_eq!(checkpoint.bytes(), Some(&b"checkpoint"[..]));
    assert_eq!(discovery.counters().bytes_read, 14);
    assert_eq!(discovery.counters().addressed_artifacts_read, 2);
    assert_eq!(discovery.counters().fixed_slots_read, 0);
}

#[test]
fn allocation_denial_is_typed_and_precedes_successful_data_read() {
    for address in [
        RecordArtifactFile::RootManifest { generation: 2 },
        RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: 2,
            block: 1,
        },
    ] {
        // This is the allocator/read boundary, not head grammar or custody
        // admission: the observed length must be available before bytes are read.
        let (_parent, observer, mut discovery) =
            discovery(|root| write_root_artifact(root, address, b"present"), 2, 32);
        let reads_before = observer.snapshot().positioned_read_attempts();
        let result = discovery.read_record_artifact_with_allocator(address, 16, |length| {
            assert_eq!(length, 7);
            Err::<Vec<u8>, _>(DeniedAllocation)
        });
        assert!(matches!(
            result,
            Err(RecoveryDiscoveryAllocationFailure::Allocation {
                artifact: RecoveryDiscoveryArtifact::Record(observed),
                offset: 0,
                requested: 7,
                cause: DeniedAllocation,
            }) if observed == address
        ));
        assert_eq!(discovery.counters().bytes_read, 0);
        assert_eq!(discovery.counters().addressed_artifacts_read, 0);
        assert_eq!(observer.snapshot().positioned_read_attempts(), reads_before);
    }
}

#[test]
fn limits_and_absence_precede_allocator_callback() {
    let present = RecordArtifactFile::RootManifest { generation: 3 };
    let absent = RecordArtifactFile::RootManifest { generation: 4 };
    let (_parent, _observer, mut discovery) =
        discovery(|root| write_root_artifact(root, present, b"present"), 2, 32);
    let calls = Cell::new(0);
    let too_small = discovery.read_record_artifact_with_allocator(present, 3, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(Vec::new())
    });
    assert!(matches!(
        too_small,
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::ByteLimitExceeded { .. }
        ))
    ));
    let absent_read = discovery
        .read_record_artifact_with_allocator(absent, 16, |_| {
            calls.set(calls.get() + 1);
            Ok::<_, DeniedAllocation>(Vec::new())
        })
        .unwrap();
    assert_eq!(absent_read.bytes(), None);
    let no_entries = discovery.read_record_artifact_with_allocator(present, 16, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(Vec::new())
    });
    assert!(matches!(
        no_entries,
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::EntryLimitExceeded { .. }
        ))
    ));
    assert_eq!(calls.get(), 0);
}

#[test]
fn range_read_checks_file_bounds_then_exact_buffer_before_data_read() {
    let address = RecordArtifactFile::ExtentArena { arena: 9 };
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            let directory = root.join("families/records/arenas");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(address.file_name()), b"abcdefgh").unwrap();
        },
        5,
        32,
    );
    let range = discovery
        .read_record_range_with_allocator(address, 2, 3, 8, |length| {
            assert_eq!(length, 3);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .unwrap();
    assert_eq!(range.offset(), 2);
    assert_eq!(range.bytes(), Some(&b"cde"[..]));
    assert_eq!(discovery.counters().bytes_read, 3);

    let calls = Cell::new(0);
    let outside = discovery.read_record_range_with_allocator(address, 7, 3, 8, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; 3])
    });
    assert!(matches!(
        outside,
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::Media { failure, .. }
        )) if failure.kind() == ArtifactTreeFailureKind::Damaged
    ));
    assert_eq!(calls.get(), 0);

    let wrong = discovery.read_record_range_with_allocator(address, 1, 4, 8, |length| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; length - 1])
    });
    assert!(matches!(
        wrong,
        Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact: RecoveryDiscoveryArtifact::Record(observed_artifact),
            offset: 1,
            requested: 4,
            observed: 3
        }) if observed_artifact == address
    ));
    assert_eq!(calls.get(), 1);
    assert_eq!(discovery.counters().bytes_read, 3);
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
}

#[test]
fn absent_and_over_limit_ranges_never_allocate() {
    let present = RecordArtifactFile::ExtentArena { arena: 10 };
    let absent = RecordArtifactFile::ExtentArena { arena: 11 };
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            let directory = root.join("families/records/arenas");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(present.file_name()), b"12345678").unwrap();
        },
        3,
        16,
    );
    let calls = Cell::new(0);
    let absent_read = discovery
        .read_record_range_with_allocator(absent, 0, 2, 4, |_| {
            calls.set(calls.get() + 1);
            Ok::<_, DeniedAllocation>(vec![0; 2])
        })
        .unwrap();
    assert_eq!(absent_read.bytes(), None);
    let oversized = discovery.read_record_range_with_allocator(present, 0, 5, 4, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; 5])
    });
    assert!(matches!(
        oversized,
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::ByteLimitExceeded { .. }
        ))
    ));
    assert_eq!(calls.get(), 0);
    assert_eq!(discovery.counters().bytes_read, 0);
}

#[test]
fn extent_relative_range_is_admitted_before_allocator_or_positioned_read() {
    let arena = ExtentArenaId::new(12).unwrap();
    let range = ExtentArenaRange::new(arena, 2, 5).unwrap();
    let artifact = RecordArtifactFile::ExtentArena { arena: arena.get() };
    let (_parent, observer, mut discovery) = discovery(
        |root| {
            let directory = root.join("families/records/arenas");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(artifact.file_name()), b"abcdefgh").unwrap();
        },
        3,
        16,
    );
    let reads_before = observer.snapshot().positioned_read_attempts();
    let calls = Cell::new(0);
    for (relative, length) in [(4, 2), (u64::MAX, 2)] {
        let denied = discovery.read_extent_range_with_allocator(range, relative, length, 8, |_| {
            calls.set(calls.get() + 1);
            Ok::<_, DeniedAllocation>(Vec::new())
        });
        assert!(matches!(
            denied,
            Err(RecoveryDiscoveryAllocationFailure::Discovery(
                RecoveryDiscoveryFailure::InvalidAddress {
                    artifact: RecoveryDiscoveryArtifact::Record(observed),
                }
            )) if observed == artifact
        ));
    }
    assert_eq!(calls.get(), 0);
    assert_eq!(observer.snapshot().positioned_read_attempts(), reads_before);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);

    let observed = discovery
        .read_extent_range_with_allocator(range, 1, 3, 8, |length| {
            calls.set(calls.get() + 1);
            assert_eq!(length, 3);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .unwrap();
    assert_eq!(
        observed.artifact(),
        &RecoveryDiscoveryArtifact::Record(artifact)
    );
    assert_eq!(observed.offset(), 3);
    assert_eq!(observed.bytes(), Some(&b"def"[..]));
    assert_eq!(calls.get(), 1);
    assert_eq!(
        observer.snapshot().positioned_read_attempts(),
        reads_before + 1
    );
    assert_eq!(discovery.counters().bytes_read, 3);
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
}
