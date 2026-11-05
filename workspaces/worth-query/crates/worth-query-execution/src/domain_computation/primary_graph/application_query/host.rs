//! What `application_query` needs from the runtime that composes it.

/// The runtime's identity plus what `application_query` reads or calls above itself.
pub trait ApplicationQueryHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    /// This host as the trait object `application_query` is handed.
    fn application_query_host(&self) -> &dyn ApplicationQueryHost<Schema>;
    fn managed_indexes(&self) -> &crate::domain_computation::primary_graph::application_query::managed_index::ManagedIndexProviderCatalog<Schema>;
    fn relational_branch_identity(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity;
    fn source_meanings(&self) -> &crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry;
}
