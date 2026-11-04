//! Sealed test doors over this module's private members.
use super::*;

impl ExternalEffectCorrelationIdentity {
    #[doc(hidden)]
    pub fn from_digest_for_test(digest: CanonicalDigestId) -> Self {
        Self::from_digest(digest)
    }
}
