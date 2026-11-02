//! Original-source assessment is an observation; actual execution keeps its restriction.
use super::*;
use crate::application_entry::mutation::authorization::{
    PreparedMutation, WorthQueryBindingAdmission,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;

impl<S, I> WorthQueryOperationDescription<S, I>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub(super) fn prepare_history_source<'owner, P: ApplicationProgramDefinition<S>>(
        &'owner self,
        runtime: &'owner WorthQueryProgramApplicationRuntime<S, P>,
        action: &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
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
        if self.source.is_some() {
            return Err(WorthQueryApplicationRequestMutationDenial::DocumentHistory(
                worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial::UnsupportedOperation));
        }
        if !std::ptr::eq(runtime.runtime(), request.application)
            || match &self.history {
                Some(restriction) => {
                    request.branch != restriction.original_source()
                        || self.branch != restriction.candidate()
                }
                None => request.branch != self.branch,
            }
        {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        let selected = runtime
            .runtime()
            .on_branch(request.branch)
            .select(phase)
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = runtime.selected_program_owner(&selected).map_err(
            crate::application_entry::mutation::selected_program::map_selected_program_owner_denial,
        )?;
        owner
            .resolve_declared_action::<I::Binding>(action)
            .map_err(|_| WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)?;
        let mut native = self.request_view(request, action);
        // This private view neither relocates an active owner nor executes/claims
        // a source operation. All actual execution uses the untouched description.
        native.request.branch = request.branch;
        native.history_constraint = None;
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
}
