//! The runtime's consumer methods over `document_history_mod` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::product_operation::WorthQueryOwnedProductHead;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryAttachmentFailure;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn attach_document_history<'head, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        head: WorthQueryOwnedProductHead<'head, S>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<
        WorthQueryDocumentHistory<'head, S, H>,
        WorthQueryDocumentHistoryAttachmentFailure<'head, S>,
    >
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::DocumentHistoryOps<
            S,
        >>::attach_document_history(self, phase, _access, head, read)
    }

    pub fn read_document_history<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query, <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::DocumentHistoryOps<
            S,
        >>::read_document_history(self, phase, _access, document, read)
    }
}
