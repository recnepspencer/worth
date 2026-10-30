#[allow(unused_imports)]
use crate::extent_validation::literal_vectors::*;

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
    let manifest = worth_store_physical_format::DurableExtentManifest::new(
        format, record, extent, 6, 16_384, 1, 16_384,
    )
    .unwrap();
    let coordinate =
        worth_store_physical_format::ExtentChunkCoordinate::new(record, extent, 6, 0, 1).unwrap();
    assert_eq!(manifest.encode(format), MANIFEST_VECTOR);
    assert_eq!(
        encode_extent_chunk(format, coordinate, b"C9EXT!").unwrap(),
        CHUNK_VECTOR
    );
}
