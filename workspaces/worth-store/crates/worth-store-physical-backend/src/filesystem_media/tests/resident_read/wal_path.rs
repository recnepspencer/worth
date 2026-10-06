use super::*;
use crate::filesystem_media::{
    ArtifactTreeDirectoryEntry, ArtifactTreeListingAllocator, ArtifactTreeListingStorageChange,
    ArtifactTreePathAllocationBoundary as Boundary, ArtifactTreePathAllocator,
    ArtifactTreeReadAllocator, ArtifactTreeStorageAllocator,
};
use crate::recovery_media::{
    ObservedWalArtifact, RecoveryWalListingAllocationMode, RecoveryWalReadStorage,
};
use std::{cell::RefCell, ffi::OsString, rc::Rc};

#[derive(Debug, PartialEq, Eq)]
struct PathDenied(Boundary);

#[derive(Default)]
struct Census {
    active: [u64; 4],
    payload_calls: usize,
    calls: Vec<(Boundary, u64)>,
}

struct Backing {
    census: Rc<RefCell<Census>>,
    index: usize,
    bytes: u64,
}
impl Drop for Backing {
    fn drop(&mut self) {
        self.census.borrow_mut().active[self.index] -= self.bytes;
    }
}

struct Storage {
    census: Rc<RefCell<Census>>,
    deny: Option<Boundary>,
}
impl ArtifactTreeStorageAllocator for Storage {
    type Denial = PathDenied;
}
impl ArtifactTreePathAllocator for Storage {
    type PathBacking = Backing;
    fn admit_path_backing(
        &mut self,
        boundary: Boundary,
        bytes: u64,
    ) -> Result<Backing, PathDenied> {
        let index = match boundary {
            Boundary::DirectoryAddress => 0,
            Boundary::FileAddress => 1,
            Boundary::DirectoryOpen => 2,
            Boundary::FileOpen => 3,
        };
        self.census.borrow_mut().calls.push((boundary, bytes));
        if self.deny == Some(boundary) {
            self.deny = None;
            return Err(PathDenied(boundary));
        }
        self.census.borrow_mut().active[index] += bytes;
        Ok(Backing {
            census: self.census.clone(),
            index,
            bytes,
        })
    }
}
impl ArtifactTreeListingAllocator for Storage {
    fn listing_storage_change(
        &mut self,
        _: ArtifactTreeListingStorageChange,
    ) -> Result<(), PathDenied> {
        Ok(())
    }
    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, PathDenied> {
        Ok(Vec::with_capacity(count))
    }
}
impl ArtifactTreeReadAllocator for Storage {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, PathDenied> {
        let mut census = self.census.borrow_mut();
        assert!(
            census.active[0] > 0,
            "the outer WAL directory address remains owned"
        );
        assert_eq!(
            &census.active[1..],
            &[0, 0, 0],
            "file address and both cap-open scratch grants are disposed before payload allocation"
        );
        census.payload_calls += 1;
        Ok(vec![0; length])
    }
}
impl RecoveryWalReadStorage for Storage {
    fn listing_allocation_mode(&self) -> RecoveryWalListingAllocationMode {
        RecoveryWalListingAllocationMode::AdmissionCallbacks
    }
    fn allocate_wal_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ObservedWalArtifact>, PathDenied> {
        Ok(Vec::with_capacity(count))
    }
    fn allocate_context(&mut self, count: usize) -> Result<OsString, PathDenied> {
        Ok(OsString::with_capacity(count))
    }
}

#[cfg(windows)]
#[test]
fn wal_path_denials_precede_their_open_and_same_owner_retry_disposes_backing() {
    // This real C.4 owner exercises allocator ordering and confinement, not
    // native residency or WAL grammar. Native grant pressure is Store-owned.
    for boundary in [
        Boundary::DirectoryAddress,
        Boundary::DirectoryOpen,
        Boundary::FileAddress,
        Boundary::FileOpen,
    ] {
        let name = ".ΐ-actual.wal";
        let (parent, observer, mut discovery) = discovery(
            |root| {
                std::fs::create_dir_all(root.join("families/wal")).unwrap();
                std::fs::write(root.join("families/wal").join(name), b"payload").unwrap();
            },
            8,
            64,
        );
        assert!(discovery.wal_path_storage_is_qualified());
        let census = Rc::new(RefCell::new(Census::default()));
        let mut storage = Storage {
            census: census.clone(),
            deny: Some(boundary),
        };
        let before = observer.snapshot();
        let failure = discovery
            .read_wal_artifacts_with_storage(segments(4), 32, &mut storage)
            .unwrap_err();
        assert!(
            matches!(failure, RecoveryDiscoveryAllocationFailure::Allocation { requested, cause: PathDenied(actual), .. } if requested > 0 && actual == boundary)
        );
        assert_eq!(census.borrow().active, [0; 4]);
        assert_eq!(census.borrow().payload_calls, 0);
        assert_eq!(discovery.counters().wal_bytes_read, 0);
        assert_eq!(
            observer.snapshot().positioned_read_attempts(),
            before.positioned_read_attempts()
        );
        if boundary == Boundary::DirectoryAddress || boundary == Boundary::DirectoryOpen {
            assert_eq!(
                observer
                    .snapshot()
                    .attempts_for(crate::filesystem_media::MediaOperationRole::OpenDirectory),
                before.attempts_for(crate::filesystem_media::MediaOperationRole::OpenDirectory)
            );
        }
        if boundary == Boundary::FileOpen {
            assert_eq!(
                observer
                    .snapshot()
                    .attempts_for(crate::filesystem_media::MediaOperationRole::OpenExisting),
                before.attempts_for(crate::filesystem_media::MediaOperationRole::OpenExisting)
            );
        }
        let observed = discovery
            .read_wal_artifacts_with_storage(segments(4), 32, &mut storage)
            .unwrap();
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].name(), std::ffi::OsStr::new(name));
        assert_eq!(observed[0].bytes(), Some(b"payload".as_slice()));
        assert_eq!(census.borrow().payload_calls, 1);
        assert_eq!(census.borrow().active, [0; 4]);
        assert_eq!(discovery.finish().recovery_effect_count(), 0);
        assert_eq!(
            std::fs::read(parent.path().join("store/families/wal").join(name)).unwrap(),
            b"payload"
        );
    }
}

