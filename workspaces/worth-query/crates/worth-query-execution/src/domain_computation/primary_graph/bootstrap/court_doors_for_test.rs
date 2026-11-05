//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn managed_indexes_for_test(
        &self,
    ) -> &super::super::application_query::managed_index::ManagedIndexProviderCatalog<Schema> {
        &self.managed_indexes
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn managed_indexes_mut_for_test(
        &mut self,
    ) -> &mut super::super::application_query::managed_index::ManagedIndexProviderCatalog<Schema>
    {
        &mut self.managed_indexes
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn graph_for_test(&self) -> &WorthQueryPrimaryGraph {
        &self.graph
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema> {
    #[doc(hidden)]
    pub fn mutation_handlers_for_test(
        &self,
    ) -> &super::super::handler::PendingMutationHandlerRegistry<Schema> {
        &self.mutation_handlers
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn publish_for_test(
        self,
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial> {
        Self::publish(self, phase, runtime, authority)
    }
}
