use std::sync::Arc;

use worth_foundational::facade::{
    admit_foundational_authority_identity, admit_foundational_external_identity_token,
    derive_foundational_digest_identity_evidence, project_foundational_identity,
    CanonicalDerivedDigest, FoundationalExternalIdentityToken, FoundationalIdentityKind,
};

use super::{
    bridge_truth_authority, BridgeCanonicalDigestIdentityBasis,
    BridgeEvidenceReferenceIdentityKind, BridgeTruthAuthorityIdentity,
    BridgeTruthDigestIdentityEvidence, BridgeTruthExternalIdentityToken,
    BridgeTruthProjectionIdentity,
};

pub(crate) fn admit_bridge_truth_authority_identity(
    value: impl Into<Arc<str>>,
) -> BridgeTruthAuthorityIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind> {
    admit_foundational_authority_identity(value.into(), bridge_truth_authority())
}

pub(crate) fn admit_bridge_truth_authority_identity_for_kind<Value, Kind>(
    value: Value,
) -> BridgeTruthAuthorityIdentity<Value, Kind>
where
    Kind: FoundationalIdentityKind,
{
    admit_foundational_authority_identity(value, bridge_truth_authority())
}

pub fn bridge_truth_external_identity_token(
    value: impl Into<Arc<str>>,
) -> BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind> {
    FoundationalExternalIdentityToken::new(value.into())
}

pub fn bridge_truth_projection_identity_from_external_token(
    token: BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    label: impl Into<Arc<str>>,
) -> BridgeTruthProjectionIdentity<Arc<str>, BridgeEvidenceReferenceIdentityKind> {
    let authority = bridge_truth_authority();
    let identity = admit_foundational_external_identity_token(token, authority);
    project_foundational_identity(&identity, label.into(), bridge_truth_authority())
}

pub fn bridge_truth_digest_identity_evidence_from_external_token(
    token: BridgeTruthExternalIdentityToken<Arc<str>, BridgeEvidenceReferenceIdentityKind>,
    digest: CanonicalDerivedDigest,
) -> BridgeTruthDigestIdentityEvidence<
    BridgeCanonicalDigestIdentityBasis,
    BridgeEvidenceReferenceIdentityKind,
> {
    let authority = bridge_truth_authority();
    let identity = admit_foundational_external_identity_token(token, authority);
    derive_foundational_digest_identity_evidence(&identity, digest, bridge_truth_authority())
}
