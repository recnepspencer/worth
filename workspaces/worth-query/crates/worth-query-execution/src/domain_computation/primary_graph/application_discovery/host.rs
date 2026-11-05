//! What `application_discovery` needs from the runtime that composes it.

/// The runtime's identity plus what `application_discovery` reads or calls above itself.
pub trait ApplicationDiscoveryHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `application_discovery` is handed.
    fn application_discovery_host(&self) -> &dyn ApplicationDiscoveryHost<Schema>;
}
