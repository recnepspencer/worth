pub(crate) use {worth_store_physical_format::integrity_declarations::families::WAL_FRAME_INTEGRITY_DECLARATION, worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm, worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily, worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary};

pub(crate) use {crate::phase_4_literal_vectors::oracle::assert_declaration, crate::phase_4_literal_vectors::oracle::assert_literal_vector, crate::phase_4_literal_vectors::oracle::decode_hex, crate::phase_4_literal_vectors::oracle::DeclarationChecksumExpectation, crate::phase_4_literal_vectors::oracle::LiteralChecksum, crate::phase_4_literal_vectors::oracle::LiteralChecksumExpectation, crate::phase_4_literal_vectors::oracle::LiteralVector};

pub(crate) const FRAME_HEX: &str = concat!(
    "574f52544857414c010074000100000000000000020000000000000003000000",
    "0000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a",
    "084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a3",
    "7f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d8",
    "3ab40999718d0fcaf7354454bff68c9a716003b636fa681",
);
pub(crate) const PAYLOAD_SHA256: [u8; 32] = [
    0x8e, 0x13, 0x36, 0xab, 0x78, 0xeb, 0xe6, 0x87, 0xfd, 0x80, 0x56, 0xa3, 0x7f, 0x2d, 0x3b, 0x0c,
    0x32, 0xf4, 0xcf, 0x8f, 0xa8, 0xb6, 0x91, 0xb6, 0x53, 0x80, 0x0f, 0xa6, 0x93, 0xd5, 0x70, 0xb9,
];
pub(crate) const FRAME_SHA256: [u8; 32] = [
    0xc8, 0xb5, 0xe3, 0x84, 0x93, 0xd6, 0x39, 0x7d, 0x83, 0xab, 0x40, 0x99, 0x97, 0x18, 0xd0, 0xfc,
    0xaf, 0x73, 0x54, 0x45, 0x4b, 0xff, 0x68, 0xc9, 0xa7, 0x16, 0x00, 0x3b, 0x63, 0x6f, 0xa6, 0x81,
];
pub(crate) const DECLARATION_PAYLOAD_RANGE: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[(
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(116),
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];
pub(crate) const DECLARATION_FRAME_RANGE: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[(
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(0),
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];

pub(crate) fn verify() {
    assert_declaration(
        WAL_FRAME_INTEGRITY_DECLARATION,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame,
        1,
        None,
        &[
            DeclarationChecksumExpectation {
                algorithm: worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Sha256,
                ranges: DECLARATION_PAYLOAD_RANGE,
                field: (
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(84),
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(116),
                ),
            },
            DeclarationChecksumExpectation {
                algorithm: worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Sha256,
                ranges: DECLARATION_FRAME_RANGE,
                field: (
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::PayloadEnd,
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::ArtifactEnd,
                ),
            },
        ],
    );
    let bytes = decode_hex(FRAME_HEX);
    assert_eq!(bytes.len(), 151);
    const PAYLOAD_END: usize = 119;
    const PAYLOAD_RANGES: &[(usize, usize)] = &[(116, PAYLOAD_END)];
    const FRAME_RANGES: &[(usize, usize)] = &[(0, PAYLOAD_END)];
    const CHECKSUMS: &[LiteralChecksumExpectation<'static>] = &[
        LiteralChecksumExpectation {
            checksum: crate::phase_4_literal_vectors::oracle::LiteralChecksum::Sha256(PAYLOAD_SHA256),
            ranges: PAYLOAD_RANGES,
            field: (84, 116),
        },
        LiteralChecksumExpectation {
            checksum: crate::phase_4_literal_vectors::oracle::LiteralChecksum::Sha256(FRAME_SHA256),
            ranges: FRAME_RANGES,
            field: (PAYLOAD_END, 151),
        },
    ];
    assert_literal_vector(LiteralVector {
        name: "WAL v1 frame",
        bytes,
        checksums: CHECKSUMS,
    });
}