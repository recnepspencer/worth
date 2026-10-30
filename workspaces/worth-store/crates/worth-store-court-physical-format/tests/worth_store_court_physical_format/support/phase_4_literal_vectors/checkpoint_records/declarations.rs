pub(crate) use worth_store_physical_format::integrity_declarations::{
    families::checkpoint::CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
    families::checkpoint::CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
    families::checkpoint::CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
    families::checkpoint::CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
    families::checkpoint::CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityFormatDeclaration,
};

pub(crate) use crate::oracle::support::literal_vectors::{
    assert_declaration, assert_literal_vector, decode_hex, DeclarationChecksumExpectation,
    LiteralChecksum, LiteralChecksumExpectation, LiteralVector,
};

pub(crate) const HEADER: &str = concat!(
    "574350375245430001010000900000000102030405060708090a0b0c0d0e0f1007000000000000000a00000000000000",
    "140000000000000003000000000000000400000000000000050000000000000001000000000000000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "00000000000000000000000000000000d326f15d",
);
pub(crate) const DIRTY_BASIS: &str = concat!(
    "574350375245430001020000300000000100000000000000000000000000000000000000000000004000000000000000",
    "100000000000000006000000000000001163a06b",
);
pub(crate) const BINDING_COMPACTION: &str =
    "5743503752454300010300001000000009000000000000001200000000000000f3a441f8";
pub(crate) const BINDING: &str = "57435037524543000104000003000000aabbcc9133843e";
pub(crate) const FOOTER: &str = concat!(
    "574350375245430001050000880000000102030405060708090a0b0c0d0e0f1007000000000000000100000000000000",
    "b224b50fd7741f1c3141020f37c2517dba7644e9b09562fa7ab38c6c4c05b4efe8000000000000000900000000000000",
    "12000000000000000100000000000000170000000000000081047fa20cd00db126df9ae50af3e04d4a0ee9aae244d20c",
    "ad55713251c32e7c9c179ea6",
);

pub(crate) const DECLARATION_RANGES: &[(
    PhysicalIntegrityCoverageBoundary,
    PhysicalIntegrityCoverageBoundary,
)] = &[(
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::Fixed(0),
    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];

pub(crate) struct Case {
    pub(crate) name: &'static str,
    pub(crate) declaration: PhysicalIntegrityFormatDeclaration,
    pub(crate) family: PhysicalIntegrityArtifactFamily,
    pub(crate) hex: &'static str,
    pub(crate) checksum: u32,
}

pub(crate) fn verify() {
    for case in cases() {
        assert_declaration(
            case.declaration,
            case.family,
            1,
            None,
            &[DeclarationChecksumExpectation {
                algorithm: worth_store_physical_format::integrity_declarations::PhysicalIntegrityAlgorithm::Crc32c,
                ranges: DECLARATION_RANGES,
                field: (
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::PayloadEnd,
                    worth_store_physical_format::integrity_declarations::PhysicalIntegrityCoverageBoundary::ArtifactEnd,
                ),
            }],
        );
        let bytes = decode_hex(case.hex);
        let payload_end = bytes.len() - 4;
        let ranges = [(0, payload_end)];
        let checksums = [LiteralChecksumExpectation {
            checksum: crate::oracle::support::literal_vectors::LiteralChecksum::Crc32c(
                case.checksum,
            ),
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

pub(crate) fn cases() -> [Case; 5] {
    [
        Case {
            name: "checkpoint stream header",
            declaration: CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
            family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            hex: HEADER,
            checksum: 0x5df1_26d3,
        },
        Case {
            name: "checkpoint dirty-basis record",
            declaration: CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
            family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            hex: DIRTY_BASIS,
            checksum: 0x6ba0_6311,
        },
        Case {
            name: "checkpoint binding-compaction record",
            declaration: CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
            family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            hex: BINDING_COMPACTION,
            checksum: 0xf841_a4f3,
        },
        Case {
            name: "checkpoint binding record",
            declaration: CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
            family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointBinding,
            hex: BINDING,
            checksum: 0x3e84_3391,
        },
        Case {
            name: "checkpoint footer",
            declaration: CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
            family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::CheckpointFooter,
            hex: FOOTER,
            checksum: 0xa69e_179c,
        },
    ]
}
