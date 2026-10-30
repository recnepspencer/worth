use super::encoder::WorthQueryEvidenceIdentityEncoder;
use super::scheme::WorthQueryEvidenceIdentityScheme;
use super::scope::WorthQueryEvidenceScope;
use super::sealed::SealedWorthQueryEvidenceIdentity;
use worth_foundational::facade::{CanonicalDerivedDigest, CanonicalDigestId};
use worth_runtime_bridge::facade::{
    bridge_truth_digest_identity_evidence_from_external_token,
    bridge_truth_external_identity_token, bridge_truth_projection_identity_from_external_token,
    BridgeIdentityEvidence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEvidenceIdentityComparisonError {
    SchemeMismatch {
        left: WorthQueryEvidenceIdentityScheme,
        right: WorthQueryEvidenceIdentityScheme,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEvidenceIdentity {
    scope: WorthQueryEvidenceScope,
    scheme: WorthQueryEvidenceIdentityScheme,
    digest_token: String,
    canonical_digest: CanonicalDerivedDigest,
}

impl WorthQueryEvidenceIdentity {
    pub fn compose(scope: WorthQueryEvidenceScope) -> WorthQueryEvidenceIdentityEncoder {
        WorthQueryEvidenceIdentityEncoder::new(scope, WorthQueryEvidenceIdentityScheme::V1)
    }

    pub fn compose_with_scheme(
        scope: WorthQueryEvidenceScope,
        scheme: WorthQueryEvidenceIdentityScheme,
    ) -> WorthQueryEvidenceIdentityEncoder {
        WorthQueryEvidenceIdentityEncoder::new(scope, scheme)
    }

    pub(crate) fn new(sealed: SealedWorthQueryEvidenceIdentity) -> Self {
        Self {
            scope: sealed.scope,
            scheme: sealed.scheme,
            digest_token: sealed.digest_token,
            canonical_digest: sealed.canonical_digest,
        }
    }

    pub fn scope(&self) -> WorthQueryEvidenceScope {
        self.scope
    }

    pub fn scheme(&self) -> WorthQueryEvidenceIdentityScheme {
        self.scheme
    }

    pub fn as_str(&self) -> &str {
        &self.digest_token
    }

    pub fn reporting_projection(&self) -> &str {
        self.as_str()
    }

    pub fn terminal_projection_for_reporting(&self) -> &str {
        self.reporting_projection()
    }

    pub fn bridge_evidence_identity(&self) -> BridgeIdentityEvidence {
        let token = bridge_truth_external_identity_token(self.as_str());
        let scope = bridge_truth_projection_identity_from_external_token(
            token.clone(),
            self.scope.as_str(),
        );
        let identity_token = bridge_truth_digest_identity_evidence_from_external_token(
            token,
            self.canonical_digest.clone(),
        );
        BridgeIdentityEvidence::from_query_evidence_identity(scope, identity_token)
    }

    pub fn bridge_external_identity_evidence(&self) -> BridgeIdentityEvidence {
        BridgeIdentityEvidence::from_external_authority(bridge_truth_external_identity_token(
            self.as_str(),
        ))
    }

    pub fn eq_same_scheme(
        &self,
        other: &Self,
    ) -> Result<bool, WorthQueryEvidenceIdentityComparisonError> {
        self.same_scheme_as(other)?;
        Ok(self.digest_token == other.digest_token)
    }

    pub fn same_scheme_as(
        &self,
        other: &Self,
    ) -> Result<(), WorthQueryEvidenceIdentityComparisonError> {
        if self.scheme == other.scheme {
            Ok(())
        } else {
            Err(WorthQueryEvidenceIdentityComparisonError::SchemeMismatch {
                left: self.scheme,
                right: other.scheme,
            })
        }
    }

    pub fn compare_same_scheme(
        &self,
        other: &Self,
    ) -> Result<std::cmp::Ordering, WorthQueryEvidenceIdentityComparisonError> {
        self.same_scheme_as(other)?;
        Ok(self.digest_token.cmp(&other.digest_token))
    }

    pub fn canonical_digest(&self) -> &CanonicalDerivedDigest {
        &self.canonical_digest
    }

    pub fn operational_key(&self) -> super::WorthQueryEvidenceIdentityKey {
        super::WorthQueryEvidenceIdentityKey::new(
            self.scope,
            self.scheme,
            *self.canonical_digest.value().bytes(),
        )
    }

    pub fn canonical_digest_id(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(*self.canonical_digest.value().bytes())
    }
}
