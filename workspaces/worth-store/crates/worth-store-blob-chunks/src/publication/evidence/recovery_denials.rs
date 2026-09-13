use worth_store_wal::LogSequenceNumber;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationTornPublicationDenial {
    torn_lsn: Option<LogSequenceNumber>,
    reason: String,
}

impl BlobPublicationTornPublicationDenial {
    pub fn new(torn_lsn: Option<LogSequenceNumber>, reason: impl Into<String>) -> Self {
        Self {
            torn_lsn,
            reason: reason.into(),
        }
    }

    pub const fn torn_lsn(&self) -> Option<LogSequenceNumber> {
        self.torn_lsn
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlobPublicationBackendResidueKind {
    StalePageImage,
    OrphanedCheckpointManifest,
    BackendDirectoryResidue,
    InvalidCompactionProduct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobPublicationNonAuthoritativeSource {
    BackendResidue,
    LiveAcknowledgmentMemory,
    LogOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationNonAuthoritativeDenial {
    source: BlobPublicationNonAuthoritativeSource,
    persisted_digest: String,
}

impl BlobPublicationNonAuthoritativeDenial {
    pub fn new(
        source: BlobPublicationNonAuthoritativeSource,
        persisted_digest: impl Into<String>,
    ) -> Self {
        Self {
            source,
            persisted_digest: persisted_digest.into(),
        }
    }

    pub const fn source(&self) -> BlobPublicationNonAuthoritativeSource {
        self.source
    }

    pub fn persisted_digest(&self) -> &str {
        &self.persisted_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobPublicationClassificationDenialKind {
    BackendResidueCannotStandInForCrashBoundaryAuthority,
    AmbiguousResidueCannotStandInForCrashBoundaryAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobPublicationClassificationDenial {
    kind: BlobPublicationClassificationDenialKind,
}

impl BlobPublicationClassificationDenial {
    pub(crate) const fn new(kind: BlobPublicationClassificationDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> BlobPublicationClassificationDenialKind {
        self.kind
    }
}
