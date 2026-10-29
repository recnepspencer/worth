#[allow(unused_imports)] use crate::extent_validation::literal_vectors::*;
use {worth_store_physical_format::integrity_declarations::families::EXTENT_CHUNK_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::families::EXTENT_MANIFEST_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary};
use {worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PhysicalPageSizeClass};
use {worth_store_physical_integrity::validate_extent_chunk, worth_store_physical_integrity::validate_extent_manifest, worth_store_physical_integrity::ExtentChunkIntegrityValidation, worth_store_physical_integrity::ExtentManifestIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::extent_validation::chunk_scope, crate::support::extent_validation::extent_cell, crate::support::extent_validation::format, crate::support::extent_validation::independent_crc32c, crate::support::extent_validation::manifest_scope, crate::support::extent_validation::record, crate::support::extent_validation::store};

#[test]
fn literal_extent_vectors_freeze_checksum_coverage_and_writer_bytes() {
    for declaration in [
        EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        EXTENT_CHUNK_INTEGRITY_DECLARATION,
    ] {
        let checksum = declaration.checksums()[0];
        assert_eq!(checksum.covered_ranges().len(), 2);
        assert_eq!(
            checksum.covered_ranges()[0].start(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(0)
        );
        assert_eq!(
            checksum.covered_ranges()[0].end(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(44)
        );
        assert_eq!(
            checksum.covered_ranges()[1].start(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(48)
        );
        assert_eq!(
            checksum.covered_ranges()[1].end(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::ArtifactEnd
        );
    }
    assert_eq!(
        independent_crc32c(&[&MANIFEST_VECTOR[..44], &MANIFEST_VECTOR[48..]]),
        MANIFEST_CRC32C
    );
    assert_eq!(
        independent_crc32c(&[&CHUNK_VECTOR[..44], &CHUNK_VECTOR[48..]]),
        CHUNK_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(MANIFEST_VECTOR[44..48].try_into().unwrap()),
        MANIFEST_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(CHUNK_VECTOR[44..48].try_into().unwrap()),
        CHUNK_CRC32C
    );

    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let record = record(0x22, 7);
    let extent = extent_cell(4, 5);
    let manifest =
        worth_store_physical_format::DurableExtentManifest::new(format, record, extent, 6, 16_384, 1, 16_384).unwrap();
    let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    assert_eq!(manifest.encode(format), MANIFEST_VECTOR);
    assert_eq!(
        encode_extent_chunk(format, coordinate, b"C9EXT!").unwrap(),
        CHUNK_VECTOR
    );
}

#[test]
fn runtime_validators_consume_the_independent_literal_extent_vectors() {
    let store = store(9);
    let format = format(worth_store_physical_format::PhysicalPageSizeClass::KiB16);
    let record = record(0x22, 7);
    let extent = extent_cell(4, 5);
    let arena_range = crate::support::extent_validation::literal_arena_range(format);
    let placement =
        worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(record, extent, 6, arena_range).unwrap();
    let manifest_scope = manifest_scope(store, format, placement, MANIFEST_VECTOR.len() as u64);
    let (worth_store_physical_integrity::ExtentManifestIntegrityValidation::Intact(manifest), _) = validate_extent_manifest(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&MANIFEST_VECTOR),
        manifest_scope,
    ) else {
        panic!("literal extent manifest rejected")
    };
    let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    let chunk_scope = chunk_scope(store, format, coordinate, CHUNK_VECTOR.len() as u64);
    let (worth_store_physical_integrity::ExtentChunkIntegrityValidation::Intact(chunk), _) = validate_extent_chunk(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&CHUNK_VECTOR),
        chunk_scope,
        &manifest,
    ) else {
        panic!("literal extent chunk rejected")
    };
    assert_eq!(chunk.logical_bytes(), 6);
    assert_eq!(chunk.logical_offset(), 0);
    assert_eq!(chunk.ordinal(), 1);
}
