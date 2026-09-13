use super::{
    BlobPublicationBackendResidueKind, BlobPublicationCrashEdge,
    BlobPublicationTornPublicationDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationEvidence {
    kind: BlobPublicationEvidenceKind,
    persisted_digest: String,
}

impl BlobPublicationEvidence {
    pub fn from_persisted_crash_edge(edge: BlobPublicationCrashEdge) -> Self {
        let persisted_digest = format!("persisted-crash-edge:{edge:?}");
        Self {
            kind: BlobPublicationEvidenceKind::PersistedCrashEdge(edge),
            persisted_digest,
        }
    }

    pub fn from_backend_residue(
        residue: BlobPublicationBackendResidueKind,
        residue_digest: impl Into<String>,
    ) -> Self {
        Self {
            kind: BlobPublicationEvidenceKind::BackendResidueOnly { residue },
            persisted_digest: residue_digest.into(),
        }
    }

    pub fn from_live_ack_memory(memory_digest: impl Into<String>) -> Self {
        Self {
            kind: BlobPublicationEvidenceKind::LiveAcknowledgmentMemoryOnly,
            persisted_digest: memory_digest.into(),
        }
    }

    pub fn from_log_only(log_digest: impl Into<String>) -> Self {
        Self {
            kind: BlobPublicationEvidenceKind::LogOnly,
            persisted_digest: log_digest.into(),
        }
    }

    pub fn from_torn_publication(denial: BlobPublicationTornPublicationDenial) -> Self {
        let persisted_digest = format!("torn-publication:{denial:?}");
        Self {
            kind: BlobPublicationEvidenceKind::TornPublication(denial),
            persisted_digest,
        }
    }

    pub fn insufficient_persisted_evidence(ambiguity_digest: impl Into<String>) -> Self {
        let ambiguity_digest = ambiguity_digest.into();
        Self {
            persisted_digest: ambiguity_digest.clone(),
            kind: BlobPublicationEvidenceKind::InsufficientPersistedEvidence { ambiguity_digest },
        }
    }

    pub(crate) const fn kind(&self) -> &BlobPublicationEvidenceKind {
        &self.kind
    }

    pub fn persisted_digest(&self) -> &str {
        &self.persisted_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BlobPublicationEvidenceKind {
    PersistedCrashEdge(BlobPublicationCrashEdge),
    BackendResidueOnly {
        residue: BlobPublicationBackendResidueKind,
    },
    LiveAcknowledgmentMemoryOnly,
    LogOnly,
    TornPublication(BlobPublicationTornPublicationDenial),
    InsufficientPersistedEvidence {
        ambiguity_digest: String,
    },
}
