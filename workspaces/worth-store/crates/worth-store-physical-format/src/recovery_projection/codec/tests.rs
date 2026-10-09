use super::*;
use crate::{
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PhysicalExtentCopyIntent,
    PhysicalExtentCopyRecord, PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
};
use sha2::{Digest, Sha256};

fn extent_projection(
    operation: PersistedPhysicalRecoveryOperation,
) -> Option<PersistedPhysicalRecoveryProjection> {
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
    PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        root,
        vec![record],
        vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        operation,
    )
}

fn limits() -> PhysicalRecoveryProjectionDecodeLimits {
    PhysicalRecoveryProjectionDecodeLimits {
        frames: 1,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 2,
        inline_allocations: 0,
    }
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn binding() -> PersistedBlobSemanticRecordBinding {
    PersistedBlobSemanticRecordBinding::new(
        PersistedRecordIdentity::new([7; 16], 9).unwrap(),
        [5; 32],
        12,
    )
    .unwrap()
}

fn with_domain(encoded: &[u8], domain: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    field(&mut result, domain);
    result.extend_from_slice(&encoded[8 + CURRENT_RECOVERY_PROJECTION_DOMAIN.len()..]);
    result
}

#[test]
fn current_domain_roundtrips_frames_and_rejects_retired_domains_before_body() {
    let projection = extent_projection(PersistedPhysicalRecoveryOperation::None).unwrap();
    let bytes = projection.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format()),
        Ok(projection),
    );
    for version in 1..=15 {
        let retired = with_domain(
            &bytes,
            format!("store.physical.recovery-projection.v{version}").as_bytes(),
        );
        assert_eq!(
            PersistedPhysicalRecoveryProjection::decode(&retired, limits(), format()),
            Err(PhysicalRecoveryProjectionDenial::UnsupportedVersion(
                version
            )),
        );
    }
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(
            &with_domain(&bytes, b"not-a-recovery-projection"),
            limits(),
            format(),
        ),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn current_operation_variants_preserve_exact_binding_and_explicit_unknown_route() {
    let binding = binding();
    let operations = [
        PersistedPhysicalRecoveryOperation::SessionDeclared(binding),
        PersistedPhysicalRecoveryOperation::GenerationPublished(binding),
        PersistedPhysicalRecoveryOperation::SessionFrontier(binding),
        PersistedPhysicalRecoveryOperation::SessionAbandoned(binding),
        PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: None,
            directory_replacement: None,
        },
        PersistedPhysicalRecoveryOperation::ChunkReused(binding),
        PersistedPhysicalRecoveryOperation::DedupeQuarantined(binding),
    ];
    for operation in operations {
        let projection = extent_projection(operation).unwrap();
        let bytes = projection.encode();
        let decoded =
            PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format()).unwrap();
        assert_eq!(decoded, projection);
        assert!(decoded.placements()[0].route_metadata().is_legacy_unknown());
    }
    let wrong = PersistedBlobSemanticRecordBinding::new(binding.record(), [5; 32], 13).unwrap();
    assert!(
        extent_projection(PersistedPhysicalRecoveryOperation::GenerationPublished(
            wrong
        ))
        .is_none()
    );
}

#[test]
fn directory_quarantine_tristate_and_retirement_are_current_operation_facts() {
    let record = binding().record();
    let prior = PersistedRecordIdentity::new([7; 16], 8).unwrap();
    let source = crate::IndexedThroughBlobPublication::new(11, record, [6; 32]).unwrap();
    let predecessor = crate::DerivedFamilyRootDirectoryBinding::new(prior, Some(source));
    assert!(PersistedDerivedDirectoryRetirement::new(Some(predecessor), vec![]).is_none());
    assert!(
        PersistedDerivedDirectoryRetirement::new(Some(predecessor), vec![prior, prior]).is_none()
    );
    let retirement =
        PersistedDerivedDirectoryRetirement::new(Some(predecessor), vec![prior]).unwrap();
    for directory in [
        PersistedDerivedDirectoryRecordBinding::new(binding(), Some(source)),
        PersistedDerivedDirectoryRecordBinding::new_with_quarantine(binding(), Some(source), None),
        PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
            binding(),
            Some(source),
            Some(prior),
        ),
    ] {
        let operation = PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: directory,
            retirement: Some(retirement.clone()),
        };
        let projection = extent_projection(operation).unwrap();
        assert_eq!(
            PersistedPhysicalRecoveryProjection::decode(&projection.encode(), limits(), format()),
            Ok(projection),
        );
    }
    let overlapping = PersistedDerivedDirectoryRetirement::new(None, vec![record]).unwrap();
    assert!(
        extent_projection(PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new(binding(), None),
            retirement: Some(overlapping),
        })
        .is_none()
    );
}

#[test]
fn variant_attachment_bytes_cannot_be_reinterpreted_as_other_operations() {
    let prior = PersistedRecordIdentity::new([7; 16], 8).unwrap();
    let retirement = PersistedDerivedDirectoryRetirement::new(None, vec![prior]).unwrap();
    let operation = PersistedPhysicalRecoveryOperation::DerivedDirectory {
        binding: PersistedDerivedDirectoryRecordBinding::new(binding(), None),
        retirement: Some(retirement),
    };
    let projection = extent_projection(operation.clone()).unwrap();
    let mut encoded = projection.encode();
    let mut operation_wire = Vec::new();
    write_operation(&mut operation_wire, &operation);
    let tag_at = encoded.len() - operation_wire.len() + 8;
    assert_eq!(encoded[tag_at], 6);
    encoded[tag_at] = 5;
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&encoded, limits(), format()),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
    let mut encoded = projection.encode();
    encoded.push(0);
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&encoded, limits(), format()),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}

#[test]
fn current_source_copy_roundtrips_without_operation_or_frame_format_default() {
    let format = format();
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
    let projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, root, recipe).unwrap();
    let bytes = projection.encode();
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits(), format),
        Ok(projection),
    );
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode_frames(&bytes, limits()),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );
}
