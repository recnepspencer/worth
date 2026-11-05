//! What `resolution` needs from the runtime that composes it.

/// The runtime's identity plus what `resolution` reads or calls above itself.
pub trait ResolutionHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `resolution` is handed.
    fn resolution_host(&self) -> &dyn ResolutionHost<Schema>;
}
