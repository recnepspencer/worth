//! The runtime's consumer methods over `document_history_preview` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::application_query::WorthQueryHistoryPreviewReadAdmission;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryNodeId;
use crate::domain_computation::primary_graph::WorthQueryHistoryPreview;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn open_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<WorthQueryHistoryPreview, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::open_document_history_preview(self, phase, access, document, read, node)
    }

    pub fn prepare_document_history_preview_read<'phase, 'scope, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        preview: &WorthQueryHistoryPreview,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryPreviewReadAdmission<'phase, 'scope>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::prepare_document_history_preview_read(self, phase, access, document, preview, read)
    }

    pub fn read_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        admission: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query, <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::read_document_history_preview(self, phase, _access, document, admission, read)
    }
}
