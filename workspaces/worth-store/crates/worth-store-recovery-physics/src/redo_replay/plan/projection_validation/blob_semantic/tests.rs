use super::*;
use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobReclaimDescriptorV2, BlobReclaimSourceKind,
    BlobSessionAbandonedV1, BlobSessionDeclarationV1, BlobSessionFrontierV1,
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryV1, DurableExtentRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, IndexedThroughBlobPublication,
    PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryRootState, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
};

fn declaration() -> Vec<u8> {
    BlobSessionDeclarationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        [4; 32],
        64 << 10,
        100_000,
        1 << 20,
        12,
    )
    .unwrap()
    .encode()
}

#[test]
fn released_drop_replay_requires_exact_selected_descriptor_binding() {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let manifest = PersistedRecordIdentity::new([7; 16], 8).unwrap();
    let bytes = BlobReclaimDescriptorV2::new(
        [1; 16],
        [2; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        [3; 32],
        manifest,
        [4; 32],
        2,
        11,
        12,
        None,
        2,
        false,
    )
    .unwrap()
    .encode();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let selected = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: None,
        },
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &selected),
        Ok(())
    );
    assert_eq!(
        validate_blob_semantic(
            &bytes,
            record,
            [1; 16],
            &projection(&bytes, PersistedPhysicalRecoveryOperation::None),
        ),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    let wrong = PersistedBlobSemanticRecordBinding::new(record, [9; 32], 12).unwrap();
    assert_eq!(
        validate_blob_semantic(
            &bytes,
            record,
            [1; 16],
            &projection(
                &bytes,
                PersistedPhysicalRecoveryOperation::RecordsDropped {
                    binding: wrong,
                    head_effect: None,
                }
            ),
        ),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
}

#[test]
fn frontier_replay_requires_exact_typed_operation_binding() {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let frontier = BlobSessionFrontierV1::new(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [5; 32],
        1,
        64 << 10,
        PersistedRecordIdentity::new([6; 16], 8).unwrap(),
        [8; 32],
    )
    .unwrap();
    let bytes = frontier.encode();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let selected = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::SessionFrontier(binding),
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &selected),
        Ok(())
    );
    let omitted = projection(&bytes, PersistedPhysicalRecoveryOperation::None);
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &omitted),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    let mislabeled = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::GenerationPublished(binding),
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &mislabeled),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}

#[test]
fn abandonment_replay_requires_exact_typed_operation_binding() {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let abandoned = BlobSessionAbandonedV1::new(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [5; 32],
        BlobAbandonmentReasonV1::ExplicitAbort,
    )
    .unwrap();
    let bytes = abandoned.encode();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let selected = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::SessionAbandoned(binding),
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &selected),
        Ok(())
    );
    let expired = BlobSessionAbandonedV1::new(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [5; 32],
        BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence: std::num::NonZeroU64::new(17).unwrap(),
        },
    )
    .unwrap()
    .encode();
    let expiry_binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&expired).into(), 12)
            .unwrap();
    assert_eq!(
        validate_blob_semantic(
            &expired,
            record,
            [1; 16],
            &projection(
                &expired,
                PersistedPhysicalRecoveryOperation::SessionAbandoned(expiry_binding)
            )
        ),
        Ok(())
    );
    for other in [
        PersistedPhysicalRecoveryOperation::None,
        PersistedPhysicalRecoveryOperation::SessionFrontier(binding),
    ] {
        assert_eq!(
            validate_blob_semantic(&bytes, record, [1; 16], &projection(&bytes, other)),
            Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
        );
    }
    assert_eq!(
        validate_blob_semantic(&bytes, record, [8; 16], &selected),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    let wrong_digest = PersistedBlobSemanticRecordBinding::new(record, [9; 32], 12).unwrap();
    assert_eq!(
        validate_blob_semantic(
            &bytes,
            record,
            [1; 16],
            &projection(
                &bytes,
                PersistedPhysicalRecoveryOperation::SessionAbandoned(wrong_digest)
            )
        ),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}

fn projection(
    bytes: &[u8],
    semantic: PersistedPhysicalRecoveryOperation,
) -> PersistedPhysicalRecoveryProjection {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement =
        DurableExtentRecordPlacement::legacy_unknown(record, extent, bytes.len() as u64, range)
            .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(
            ExtentChunkCoordinate::new(record, extent, bytes.len() as u64, 0, bytes.len() as u32)
                .unwrap(),
        ),
        RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena { arena: 2 },
            0,
            bytes.len() as u32,
        )
        .unwrap(),
        bytes,
    )
    .unwrap();
    PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![record],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        semantic,
    )
    .unwrap()
}

#[test]
fn directory_replay_requires_typed_operation_and_exact_payload_source() {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let source = IndexedThroughBlobPublication::new(11, record, [4; 32]).unwrap();
    let bytes = DerivedFamilyRootDirectoryV1::new(vec![])
        .unwrap()
        .with_indexed_through(source)
        .encode();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let selected = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new(binding, Some(source)),
            retirement: None,
        },
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &selected),
        Ok(())
    );
    let omitted = projection(&bytes, PersistedPhysicalRecoveryOperation::None);
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &omitted),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    let wrong_source = IndexedThroughBlobPublication::new(11, record, [5; 32]).unwrap();
    let mislabeled = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new(binding, Some(wrong_source)),
            retirement: None,
        },
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &mislabeled),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
}

#[test]
fn directory_replay_rejects_hash_valid_mismatched_quarantine_marker() {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let marker = PersistedRecordIdentity::new([7; 16], 10).unwrap();
    let bytes = DerivedFamilyRootDirectoryV1::new(vec![])
        .unwrap()
        .with_indexed_through_quarantine(Some(marker))
        .encode();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let correct = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                binding,
                None,
                Some(marker),
            ),
            retirement: None,
        },
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &correct),
        Ok(())
    );
    let mismatched = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                binding, None, None,
            ),
            retirement: None,
        },
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &mismatched),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
}

#[test]
fn declaration_replay_requires_exact_record_payload_and_successor_binding() {
    let bytes = declaration();
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let binding =
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&bytes).into(), 12).unwrap();
    let selected = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::SessionDeclared(binding),
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &selected),
        Ok(())
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [8; 16], &selected),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    assert_eq!(
        validate_blob_semantic(
            &bytes,
            PersistedRecordIdentity::new([8; 16], 9).unwrap(),
            [1; 16],
            &selected
        ),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    let mut altered = bytes.clone();
    altered[16] ^= 1;
    assert_eq!(
        validate_blob_semantic(&altered, record, [1; 16], &selected),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    let wrong_digest = PersistedBlobSemanticRecordBinding::new(record, [9; 32], 12).unwrap();
    let wrong = projection(
        &bytes,
        PersistedPhysicalRecoveryOperation::SessionDeclared(wrong_digest),
    );
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &wrong),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
    let omitted = projection(&bytes, PersistedPhysicalRecoveryOperation::None);
    assert_eq!(
        validate_blob_semantic(&bytes, record, [1; 16], &omitted),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
    );
}
