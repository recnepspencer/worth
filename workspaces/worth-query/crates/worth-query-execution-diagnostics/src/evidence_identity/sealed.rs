use worth_foundational::facade::CanonicalDerivedDigest;

use super::scheme::WorthQueryEvidenceIdentityScheme;
use super::scope::WorthQueryEvidenceScope;

pub(crate) struct SealedWorthQueryEvidenceIdentity {
    pub(crate) scope: WorthQueryEvidenceScope,
    pub(crate) scheme: WorthQueryEvidenceIdentityScheme,
    pub(crate) digest_token: String,
    pub(crate) canonical_digest: CanonicalDerivedDigest,
}

impl SealedWorthQueryEvidenceIdentity {
    pub(crate) fn new(
        scope: WorthQueryEvidenceScope,
        scheme: WorthQueryEvidenceIdentityScheme,
        digest_token: String,
        canonical_digest: CanonicalDerivedDigest,
    ) -> Self {
        Self {
            scope,
            scheme,
            digest_token,
            canonical_digest,
        }
    }
}
