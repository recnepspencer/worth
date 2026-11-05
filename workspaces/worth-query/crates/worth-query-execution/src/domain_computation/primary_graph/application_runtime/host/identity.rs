//! The runtime's identity and wiring, lent to every module's host trait.
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use {crate::domain_computation::execution_runtime::WorthQueryApplicationIdentityHost, crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity};
use crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;

impl<Schema: ApplicationSchema> WorthQueryApplicationIdentityHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema> {
        &self.installed_schema
    }
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime.authority_identity()
    }
    fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime
    {
        &self.runtime.product_runtime
    }
    fn installed_operations(
        &self,
    ) -> &std::sync::Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.runtime.installed_operations
    }
    fn authorization_clock(
        &self,
    ) -> &std::sync::Arc<crate::domain_computation::runtime_time::WorthQueryRuntimeClock> {
        &self.runtime.authorization_clock
    }
}

impl<Schema: ApplicationSchema> WorthQueryRuntimeIdentityHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn provider_graph(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.runtime.primary_provider.graph
    }
    fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        self.runtime.primary_graph()
    }
    fn application_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self
    }
    fn bridge(&self) -> &crate::domain_computation::primary_graph::vocabulary::managed_bridge::WorthQueryInstalledApplicationBridge{
        &self.runtime.bridge
    }
    fn granular_invalidation_installation(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation {
        self.runtime.granular_invalidation_installation()
    }
}
