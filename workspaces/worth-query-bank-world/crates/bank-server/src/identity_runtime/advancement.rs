//! Bank host-call custody over its installed Query application.
use super::BankIdentityRuntime;

impl BankIdentityRuntime {
    /// Lends the installed phase to a host call without exposing the runtime.
    pub fn with_application_advancement<R>(
        &self,
        scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
        body: impl for<'phase> FnOnce(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'phase>,
        ) -> R,
    ) -> Result<R, worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial>
    {
        self.application_runtime()
            .with_application_advancement(scope, body)
    }
}
