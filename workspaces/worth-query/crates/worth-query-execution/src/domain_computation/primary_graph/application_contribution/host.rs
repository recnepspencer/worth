//! What `application_contribution` needs from the runtime that composes it.

/// The runtime's identity plus what `application_contribution` reads or calls above itself.
pub trait ApplicationContributionHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
    + crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost<Schema>
    + crate::domain_computation::product_publication::host::ProductPublicationHost<Schema>
{
    /// This host as the trait object `application_contribution` is handed.
    fn application_contribution_host(&self) -> &dyn ApplicationContributionHost<Schema>;
    fn output_production(&self) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryOutputProduction<Schema>;
}
