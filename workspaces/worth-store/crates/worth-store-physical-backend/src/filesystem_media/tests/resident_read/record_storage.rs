//! Real C4 record media proves path/open/payload admission and disposal order.

use super::*;
use crate::filesystem_media::{
    ArtifactTreePathAllocationBoundary as Boundary, ArtifactTreePathAllocator,
    ArtifactTreeReadAllocator, ArtifactTreeStorageAllocator, MediaOperationRole,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum StorageDenied {
    Path(Boundary),
    Payload,
}

#[derive(Default)]
struct Census {
    active: [u64; 4],
    path_calls: usize,
    payload_calls: usize,
}

pub(super) struct PathBacking {
    census: Rc<RefCell<Census>>,
    slot: usize,
    bytes: u64,
}
impl Drop for PathBacking {
    fn drop(&mut self) {
        self.census.borrow_mut().active[self.slot] -= self.bytes;
    }
}
pub(super) struct Storage {
    census: Rc<RefCell<Census>>,
    deny_path: Option<Boundary>,
    deny_payload: bool,
}
impl ArtifactTreeStorageAllocator for Storage {
    type Denial = StorageDenied;
}
impl ArtifactTreePathAllocator for Storage {
    type PathBacking = PathBacking;
    fn admit_path_backing(
        &mut self,
        boundary: Boundary,
        bytes: u64,
    ) -> Result<PathBacking, StorageDenied> {
        self.census.borrow_mut().path_calls += 1;
        if self.deny_path == Some(boundary) {
            self.deny_path = None;
            return Err(StorageDenied::Path(boundary));
        }
        let slot = match boundary {
            Boundary::DirectoryAddress => 0,
            Boundary::FileAddress => 1,
            Boundary::DirectoryOpen => 2,
            Boundary::FileOpen => 3,
        };
        self.census.borrow_mut().active[slot] += bytes;
        Ok(PathBacking {
            census: self.census.clone(),
            slot,
            bytes,
        })
    }
}
impl ArtifactTreeReadAllocator for Storage {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, StorageDenied> {
        let mut census = self.census.borrow_mut();
        assert_eq!(
            census.active, [0; 4],
            "record addresses and cap scratch die before payload"
        );
        census.payload_calls += 1;
        if self.deny_payload {
            self.deny_payload = false;
            return Err(StorageDenied::Payload);
        }
        Ok(vec![0; length])
    }
}
pub(super) fn storage(deny_path: Option<Boundary>, deny_payload: bool) -> Storage {
    Storage {
        census: Rc::new(RefCell::new(Census::default())),
        deny_path,
        deny_payload,
    }
}

#[cfg(windows)]
#[test]
fn record_path_and_open_denials_precede_payload_then_same_discovery_retries() {
    let address = RecordArtifactFile::ReleaseCustodyHeadBlock {
        generation: 7,
        block: 3,
    };
    for boundary in [
        Boundary::DirectoryAddress,
        Boundary::FileAddress,
        Boundary::DirectoryOpen,
        Boundary::FileOpen,
    ] {
        let (_parent, observer, mut discovery) =
            discovery(|root| write_root_artifact(root, address, b"head"), 8, 64);
        let mut storage = storage(Some(boundary), false);
        let before = observer.snapshot();
        let failure = discovery
            .read_record_artifact_with_storage(address, 16, &mut storage)
            .unwrap_err();
        assert!(
            matches!(failure, RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::Record(observed), offset: 0,
            requested, cause: StorageDenied::Path(actual),
        } if observed == address && requested > 0 && actual == boundary)
        );
        assert_eq!(storage.census.borrow().active, [0; 4]);
        assert_eq!(storage.census.borrow().payload_calls, 0);
        assert_eq!(
            observer.snapshot().positioned_read_attempts(),
            before.positioned_read_attempts()
        );
        if matches!(
            boundary,
            Boundary::DirectoryAddress | Boundary::FileAddress | Boundary::DirectoryOpen
        ) {
            assert_eq!(
                observer
                    .snapshot()
                    .attempts_for(MediaOperationRole::OpenDirectory),
                before.attempts_for(MediaOperationRole::OpenDirectory)
            );
        }
        if boundary == Boundary::FileOpen {
            assert_eq!(
                observer
                    .snapshot()
                    .attempts_for(MediaOperationRole::OpenExisting),
                before.attempts_for(MediaOperationRole::OpenExisting)
            );
        }
        let observed = discovery
            .read_record_artifact_with_storage(address, 16, &mut storage)
            .unwrap();
        assert_eq!(observed.bytes(), Some(&b"head"[..]));
        assert_eq!(
            observed.artifact(),
            &RecoveryDiscoveryArtifact::Record(address)
        );
        assert_eq!(storage.census.borrow().payload_calls, 1);
        assert_eq!(storage.census.borrow().active, [0; 4]);
        assert_eq!(discovery.counters().bytes_read, 4);
        assert_eq!(discovery.finish().recovery_effect_count(), 0);
    }
}

