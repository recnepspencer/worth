//! What `conditional_operation` needs from the runtime that composes it.

/// The runtime's identity plus what `conditional_operation` reads or calls above itself.
pub trait ConditionalOperationHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    /// This host as the trait object `conditional_operation` is handed.
    fn conditional_operation_host(&self) -> &dyn ConditionalOperationHost<Schema>;
    fn conditional_operations(&self) -> &std::sync::Mutex<crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>>;
    fn bridge(&self) -> &crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge;
    fn granular_invalidation(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation;
    fn installed_resources(&self) -> &crate::domain_computation::primary_graph::application_resource::installation::InstalledResourceRegistry<Schema>;
    fn retain_installed_packages(
        &self,
    ) -> std::sync::Arc<worth_query_installation::facade::WorthQueryInstalledPackageIndex>;
    fn release_conditional_runtime_resources(&mut self);
}
