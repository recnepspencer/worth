use crate::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary, PhysicalIntegrityCoveredRange,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

pub const WAL_FRAME_V1_VERSION: u16 = 1;
pub const WAL_FRAME_V1_HEADER_BYTES: usize = 116;
pub const WAL_FRAME_V1_FOOTER_BYTES: usize = 32;

const PAYLOAD_RANGES: &[PhysicalIntegrityCoveredRange] = &[PhysicalIntegrityCoveredRange::new(
    PhysicalIntegrityCoverageBoundary::Fixed(WAL_FRAME_V1_HEADER_BYTES as u64),
    PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];
const FRAME_RANGES: &[PhysicalIntegrityCoveredRange] = &[PhysicalIntegrityCoveredRange::new(
    PhysicalIntegrityCoverageBoundary::Fixed(0),
    PhysicalIntegrityCoverageBoundary::PayloadEnd,
)];
const CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] = &[
    PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        PAYLOAD_RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::Fixed(84),
            PhysicalIntegrityCoverageBoundary::Fixed(WAL_FRAME_V1_HEADER_BYTES as u64),
        ),
    ),
    PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        FRAME_RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::PayloadEnd,
            PhysicalIntegrityCoverageBoundary::ArtifactEnd,
        ),
    ),
];

pub const WAL_FRAME_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::WalFrame,
        PhysicalIntegrityFormatVersion::new(WAL_FRAME_V1_VERSION, None),
        CHECKSUMS,
    );
