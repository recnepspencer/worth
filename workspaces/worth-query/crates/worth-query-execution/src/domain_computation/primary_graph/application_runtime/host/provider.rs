//! The runtime composes `provider`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::provider::host::ProviderHost;

impl<Schema: ApplicationSchema> ProviderHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn provider_host(&self) -> &dyn ProviderHost<Schema> {
        self
    }
    fn primary_provider(
        &self,
    ) -> &std::sync::Arc<
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    > {
        &self.runtime.primary_provider
    }
    fn installed_operations(
        &self,
    ) -> &std::sync::Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.runtime.installed_operations
    }
    fn bridge(&self) -> &crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge{
        &self.runtime.bridge
    }
    fn recovered_program_source_delivery(&self) -> &crate::domain_computation::primary_graph::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration{
        &self.runtime.recovered_program_source_delivery
    }
}
