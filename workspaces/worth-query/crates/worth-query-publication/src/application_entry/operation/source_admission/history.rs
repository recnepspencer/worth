//! The original action prepares fresh source-side observation and candidate-side restrictions.
use super::*;
use crate::application_entry::mutation::authorization::PreparedMutation;

impl<S, I, P, A, Mode> WorthQuerySourceOperationAdmission<S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub(in crate::application_entry::operation) fn prepare_history_source<'owner>(
        &'owner self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            PreparedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                'owner,
                S,
            >,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        if self.workflow.is_some()
            || self.recorded_correction.is_some()
            || self.description.source.is_some()
        {
            return Err(WorthQueryApplicationRequestMutationDenial::DocumentHistory(
                worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial::UnsupportedOperation,
            ));
        }
        self.description.prepare_history_source(
            &self.runtime,
            &self.installed_action,
            phase,
            request,
        )
    }
}
