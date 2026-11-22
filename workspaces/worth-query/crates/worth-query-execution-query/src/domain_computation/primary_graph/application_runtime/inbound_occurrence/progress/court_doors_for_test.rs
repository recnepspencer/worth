//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn progress_accepted_inbound_occurrence_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceiptPosture, WorthQueryInboundAdmissionDenial> {
        Self::progress_accepted_inbound_occurrence(self, execution, phase, accepted, request)
    }
}
