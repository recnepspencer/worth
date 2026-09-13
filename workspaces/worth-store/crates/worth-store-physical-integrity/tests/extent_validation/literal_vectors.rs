use worth_store_physical_format::integrity_declarations::{
    families::{EXTENT_CHUNK_INTEGRITY_DECLARATION, EXTENT_MANIFEST_INTEGRITY_DECLARATION},
    PhysicalIntegrityCoverageBoundary,
};
use worth_store_physical_format::{
    encode_extent_chunk, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentChunkCoordinate, PhysicalPageSizeClass,
};
use worth_store_physical_integrity::{
    validate_extent_chunk, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::support::{
    chunk_scope, extent_cell, format, independent_crc32c, manifest_scope, record, store,
};

const MANIFEST_CRC32C: u32 = 0xa82f_6ba8;
const CHUNK_CRC32C: u32 = 0x3adf_7d7a;

const MANIFEST_VECTOR: [u8; 104] = [
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x06, 0x02, 0x01, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa8, 0x6b, 0x2f, 0xa8,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const CHUNK_VECTOR: [u8; 118] = [
    0x57, 0x52, 0x43, 0x35, 0x46, 0x52, 0x4d, 0x00, 0x04, 0x02, 0x01, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x01, 0x01, 0x01, 0x18, 0x30, 0x00, 0x00, 0x00, 0x46, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7a, 0x7d, 0xdf, 0x3a,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x43, 0x39, 0x45, 0x58, 0x54, 0x21,
];

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
            PhysicalIntegrityCoverageBoundary::Fixed(0)
        );
        assert_eq!(
            checksum.covered_ranges()[0].end(),
            PhysicalIntegrityCoverageBoundary::Fixed(44)
        );
        assert_eq!(
            checksum.covered_ranges()[1].start(),
            PhysicalIntegrityCoverageBoundary::Fixed(48)
        );
        assert_eq!(
            checksum.covered_ranges()[1].end(),
            PhysicalIntegrityCoverageBoundary::ArtifactEnd
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

    let format = format(PhysicalPageSizeClass::KiB16);
    let record = record(0x22, 7);
    let extent = extent_cell(4, 5);
    let manifest = DurableExtentManifest::new(format, record, extent, 6, 16_384, 1).unwrap();
    let coordinate = ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    assert_eq!(manifest.encode(format), MANIFEST_VECTOR);
    assert_eq!(
        encode_extent_chunk(format, coordinate, b"C9EXT!").unwrap(),
        CHUNK_VECTOR
    );
}

#[test]
fn runtime_validators_consume_the_independent_literal_extent_vectors() {
    let store = store(9);
    let format = format(PhysicalPageSizeClass::KiB16);
    let record = record(0x22, 7);
    let extent = extent_cell(4, 5);
    let placement = DurableExtentRecordPlacement::new(record, extent, 6).unwrap();
    let manifest_scope = manifest_scope(store, format, placement, MANIFEST_VECTOR.len() as u64);
    let (ExtentManifestIntegrityValidation::Intact(manifest), _) = validate_extent_manifest(
        UntrustedPhysicalArtifact::from_bounded_bytes(&MANIFEST_VECTOR),
        manifest_scope,
    ) else {
        panic!("literal extent manifest rejected")
    };
    let coordinate = ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    let chunk_scope = chunk_scope(store, format, coordinate, CHUNK_VECTOR.len() as u64);
    let (ExtentChunkIntegrityValidation::Intact(chunk), _) = validate_extent_chunk(
        UntrustedPhysicalArtifact::from_bounded_bytes(&CHUNK_VECTOR),
        chunk_scope,
        &manifest,
    ) else {
        panic!("literal extent chunk rejected")
    };
    assert_eq!(chunk.logical_bytes(), 6);
    assert_eq!(chunk.logical_offset(), 0);
    assert_eq!(chunk.ordinal(), 1);
}