#[cfg(windows)]
#[test]
fn overflowing_record_range_precedes_rejecting_storage_and_same_discovery_retries() {
    let address = RecordArtifactFile::ExtentArena { arena: 1 };
    let (_parent, observer, mut discovery) = discovery(
        |root| {
            std::fs::create_dir_all(root.join("families/records/arenas")).unwrap();
            std::fs::write(
                root.join("families/records/arenas")
                    .join(address.file_name()),
                b"abcdef",
            )
            .unwrap();
        },
        1,
        64,
    );
    let mut storage = storage(Some(Boundary::DirectoryAddress), false);
    let before = observer.snapshot();
    let discovery_before = discovery.counters();
    let failure = discovery
        .read_record_artifact_range_with_storage(address, u64::MAX, 1, 16, &mut storage)
        .unwrap_err();
    assert!(matches!(
        failure,
        RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::Record(actual)
            }
        ) if actual == address
    ));
    assert_eq!(storage.census.borrow().path_calls, 0);
    assert_eq!(storage.census.borrow().payload_calls, 0);
    assert_eq!(storage.census.borrow().active, [0; 4]);
    assert_eq!(discovery.counters(), discovery_before);
    assert_eq!(
        observer.snapshot(),
        before,
        "no address, open or read attempt"
    );

    let failure = discovery
        .read_record_artifact_range_with_storage(address, 2, 3, 16, &mut storage)
        .unwrap_err();
    assert!(matches!(
        failure,
        RecoveryDiscoveryAllocationFailure::Allocation {
            cause: StorageDenied::Path(Boundary::DirectoryAddress),
            ..
        }
    ));
    assert_eq!(storage.census.borrow().path_calls, 1);
    assert_eq!(storage.census.borrow().payload_calls, 0);
    assert_eq!(discovery.counters(), discovery_before);
    assert_eq!(observer.snapshot(), before);
    let observed = discovery
        .read_record_artifact_range_with_storage(address, 2, 3, 16, &mut storage)
        .unwrap();
    assert_eq!(observed.bytes(), Some(&b"cde"[..]));
    assert_eq!(observed.offset(), 2);
    assert_eq!(storage.census.borrow().active, [0; 4]);
    assert_eq!(storage.census.borrow().payload_calls, 1);
    assert_eq!(discovery.counters().bytes_read, 3);
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[cfg(windows)]
#[test]
fn record_payload_denial_and_backed_range_preserve_exact_read_semantics() {
    let address = RecordArtifactFile::ExtentArena { arena: 1 };
    let (_parent, observer, mut discovery) = discovery(
        |root| {
            std::fs::create_dir_all(root.join("families/records/arenas")).unwrap();
            std::fs::write(
                root.join("families/records/arenas")
                    .join(address.file_name()),
                b"abcdef",
            )
            .unwrap();
        },
        8,
        64,
    );
    let mut storage = storage(None, true);
    let before = observer.snapshot().positioned_read_attempts();
    let failure = discovery
        .read_record_artifact_range_with_storage(address, 2, 3, 16, &mut storage)
        .unwrap_err();
    assert!(
        matches!(failure, RecoveryDiscoveryAllocationFailure::Allocation {
        artifact: RecoveryDiscoveryArtifact::Record(observed), offset: 2,
        requested: 3, cause: StorageDenied::Payload,
    } if observed == address)
    );
    assert_eq!(storage.census.borrow().active, [0; 4]);
    assert_eq!(observer.snapshot().positioned_read_attempts(), before);
    assert_eq!(discovery.counters().bytes_read, 0);
    let observed = discovery
        .read_record_artifact_range_with_storage(address, 2, 3, 16, &mut storage)
        .unwrap();
    assert_eq!(observed.bytes(), Some(&b"cde"[..]));
    assert_eq!(observed.offset(), 2);
    assert_eq!(discovery.counters().bytes_read, 3);
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
    assert_eq!(storage.census.borrow().active, [0; 4]);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[cfg(windows)]
#[test]
fn checkpoint_and_fixed_slot_backed_reads_preserve_success_absence_and_counters() {
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            std::fs::write(root.join("families/checkpoint.current"), b"checkpoint").unwrap();
        },
        8,
        64,
    );
    let mut storage = storage(None, true);
    let failure = discovery
        .read_current_checkpoint_with_storage(16, &mut storage)
        .unwrap_err();
    assert!(matches!(
        failure,
        RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            requested: 10,
            cause: StorageDenied::Payload,
            ..
        }
    ));
    let checkpoint = discovery
        .read_current_checkpoint_with_storage(16, &mut storage)
        .unwrap();
    assert_eq!(checkpoint.bytes(), Some(&b"checkpoint"[..]));
    let absent = discovery
        .read_record_artifact_with_storage(
            RecordArtifactFile::PreviousRootSelector,
            16,
            &mut storage,
        )
        .unwrap();
    assert_eq!(absent.bytes(), None);
    let missing = discovery
        .read_record_artifact_range_with_storage(
            RecordArtifactFile::RootManifest { generation: 99 },
            1,
            3,
            16,
            &mut storage,
        )
        .unwrap();
    assert_eq!(missing.bytes(), None);
    assert_eq!(missing.offset(), 1);
    assert_eq!(discovery.counters().bytes_read, 10);
    assert_eq!(discovery.counters().fixed_slots_read, 1);
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
    assert_eq!(storage.census.borrow().payload_calls, 2);
    assert_eq!(storage.census.borrow().active, [0; 4]);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[cfg(not(windows))]
#[test]
fn unqualified_record_storage_rejects_before_address_or_payload_callbacks() {
    let (_parent, _observer, mut discovery) = discovery(|_| {}, 4, 64);
    let mut storage = storage(None, false);
    let result = discovery.read_current_checkpoint_with_storage(16, &mut storage);
    assert!(matches!(
        result,
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint
            }
        ))
    ));
    assert_eq!(storage.census.borrow().path_calls, 0);
    assert_eq!(storage.census.borrow().payload_calls, 0);
}
