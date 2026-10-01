use super::*;
use crate::PhysicalByteRange;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::*;

#[test]
fn routed_arena_geometry_and_generation_constrain_manifest_and_chunk_admission() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
    let record = PersistedRecordIdentity::new([4; 16], 1).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
    let arena = ExtentArenaRange::new(ExtentArenaId::new(9).unwrap(), 4096, 20480).unwrap();
    let placement = DurableExtentRecordPlacement::legacy_unknown(record, extent, 3, arena).unwrap();
    let manifest = DurableExtentManifest::new(format, record, extent, 3, 16384, 1, 4096)
        .unwrap()
        .encode(format);
    let scope = PhysicalArtifactScope::extent_manifest(
        store,
        format,
        placement,
        PhysicalByteRange::new(4096, 104).unwrap(),
    );
    let validation = validate_extent_arena_frame(
        UntrustedPhysicalArtifact::from_bounded_bytes(&manifest),
        ExtentArenaFrameExpectation::Manifest(scope),
    )
    .0;
    let ExtentArenaFrameIntegrityValidation::Manifest(validated) = validation else {
        panic!("valid manifest rejected")
    };
    let membership = validated.membership();
    assert_eq!(membership.frame_layout().chunk_offset(1), Some(4096));
    let evidence = validated.into_validation_record();
    let retained =
        IntegrityValidatedExtentMembership::from_validation_record(evidence, scope).unwrap();
    assert_eq!(retained.alignment(), 4096);
    assert_eq!(retained.arena_range(), arena);

    let coordinate = ExtentChunkCoordinate::new(record, extent, 3, 0, 1).unwrap();
    let chunk = encode_extent_chunk(format, coordinate, b"abc").unwrap();
    let chunk_scope = |offset, range| {
        PhysicalArtifactScope::extent_chunk(
            store,
            format,
            coordinate,
            PhysicalByteRange::new(offset, 115).unwrap(),
            range,
        )
    };
    for (offset, range, intact) in [
        (8192, arena, true),
        (4096, arena, false),
        (
            8192,
            ExtentArenaRange::new(ExtentArenaId::new(10).unwrap(), 4096, 20480).unwrap(),
            false,
        ),
    ] {
        let validation = validate_extent_arena_frame(
            UntrustedPhysicalArtifact::from_bounded_bytes(&chunk),
            ExtentArenaFrameExpectation::Chunk {
                scope: chunk_scope(offset, range),
                membership,
            },
        )
        .0;
        assert_eq!(
            matches!(validation, ExtentArenaFrameIntegrityValidation::Chunk(_)),
            intact
        );
    }
    let newer = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent.extent_id())
        .with_extent_generation(PhysicalGeneration::from_raw(3).unwrap());
    let stale_scope = PhysicalArtifactScope::extent_manifest(
        store,
        format,
        DurableExtentRecordPlacement::legacy_unknown(record, newer, 3, arena).unwrap(),
        scope.byte_range(),
    );
    assert!(matches!(
        validate_extent_arena_frame(
            UntrustedPhysicalArtifact::from_bounded_bytes(&manifest),
            ExtentArenaFrameExpectation::Manifest(stale_scope)
        )
        .0,
        ExtentArenaFrameIntegrityValidation::Rejected(_)
    ));
}
