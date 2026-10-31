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
        let selected = match self.history.as_ref() {
            None => self.select(phase, request)?.0,
            Some(restriction) => {
                if !std::ptr::eq(self.runtime.runtime(), request.application)
                    || request.branch != restriction.original_source()
                    || self.description.branch != restriction.candidate()
                {
                    return Err(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch,
                    );
                }
                let selected = self
                    .runtime
                    .runtime()
                    .on_branch(request.branch)
                    .select(phase)
                    .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
                let owner = self.runtime.selected_program_owner(&selected)
                    .map_err(crate::application_entry::mutation::selected_program::map_selected_program_owner_denial)?;
                owner
                    .resolve_declared_action::<I::Binding>(&self.installed_action)
                    .map_err(|_| {
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
                    })?;
                selected
            }
        };
        let mut native = self
            .description
            .request_view(request, &self.installed_action);
        // This temporary view authorizes the document transition. It neither
        // relocates the active candidate owner nor resolves/claims any native key.
        native.request.branch = request.branch;
        let staged = native.stage()?;
        let identities = native.identities()?;
        let prepared =
            <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(
                phase,
                &native,
                &identities,
                staged,
                &selected,
            )?;
        Ok((prepared, selected))
    }

    pub(in crate::application_entry::operation) fn validate_history_prepared(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        prepared: &PreparedMutation<S, I::Binding>,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial> {
        if let Some(restriction) = &self.history {
            self.runtime
                .runtime()
                .validate_history_operation(
                    phase,
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    restriction,
                    &prepared.admission,
                    prepared.idempotency,
                )
                .map_err(WorthQueryApplicationRequestMutationDenial::DocumentHistory)?;
        }
        Ok(())
    }
}
