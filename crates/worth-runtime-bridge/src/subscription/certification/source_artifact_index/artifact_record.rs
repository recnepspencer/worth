use std::cmp::Ordering;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::artifact_evidence::{
    BridgeSubscriptionSourceArtifactEvidence, BridgeSubscriptionSourceArtifactKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionSourceArtifactInput {
    artifact_kind: BridgeSubscriptionSourceArtifactKind,
    artifact_identity: Arc<str>,
    artifact_digest: Arc<str>,
}

impl BridgeSubscriptionSourceArtifactInput {
    pub fn from_evidence(evidence: BridgeSubscriptionSourceArtifactEvidence) -> Self {
        Self {
            artifact_kind: evidence.artifact_kind(),
            artifact_identity: evidence.identity(),
            artifact_digest: evidence.digest(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionSourceArtifactRecord {
    artifact_kind: BridgeSubscriptionSourceArtifactKind,
    artifact_identity: Arc<str>,
    artifact_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionSourceArtifactRecord {
    pub(super) fn from_input(input: BridgeSubscriptionSourceArtifactInput) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-source-artifact|kind={}|identity={}|digest={}",
            input.artifact_kind.as_str(),
            input.artifact_identity,
            input.artifact_digest,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            artifact_kind: input.artifact_kind,
            artifact_identity: input.artifact_identity,
            artifact_digest: input.artifact_digest,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-source-artifact:sha256:{digest:x}"
            )),
        }
    }

    pub fn artifact_kind(&self) -> BridgeSubscriptionSourceArtifactKind {
        self.artifact_kind
    }

    pub fn artifact_identity(&self) -> &str {
        self.artifact_identity.as_ref()
    }

    pub fn artifact_digest(&self) -> &str {
        self.artifact_digest.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub(super) fn same_source_artifact_identity(&mut self, other: &mut Self) -> bool {
        self.artifact_kind == other.artifact_kind
            && self.artifact_identity == other.artifact_identity
            && self.artifact_digest == other.artifact_digest
    }
}

pub(super) fn source_artifact_record_ordering(
    left: &BridgeSubscriptionSourceArtifactRecord,
    right: &BridgeSubscriptionSourceArtifactRecord,
) -> Ordering {
    left.artifact_kind()
        .cmp(&right.artifact_kind())
        .then_with(|| left.artifact_identity().cmp(right.artifact_identity()))
        .then_with(|| left.artifact_digest().cmp(right.artifact_digest()))
}
