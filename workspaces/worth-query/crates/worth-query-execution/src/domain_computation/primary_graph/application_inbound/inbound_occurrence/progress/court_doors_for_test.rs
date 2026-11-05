//! Sealed test doors over this module's private members.
use super::*;

pub trait ProgressDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn progress_accepted_inbound_occurrence_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceiptPosture, WorthQueryInboundAdmissionDenial> {
        Self::progress_accepted_inbound_occurrence(self, execution, phase, accepted, request)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> ProgressDoorsOps<Schema> for HostRuntime {}
