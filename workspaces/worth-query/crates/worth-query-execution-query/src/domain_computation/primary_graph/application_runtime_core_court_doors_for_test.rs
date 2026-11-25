//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn primary_provider_for_test(&self) -> &std::sync::Arc<WorthQueryPrimaryGraphProvider> {
        &self.primary_provider
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn product_runtime_for_test(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.product_runtime
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn authority_identity_for_test(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        Self::authority_identity(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn primary_graph_for_test(&self) -> Option<&super::super::super::WorthQueryPrimaryGraph> {
        Self::primary_graph(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn authorization_for_test(&self) -> &Arc<WorthQueryInstalledAuthorizationRegistry> {
        &self.authorization
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn installed_packages_for_test(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex {
        Self::installed_packages(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn bridge_for_test(
        &self,
    ) -> &super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge {
        &self.bridge
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn installed_operations_for_test(
        &self,
    ) -> &Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.installed_operations
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn output_demands_for_test(
        &self,
    ) -> &super::super::super::application_output_demand::WorthQueryOutputDemandRegistry {
        &self.output_demands
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn output_producer_routes_for_test(
        &self,
    ) -> &super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes
    {
        &self.output_producer_routes
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn relational_branch_identity_for_test(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity {
        &self.relational_branch_identity
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn execution_for_test(&self) -> &WorthQueryExecutionRuntime {
        Self::execution(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn execution_mut_for_test(&mut self) -> &mut WorthQueryExecutionRuntime {
        Self::execution_mut(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn primary_graph_authority_for_test(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority {
        Self::primary_graph_authority(self)
    }
}

impl WorthQueryApplicationRuntimeCore {
    #[doc(hidden)]
    pub fn output_producer_routes_mut_for_test(
        &mut self,
    ) -> &mut super::super::super::application_contribution::WorthQueryInstalledOutputProducerRoutes
    {
        &mut self.output_producer_routes
    }
}

impl WorthQueryApplicationRuntimeCore {
    /// One capture of a durable Home's World original history and its owner,
    /// as `cancellation::require` takes them; `None` on a Memory Home.
    #[doc(hidden)]
    pub fn durable_originals_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<(
        worth_runtime_world::facade::WorldProgramActivationHistory,
        worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    )> {
        use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
        let owner = self.product_runtime.owner_ref();
        owner.owner_identity().durable()?;
        let originals = owner
            .inspection_port()
            .observe_program_activation_history(execution)
            .unwrap_or_else(|denial| panic!("original history: {denial:?}"));
        Some((originals, owner.owner_identity()))
    }
}
