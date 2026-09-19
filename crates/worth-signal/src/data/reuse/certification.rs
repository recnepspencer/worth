use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::reuse::basis::{ReuseCrossing, ReuseOrigin, ReuseSource, ReuseStrategy};
use crate::data::reuse::context::ReuseBoundaryAuthority;
use crate::data::reuse::contract::ArtifactSemanticBoundary;

/// Cold-path proof that a reuse decision respected the node's reuse contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReuseCertificationRecord {
    pub strategy: ReuseStrategy,
    pub origin: ReuseOrigin,
    pub source: ReuseSource,
    pub crossing: ReuseCrossing,
    #[serde(default)]
    pub proofs: Vec<ReuseBoundaryProof>,
}

/// One semantic boundary that was checked during reuse certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseBoundaryProof {
    pub boundary: ArtifactSemanticBoundary,
    #[serde(default)]
    pub satisfied: bool,
}

/// Structured cold-path failure for illegal reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseCertificationFailure {
    pub strategy: Option<ReuseStrategy>,
    pub source: ReuseSource,
    pub crossing: ReuseCrossing,
    pub failure: ReuseBoundaryFailure,
}

/// Specific semantic boundary that blocked reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReuseBoundaryFailure {
    UnsupportedStrategyFamily(ReuseStrategy),
    ContractStrategyDisallowed(ReuseStrategy),
    BoundaryMismatch(ArtifactSemanticBoundary),
    BoundaryContextUnavailable(ArtifactSemanticBoundary),
    PersistentCorrespondenceEvidenceMissing,
    PersistentCorrespondenceEvidenceInvalid,
    CompositionRegionLegalityFailure,
    MixedBasisInsufficiency,
    SnapshotReuseNotAllowed,
    AuthorityReuseNotAllowed,
}

/// Current and prior evidence used to certify a reuse decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseBoundaryEvidence {
    pub current: ReuseBoundaryAuthority,
    #[serde(default)]
    pub previous: Option<ReuseBoundaryAuthority>,
}
