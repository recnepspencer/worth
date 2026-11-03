//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn artifact_contracts_for_test(&self) -> &WorthQueryInstalledWorkflowArtifactContracts {
        &self.artifact_contracts
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn direct_resource_topology_for_test(&self) -> &WorthQueryExecutionResourceTopology {
        &self.direct_resource_topology
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn installation_runtime_ordinal_for_test(&self) -> &u64 {
        &self.installation_runtime_ordinal
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn provider_plan_declarations_for_test(&self) -> &Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations> {
        &self.provider_plan_declarations
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn topology_for_test(&self) -> &WorthQueryExecutionResourceTopology {
        &self.topology
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn artifact_contracts_mut_for_test(&mut self) -> &mut WorthQueryInstalledWorkflowArtifactContracts {
        &mut self.artifact_contracts
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn direct_resource_topology_mut_for_test(&mut self) -> &mut WorthQueryExecutionResourceTopology {
        &mut self.direct_resource_topology
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn installation_runtime_ordinal_mut_for_test(&mut self) -> &mut u64 {
        &mut self.installation_runtime_ordinal
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn provider_plan_declarations_mut_for_test(&mut self) -> &mut Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations> {
        &mut self.provider_plan_declarations
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn topology_mut_for_test(&mut self) -> &mut WorthQueryExecutionResourceTopology {
        &mut self.topology
    }
}
