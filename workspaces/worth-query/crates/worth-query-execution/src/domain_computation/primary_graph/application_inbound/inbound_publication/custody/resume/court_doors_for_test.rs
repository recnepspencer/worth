//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn resume_installed_transport_completion_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        correlation: &ExternalEffectCorrelationIdentity,
        request: &WorthQueryRequestScope,
    ) -> InstalledTransportResumeOutcome {
        Self::resume_installed_transport_completion(self, execution, phase, correlation, request)
    }
}
