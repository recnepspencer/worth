//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRecoveryEffectAuthority {
    #[doc(hidden)]
    pub fn mint_for_test(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
    ) -> Self {
        Self::mint(runtime_authority, handle_authority)
    }
}

impl WorthQueryRecoveryInspectAuthority {
    #[doc(hidden)]
    pub fn mint_for_test(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
    ) -> Self {
        Self::mint(runtime_authority, handle_authority)
    }
}
