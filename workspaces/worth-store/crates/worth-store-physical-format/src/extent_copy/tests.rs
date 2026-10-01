use super::*;
use crate::{ExtentArenaId, PersistedRecordIdentity, PhysicalExtentId};

fn fixture() -> (PhysicalRecordFormatDeclaration, PhysicalExtentCopyIntent) {
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
    (
        format,
        PhysicalExtentCopyIntent::new(format, [19; 32], 12, source, destination, 4096, [23; 32])
            .unwrap(),
    )
}

#[test]
fn intent_wire_binds_geometry_without_claiming_final_root_or_lsn() {
    let (format, intent) = fixture();
    let bytes = PhysicalExtentCopyRecord::Intent(intent).encode();
    assert!(payload_is_extent_copy_any(&bytes));
    assert!(!payload_is_extent_copy_any(
        b"store.physical.extent-copy.v3"
    ));
    let body = &bytes[EXTENT_COPY_DOMAIN.len() + 1..];
    assert_eq!(body.len(), 200);
    assert_eq!(&body[32..40], &12_u64.to_le_bytes());
    assert_eq!(&body[72..80], &3_u64.to_le_bytes());
    assert_eq!(&body[80..88], &4_u64.to_le_bytes());
    assert_eq!(&body[88..96], &5_u64.to_le_bytes());
    assert_eq!(&body[96..104], &2_u64.to_le_bytes());
    assert_eq!(&body[120..128], &8_u64.to_le_bytes());
    assert_eq!(&body[144..152], &40_000_u64.to_le_bytes());
    assert_eq!(&body[164..168], &3_u32.to_le_bytes());
    assert_eq!(
        PhysicalExtentCopyRecord::decode(&bytes, format),
        Ok(PhysicalExtentCopyRecord::Intent(intent))
    );
    for offset in [88, 112, 152, 160, 164] {
        let mut damaged = bytes.clone();
        damaged[EXTENT_COPY_DOMAIN.len() + 1 + offset] ^= 1;
        assert!(
            PhysicalExtentCopyRecord::decode(&damaged, format).is_err(),
            "offset {offset}"
        );
    }
    let mut empty = bytes;
    let logical = EXTENT_COPY_DOMAIN.len() + 1 + 144;
    empty[logical..logical + 8].fill(0);
    assert!(PhysicalExtentCopyRecord::decode(&empty, format).is_err());
}

#[test]
fn copy_cannot_reserve_its_source_arena_or_reinterpret_trailing_bytes() {
    let (format, intent) = fixture();
    let same_arena =
        ExtentArenaRange::new(intent.source.arena_range().arena(), 106_496, 53_248).unwrap();
    assert!(PhysicalExtentCopyIntent::new(
        format,
        [19; 32],
        12,
        intent.source,
        same_arena,
        4096,
        [23; 32]
    )
    .is_none());
    let mut bytes = PhysicalExtentCopyRecord::Intent(intent).encode();
    bytes.push(0);
    assert_eq!(
        PhysicalExtentCopyRecord::decode(&bytes, format),
        Err(PhysicalExtentCopyDenial::Length)
    );
}

#[test]
fn resolution_separates_intent_lsn_from_later_publication_lsn() {
    let (format, _) = fixture();
    for kind in [
        PhysicalExtentCopyResolutionKind::Cancelled,
        PhysicalExtentCopyResolutionKind::Published {
            root_generation: 30,
            publication_lsn: 42,
        },
    ] {
        let resolution = PhysicalExtentCopyResolution::new([19; 32], [29; 32], 41, kind).unwrap();
        let encoded = PhysicalExtentCopyRecord::Resolved(resolution).encode();
        assert_eq!(encoded.len(), EXTENT_COPY_DOMAIN.len() + 90);
        assert_eq!(
            PhysicalExtentCopyRecord::decode(&encoded, format),
            Ok(PhysicalExtentCopyRecord::Resolved(resolution))
        );
    }
    assert!(PhysicalExtentCopyResolution::new(
        [19; 32],
        [29; 32],
        41,
        PhysicalExtentCopyResolutionKind::Published {
            root_generation: 30,
            publication_lsn: 41
        }
    )
    .is_none());
}

