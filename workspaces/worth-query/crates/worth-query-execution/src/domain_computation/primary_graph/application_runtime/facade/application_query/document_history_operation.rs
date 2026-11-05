//! The runtime's consumer methods over `document_history_operation` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryModel;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn assess_document_history_operation<H, M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        admitted: &WorthQueryAdmittedApplicationOperation<S, M::Operation, M::Input, <<M as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        M: ApplicationMutationBinding<S>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::OperationOps<S>>::assess_document_history_operation::<H, M>(self, phase, _access, document, installed, selected, admitted)
    }
}
