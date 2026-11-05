//! Moved whole from `domain_computation::operation_binding::installed_domain_authority`: the items below and every impl they own.
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use worth_query_installation::facade::WorthQueryInstallationGeneration;

/// Runtime-affine proof that one installed domain remains current for an
/// execution binding.
pub struct WorthQueryInstalledDomainExecutionAuthority {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    owner: Arc<str>,
    generation: WorthQueryInstallationGeneration,
    current_generation: Arc<AtomicU64>,
}

impl WorthQueryInstalledDomainExecutionAuthority {
    pub(crate) fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        owner: &str,
        generation: WorthQueryInstallationGeneration,
        current_generation: Arc<AtomicU64>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_authority,
            owner: owner.into(),
            generation,
            current_generation,
        })
    }

    pub fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn installation_generation(&self) -> WorthQueryInstallationGeneration {
        self.generation
    }

    pub fn is_current_installation_generation(&self) -> bool {
        self.current_generation.load(Ordering::Acquire) == self.generation.ordinal()
    }
}

impl WorthQueryInstalledDomainExecutionAuthority {
    #[doc(hidden)]
    pub fn mint_for_test(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        owner: &str,
        generation: WorthQueryInstallationGeneration,
        current_generation: Arc<AtomicU64>,
    ) -> Arc<Self> {
        Self::mint(runtime_authority, owner, generation, current_generation)
    }
}
