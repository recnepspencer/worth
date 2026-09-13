use super::{
    BlobPublicationBackendResidueKind, BlobPublicationCrashEdge, BlobPublicationEvidence,
    BlobPublicationPersistedBytes, BlobPublicationTornPublicationDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobPublicationObservedSource {
    PersistedCrashEdge(BlobPublicationCrashEdge),
    BackendResidue {
        residue: BlobPublicationBackendResidueKind,
        residue_digest: String,
    },
    LiveAcknowledgmentMemory {
        memory_digest: String,
    },
    LogOnly {
        log_digest: String,
    },
    TornPublication(BlobPublicationTornPublicationDenial),
    InsufficientPersistedEvidence {
        ambiguity_digest: String,
    },
}

impl BlobPublicationObservedSource {
    pub fn persisted_crash_edge(edge: BlobPublicationCrashEdge) -> Self {
        Self::PersistedCrashEdge(edge)
    }

    pub fn backend_residue(
        residue: BlobPublicationBackendResidueKind,
        residue_digest: impl Into<String>,
    ) -> Self {
        Self::BackendResidue {
            residue,
            residue_digest: residue_digest.into(),
        }
    }

    pub fn live_ack_memory(memory_digest: impl Into<String>) -> Self {
        Self::LiveAcknowledgmentMemory {
            memory_digest: memory_digest.into(),
        }
    }

    pub fn log_only(log_digest: impl Into<String>) -> Self {
        Self::LogOnly {
            log_digest: log_digest.into(),
        }
    }

    pub fn torn_publication(denial: BlobPublicationTornPublicationDenial) -> Self {
        Self::TornPublication(denial)
    }

    pub fn insufficient_persisted_evidence(ambiguity_digest: impl Into<String>) -> Self {
        Self::InsufficientPersistedEvidence {
            ambiguity_digest: ambiguity_digest.into(),
        }
    }

    pub(crate) fn into_evidence(self) -> BlobPublicationEvidence {
        match self {
            Self::PersistedCrashEdge(edge) => {
                BlobPublicationEvidence::from_persisted_crash_edge(edge)
            }
            Self::BackendResidue {
                residue,
                residue_digest,
            } => BlobPublicationEvidence::from_backend_residue(residue, residue_digest),
            Self::LiveAcknowledgmentMemory { memory_digest } => {
                BlobPublicationEvidence::from_live_ack_memory(memory_digest)
            }
            Self::LogOnly { log_digest } => BlobPublicationEvidence::from_log_only(log_digest),
            Self::TornPublication(denial) => BlobPublicationEvidence::from_torn_publication(denial),
            Self::InsufficientPersistedEvidence { ambiguity_digest } => {
                BlobPublicationEvidence::insufficient_persisted_evidence(ambiguity_digest)
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlobPublicationObservationSet {
    torn_publication: Option<BlobPublicationObservedSource>,
    persisted_crash_edge: Option<BlobPublicationObservedSource>,
    backend_residue: Option<BlobPublicationObservedSource>,
    live_ack_memory: Option<BlobPublicationObservedSource>,
    log_only: Option<BlobPublicationObservedSource>,
    insufficient_persisted_evidence: Option<BlobPublicationObservedSource>,
}

impl BlobPublicationObservationSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_persisted_bytes(self, bytes: BlobPublicationPersistedBytes) -> Self {
        self.with_observed_source(bytes.observe())
    }

    pub fn with_persisted_crash_edge(self, edge: BlobPublicationCrashEdge) -> Self {
        self.with_observed_source(BlobPublicationObservedSource::persisted_crash_edge(edge))
    }

    pub fn with_backend_residue(
        self,
        residue: BlobPublicationBackendResidueKind,
        residue_digest: impl Into<String>,
    ) -> Self {
        self.with_observed_source(BlobPublicationObservedSource::backend_residue(
            residue,
            residue_digest,
        ))
    }

    pub fn with_live_ack_memory(self, memory_digest: impl Into<String>) -> Self {
        self.with_observed_source(BlobPublicationObservedSource::live_ack_memory(
            memory_digest,
        ))
    }

    pub fn with_log_only(self, log_digest: impl Into<String>) -> Self {
        self.with_observed_source(BlobPublicationObservedSource::log_only(log_digest))
    }

    pub fn with_torn_publication(self, denial: BlobPublicationTornPublicationDenial) -> Self {
        self.with_observed_source(BlobPublicationObservedSource::torn_publication(denial))
    }

    pub fn with_insufficient_persisted_evidence(self, ambiguity_digest: impl Into<String>) -> Self {
        self.with_observed_source(
            BlobPublicationObservedSource::insufficient_persisted_evidence(ambiguity_digest),
        )
    }

    pub fn with_observed_source(mut self, source: BlobPublicationObservedSource) -> Self {
        match source {
            BlobPublicationObservedSource::TornPublication(_) => {
                self.torn_publication = Some(source);
            }
            BlobPublicationObservedSource::PersistedCrashEdge(_) => {
                self.persisted_crash_edge = Some(source);
            }
            BlobPublicationObservedSource::BackendResidue { .. } => {
                self.backend_residue = Some(source);
            }
            BlobPublicationObservedSource::LiveAcknowledgmentMemory { .. } => {
                self.live_ack_memory = Some(source);
            }
            BlobPublicationObservedSource::LogOnly { .. } => {
                self.log_only = Some(source);
            }
            BlobPublicationObservedSource::InsufficientPersistedEvidence { .. } => {
                self.insufficient_persisted_evidence = Some(source);
            }
        }
        self
    }

    pub(crate) fn into_sources(self) -> BlobPublicationObservationSources {
        BlobPublicationObservationSources {
            torn_publication: self.torn_publication,
            persisted_crash_edge: self.persisted_crash_edge,
            backend_residue: self.backend_residue,
            live_ack_memory: self.live_ack_memory,
            log_only: self.log_only,
            insufficient_persisted_evidence: self.insufficient_persisted_evidence,
        }
    }
}

pub(crate) struct BlobPublicationObservationSources {
    pub(crate) torn_publication: Option<BlobPublicationObservedSource>,
    pub(crate) persisted_crash_edge: Option<BlobPublicationObservedSource>,
    pub(crate) backend_residue: Option<BlobPublicationObservedSource>,
    pub(crate) live_ack_memory: Option<BlobPublicationObservedSource>,
    pub(crate) log_only: Option<BlobPublicationObservedSource>,
    pub(crate) insufficient_persisted_evidence: Option<BlobPublicationObservedSource>,
}
