//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn progress_retained_inbound_occurrence_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        correlation_token: [u8; 32],
        request: &WorthQueryRequestScope,
    ) -> Result<Posture, Denial> {
        Self::progress_retained_inbound_occurrence(
            self,
            execution,
            phase,
            correlation_token,
            request,
        )
    }
}
