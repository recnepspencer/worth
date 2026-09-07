use worth_store_physical_format::integrity_declarations::{
    families::checkpoint::{
        CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
        CHECKPOINT_BINDING_INTEGRITY_DECLARATION, CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
        CHECKPOINT_FOOTER_INTEGRITY_DECLARATION, CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
    },
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityFormatDeclaration,
};

use super::oracle::{
    assert_declaration, assert_literal_vector, decode_hex, DeclarationChecksumExpectation,
    LiteralChecksum, LiteralChecksumExpectation, LiteralVector,
};

pub(super) const HEADER: &str = concat!(
    "574350375245430001010000900000000102030405060708090a0b0c0d0e0f1007000000000000000a00000000000000",
    "140000000000000003000000000000000400000000000000050000000000000001000000000000000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "00000000000000000000000000000000d326f15d",
);
pub(super) const DIRTY_BASIS: &str = concat!(
    "574350375245430001020000300000000100000000000000000000000000000000000000000000004000000000000000",
    "100000000000000006000000000000001163a06b",
);
pub(super) const BINDING_COMPACTION: &str =
    "5743503752454300010300001000000009000000000000001200000000000000f3a441f8";
pub(super) const BINDING: &str = "57435037524543000104000003000000aabbcc9133843e";
pub(super) const FOOTER: &str = concat!(
    "574350375245430001050000880000000102030405060708090a0b0c0d0e0f1007000000000000000100000000000000",
    "b224b50fd7741f1c3141020f37c2517dba7644e9b09562fa7ab38c6c4c05b4efe8000000000000000900000000000000",
    "12000000000000000100000000000000170000000000000081047fa20cd00db126df9ae50af3e04d4a0ee9aae244d20c",
    "ad55713251c32e7c9c179ea6",
);

const DECLARATION_RANGES: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[(
    PhysicalIntegrityCoverageBoundary::Fixed(0),
    PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];

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
            None,
            &[DeclarationChecksumExpectation {
                algorithm: PhysicalIntegrityAlgorithm::Crc32c,
                ranges: DECLARATION_RANGES,
                field: (
                    PhysicalIntegrityCoverageBoundary::PayloadEnd,
                    PhysicalIntegrityCoverageBoundary::ArtifactEnd,
                ),
            }],
        );
        let bytes = decode_hex(case.hex);
        let payload_end = bytes.len() - 4;
        let ranges = [(0, payload_end)];
        let checksums = [LiteralChecksumExpectation {
            checksum: LiteralChecksum::Crc32c(case.checksum),
            ranges: &ranges,
            field: (payload_end, bytes.len()),
        }];
        assert_literal_vector(LiteralVector {
            name: case.name,
            bytes,
            checksums: &checksums,
        });
    }
}

fn cases() -> [Case; 5] {
    [
        Case {
            name: "checkpoint stream header",
            declaration: CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            hex: HEADER,
            checksum: 0x5df1_26d3,
        },
        Case {
            name: "checkpoint dirty-basis record",
            declaration: CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            hex: DIRTY_BASIS,
            checksum: 0x6ba0_6311,
        },
        Case {
            name: "checkpoint binding-compaction record",
            declaration: CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            hex: BINDING_COMPACTION,
            checksum: 0xf841_a4f3,
        },
        Case {
            name: "checkpoint binding record",
            declaration: CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::CheckpointBinding,
            hex: BINDING,
            checksum: 0x3e84_3391,
        },
        Case {
            name: "checkpoint footer",
            declaration: CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
            family: PhysicalIntegrityArtifactFamily::CheckpointFooter,
            hex: FOOTER,
            checksum: 0xa69e_179c,
        },
    ]
}
