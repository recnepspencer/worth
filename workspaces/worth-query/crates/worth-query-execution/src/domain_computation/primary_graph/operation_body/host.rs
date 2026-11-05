//! What `operation_body` needs from the runtime that composes it.

/// The runtime's identity plus what `operation_body` reads or calls above itself.
pub trait OperationBodyHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `operation_body` is handed.
    fn operation_body_host(&self) -> &dyn OperationBodyHost<Schema>;
}
