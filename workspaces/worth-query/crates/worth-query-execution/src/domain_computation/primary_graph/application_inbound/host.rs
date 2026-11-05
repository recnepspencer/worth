//! What `application_inbound` needs from the runtime that composes it.

/// The runtime's identity plus what `application_inbound` reads or calls above itself.
pub trait ApplicationInboundHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::product_publication::host::ProductPublicationHost<Schema>
    + crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>
{
    /// This host as the trait object `application_inbound` is handed.
    fn application_inbound_host(&self) -> &dyn ApplicationInboundHost<Schema>;
    fn inbound(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_inbound::WorthQueryInboundProcessing;
}
