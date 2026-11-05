//! What `application_installation` needs from the runtime that composes it.

/// The runtime's identity plus what `application_installation` reads or calls above itself.
pub trait ApplicationInstallationHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `application_installation` is handed.
    fn application_installation_host(&self) -> &dyn ApplicationInstallationHost<Schema>;
    fn recovered_outputs(&self) -> &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs;
}
