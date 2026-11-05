//! The runtime's consumer methods over `document_history_retention` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryNodeId;
use crate::domain_computation::primary_graph::WorthQueryHistoryRetentionStatus;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBookmarkBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps;
use worth_query_declaration::facade::application_history::ApplicationHistoryWindowBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn change_document_history_window<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        window: ApplicationHistoryRecentSteps,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryWindowBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::change_document_history_window::<H, N>(self, phase, access, document, history_access, read, window)
    }

    pub fn change_document_history_bookmark<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryBookmarkBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::change_document_history_bookmark::<H, N>(self, phase, access, document, history_access, read, node, pin)
    }

    pub fn read_document_history_retention<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::read_document_history_retention(self, phase, access, document, read)
    }
}
