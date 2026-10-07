//! Mechanical borrowed reads use real confined media, not recovery authority.
use super::*;
use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator,
    ArtifactTreeStorageAllocator, FilesystemAccessPosture, FilesystemMediaOwner,
    FilesystemQualificationRequest, MediaOperationRole,
};
use crate::recovery_media::{
    ArtifactCeiling, ArtifactDamage, PageAddress, ReadGrant, RecoveryDiscoveryArtifact,
    UnchargedRead,
};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

#[derive(Default)]
struct Storage {
    address_calls: usize,
    payload_calls: usize,
}
impl ArtifactTreeStorageAllocator for Storage {
    type Denial = ();
}
impl ArtifactTreePathAllocator for Storage {
    type PathBacking = ();
    fn admit_path_backing(
        &mut self,
        _: ArtifactTreePathAllocationBoundary,
        _: u64,
    ) -> Result<(), ()> {
        self.address_calls += 1;
        Ok(())
    }
}
impl ArtifactTreeReadAllocator for Storage {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, ()> {
        self.payload_calls += 1;
        Ok(vec![0; length])
    }
}

#[test]
fn borrowed_record_range_overflow_denies_before_address_or_payload_allocation() {
    let (_root, media) = qualified();
    let mut storage = Storage::default();
    let mut observation = media.bounded_record_observation(2, 64).unwrap();
    let before = media.counters();
    let address = RecordArtifactFile::ReleaseCustodyHeadBlock {
        generation: 7,
        block: 2,
    };
    assert!(matches!(
        observation.read_record_artifact_range_with_storage(
            address, u64::MAX, 2, ReadGrant::ceiling_only(), &mut storage,
        ),
        TransitionOutcome::Failed(crate::recovery_media::AllocatedReadFailure::Damage(
            ArtifactDamage::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::Record(observed),
            },
        )) if observed == address
    ));
    assert_eq!((storage.address_calls, storage.payload_calls), (0, 0));
    assert_eq!(observation.counters(), Default::default());
    assert_eq!(media.counters(), before);
    assert!(matches!(
        media.bounded_record_observation(0, 64),
        Err(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit)
    ));
    // An observation admitted no bytes is an observation every read
    // refuses with its real length.
    assert!(media.bounded_record_observation(2, 0).is_ok());
}

#[cfg(windows)]
#[test]
fn borrowed_record_reads_preserve_actual_head_bytes_ranges_and_absence() {
    let (root, media) = qualified();
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    media.artifact_tree().create_directory(&records).unwrap();
    let directory = records.child("roots").unwrap();
    media.artifact_tree().create_directory(&directory).unwrap();
    let address = RecordArtifactFile::ReleaseCustodyHeadBlock {
        generation: 7,
        block: 2,
    };
    let input = b"actual addressed head read bytes";
    media
        .artifact_tree()
        .write_new(&directory.file(&address.file_name()).unwrap(), input)
        .unwrap();
    let before = media.counters();
    let mut observation = media.bounded_record_observation(4, 128).unwrap();
    assert_eq!(observation.store_identity(), media.store_identity());
    let mut storage = Storage::default();
    let head = |block| {
        ArtifactCeiling::page(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            PageAddress::ReleaseCustodyHeadBlock {
                generation: 7,
                block,
            },
        )
    };
    let whole = observation
        .read_with_storage(head(2), ReadGrant::ceiling_only(), &mut storage)
        .observed()
        .unwrap();
    assert_eq!(whole.bytes(), Some(input.as_slice()));
    let range = observation
        .read_record_artifact_range_with_storage(
            address,
            7,
            9,
            ReadGrant::ceiling_only(),
            &mut storage,
        )
        .observed()
        .unwrap();
    assert_eq!(range.bytes(), Some(&input[7..16]));
    let absent = observation
        .read_with_storage(head(3), ReadGrant::ceiling_only(), &mut storage)
        .observed()
        .unwrap();
    assert!(absent.bytes().is_none());
    assert_eq!(observation.counters().bytes_read, input.len() as u64 + 9);
    assert_eq!(observation.counters().addressed_artifacts_read, 2);
    assert_eq!(storage.payload_calls, 2);
    let after = media.counters();
    for role in [
        MediaOperationRole::CreateNew,
        MediaOperationRole::PositionedWrite,
        MediaOperationRole::Truncate,
        MediaOperationRole::Delete,
        MediaOperationRole::AtomicReplace,
    ] {
        assert_eq!(after.attempts_for(role), before.attempts_for(role));
    }
    assert_eq!(
        std::fs::read(
            root.path()
                .join("store/families/records/roots")
                .join(address.file_name())
        )
        .unwrap(),
        input
    );
    drop(observation);
    media.close();
}

fn qualified() -> (tempfile::TempDir, QualifiedFilesystemMedia) {
    let root = tempfile::tempdir().unwrap();
    let request = FilesystemQualificationRequest::certification(
        root.path().join("store"),
        FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let TransitionOutcome::Success(media) = FilesystemMediaOwner::qualify(request).into_raw()
    else {
        panic!("real qualified media must initialize");
    };
    (root, media)
}
