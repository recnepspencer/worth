//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity;

pub trait ResumeDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn resume_installed_transport_completion_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        correlation: &ExternalEffectCorrelationIdentity,
        request: &WorthQueryRequestScope,
    ) -> InstalledTransportResumeOutcome {
        Self::resume_installed_transport_completion(self, execution, phase, correlation, request)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> ResumeDoorsOps<Schema> for HostRuntime {}
