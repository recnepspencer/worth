#[allow(unused_imports)]
use crate::extent_validation::literal_vectors::*;

#[test]
fn runtime_validators_consume_the_independent_literal_extent_vectors() {
    let store = store(9);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let record = record(0x22, 7);
    let extent = extent_cell(4, 5);
    let arena_range = crate::support::extent_validation::literal_arena_range(format);
    let placement = worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
        record,
        extent,
        6,
        arena_range,
    )
    .unwrap();
    let manifest_scope = manifest_scope(store, format, placement, MANIFEST_VECTOR.len() as u64);
    let (worth_store_physical_integrity::ExtentManifestIntegrityValidation::Intact(manifest), _) =
        validate_extent_manifest(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &MANIFEST_VECTOR,
            ),
            manifest_scope,
        )
    else {
        panic!("literal extent manifest rejected")
    };
    let coordinate =
        worth_store_physical_format::ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    let chunk_scope = chunk_scope(store, format, coordinate, CHUNK_VECTOR.len() as u64);
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(chunk), _) =
        validate_extent_chunk(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &CHUNK_VECTOR,
            ),
            chunk_scope,
            &manifest,
        )
    else {
        panic!("literal extent chunk rejected")
    };
    assert_eq!(chunk.logical_bytes(), 6);
    assert_eq!(chunk.logical_offset(), 0);
    assert_eq!(chunk.ordinal(), 1);
}
