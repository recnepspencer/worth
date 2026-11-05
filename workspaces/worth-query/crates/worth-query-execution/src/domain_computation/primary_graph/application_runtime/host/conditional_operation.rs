//! The runtime composes `conditional_operation`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::conditional_operation::host::ConditionalOperationHost;

impl<Schema: ApplicationSchema> ConditionalOperationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn conditional_operation_host(&self) -> &dyn ConditionalOperationHost<Schema> {
        self
    }
    fn conditional_operations(&self) -> &std::sync::Mutex<crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>>{
        &self.conditional_operations
    }
    fn bridge(&self) -> &crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge{
        &self.runtime.bridge
    }
    fn granular_invalidation(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation {
        &self.runtime.granular_invalidation
    }
    fn installed_resources(&self) -> &crate::domain_computation::primary_graph::application_resource::installation::InstalledResourceRegistry<Schema>{
        &self.installed_resources
    }
    fn retain_installed_packages(
        &self,
    ) -> std::sync::Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex> {
        self.runtime.retain_installed_packages()
    }
    fn release_conditional_runtime_resources(&mut self) {
        Self::release_conditional_runtime_resources(self)
    }
}
