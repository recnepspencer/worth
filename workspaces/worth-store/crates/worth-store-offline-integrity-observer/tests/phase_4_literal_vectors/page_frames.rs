use worth_store_physical_format::integrity_declarations::{
    families::PAGE_FRAME_INTEGRITY_DECLARATION, PhysicalIntegrityAlgorithm,
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
};

use super::oracle::{
    assert_declaration, assert_literal_vector, DeclarationChecksumExpectation, LiteralChecksum,
    LiteralChecksumExpectation, LiteralVector,
};

const DECLARATION_RANGES: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[
    (
        PhysicalIntegrityCoverageBoundary::Fixed(0),
        PhysicalIntegrityCoverageBoundary::Fixed(44),
    ),
    (
        PhysicalIntegrityCoverageBoundary::Fixed(48),
        PhysicalIntegrityCoverageBoundary::ArtifactEnd,
    ),
];

struct PageCase {
    name: &'static str,
    byte_count: usize,
    format_bytes: [u8; 10],
    checksum: u32,
}

pub(super) fn verify() {
    assert_declaration(
        PAGE_FRAME_INTEGRITY_DECLARATION,
        PhysicalIntegrityArtifactFamily::PageFrame,
        1,
        Some(2),
        &[DeclarationChecksumExpectation {
            algorithm: PhysicalIntegrityAlgorithm::Crc32c,
            ranges: DECLARATION_RANGES,
            field: (
                PhysicalIntegrityCoverageBoundary::Fixed(44),
                PhysicalIntegrityCoverageBoundary::Fixed(48),
            ),
        }],
    );
    for (case, (_, bytes)) in cases().iter().zip(reader_vectors()) {
        let ranges = [(0, 44), (48, bytes.len())];
        let checksums = [LiteralChecksumExpectation {
            checksum: LiteralChecksum::Crc32c(case.checksum),
            ranges: &ranges,
            field: (44, 48),
        }];
        assert_literal_vector(LiteralVector {
            name: case.name,
            bytes,
            checksums: &checksums,
        });
    }
}

fn cases() -> [PageCase; 3] {
    [
        PageCase {
            name: "16 KiB inline page",
            byte_count: 16 * 1024,
            format_bytes: [1, 0, 0, 64, 0, 0, 1, 1, 1, 24],
            checksum: 0xecf1_54f9,
        },
        PageCase {
            name: "32 KiB inline page",
            byte_count: 32 * 1024,
            format_bytes: [1, 0, 0, 128, 0, 0, 1, 1, 1, 24],
            checksum: 0xae22_45ae,
        },
        PageCase {
            name: "64 KiB inline page",
            byte_count: 64 * 1024,
            format_bytes: [1, 0, 0, 0, 1, 0, 1, 1, 1, 24],
            checksum: 0x7d1b_576a,
        },
    ]
}

fn literal_page(case: &PageCase) -> Vec<u8> {
    let payload_bytes = case.byte_count - 48;
    let mut bytes = vec![0_u8; case.byte_count];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = 3;
    bytes[9] = 2;
    bytes[10..20].copy_from_slice(&case.format_bytes);
    bytes[20..22].copy_from_slice(&48_u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&(payload_bytes as u32).to_le_bytes());
    bytes[28..36].copy_from_slice(&11_u64.to_le_bytes());
    bytes[36..44].copy_from_slice(&0x2122_2324_2526_2728_u64.to_le_bytes());
    bytes[44..48].copy_from_slice(&case.checksum.to_le_bytes());
    bytes[48..56].copy_from_slice(&0x0102_0304_0506_0708_u64.to_le_bytes());
    bytes[56..64].copy_from_slice(&0x1112_1314_1516_1718_u64.to_le_bytes());
    bytes[64..66].copy_from_slice(&1_u16.to_le_bytes());
    bytes[72..88].fill(0xab);
    bytes[88..96].copy_from_slice(&7_u64.to_le_bytes());
    bytes[96..100].copy_from_slice(&((payload_bytes - 3) as u32).to_le_bytes());
    bytes[100..104].copy_from_slice(&3_u32.to_le_bytes());
    bytes[104..112].copy_from_slice(&13_u64.to_le_bytes());
    bytes[case.byte_count - 3..].copy_from_slice(&[0xde, 0xad, 0x5a]);
    bytes
}

pub(super) fn reader_vectors() -> Vec<([u8; 10], Vec<u8>)> {
    cases()
        .iter()
        .map(|case| (case.format_bytes, literal_page(case)))
        .collect()
}
