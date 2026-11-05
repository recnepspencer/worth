//! What `application_aftermath` needs from the runtime that composes it.

/// The application identity plus what `application_aftermath` reads or calls above itself.
pub trait ApplicationAftermathHost<Schema>:
    crate::domain_computation::execution_runtime::WorthQueryApplicationIdentityHost<Schema>
{
    /// This host as the trait object `application_aftermath` is handed.
    fn application_aftermath_host(&self) -> &dyn ApplicationAftermathHost<Schema>;
    fn recovery_handles(
        &self,
    ) -> &std::sync::Arc<crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry>;
}
