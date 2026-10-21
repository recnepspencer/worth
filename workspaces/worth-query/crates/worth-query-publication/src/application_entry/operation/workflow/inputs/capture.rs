//! Borrow original typed custody while native capture derives descriptive correspondence.
use super::pack::WorthQueryWorkflowInputDenial;
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
use crate::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestMutationDenial as RequestDenial,
    WorthQueryOperationDescription,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
        ApplicationMutationScopeResolution,
    },
    application_program::ApplicationWorkflowNodeIdentity,
};
use worth_query_execution::{
    facade::{
        application_contribution::WorthQueryAdvancementPhase,
        primary_graph::{WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation},
    },
    publication_boundary::{
        program_publication_access,
        workflow_proposal::{PreparedWorkflowCapturedInput, WorkflowCapturedInputSource},
    },
};
use worth_query_installation::facade::ApplicationSchema;

/// Opaque traversal context. Only proposal preparation can construct it;
/// it grants neither action admission nor a workflow operation slot.
#[doc(hidden)]
pub struct WorthQueryWorkflowInputCapture<'a, 'p, 'r, S> {
    pub(in crate::application_entry) request: &'a WorthQueryApplicationRequest<'r, 'r, 'r, S>,
    pub(in crate::application_entry) phase: &'a WorthQueryAdvancementPhase<'p>,
    pub(in crate::application_entry) selected: &'a WorthQuerySelectedProductOperation<'r, S>,
    pub(in crate::application_entry) inputs: Vec<(
        ApplicationWorkflowNodeIdentity,
        PreparedWorkflowCapturedInput,
    )>,
}

impl<S: ApplicationSchema> WorthQueryWorkflowInputCapture<'_, '_, '_, S> {
    pub(super) fn retain<I>(
        &mut self,
        target: &ApplicationWorkflowNodeIdentity,
        description: &WorthQueryOperationDescription<S, I>,
    ) -> Result<(), WorthQueryWorkflowInputDenial>
    where
        I: ApplicationMutationIntent<S>,
        <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        let result = (|| {
            let binding = self.request.application.installed_schema().installed_mutation_binding::<I::Binding>()
                .map_err(RequestDenial::BindingInstallation)?;
            let principal = self.selected.resolve_authenticated_principal(
                self.phase, binding.principal_binding(), self.request.principal,
                self.request.scope, WorthQueryPrincipalResolutionMode::Ordinary,
            ).map_err(RequestDenial::PrincipalResolution)?;
            let (field, value) = description.intent.scope_binding().into_field_parts(principal.principal_identity());
            let scope = self.selected.resolve_entity(self.phase, field, value,
                self.request.scope, WorthQueryPrincipalResolutionMode::Ordinary)
                .map_err(RequestDenial::ScopeResolution)?;
            let input = description.encoded_input.as_ref().map_err(|cause| RequestDenial::Identity(
                worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial::Input(cause.clone())))?;
            let identities = ApplicationMutationIdentities::<S, I::Binding>::with_encoded_input(&description.key, input)
                .map_err(RequestDenial::Identity)?;
            Ok((scope, identities))
        })().map_err(WorthQueryWorkflowInputDenial::CaptureAdmission)?;
        if description.branch != self.request.branch {
            return Err(WorthQueryWorkflowInputDenial::CaptureBranchMismatch);
        }
        let source = match &description.source {
            None => WorkflowCapturedInputSource::None,
            Some(WorthQueryMutationExpectedSource::Row(source)) => {
                WorkflowCapturedInputSource::Row(source)
            }
            Some(WorthQueryMutationExpectedSource::ResultSet(source)) => {
                WorkflowCapturedInputSource::ResultSet(source)
            }
        };
        let captured = self
            .selected
            .prepare_workflow_captured_input::<I::Binding>(
                self.phase,
                &program_publication_access(),
                &result.0,
                &result.1,
                &description.preconditions,
                source,
            )
            .map_err(WorthQueryWorkflowInputDenial::Capture)?;
        self.inputs.push((target.clone(), captured));
        Ok(())
    }
}
