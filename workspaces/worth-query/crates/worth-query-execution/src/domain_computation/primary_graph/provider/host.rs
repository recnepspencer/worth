//! What `provider` needs from the runtime that composes it.

/// The runtime's identity plus what `provider` reads or calls above itself.
pub trait ProviderHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::product_publication::host::ProductPublicationHost<Schema>
{
    /// This host as the trait object `provider` is handed.
    fn provider_host(&self) -> &dyn ProviderHost<Schema>;
    fn primary_provider(
        &self,
    ) -> &std::sync::Arc<
        crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    >;
    fn recovered_program_source_delivery(&self) -> &crate::domain_computation::primary_graph::provider::recovered_program_source_delivery::RecoveredProgramSourceDeliveryRegistration;
}
