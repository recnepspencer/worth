use serde::{Deserialize, Serialize};

use crate::schema::data::{DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion};

use super::{ReplayMismatchClass, ReplayVerificationLayer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescriptorAuthorityKind {
    SchemaTransitionArtifact,
    SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor,
    SchemaLineageArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplaySurfaceAuthorityKind {
    Patch,
    Diagnostics,
    History,
    Snapshot,
    BranchHead,
    Lineage,
    Strategy,
    DerivedIndexes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedDescriptorDigest {
    pub kind: DescriptorAuthorityKind,
    pub digest: worth_foundational::CanonicalDigestId,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
    pub canonical_basis_version: Option<DescriptorCanonicalBasisVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptorComparisonBasis {
    pub kind: DescriptorAuthorityKind,
    pub exact_digest: Option<VerifiedDescriptorDigest>,
    pub summary_digest: Option<worth_foundational::CanonicalDigestId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedReplaySurfaceDigest {
    pub kind: ReplaySurfaceAuthorityKind,
    pub digest: worth_foundational::CanonicalDigestId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySurfaceComparisonBasis {
    pub kind: ReplaySurfaceAuthorityKind,
    pub exact_digest: Option<VerifiedReplaySurfaceDigest>,
    pub summary_digest: Option<worth_foundational::CanonicalDigestId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescriptorParityCheck {
    ExactDigestMatch {
        kind: DescriptorAuthorityKind,
    },
    SummaryMatchDigestUnavailable {
        kind: DescriptorAuthorityKind,
    },
    Drift {
        kind: DescriptorAuthorityKind,
        layer: ReplayVerificationLayer,
        mismatch_class: ReplayMismatchClass,
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplaySurfaceParityCheck {
    ExactDigestMatch {
        kind: ReplaySurfaceAuthorityKind,
    },
    SummaryMatchDigestUnavailable {
        kind: ReplaySurfaceAuthorityKind,
    },
    Drift {
        kind: ReplaySurfaceAuthorityKind,
        layer: ReplayVerificationLayer,
        mismatch_class: ReplayMismatchClass,
        detail: String,
    },
}

impl VerifiedDescriptorDigest {
    pub fn from_digest(
        kind: DescriptorAuthorityKind,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        canonical_basis_version: Option<DescriptorCanonicalBasisVersion>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            kind,
            digest: worth_foundational::CanonicalDigestId::new(digest),
            descriptor_semantics_version,
            canonical_basis_version,
        }
    }
}

impl VerifiedReplaySurfaceDigest {
    pub fn from_digest(kind: ReplaySurfaceAuthorityKind, digest: [u8; 32]) -> Self {
        Self {
            kind,
            digest: worth_foundational::CanonicalDigestId::new(digest),
        }
    }
}

impl DescriptorComparisonBasis {
    pub fn new(
        kind: DescriptorAuthorityKind,
        exact_digest: Option<VerifiedDescriptorDigest>,
        summary_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            kind,
            exact_digest,
            summary_digest: summary_digest.map(worth_foundational::CanonicalDigestId::new),
        }
    }

    pub fn compare(
        &self,
        other: &Self,
        mismatch_class: ReplayMismatchClass,
        detail: impl Into<String>,
    ) -> DescriptorParityCheck {
        let detail = detail.into();
        if self.kind != other.kind {
            return DescriptorParityCheck::Drift {
                kind: self.kind,
                layer: ReplayVerificationLayer::DigestParity,
                mismatch_class,
                detail,
            };
        }
        match (&self.exact_digest, &other.exact_digest) {
            (Some(expected), Some(observed)) if expected == observed => {
                return DescriptorParityCheck::ExactDigestMatch { kind: self.kind };
            }
            (None, None) => {}
            _ => {
                return DescriptorParityCheck::Drift {
                    kind: self.kind,
                    layer: ReplayVerificationLayer::DigestParity,
                    mismatch_class,
                    detail,
                };
            }
        }
        match (self.summary_digest, other.summary_digest) {
            (Some(expected), Some(observed)) if expected == observed => {
                DescriptorParityCheck::SummaryMatchDigestUnavailable { kind: self.kind }
            }
            _ => DescriptorParityCheck::Drift {
                kind: self.kind,
                layer: ReplayVerificationLayer::SummaryParity,
                mismatch_class,
                detail,
            },
        }
    }
}

impl ReplaySurfaceComparisonBasis {
    pub fn new(
        kind: ReplaySurfaceAuthorityKind,
        exact_digest: Option<VerifiedReplaySurfaceDigest>,
        summary_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            kind,
            exact_digest,
            summary_digest: summary_digest.map(worth_foundational::CanonicalDigestId::new),
        }
    }

    pub fn compare(
        &self,
        other: &Self,
        mismatch_class: ReplayMismatchClass,
        detail: impl Into<String>,
    ) -> ReplaySurfaceParityCheck {
        let detail = detail.into();
        if self.kind != other.kind {
            return ReplaySurfaceParityCheck::Drift {
                kind: self.kind,
                layer: ReplayVerificationLayer::DigestParity,
                mismatch_class,
                detail,
            };
        }
        match (&self.exact_digest, &other.exact_digest) {
            (Some(expected), Some(observed)) if expected == observed => {
                return ReplaySurfaceParityCheck::ExactDigestMatch { kind: self.kind };
            }
            (None, None) => {}
            _ => {
                return ReplaySurfaceParityCheck::Drift {
                    kind: self.kind,
                    layer: ReplayVerificationLayer::DigestParity,
                    mismatch_class,
                    detail,
                };
            }
        }
        match (self.summary_digest, other.summary_digest) {
            (Some(expected), Some(observed)) if expected == observed => {
                ReplaySurfaceParityCheck::SummaryMatchDigestUnavailable { kind: self.kind }
            }
            _ => ReplaySurfaceParityCheck::Drift {
                kind: self.kind,
                layer: ReplayVerificationLayer::SummaryParity,
                mismatch_class,
                detail,
            },
        }
    }
}
