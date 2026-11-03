//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryArtifactProductionAuthority {
    #[doc(hidden)]
    pub fn mint_for_test(parts: WorthQueryArtifactProductionAuthorityParts) -> Arc<Self> {
        Self::mint(parts)
    }
}
