use crate::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary as Boundary, PhysicalIntegrityCoveredRange,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

const NODE_RANGES: &[PhysicalIntegrityCoveredRange] = &[
    PhysicalIntegrityCoveredRange::new(Boundary::Fixed(0), Boundary::Fixed(96)),
    PhysicalIntegrityCoveredRange::new(Boundary::Fixed(100), Boundary::ArtifactEnd),
];
const NODE_CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Crc32c,
        NODE_RANGES,
        PhysicalIntegrityChecksumField::new(Boundary::Fixed(96), Boundary::Fixed(100)),
    )];

/// Inner node-payload checksum; the containing C.5 page has its own frame checksum.
pub const BTREE_NODE_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::BTreeNode,
        PhysicalIntegrityFormatVersion::new(1, None),
        NODE_CHECKSUMS,
    );
