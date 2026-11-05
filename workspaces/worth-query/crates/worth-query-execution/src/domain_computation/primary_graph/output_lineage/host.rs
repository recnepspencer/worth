//! What `output_lineage` needs from the runtime that composes it.

/// The runtime's identity plus what `output_lineage` reads or calls above itself.
pub trait OutputLineageHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `output_lineage` is handed.
    fn output_lineage_host(&self) -> &dyn OutputLineageHost<Schema>;
}
