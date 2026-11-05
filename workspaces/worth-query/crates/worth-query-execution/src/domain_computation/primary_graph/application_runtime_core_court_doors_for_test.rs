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
