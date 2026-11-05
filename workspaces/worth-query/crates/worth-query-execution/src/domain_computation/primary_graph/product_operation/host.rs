//! What `product_operation` needs from the runtime that composes it.

/// The runtime's identity plus what `product_operation` reads or calls above itself.
pub trait ProductOperationHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::primary_graph::product_activation::host::ProductActivationHost<
        Schema,
    > + crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<
        Schema,
    > + crate::domain_computation::primary_graph::conditional_operation::host::ConditionalOperationHost<
        Schema,
    > + crate::basis::WorthQuerySourceProgramResolver
{
    /// This host as the trait object `product_operation` is handed.
    fn product_operation_host(&self) -> &dyn ProductOperationHost<Schema>;
    fn application_readiness_schema_token(&self) -> &str;
    fn basis_leases(&self) -> &crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisRegistry;
    fn installed_producers(&self) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>;
    fn workflow_coverage(&self) -> &crate::domain_computation::primary_graph::application_attempt::workflow::adoption::WorkflowVocabularyCoverageRegistry;
}