#[test]
fn source_copy_projection_has_explicit_recipe_and_no_fake_frame_set() {
    use crate::{
        PersistedExtentCopyRecipe, PersistedPhysicalRecoveryPayload,
        PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
        PhysicalRecoveryProjectionDecodeLimits,
    };
    use sha2::{Digest, Sha256};
    let (format, intent) = fixture();
    let digest = Sha256::digest(PhysicalExtentCopyRecord::Intent(intent).encode()).into();
    let recipe = PersistedExtentCopyRecipe::new(intent, 41, digest).unwrap();
    assert!(PersistedExtentCopyRecipe::new(intent, 41, [0; 32]).is_none());
    assert!(PersistedExtentCopyRecipe::new(intent, 0, digest).is_none());
    let root = PersistedPhysicalRecoveryRootState::new(65_536, 1, 32, vec![], None, None).unwrap();
    assert!(
        PersistedPhysicalRecoveryProjection::from_source_copy(11, root.clone(), recipe).is_none()
    );
    let projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, root, recipe).unwrap();
    assert!(projection.frames().is_none());
    assert_eq!(
        projection.payload(),
        &PersistedPhysicalRecoveryPayload::SourceCopy(recipe)
    );
    let bytes = projection.encode();
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 0,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 1,
        inline_allocations: 0,
    };
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits, format),
        Ok(projection)
    );
    let wrong_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(crate::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    assert!(PersistedPhysicalRecoveryProjection::decode(&bytes, limits, wrong_format).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(PersistedPhysicalRecoveryProjection::decode(&trailing, limits, format).is_err());
}

#[test]
fn classified_copy_binds_route_class_in_intent_and_recovery_projection() {
    use crate::{
        BlobRecordKind, PersistedExtentCopyRecipe, PersistedPhysicalRecoveryProjection,
        PersistedPhysicalRecoveryRootState, PhysicalRecoveryProjectionDecodeLimits,
        SelectedRecordContentClass, SelectedRecordRouteMetadata,
    };
    use sha2::{Digest, Sha256};

    let (format, legacy) = fixture();
    let metadata = SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Blob(
        BlobRecordKind::Chunk,
    ))
    .unwrap();
    let source = DurableExtentRecordPlacement::new_selected(
        legacy.source().record(),
        legacy.source().extent_cell(),
        legacy.source().payload_bytes(),
        legacy.source().arena_range(),
        metadata,
    )
    .unwrap();
    let intent = PhysicalExtentCopyIntent::new(
        format,
        legacy.operation(),
        legacy.source_root(),
        source,
        legacy.destination().arena_range(),
        legacy.alignment(),
        legacy.source_digest(),
    )
    .unwrap();
    assert_eq!(intent.destination().route_metadata(), metadata);
    let encoded = PhysicalExtentCopyRecord::Intent(intent).encode();
    assert!(encoded.starts_with(EXTENT_COPY_V2_DOMAIN));
    assert!(payload_is_extent_copy_any(&encoded));
    assert_eq!(
        PhysicalExtentCopyRecord::decode(&encoded, format),
        Ok(PhysicalExtentCopyRecord::Intent(intent))
    );
    let mut forged_tier = encoded.clone();
    let destination_tier_offset = forged_tier.len() - 3;
    forged_tier[destination_tier_offset] = 0xff;
    assert_eq!(
        PhysicalExtentCopyRecord::decode(&forged_tier, format),
        Err(PhysicalExtentCopyDenial::Identity)
    );
    let cold = PhysicalExtentCopyIntent::new_with_target_tier(
        format,
        legacy.operation(),
        legacy.source_root(),
        source,
        legacy.destination().arena_range(),
        legacy.alignment(),
        legacy.source_digest(),
        crate::PhysicalTierClass::Cold,
    )
    .unwrap();
    assert_eq!(cold.source().route_metadata(), metadata);
    assert_eq!(
        cold.destination().tier_class(),
        crate::PhysicalTierClass::Cold
    );
    let cold_encoded = PhysicalExtentCopyRecord::Intent(cold).encode();
    assert_eq!(
        PhysicalExtentCopyRecord::decode(&cold_encoded, format),
        Ok(PhysicalExtentCopyRecord::Intent(cold))
    );
    assert_ne!(cold_encoded, encoded);
    let cold_recipe =
        PersistedExtentCopyRecipe::new(cold, 41, Sha256::digest(&cold_encoded).into()).unwrap();
    let cold_root =
        PersistedPhysicalRecoveryRootState::new(65_536, 1, 32, vec![], None, None).unwrap();
    let cold_projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, cold_root, cold_recipe).unwrap();
    let cold_limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 0,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 1,
        inline_allocations: 0,
    };
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&cold_projection.encode(), cold_limits, format,),
        Ok(cold_projection)
    );
    let digest = Sha256::digest(&encoded).into();
    let recipe = PersistedExtentCopyRecipe::new(intent, 41, digest).unwrap();
    let root = PersistedPhysicalRecoveryRootState::new(65_536, 1, 32, vec![], None, None).unwrap();
    let projection =
        PersistedPhysicalRecoveryProjection::from_source_copy(17, root, recipe).unwrap();
    let bytes = projection.encode();
    assert!(bytes
        .windows(b"store.physical.recovery-projection.v15".len())
        .any(|window| window == b"store.physical.recovery-projection.v15"));
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 0,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 1,
        inline_allocations: 0,
    };
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits, format),
        Ok(projection)
    );
}
