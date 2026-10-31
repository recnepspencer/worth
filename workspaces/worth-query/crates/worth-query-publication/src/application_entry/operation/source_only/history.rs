//! Only an initial ordinary action can take the document-created candidate target.
use super::*;
use worth_query_declaration::facade::{
    application_operation::NoApplicationMutationSource,
    application_program::ApplicationSourceCompletion,
};
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistoryDenial as Denial, WorthQueryDocumentHistoryOperationConstraint,
};

impl<S, I, P, A> WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    pub(in crate::application_entry) fn history_target(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.admission.description.branch
    }

    pub(in crate::application_entry) fn bind_history_candidate(
        mut self,
        restriction: WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<Self, (Self, Denial)> {
        if !matches!(self.state, SourceState::<S, I>::Initial)
            || self.admission.workflow.is_some()
            || self.admission.recorded_correction.is_some()
            || self.admission.description.history.is_some()
            || I::Binding::WORKFLOW_CONTROL
            || I::Binding::REQUIRES_WORKFLOW_AUTHORITY
            || I::Binding::workflow_execution_requirements().is_applicable()
        {
            return Err((self, Denial::UnsupportedOperation));
        }
        if self.admission.description.branch != restriction.original_source()
            || !self
                .admission
                .runtime
                .runtime()
                .product_runtime()
                .owns_branch_selector(restriction.candidate())
        {
            return Err((self, Denial::ForeignRecording));
        }
        // No active native attempt is relocated. Every original description field
        // stays owned; only this initial action's private execution target changes.
        self.admission.description.branch = restriction.candidate();
        self.admission.description.history = Some(restriction);
        Ok(self)
    }
}

impl<S, I, P, A> WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn close_for_history(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::application_entry::WorthQueryClosedOperation,
        crate::application_entry::WorthQueryOperationCloseDenial,
    > {
        let closed = self.close_in_place(request)?;
        // Terminal custody cannot advance. The restriction's original-source pin
        // is no longer needed once actual native cleanup has discharged.
        self.operation.admission.description.history = None;
        Ok(closed)
    }
}
