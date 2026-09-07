use worth_foundational::{PhysicalByteRange, PhysicalIntegrityPosture};

use super::OfflinePhysicalDamageLocalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineUnsupportedVersionAxis {
    NamespaceEncoding,
    NamespaceSchema,
    EnvelopeSchema,
    PhysicalRecordFormat,
    PageSize,
    ByteOrder,
    RootProtocol,
    IntegrityAlgorithm,
    RecordIdentityWidth,
    WalFrame,
    CheckpointRecord,
    PhysicalWork,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineUnsupportedPhysicalVersion {
    axis: OfflineUnsupportedVersionAxis,
    observed: u64,
    supported: Box<str>,
    range: PhysicalByteRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineUnknownPhysicalReason {
    UnrecognizedFile,
    UnrecognizedDirectory,
    UnrecognizedOtherEntry,
    SelectorUnavailable,
    RootNotAddressed,
    StoreIdentityUnavailable,
    FilesystemEntryUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIndeterminatePhysicalReason {
    SourceChanged,
    EntryBoundExceeded,
    ByteBoundExceeded,
    OpenFileBoundExceeded,
    DepthBoundExceeded,
    SymlinkRefused,
    SymlinkBoundExceeded,
    ElapsedBoundExceeded,
    PhysicalIdentityUnavailable,
    IoFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineIntegrityOutcome {
    Intact,
    Damaged(OfflinePhysicalDamageLocalization),
    Unsupported(OfflineUnsupportedPhysicalVersion),
    Unknown(OfflineUnknownPhysicalReason),
    Indeterminate(OfflineIndeterminatePhysicalReason),
}

impl OfflineUnsupportedPhysicalVersion {
    pub(crate) fn new(
        axis: OfflineUnsupportedVersionAxis,
        observed: u64,
        supported: impl Into<Box<str>>,
        range: PhysicalByteRange,
    ) -> Self {
        Self {
            axis,
            observed,
            supported: supported.into(),
            range,
        }
    }
    pub const fn axis(&self) -> OfflineUnsupportedVersionAxis {
        self.axis
    }
    pub const fn observed(&self) -> u64 {
        self.observed
    }
    pub fn supported(&self) -> &str {
        &self.supported
    }
    pub const fn range(&self) -> PhysicalByteRange {
        self.range
    }
}

impl OfflineIntegrityOutcome {
    pub const fn posture(&self) -> PhysicalIntegrityPosture {
        match self {
            Self::Intact => PhysicalIntegrityPosture::Intact,
            Self::Damaged(_) => PhysicalIntegrityPosture::Damaged,
            Self::Unsupported(_) => PhysicalIntegrityPosture::Unsupported,
            Self::Unknown(_) => PhysicalIntegrityPosture::Unknown,
            Self::Indeterminate(_) => PhysicalIntegrityPosture::Indeterminate,
        }
    }
}
