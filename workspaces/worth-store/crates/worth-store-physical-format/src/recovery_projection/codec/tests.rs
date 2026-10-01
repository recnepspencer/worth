use super::*;
use crate::{
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PhysicalExtentCopyIntent,
    PhysicalExtentCopyRecord, PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
};
use sha2::{Digest, Sha256};

fn extent_projection() -> PersistedPhysicalRecoveryProjection {
    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement = DurableExtentRecordPlacement::legacy_unknown(record, extent, 1, range).unwrap();
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 0, 1).unwrap();
    let subject = PersistedPhysicalDataFrameSubject::ExtentChunk(
        ExtentChunkCoordinate::new(record, extent, 1, 0, 1).unwrap(),
    );
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap();
    PersistedPhysicalRecoveryProjection::new(
        11,
        root,
        vec![record],
        vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
    )
    .unwrap()
}

fn limits() -> PhysicalRecoveryProjectionDecodeLimits {
    PhysicalRecoveryProjectionDecodeLimits {
        frames: 1,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 1,
        inline_allocations: 0,
    }
}

#[test]
fn v5_frames_reencode_exactly_and_v6_semantics_bind_one_extent_record() {
    let mut projection = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let v6 = projection.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&v6, limits(), format),
        Ok(projection.clone())
    );

    projection.version = RecoveryProjectionVersion::V5;
    let v5 = projection.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&v5, limits(), format),
        Ok(projection.clone())
    );
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&v5, limits(), format)
            .unwrap()
            .encode(),
        v5
    );
    assert_ne!(v5, v6);

    let record = projection.record_identities()[0];
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let semantic = PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding);
    let mut declared = extent_projection();
    declared.blob_semantic = semantic;
    let bytes = declared.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(declared)
    );
    let mut frontier = extent_projection();
    frontier.blob_semantic = PersistedPhysicalRecoveryBlobSemantic::SessionFrontier(binding);
    let bytes = frontier.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(frontier)
    );
    let mut abandoned = extent_projection();
    abandoned.blob_semantic = PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(binding);
    let bytes = abandoned.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(abandoned)
    );
    assert!(PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        projection.root_state().clone(),
        vec![record],
        projection.frames().unwrap().to_vec(),
        projection.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::GenerationPublished(
            PersistedBlobSemanticRecordBinding::new(record, [5; 32], 13).unwrap()
        ),
    )
    .is_none());
}

#[test]
fn v7_drop_binding_is_additive_and_v6_cannot_reinterpret_its_tag() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let projected = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding),
    )
    .unwrap();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&projected.encode(), limits(), format),
        Ok(projected.clone())
    );
    let mut mislabeled = projected.encode();
    let v7 = V7_DOMAIN;
    let v6 = V6_DOMAIN;
    assert_eq!(v7.len(), v6.len());
    let domain_offset = 8;
    mislabeled[domain_offset..domain_offset + v6.len()].copy_from_slice(v6);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&mislabeled, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::Malformed)
    );
}

