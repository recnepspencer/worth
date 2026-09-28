use super::super::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

#[test]
fn arena_creation_extension_and_reuse_preserve_every_outside_byte() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    let media = match FilesystemMediaOwner::qualify(FilesystemQualificationRequest::certification(
        &root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    ))
    .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("qualified real filesystem owner required"),
    };
    let tree = media.artifact_tree();
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    tree.create_directory(&records).unwrap();
    let arenas = records.child("arenas").unwrap();
    tree.create_directory(&arenas).unwrap();
    let logical = RecordArtifactFile::ExtentArena { arena: 1 };
    let physical = arenas.file(&logical.file_name()).unwrap();
    let creation = match tree.write_new_exact(
        &physical,
        ArtifactNewWriteRange::at(16, 8).unwrap(),
        b"original",
    ) {
        ArtifactNewWriteOutcome::Completed(receipt) => receipt,
        failure => panic!("nonzero first frame creation: {failure:?}"),
    };
    assert_eq!(creation.range().offset(), 16);
    assert_ne!(creation.create_operation(), creation.write_operation());
    for (offset, bytes) in [(4, &b"HEAD"[..]), (20, &b"TAIL"[..]), (32, &b"next"[..])] {
        let coordinate = RecordFrameCoordinate::new(logical, offset, bytes.len() as u32).unwrap();
        let receipt = match tree.write_arena_range_exact_at(
            &physical,
            coordinate,
            bytes,
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ) {
            ArtifactRangeWriteOutcome::Completed(receipt) => receipt,
            failure => panic!("exact arena write: {failure:?}"),
        };
        assert_eq!(receipt.coordinate(), coordinate);
        assert_eq!(receipt.completed_bytes(), bytes.len() as u64);
    }
    let expected = [
        vec![0; 4],
        b"HEAD".to_vec(),
        vec![0; 8],
        b"origTAIL".to_vec(),
        vec![0; 8],
        b"next".to_vec(),
    ]
    .concat();
    assert_eq!(
        std::fs::read(
            root.join("families/records/arenas")
                .join(logical.file_name())
        )
        .unwrap(),
        expected
    );
    let foreign = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 1,
        },
        0,
        4,
    )
    .unwrap();
    assert!(matches!(
        tree.write_arena_range_exact_at(
            &physical,
            foreign,
            b"deny",
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite
        ),
        ArtifactRangeWriteOutcome::DeniedBeforeEffect(_)
    ));
    assert_eq!(tree.read_bounded(&physical, 64).unwrap(), expected);
    media.close();
}
