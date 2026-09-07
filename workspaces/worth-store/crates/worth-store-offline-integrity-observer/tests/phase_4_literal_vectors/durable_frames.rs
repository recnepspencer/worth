use worth_store_physical_format::integrity_declarations::{
    families::{
        free_space::{
            FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
            FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
        },
        root::{BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION, ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION},
        EXTENT_CHUNK_INTEGRITY_DECLARATION, EXTENT_MANIFEST_INTEGRITY_DECLARATION,
        SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
    },
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityFormatDeclaration,
};

use super::oracle::{
    assert_declaration, assert_literal_vector, decode_hex, DeclarationChecksumExpectation,
    LiteralChecksum, LiteralChecksumExpectation, LiteralVector,
};

pub(super) const BOOTSTRAP: &str = concat!(
    "5752433546524d0001020100004000000101011830000000220000000b00000000000000000000000000000083852f18",
    "070707070707070707070707070707070b0000000000000001000040000001010118",
);
pub(super) const ROOT_ROUTING: &str = concat!(
    "5752433546524d000802010000400000010101183000000080000000030000000000000000000000000000001385587d",
    "4700000000000000030000000000000000000100010000000b000000000000000000000000000000a1a1a1a1a1a1a1a1",
    "a1a1a1a1a1a1a1a105000000000000000200000000000000000000000000000013000000000000000700000000000000",
    "0000000000000000000000000000000017000000000000000000000000000000",
);
pub(super) const SEGMENT_MEMBERSHIP: &str = concat!(
    "5752433546524d00090201000040000001010118300000005000000005000000000000000000000000000000bc92e364",
    "4900000000000000050000000000000000000100010000000b0000000000000000000000000000000d00000000000000",
    "1100000000000000050000000000000006000000000000000200000001000000",
);
pub(super) const EXTENT_MANIFEST: &str = concat!(
    "5752433546524d00060201000040000001010118300000003800000005000000000000000000000000000000a86b2fa8",
    "222222222222222222222222222222220700000000000000040000000000000006000000000000000040000001000000",
    "0000000000000000",
);
pub(super) const EXTENT_CHUNK: &str = concat!(
    "5752433546524d000402010000400000010101183000000046000000010000000000000000000000000000007a7ddf3a",
    "222222222222222222222222222222220700000000000000040000000000000005000000000000000600000000000000",
    "00000000000000000600000000000000433945585421",
);
pub(super) const FREE_SPACE_HEADER: &str = concat!(
    "5752433546524d00070201000040000001010118300000008000000006000000000000000000000000000000cab48031",
    "060000000000000008000000000000000200040000000000020000000000000008000000000000000a00000000000000",
    "0500000000000000020000000000000001000000000000000600000000000000010000000000000000000000dea0f21f",
    "0100000000000000070000000000000002000000000000000500000000000000",
);
pub(super) const FREE_SPACE_MEMBERSHIP: &str = concat!(
    "5752433546524d000a0201000040000001010118300000007800000001000000000000000000000000000000252ab68b",
    "080000000000000001000000000000000000020001000000060000000000000000000000000000000100000000000000",
    "070000000000000004000000000000000200000000000000030000000000000002000000000000000500000000000000",
    "050000000000000064000000000000000100000000000000",
);

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

struct Case {
    name: &'static str,
    declaration: PhysicalIntegrityFormatDeclaration,
    family: PhysicalIntegrityArtifactFamily,
    hex: &'static str,
    checksum: u32,
}

pub(super) fn verify() {
    for case in cases() {
        assert_declaration(
            case.declaration,
            case.family,
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
        let bytes = decode_hex(case.hex);
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

fn cases() -> [Case; 7] {
    [
        Case {
            name: "bootstrap catalog",
            declaration: BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::BootstrapCatalog,
            hex: BOOTSTRAP,
            checksum: 0x182f_8583,
        },
        Case {
            name: "root-routing block",
            declaration: ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            hex: ROOT_ROUTING,
            checksum: 0x7d58_8513,
        },
        Case {
            name: "segment-membership block",
            declaration: SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::SegmentMembership,
            hex: SEGMENT_MEMBERSHIP,
            checksum: 0x64e3_92bc,
        },
        Case {
            name: "extent manifest",
            declaration: EXTENT_MANIFEST_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::ExtentManifest,
            hex: EXTENT_MANIFEST,
            checksum: 0xa82f_6ba8,
        },
        Case {
            name: "extent chunk",
            declaration: EXTENT_CHUNK_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::ExtentChunk,
            hex: EXTENT_CHUNK,
            checksum: 0x3adf_7d7a,
        },
        Case {
            name: "free-space header",
            declaration: FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            hex: FREE_SPACE_HEADER,
            checksum: 0x3180_b4ca,
        },
        Case {
            name: "free-space-membership block",
            declaration: FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
            hex: FREE_SPACE_MEMBERSHIP,
            checksum: 0x8bb6_2a25,
        },
    ]
}
