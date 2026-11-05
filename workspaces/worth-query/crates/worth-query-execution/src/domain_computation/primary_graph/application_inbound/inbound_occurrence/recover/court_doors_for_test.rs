//! Sealed test doors over this module's private members.
use super::*;

pub trait RecoverDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn progress_retained_inbound_occurrence_for_test(
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

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> RecoverDoorsOps<Schema> for HostRuntime {}
