//! What `application_entry` needs from the runtime that composes it.

/// The runtime's identity plus what `application_entry` reads or calls above itself.
pub trait ApplicationEntryHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `application_entry` is handed.
    fn application_entry_host(&self) -> &dyn ApplicationEntryHost<Schema>;
}
