//! What `product_activation` needs from the runtime that composes it.

/// The runtime's identity plus what `product_activation` reads or calls above itself.
pub trait ProductActivationHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `product_activation` is handed.
    fn product_activation_host(&self) -> &dyn ProductActivationHost<Schema>;
    fn installed_program_support(&self) -> Option<&crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>>;
    fn bridge_conditional(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>;
}
