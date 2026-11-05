//! The runtime's consumer methods over `document_history_metadata` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryStep;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn read_document_history_step<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::MetadataOps<S>>::read_document_history_step(self, phase, access, document, read)
    }

    pub fn read_document_history_selection<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<crate::domain_computation::primary_graph::WorthQueryHistorySelection, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::MetadataOps<S>>::read_document_history_selection(self, phase, access, document, read)
    }
}