#[test]
fn v8_directory_binding_is_typed_and_cannot_be_reinterpreted_as_v7() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let source = crate::IndexedThroughBlobPublication::new(11, record, [6; 32]).unwrap();
    let projected = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(
            PersistedDerivedDirectoryRecordBinding::new(binding, Some(source)),
        ),
    )
    .unwrap();
    let bytes = projected.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projected),
    );
    let mut mislabeled = bytes;
    mislabeled[8..8 + V7_DOMAIN.len()].copy_from_slice(V7_DOMAIN);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&mislabeled, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn v10_directory_retirement_binds_exact_prior_record_and_bounded_drops() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let prior_record = PersistedRecordIdentity::new([7; 16], 8).unwrap();
    let source = crate::IndexedThroughBlobPublication::new(11, record, [6; 32]).unwrap();
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let base = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(
            PersistedDerivedDirectoryRecordBinding::new(binding, Some(source)),
        ),
    )
    .unwrap();
    let predecessor = crate::DerivedFamilyRootDirectoryBinding::new(prior_record, Some(source));
    assert!(base
        .clone()
        .with_derived_retirement(Some(predecessor), vec![])
        .is_none());
    assert!(base
        .clone()
        .with_derived_retirement(Some(predecessor), vec![prior_record, prior_record])
        .is_none());
    let projected = base
        .with_derived_retirement(Some(predecessor), vec![prior_record])
        .unwrap();
    let bytes = projected.encode();
    let mut admitted_limits = limits();
    admitted_limits.total_entries = 2;
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, admitted_limits, format),
        Ok(projected),
    );
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::EntryLimit),
    );
    let encoded_prior_record = [
        prior_record.allocation_epoch().as_slice(),
        &prior_record.ordinal().to_le_bytes(),
    ]
    .concat();
    let predecessor_offset = bytes
        .windows(encoded_prior_record.len())
        .position(|window| window == encoded_prior_record)
        .expect("encoded predecessor appears in V10 projection");
    let mut malformed = bytes.clone();
    malformed[predecessor_offset - 1] = 2;
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&malformed, admitted_limits, format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
    malformed = bytes;
    malformed[predecessor_offset + encoded_prior_record.len()] = 2;
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&malformed, admitted_limits, format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn v12_directory_binds_quarantine_marker_even_when_retirement_is_empty() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let marker = PersistedRecordIdentity::new([7; 16], 8).unwrap();
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let projected = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(
            PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
                binding,
                None,
                Some(marker),
            ),
        ),
    )
    .unwrap()
    .with_derived_retirement(None, vec![])
    .unwrap();
    let bytes = projected.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projected)
    );
    let mut mislabeled = bytes;
    mislabeled[8..8 + V10_DOMAIN.len()].copy_from_slice(V10_DOMAIN);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&mislabeled, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn v9_reuse_claim_binding_is_typed_and_cannot_be_reinterpreted_as_v8() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let projected = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::ChunkReused(binding),
    )
    .unwrap();
    let bytes = projected.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projected),
    );
    let mut mislabeled = bytes;
    mislabeled[8..8 + V8_DOMAIN.len()].copy_from_slice(V8_DOMAIN);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&mislabeled, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn v11_quarantine_marker_is_typed_and_cannot_be_reinterpreted_as_v10() {
    let old = extent_projection();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = old.record_identities()[0];
    let binding = PersistedBlobSemanticRecordBinding::new(record, [5; 32], 12).unwrap();
    let projected = PersistedPhysicalRecoveryProjection::new_with_blob_semantic(
        11,
        old.root_state().clone(),
        vec![record],
        old.frames().unwrap().to_vec(),
        old.placements().to_vec(),
        vec![],
        vec![],
        PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(binding),
    )
    .unwrap();
    let bytes = projected.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projected)
    );
    let mut mislabeled = bytes;
    mislabeled[8..8 + V10_DOMAIN.len()].copy_from_slice(V10_DOMAIN);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&mislabeled, limits(), format),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn v5_source_copy_and_v6_source_copy_preserve_distinct_payload_variant() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source = DurableExtentRecordPlacement::legacy_unknown(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
        40_000,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
    )
    .unwrap();
    let destination = ExtentArenaRange::new(ExtentArenaId::new(8).unwrap(), 0, 53_248).unwrap();
    let intent =
        PhysicalExtentCopyIntent::new(format, [19; 32], 12, source, destination, 4096, [23; 32])
            .unwrap();
    let digest = Sha256::digest(PhysicalExtentCopyRecord::Intent(intent).encode()).into();
    let recipe = PersistedExtentCopyRecipe::new(intent, 41, digest).unwrap();
    let root = PersistedPhysicalRecoveryRootState::new(65_536, 1, 32, vec![], None, None).unwrap();
    let mut projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, root, recipe).unwrap();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&projection.encode(), limits(), format),
        Ok(projection.clone())
    );
    projection.version = RecoveryProjectionVersion::V5;
    let bytes = projection.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projection)
    );
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format)
            .unwrap()
            .encode(),
        bytes
    );
}
