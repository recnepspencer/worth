//! What `handler` needs from the runtime that composes it.

/// The runtime's identity plus what `handler` reads or calls above itself.
pub trait HandlerHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `handler` is handed.
    fn handler_host(&self) -> &dyn HandlerHost<Schema>;
    fn mutation_handlers(
        &self,
    ) -> &crate::domain_computation::primary_graph::handler::InstalledMutationHandlerRegistry<Schema>;
}
