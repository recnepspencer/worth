use worth_store_physical_format::integrity_declarations::{
    families::PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION, PhysicalIntegrityAlgorithm,
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
};

use super::oracle::{
    assert_declaration, assert_literal_vector, decode_hex, DeclarationChecksumExpectation,
    LiteralChecksum, LiteralChecksumExpectation, LiteralVector,
};

pub(super) const OPERATION_3_HEX: &str = concat!(
    "575045464645435406060000000000000102030405060708090a0b0c0d0e0f10",
    "0100000000000000020000000000000003000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000050000000000000000000000000000000000000000000000",
    "b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c",
);
const OPERATION_4_HEX: &str = "575045464645435406050000000000000102030405060708090a0b0c0d0e0f1001000000000000000200000000000000040000000000000009000000000000000a00000000000000abababababababababababababababababababababababababababababababab0601000000000000070000000000000008000000000000004cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";

const OPERATION_3_SHA256: [u8; 32] = [
    0xb2, 0xa1, 0x8a, 0xee, 0x7d, 0xbc, 0x13, 0x8e, 0xc7, 0x38, 0x50, 0x7b, 0x8f, 0xcd, 0x7f, 0x72,
    0x22, 0x80, 0xd4, 0x99, 0xd9, 0xbf, 0x0f, 0x39, 0x62, 0x29, 0x4f, 0xcc, 0xfe, 0x50, 0xbd, 0x0c,
];
const OPERATION_4_SHA256: [u8; 32] = [
    0x4c, 0xad, 0x4b, 0xe6, 0x80, 0x9b, 0x3b, 0x05, 0x3c, 0xf2, 0x95, 0x18, 0x15, 0xf2, 0x0c, 0x0d,
    0x1c, 0xb2, 0x64, 0x9b, 0x87, 0xa6, 0x1b, 0x58, 0x2c, 0x1a, 0x0a, 0x3a, 0x89, 0x01, 0x0b, 0x90,
];
const RANGES: &[(usize, usize)] = &[(0, 128)];
const DECLARATION_RANGES: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[(
    PhysicalIntegrityCoverageBoundary::Fixed(0),
    PhysicalIntegrityCoverageBoundary::Fixed(128),
)];

pub(super) fn verify() {
    assert_declaration(
        PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
        PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
        6,
        None,
        &[DeclarationChecksumExpectation {
            algorithm: PhysicalIntegrityAlgorithm::Sha256,
            ranges: DECLARATION_RANGES,
            field: (
                PhysicalIntegrityCoverageBoundary::Fixed(128),
                PhysicalIntegrityCoverageBoundary::Fixed(160),
            ),
        }],
    );
    for (name, hex, checksum) in [
        (
            "physical-work durability-barrier obligation v6",
            OPERATION_3_HEX,
            OPERATION_3_SHA256,
        ),
        (
            "physical-work WAL-append obligation v6",
            OPERATION_4_HEX,
            OPERATION_4_SHA256,
        ),
    ] {
        let bytes = decode_hex(hex);
        assert_eq!(bytes.len(), 160);
        let checksums = [LiteralChecksumExpectation {
            checksum: LiteralChecksum::Sha256(checksum),
            ranges: RANGES,
            field: (128, 160),
        }];
        assert_literal_vector(LiteralVector {
            name,
            bytes,
            checksums: &checksums,
        });
    }
}
