use std::cell::Cell;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    ExtentArenaId, ExtentArenaRange, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::super::{
    namespace_identity_admission, recovery_qualification, ArtifactTreeFailureKind,
    FilesystemMediaAdmissionAuthority, FilesystemMediaOwner, MediaCounterObserver,
};
use crate::recovery_media::grant_for_test::grant;
use crate::recovery_media::{
    AdmittedRecoveryFilesystemMedia, AllocatedReadFailure, ArtifactCeiling, ArtifactDamage,
    BoundedRecoveryFilesystemDiscovery, FilesystemObservationBound, ObservedRecoveryArtifact,
    PageAddress, ReadGrant, ReadRefusal, RecoveryDiscoveryAllocationFailure,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure, UnchargedRead,
};

#[derive(Debug, PartialEq, Eq)]
struct DeniedAllocation;

/// A WAL inventory's segment ceiling, which is never nothing.
fn segments(count: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(count).expect("a segment ceiling of at least one")
}

/// A refusal's bound and both counts, or `None` when it is not a limit.
fn named(failure: &RecoveryDiscoveryFailure) -> Option<(FilesystemObservationBound, u64, u64)> {
    match failure {
        RecoveryDiscoveryFailure::Limit(past) => {
            Some((past.dimension(), past.observed(), past.admitted()))
        }
        _ => None,
    }
}

mod stops;
use stops::{format, page, root_ceiling, stopped, uncharged, Stop};

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
        .read_with_allocator(root_ceiling(1), uncharged(), |length| {
            assert_eq!(length, 4);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .observed()
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
    let head = PageAddress::ReleaseCustodyHeadBlock {
        generation: 2,
        block: 1,
    };
    for (address, ceiling) in [
        (
            RecordArtifactFile::RootManifest { generation: 2 },
            root_ceiling(2),
        ),
        (
            RecordArtifactFile::ReleaseCustodyHeadBlock {
                generation: 2,
                block: 1,
            },
            ArtifactCeiling::page(format(), head),
        ),
    ] {
        // This is the allocator/read boundary, not head grammar or custody
        // admission: the observed length must be available before bytes are read.
        let (_parent, observer, mut discovery) =
            discovery(|root| write_root_artifact(root, address, b"present"), 2, 32);
        let reads_before = observer.snapshot().positioned_read_attempts();
        let result = discovery.read_with_allocator(ceiling, uncharged(), |length| {
            assert_eq!(length, 7);
            Err::<Vec<u8>, _>(DeniedAllocation)
        });
        assert!(matches!(
            result,
            TransitionOutcome::Failed(AllocatedReadFailure::Allocation {
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
    let (_parent, _observer, mut discovery) =
        discovery(|root| write_root_artifact(root, present, b"present"), 2, 32);
    let calls = Cell::new(0);
    let past_grant = discovery.read_with_allocator(root_ceiling(3), grant(3), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(Vec::new())
    });
    assert_eq!(
        stopped(&past_grant),
        Some(Stop::PastGrant {
            granted: 3,
            length: 7
        })
    );
    let absent_read = discovery
        .read_with_allocator(root_ceiling(4), uncharged(), |_| {
            calls.set(calls.get() + 1);
            Ok::<_, DeniedAllocation>(Vec::new())
        })
        .observed()
        .unwrap();
    assert_eq!(absent_read.bytes(), None);
    let no_entries = discovery.read_with_allocator(root_ceiling(3), uncharged(), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(Vec::new())
    });
    // Two reads were admitted, and the third is past them.
    assert_eq!(
        stopped(&no_entries),
        Some(Stop::Observation(FilesystemObservationBound::Reads, 3, 2))
    );
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
        .read_record_range_with_allocator(address, 2, 3, uncharged(), |length| {
            assert_eq!(length, 3);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .observed()
        .unwrap();
    assert_eq!(range.offset(), 2);
    assert_eq!(range.bytes(), Some(&b"cde"[..]));
    assert_eq!(discovery.counters().bytes_read, 3);

    let calls = Cell::new(0);
    let outside = discovery.read_record_range_with_allocator(address, 7, 3, uncharged(), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; 3])
    });
    assert_eq!(
        stopped(&outside),
        Some(Stop::Media(ArtifactTreeFailureKind::Damaged))
    );
    assert_eq!(calls.get(), 0);

    let wrong = discovery.read_record_range_with_allocator(address, 1, 4, uncharged(), |length| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; length - 1])
    });
    assert!(matches!(
        wrong,
        TransitionOutcome::Failed(AllocatedReadFailure::BufferLengthMismatch {
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
fn absent_and_over_grant_ranges_never_allocate() {
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
        .read_record_range_with_allocator(absent, 0, 2, uncharged(), |_| {
            calls.set(calls.get() + 1);
            Ok::<_, DeniedAllocation>(vec![0; 2])
        })
        .observed()
        .unwrap();
    assert_eq!(absent_read.bytes(), None);
    let oversized = discovery.read_record_range_with_allocator(present, 0, 5, grant(4), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, DeniedAllocation>(vec![0; 5])
    });
    assert_eq!(
        stopped(&oversized),
        Some(Stop::PastGrant {
            granted: 4,
            length: 5
        })
    );
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
        let denied = discovery.read_extent_range_with_allocator(
            range,
            relative,
            length,
            uncharged(),
            |_| {
                calls.set(calls.get() + 1);
                Ok::<_, DeniedAllocation>(Vec::new())
            },
        );
        assert!(matches!(
            denied,
            TransitionOutcome::Failed(AllocatedReadFailure::Damage(
                ArtifactDamage::InvalidAddress {
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
        .read_extent_range_with_allocator(range, 1, 3, uncharged(), |length| {
            calls.set(calls.get() + 1);
            assert_eq!(length, 3);
            Ok::<_, DeniedAllocation>(vec![0; length])
        })
        .observed()
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

mod ceiling_grant;
mod damage;
mod range_refusal;
mod record_storage;
mod wal_context;
mod wal_limits;
mod wal_path;
