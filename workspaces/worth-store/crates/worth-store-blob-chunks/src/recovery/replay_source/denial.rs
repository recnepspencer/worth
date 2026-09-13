#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReplayAdmissionDenialKind {
    BackendResidueRejected,
    MissingWalSource,
    MissingCheckpointSource,
    MissingManifestSource,
    WrongReplaySourceForResumeSession,
    MissingStoreAuthorityReadmission,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobReplayAdmissionDenial {
    kind: BlobReplayAdmissionDenialKind,
    source_digest: Option<String>,
}

impl BlobReplayAdmissionDenial {
    pub(crate) fn new(kind: BlobReplayAdmissionDenialKind, digest: Option<String>) -> Self {
        Self {
            kind,
            source_digest: digest,
        }
    }

    pub const fn kind(&self) -> BlobReplayAdmissionDenialKind {
        self.kind
    }

    pub fn source_digest(&self) -> Option<&str> {
        self.source_digest.as_deref()
    }
}
