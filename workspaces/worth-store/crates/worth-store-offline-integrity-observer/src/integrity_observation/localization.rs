use worth_foundational::PhysicalByteRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflinePhysicalDamageCause {
    ChecksumMismatch,
    Framing,
    ScopeMismatch,
    Pointer,
    Truncation,
    MissingArtifact,
    DuplicateIdentity,
    MalformedPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflinePhysicalFormatField {
    Magic,
    EncodingVersion,
    NamespaceVersion,
    RecordLength,
    FieldCount,
    IdentityField,
    FamilyKind,
    EnvelopeSchema,
    FormatVersion,
    PageSize,
    ByteOrder,
    RootProtocol,
    IntegrityAlgorithm,
    RecordIdentityWidth,
    HeaderLength,
    PayloadLength,
    FrameIdentity,
    Checksum,
    StoreIdentity,
    SelectorRole,
    RootGeneration,
    LinkedSelector,
    EmbeddedFormat,
    ManifestGeneration,
    ManifestPointer,
    Reserved,
    TreeIdentity,
    TreeLevel,
    WalLsn,
    CheckpointAggregate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflinePhysicalBlastRadius {
    Field,
    Frame,
    Artifact,
    ReachableRootSubtree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflinePhysicalDamageLocalization {
    cause: OfflinePhysicalDamageCause,
    damaged_range: Option<PhysicalByteRange>,
    field: Option<OfflinePhysicalFormatField>,
    blast_radius: OfflinePhysicalBlastRadius,
}

impl OfflinePhysicalDamageLocalization {
    pub(crate) fn new(
        cause: OfflinePhysicalDamageCause,
        offset: Option<(u64, u64)>,
        field: Option<OfflinePhysicalFormatField>,
        blast_radius: OfflinePhysicalBlastRadius,
    ) -> Self {
        Self {
            cause,
            damaged_range: offset.map(|(start, length)| {
                PhysicalByteRange::new(start, length).expect("localization range is nonempty")
            }),
            field,
            blast_radius,
        }
    }

    pub const fn cause(&self) -> OfflinePhysicalDamageCause {
        self.cause
    }
    pub const fn damaged_range(&self) -> Option<PhysicalByteRange> {
        self.damaged_range
    }
    pub const fn field(&self) -> Option<OfflinePhysicalFormatField> {
        self.field
    }
    pub const fn blast_radius(&self) -> OfflinePhysicalBlastRadius {
        self.blast_radius
    }
}
