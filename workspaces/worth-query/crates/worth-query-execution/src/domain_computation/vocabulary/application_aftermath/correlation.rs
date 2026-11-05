//! Moved whole from `domain_computation::application_aftermath::external_effect::correlation`: the items below and every impl they own.
use worth_foundational::facade::CanonicalDigestId;

/// Query-owned correlation identity. Equality is digest equality only.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExternalEffectCorrelationIdentity {
    pub(in crate::domain_computation) digest: CanonicalDigestId,
}

impl ExternalEffectCorrelationIdentity {
    pub(crate) const fn from_digest(digest: CanonicalDigestId) -> Self {
        Self { digest }
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub fn bytes(&self) -> &[u8; 32] {
        self.digest.bytes()
    }
}

impl ExternalEffectCorrelationIdentity {
    #[doc(hidden)]
    pub fn from_digest_for_test(digest: CanonicalDigestId) -> Self {
        Self::from_digest(digest)
    }
}
