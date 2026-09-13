use crate::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary, PhysicalIntegrityCoveredRange,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

pub const PHYSICAL_WORK_OBLIGATION_V6_VERSION: u8 = 6;
pub const PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES: usize = 160;

const RANGES: &[PhysicalIntegrityCoveredRange] = &[PhysicalIntegrityCoveredRange::new(
    PhysicalIntegrityCoverageBoundary::Fixed(0),
    PhysicalIntegrityCoverageBoundary::Fixed(128),
)];
const CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::Fixed(128),
            PhysicalIntegrityCoverageBoundary::Fixed(160),
        ),
    )];

pub const PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
        PhysicalIntegrityFormatVersion::new(PHYSICAL_WORK_OBLIGATION_V6_VERSION as u16, None),
        CHECKSUMS,
    );
