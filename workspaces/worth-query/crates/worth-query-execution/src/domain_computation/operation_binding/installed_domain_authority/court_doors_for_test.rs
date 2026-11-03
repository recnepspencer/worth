//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInstalledDomainExecutionAuthority {
    #[doc(hidden)]
    pub fn mint_for_test( runtime_authority: WorthQueryRuntimeAuthorityIdentity, owner: &str, generation: WorthQueryInstallationGeneration, current_generation: Arc<AtomicU64>) -> Arc<Self> {
        Self::mint(runtime_authority, owner, generation, current_generation)
    }
}