#[cfg(not(windows))]
#[test]
fn unqualified_path_profile_denies_before_address_admission_or_payload() {
    let (_parent, observer, mut discovery) = discovery(
        |root| {
            std::fs::create_dir_all(root.join("families/wal")).unwrap();
            std::fs::write(root.join("families/wal/actual.wal"), b"payload").unwrap();
        },
        8,
        64,
    );
    let census = Rc::new(RefCell::new(Census::default()));
    let mut storage = Storage {
        census: census.clone(),
        deny: None,
    };
    assert!(!discovery.wal_path_storage_is_qualified());
    let before = observer.snapshot().positioned_read_attempts();
    assert!(matches!(
        discovery.read_wal_artifacts_with_storage(segments(4), 32, &mut storage),
        Err(RecoveryDiscoveryAllocationFailure::Discovery(
            RecoveryDiscoveryFailure::Damage(ArtifactDamage::InvalidAddress { .. })
        ))
    ));
    assert!(census.borrow().calls.is_empty());
    assert_eq!(observer.snapshot().positioned_read_attempts(), before);
}

#[cfg(windows)]
#[test]
fn borrowed_serving_wal_observation_uses_the_live_owner_and_unique_reads() {
    use crate::filesystem_media::{
        ArtifactTreeDirectory, FilesystemAccessPosture, FilesystemMediaOwner,
        FilesystemQualificationRequest,
    };
    use worth_proof::TransitionOutcome;
    let directory = tempfile::tempdir().unwrap();
    let request = FilesystemQualificationRequest::certification(
        directory.path().join("store"),
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let TransitionOutcome::Success(media) = FilesystemMediaOwner::qualify(request).into_raw()
    else {
        panic!("real qualified owner required")
    };
    let wal = ArtifactTreeDirectory::families().child("wal").unwrap();
    media.artifact_tree().create_directory(&wal).unwrap();
    media
        .artifact_tree()
        .write_new(&wal.file("actual.wal").unwrap(), b"payload")
        .unwrap();
    let before = media.counter_observer().snapshot();
    let census = Rc::new(RefCell::new(Census::default()));
    let mut storage = Storage {
        census: census.clone(),
        deny: Some(Boundary::FileOpen),
    };
    let mut observation = media.bounded_wal_observation(4, 32).unwrap();
    assert_eq!(observation.store_identity(), media.store_identity());
    let failure = observation
        .read_wal_artifacts_with_storage(segments(4), 32, &mut storage)
        .unwrap_err();
    assert!(matches!(
        failure,
        RecoveryDiscoveryAllocationFailure::Allocation {
            cause: PathDenied(Boundary::FileOpen),
            ..
        }
    ));
    assert_eq!(
        media.counters().positioned_read_attempts(),
        before.positioned_read_attempts()
    );
    assert_eq!(census.borrow().active, [0; 4]);
    let first = observation
        .read_wal_artifacts_with_storage(segments(4), 32, &mut storage)
        .unwrap();
    assert_eq!(first[0].store_identity(), media.store_identity());
    assert_eq!(first[0].bytes(), Some(b"payload".as_slice()));
    let second = media
        .bounded_wal_observation(4, 32)
        .unwrap()
        .read_wal_artifacts_with_storage(segments(4), 32, &mut storage)
        .unwrap();
    assert_ne!(
        first[0].observation_identity(),
        second[0].observation_identity()
    );
    assert_eq!(second[0].bytes(), first[0].bytes());
    assert_eq!(census.borrow().active, [0; 4]);
    drop((first, second, observation));
    assert_eq!(
        std::fs::read(directory.path().join("store/families/wal/actual.wal")).unwrap(),
        b"payload"
    );
    media.close();
}
