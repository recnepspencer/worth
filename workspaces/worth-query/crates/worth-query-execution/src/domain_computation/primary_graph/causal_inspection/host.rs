//! What `causal_inspection` needs from the runtime that composes it.

/// The runtime's identity plus what `causal_inspection` reads or calls above itself.
pub trait CausalInspectionHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
    + crate::domain_computation::authorization::host::AuthorizationHost<Schema>
{
    /// This host as the trait object `causal_inspection` is handed.
    fn causal_inspection_host(&self) -> &dyn CausalInspectionHost<Schema>;
    fn inspection_materialization_prepared(&self);
}
