use crate::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary, PhysicalIntegrityCoveredRange,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

const RANGES: &[PhysicalIntegrityCoveredRange] = &[PhysicalIntegrityCoveredRange::new(
    PhysicalIntegrityCoverageBoundary::Fixed(0),
    PhysicalIntegrityCoverageBoundary::Fixed(40),
)];
const CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Sha256,
        RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::Fixed(40),
            PhysicalIntegrityCoverageBoundary::Fixed(72),
        ),
    )];

pub const NAMESPACE_IDENTITY_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    PhysicalIntegrityFormatDeclaration::new(
        PhysicalIntegrityArtifactFamily::NamespaceIdentity,
        PhysicalIntegrityFormatVersion::new(1, None),
        CHECKSUMS,
    );
