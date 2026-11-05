//! What `principal_currentness_capture` needs from the runtime that composes it.

/// The runtime's identity plus what `principal_currentness_capture` reads or calls above itself.
pub trait PrincipalCurrentnessCaptureHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `principal_currentness_capture` is handed.
    fn principal_currentness_capture_host(&self) -> &dyn PrincipalCurrentnessCaptureHost<Schema>;
}
