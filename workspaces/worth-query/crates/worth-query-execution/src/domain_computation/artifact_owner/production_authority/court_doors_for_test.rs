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
    pub fn new_for_test(
        contract: Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
        domain_authority: Arc<
            crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority,
        >,
        operation_identity: String,
        binding_identity: String,
        run_identity: String,
        stage_identity: String,
        basis_identity: String,
        registry: Arc<WorthQueryWorkflowArtifactRegistry>,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> Self {
        Self {
            contract,
            domain_authority,
            operation_identity,
            binding_identity,
            run_identity,
            stage_identity,
            basis_identity,
            registry,
            production_generation,
        }
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn basis_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.basis_identity
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn binding_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.binding_identity
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn operation_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.operation_identity
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn run_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.run_identity
    }
}

impl WorthQueryArtifactProductionAuthorityParts {
    #[doc(hidden)]
    pub fn stage_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.stage_identity
    }
}
