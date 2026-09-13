use super::PhysicalIntegrityAlgorithm;

/// A stable endpoint in a checksum coverage declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalIntegrityCoverageBoundary {
    Fixed(u64),
    PayloadEnd,
    ArtifactEnd,
}

/// One half-open range included in checksum calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIntegrityCoveredRange {
    start: PhysicalIntegrityCoverageBoundary,
    end: PhysicalIntegrityCoverageBoundary,
}

/// The half-open byte range occupied by a persisted checksum field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIntegrityChecksumField {
    start: PhysicalIntegrityCoverageBoundary,
    end: PhysicalIntegrityCoverageBoundary,
}

/// One checksum field and the exact ranges that produce it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIntegrityChecksumDeclaration {
    algorithm: PhysicalIntegrityAlgorithm,
    covered_ranges: &'static [PhysicalIntegrityCoveredRange],
    field: PhysicalIntegrityChecksumField,
}

impl PhysicalIntegrityCoveredRange {
    pub const fn new(
        start: PhysicalIntegrityCoverageBoundary,
        end: PhysicalIntegrityCoverageBoundary,
    ) -> Self {
        Self { start, end }
    }

    pub const fn start(self) -> PhysicalIntegrityCoverageBoundary {
        self.start
    }

    pub const fn end(self) -> PhysicalIntegrityCoverageBoundary {
        self.end
    }
}

impl PhysicalIntegrityChecksumField {
    pub const fn new(
        start: PhysicalIntegrityCoverageBoundary,
        end: PhysicalIntegrityCoverageBoundary,
    ) -> Self {
        Self { start, end }
    }

    pub const fn start(self) -> PhysicalIntegrityCoverageBoundary {
        self.start
    }

    pub const fn end(self) -> PhysicalIntegrityCoverageBoundary {
        self.end
    }
}

impl PhysicalIntegrityChecksumDeclaration {
    pub const fn new(
        algorithm: PhysicalIntegrityAlgorithm,
        covered_ranges: &'static [PhysicalIntegrityCoveredRange],
        field: PhysicalIntegrityChecksumField,
    ) -> Self {
        Self {
            algorithm,
            covered_ranges,
            field,
        }
    }

    pub const fn algorithm(self) -> PhysicalIntegrityAlgorithm {
        self.algorithm
    }

    pub const fn covered_ranges(self) -> &'static [PhysicalIntegrityCoveredRange] {
        self.covered_ranges
    }

    pub const fn field(self) -> PhysicalIntegrityChecksumField {
        self.field
    }
}

pub(super) const DURABLE_FRAME_V2_RANGES: &[PhysicalIntegrityCoveredRange] = &[
    PhysicalIntegrityCoveredRange::new(
        PhysicalIntegrityCoverageBoundary::Fixed(0),
        PhysicalIntegrityCoverageBoundary::Fixed(44),
    ),
    PhysicalIntegrityCoveredRange::new(
        PhysicalIntegrityCoverageBoundary::Fixed(48),
        PhysicalIntegrityCoverageBoundary::ArtifactEnd,
    ),
];

pub(super) const DURABLE_FRAME_V2_CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Crc32c,
        DURABLE_FRAME_V2_RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::Fixed(44),
            PhysicalIntegrityCoverageBoundary::Fixed(48),
        ),
    )];

pub(super) const CHECKPOINT_RECORD_V1_RANGES: &[PhysicalIntegrityCoveredRange] =
    &[PhysicalIntegrityCoveredRange::new(
        PhysicalIntegrityCoverageBoundary::Fixed(0),
        PhysicalIntegrityCoverageBoundary::PayloadEnd,
    )];

pub(super) const CHECKPOINT_RECORD_V1_CHECKSUMS: &[PhysicalIntegrityChecksumDeclaration] =
    &[PhysicalIntegrityChecksumDeclaration::new(
        PhysicalIntegrityAlgorithm::Crc32c,
        CHECKPOINT_RECORD_V1_RANGES,
        PhysicalIntegrityChecksumField::new(
            PhysicalIntegrityCoverageBoundary::PayloadEnd,
            PhysicalIntegrityCoverageBoundary::ArtifactEnd,
        ),
    )];
