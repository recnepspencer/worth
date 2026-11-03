//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryArtifactProductionAuthority {
    #[doc(hidden)]
    pub fn mint_for_test(parts: WorthQueryArtifactProductionAuthorityParts) -> Arc<Self> {
        Self::mint(parts)
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn new_for_test(contract: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>, domain_authority: Arc<crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority>, operation_identity: String, binding_identity: String, run_identity: String, stage_identity: String, basis_identity: String, registry: Arc<WorthQueryWorkflowArtifactRegistry>, production_generation: WorthQueryArtifactProductionGeneration) -> Self {
        Self { contract, domain_authority, operation_identity, binding_identity, run_identity, stage_identity, basis_identity, registry, production_generation }
    }
}
