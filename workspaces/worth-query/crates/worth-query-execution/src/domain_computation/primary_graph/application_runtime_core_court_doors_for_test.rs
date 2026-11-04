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
