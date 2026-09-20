use crate::compatibility::WorthQueryPackageArchiveCompatibilityDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPackageArchiveDenialKind {
    EnvelopeByteBudgetExceeded,
    EnvelopeArchiveByteBudgetExceeded,
    EnvelopeDescriptiveTextByteBudgetExceeded,
    EnvelopeRequirementBudgetExceeded,
    EnvelopeSignatureByteBudgetExceeded,
    EmptyEnvelopeSignature,
    InvalidEnvelopeText,
    InvalidEnvelopeProtocolIdentity,
    InvalidEnvelopeProtocolVersion,
    InvalidEnvelopeLength,
    UnsupportedEnvelopeVersion,
    NonCanonicalEnvelopeRequirementSequence,
    ArchiveChecksumMismatch,
    ArchiveByteBudgetExceeded,
    ManifestFrameByteBudgetExceeded,
    RecordFrameByteBudgetExceeded,
    Truncated,
    InvalidMagic,
    UnsupportedArchiveVersion,
    UnsupportedManifestVersion,
    UnsupportedRecordVersion,
    UnsupportedApplicationProgramDescriptionVersion,
    UnsupportedRecordFamily,
    PackageRootRecordFamilyRequired,
    InvalidManifestLength,
    InvalidRecordLength,
    RecordIndexBudgetExceeded,
    InvalidUtf8,
    UnsupportedDefinitionKind,
    RecordBudgetExceeded,
    LogicalByteBudgetExceeded,
    NestedEntryBudgetExceeded,
    NestingDepthBudgetExceeded,
    CanonicalWorkBudgetExceeded,
    InvalidFamilyCount,
    RecordFamilyInventoryMismatch,
    InvalidBooleanEncoding,
    NumericWidthExceeded,
    UnsupportedRecordVariant,
    InvalidRecordShape,
    NonCanonicalRecordSequence,
    TrailingBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPackageArchiveDenial {
    kind: WorthQueryPackageArchiveDenialKind,
    compatibility: Option<WorthQueryPackageArchiveCompatibilityDenial>,
}

impl WorthQueryPackageArchiveDenial {
    pub(crate) const fn new(kind: WorthQueryPackageArchiveDenialKind) -> Self {
        Self {
            kind,
            compatibility: None,
        }
    }

    pub(crate) const fn incompatible(
        kind: WorthQueryPackageArchiveDenialKind,
        compatibility: WorthQueryPackageArchiveCompatibilityDenial,
    ) -> Self {
        Self {
            kind,
            compatibility: Some(compatibility),
        }
    }

    pub const fn kind(&self) -> WorthQueryPackageArchiveDenialKind {
        self.kind
    }

    pub const fn compatibility(&self) -> Option<WorthQueryPackageArchiveCompatibilityDenial> {
        self.compatibility
    }
}
