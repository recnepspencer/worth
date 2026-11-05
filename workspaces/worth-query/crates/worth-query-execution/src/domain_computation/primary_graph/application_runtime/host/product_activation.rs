//! The runtime composes `product_activation`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::product_activation::host::ProductActivationHost;

impl<Schema: ApplicationSchema> ProductActivationHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_activation_host(&self) -> &dyn ProductActivationHost<Schema> {
        self
    }
    fn installed_program_support(&self) -> Option<&crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>{
        Self::installed_program_support(self)
    }
    fn bridge_conditional(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>
    {
        self.runtime.bridge.conditional()
    }
}
