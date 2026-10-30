use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use worth_foundational::facade::FoundationalIdentityKind;

use crate::identity_authority::{
    admit_bridge_truth_authority_identity, BridgeCanonicalDigestIdentityBasis,
    BridgeEvidenceReferenceIdentityKind, BridgeTruthBoundaryBridgedIdentity,
    BridgeTruthDigestIdentityEvidence, BridgeTruthExternalIdentityToken,
    BridgeTruthProjectionIdentity,
};

pub struct BridgeIdentityEvidence {
    value: Arc<str>,
    payload: BridgeIdentityEvidencePayload,
}

impl BridgeIdentityEvidence {
    pub(crate) fn as_str(&self) -> &str {
        self.value.as_ref()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn terminal_projection_for_reporting(&self) -> &str {
        self.as_str()
    }

    #[cfg(test)]
    pub(crate) fn from_bridge_owner_external_authority(value: impl Into<Arc<str>>) -> Self {
        Self::from_external_authority(
            crate::identity_authority::bridge_truth_external_identity_token(value),
        )
    }

    pub fn from_external_authority(
        token: BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    ) -> Self {
        Self {
            value: token.into_value(),
            payload: BridgeIdentityEvidencePayload::ExternalAuthority,
        }
    }

    pub fn from_query_evidence_identity(
        scope: BridgeTruthProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
        identity_token: BridgeTruthDigestIdentityEvidence<
            BridgeCanonicalDigestIdentityBasis,
            BridgeEvidenceReferenceIdentityKind,
        >,
    ) -> Self {
        let _identity_token = identity_token;
        Self {
            value: Arc::clone(scope.label()),
            payload: BridgeIdentityEvidencePayload::QueryEvidenceIdentity {
                scope: scope.into_label(),
            },
        }
    }

    pub(crate) fn from_canonical_bridge_evidence(
        value: impl Into<Arc<str>>,
        scope: &'static str,
    ) -> Self {
        Self {
            value: value.into(),
            payload: BridgeIdentityEvidencePayload::CanonicalBridgeEvidence { scope },
        }
    }

    pub(crate) fn revalidate_bridge_retained_reference(
        &self,
    ) -> BridgeTruthBoundaryBridgedIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind> {
        admit_bridge_truth_authority_identity(Arc::clone(&self.value)).bridge_trust_boundary()
    }

    pub fn from_boundary_bridged_identity<Kind>(
        boundary: &BridgeTruthBoundaryBridgedIdentity<Arc<str>, Kind>,
    ) -> Self
    where
        Kind: FoundationalIdentityKind,
    {
        Self {
            value: Arc::clone(boundary.value()),
            payload: BridgeIdentityEvidencePayload::ExternalAuthority,
        }
    }
}

pub fn bridge_identity_reporting_label(evidence: &BridgeIdentityEvidence) -> &str {
    evidence.as_str()
}

impl Clone for BridgeIdentityEvidence {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            payload: self.payload.clone(),
        }
    }
}

impl PartialEq for BridgeIdentityEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.payload == other.payload
    }
}

impl Eq for BridgeIdentityEvidence {}

impl PartialOrd for BridgeIdentityEvidence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BridgeIdentityEvidence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.payload
            .cmp(&other.payload)
            .then_with(|| self.value.cmp(&other.value))
    }
}

impl Hash for BridgeIdentityEvidence {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.payload.hash(state);
        self.value.hash(state);
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum BridgeIdentityEvidencePayload {
    ExternalAuthority,
    CanonicalBridgeEvidence { scope: &'static str },
    QueryEvidenceIdentity { scope: Arc<str> },
}

impl fmt::Debug for BridgeIdentityEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeIdentityEvidence")
            .field(&"<opaque>")
            .finish()
    }
}
