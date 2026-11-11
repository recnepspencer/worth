use serde::{Deserialize, Serialize};

use crate::history::data::CommitId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayAuthorityBasisKind {
    DurableLogCanonical,
    RetainedEnvelopeCanonical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLineageAuthorityBasis {
    kind: ReplayAuthorityBasisKind,
    commit_id: CommitId,
    digest_mode: ReplayLineageDigestMode,
    lineage_event_count: usize,
    lineage_decision_count: usize,
    event_batch_digest_basis: CertifiedLineageSurfaceComparisonBasis,
    decision_log_digest_basis: CertifiedLineageSurfaceComparisonBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayLineageDigestMode {
    ExactCanonicalArtifactDigest,
    SummaryDigestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageCertifiedSurfaceKind {
    EventBatch,
    DecisionLog,
    HistoricalResolution,
    GraphExport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedLineageSurfaceDigest {
    kind: LineageCertifiedSurfaceKind,
    digest: worth_foundational::CanonicalDigestId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedLineageSurfaceComparisonBasis {
    kind: LineageCertifiedSurfaceKind,
    exact_digest: Option<CertifiedLineageSurfaceDigest>,
    summary_digest: Option<worth_foundational::CanonicalDigestId>,
}

impl CertifiedLineageSurfaceDigest {
    pub(crate) fn from_digest(kind: LineageCertifiedSurfaceKind, digest: [u8; 32]) -> Self {
        Self {
            kind,
            digest: worth_foundational::CanonicalDigestId::new(digest),
        }
    }

    pub fn kind(&self) -> LineageCertifiedSurfaceKind {
        self.kind
    }

    pub fn digest(&self) -> &[u8; 32] {
        self.digest.bytes()
    }
}

impl CertifiedLineageSurfaceComparisonBasis {
    pub(crate) fn new(
        kind: LineageCertifiedSurfaceKind,
        exact_digest: Option<CertifiedLineageSurfaceDigest>,
        summary_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            kind,
            exact_digest,
            summary_digest: summary_digest.map(worth_foundational::CanonicalDigestId::new),
        }
    }

    pub fn kind(&self) -> LineageCertifiedSurfaceKind {
        self.kind
    }

    pub fn exact_digest(&self) -> Option<&CertifiedLineageSurfaceDigest> {
        self.exact_digest.as_ref()
    }

    pub fn summary_digest(&self) -> Option<&[u8; 32]> {
        self.summary_digest
            .as_ref()
            .map(worth_foundational::CanonicalDigestId::bytes)
    }
}

impl ReplayLineageAuthorityBasis {
    pub(crate) fn new(
        kind: ReplayAuthorityBasisKind,
        commit_id: CommitId,
        digest_mode: ReplayLineageDigestMode,
        lineage_event_count: usize,
        lineage_decision_count: usize,
        event_batch_digest_basis: CertifiedLineageSurfaceComparisonBasis,
        decision_log_digest_basis: CertifiedLineageSurfaceComparisonBasis,
    ) -> Self {
        Self {
            kind,
            commit_id,
            digest_mode,
            lineage_event_count,
            lineage_decision_count,
            event_batch_digest_basis,
            decision_log_digest_basis,
        }
    }

    pub fn kind(&self) -> ReplayAuthorityBasisKind {
        self.kind
    }

    pub fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub fn digest_mode(&self) -> ReplayLineageDigestMode {
        self.digest_mode
    }

    pub fn lineage_event_count(&self) -> usize {
        self.lineage_event_count
    }

    pub fn lineage_decision_count(&self) -> usize {
        self.lineage_decision_count
    }

    pub fn event_batch_digest_basis(&self) -> &CertifiedLineageSurfaceComparisonBasis {
        &self.event_batch_digest_basis
    }

    pub fn decision_log_digest_basis(&self) -> &CertifiedLineageSurfaceComparisonBasis {
        &self.decision_log_digest_basis
    }
}
