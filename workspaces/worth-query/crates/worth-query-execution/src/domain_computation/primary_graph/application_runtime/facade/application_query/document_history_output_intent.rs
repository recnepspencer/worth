//! The runtime's consumer methods over `document_history_output_intent` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::HistoryOutputControlPlan;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputIntent;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn capture_history_output_intent<
        H: ApplicationHistoryBinding<S>,
        B: ApplicationQueryBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        unused_1: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        control: HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<WorthQueryHistoryOutputIntent, WorthQueryHistoryOutputReadDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::OutputIntentOps<S>>::capture_history_output_intent(self, phase, unused_1, document, binding, control)
    }
}
